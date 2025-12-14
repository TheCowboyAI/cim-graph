//! NATS JetStream event store implementation
//!
//! Provides persistent event storage using NATS JetStream.
//!
//! This module uses CIM Domain's subject algebra types:
//! - [`Subject`] for building validated, concrete NATS subjects
//! - [`SubjectPattern`] for building wildcard subscription patterns
//! - [`SubjectSegment`] for individual subject tokens

use crate::error::{GraphError, Result};
use crate::events::{GraphEvent, EventPayload, GraphType as SubjectGraphType, EventType};
use crate::core::ipld_chain::Cid;
use async_nats::jetstream::{self, consumer::PullConsumer, stream::Stream};
use futures::StreamExt;
use async_trait::async_trait;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use cim_domain::{Subject, SubjectSegment, SubjectPattern};
use crate::channels::default_prefix;

/// NATS-specific errors
#[derive(Debug, thiserror::Error)]
pub enum NatsError {
    /// NATS connection failed
    #[error("NATS connection error: {0}")]
    ConnectionError(String),
    
    /// JetStream operation failed
    #[error("JetStream error: {0}")]
    JetStreamError(String),
    
    /// Requested stream does not exist
    #[error("Stream not found: {0}")]
    StreamNotFound(String),
    
    /// Consumer operation failed
    #[error("Consumer error: {0}")]
    ConsumerError(String),
    
    /// Failed to serialize/deserialize data
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Subscription operation failed
    #[error("Subscription error: {0}")]
    SubscriptionError(String),
}

impl From<NatsError> for GraphError {
    fn from(err: NatsError) -> Self {
        GraphError::External(err.to_string())
    }
}

/// Configuration for JetStream connection
#[derive(Debug, Clone)]
pub struct JetStreamConfig {
    /// NATS server URL
    pub server_url: String,
    
    /// Stream name for graph events
    pub stream_name: String,
    
    /// Subject prefix for events
    pub subject_prefix: String,
    
    /// Max age for events (in seconds)
    pub max_age_secs: u64,
    
    /// Enable deduplication
    pub enable_dedup: bool,
}

impl Default for JetStreamConfig {
    fn default() -> Self {
        Self {
            server_url: "localhost:4222".to_string(),
            stream_name: "CIM_GRAPH_EVENTS".to_string(),
            subject_prefix: default_prefix(),
            max_age_secs: 86400 * 30, // 30 days
            enable_dedup: true,
        }
    }
}

/// Event envelope for JetStream storage
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EventEnvelope {
    /// Event data
    event: GraphEvent,
    
    /// CID of the event (from IPLD chain)
    cid: Option<String>,
    
    /// Sequence number in the aggregate
    sequence: u64,
    
    /// Headers for NATS
    headers: EventHeaders,
}

/// Headers for event routing and correlation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EventHeaders {
    /// Aggregate ID the event belongs to
    aggregate_id: Uuid,
    
    /// Event type for filtering
    event_type: String,
    
    /// Graph type (ipld, context, workflow, etc.)
    graph_type: String,
    
    /// Correlation ID for tracking
    correlation_id: Uuid,
    
    /// Causation ID if this event was caused by another
    causation_id: Option<Uuid>,
}

/// JetStream-based event store
#[derive(Debug)]
pub struct JetStreamEventStore {
    /// NATS client
    client: async_nats::Client,
    
    /// JetStream context
    jetstream: jetstream::Context,
    
    /// Configuration
    config: JetStreamConfig,
    
    /// Stream handle
    stream: Arc<RwLock<Option<Stream>>>,
}

impl JetStreamEventStore {
    /// Create a new JetStream event store
    pub async fn new(config: JetStreamConfig) -> Result<Self> {
        // Connect to NATS
        let client = async_nats::connect(&config.server_url)
            .await
            .map_err(|e| NatsError::ConnectionError(e.to_string()))?;
        
        // Create JetStream context
        let jetstream = jetstream::new(client.clone());
        
        let store = Self {
            client,
            jetstream,
            config,
            stream: Arc::new(RwLock::new(None)),
        };
        
        // Initialize stream
        store.ensure_stream().await?;
        
        Ok(store)
    }
    
    /// Ensure the stream exists with proper configuration
    async fn ensure_stream(&self) -> Result<()> {
        // Build wildcard pattern using SubjectPattern for all events under prefix
        let all_subjects_pattern = build_wildcard_pattern(&self.config.subject_prefix, ">");
        let subjects = vec![all_subjects_pattern];
        
        let mut stream_config = jetstream::stream::Config {
            name: self.config.stream_name.clone(),
            subjects,
            max_age: std::time::Duration::from_secs(self.config.max_age_secs),
            ..Default::default()
        };
        
        if self.config.enable_dedup {
            stream_config.duplicate_window = std::time::Duration::from_secs(120);
        }
        
        // Create or update stream
        let stream = self.jetstream
            .get_or_create_stream(stream_config)
            .await
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;
        
        *self.stream.write().await = Some(stream);
        
        Ok(())
    }
    
    /// Publish an event to JetStream
    pub async fn publish_event(&self, event: GraphEvent, cid: Option<Cid>) -> Result<u64> {
        // Determine event type and graph type from payload
        let (event_type, graph_type) = determine_event_type(&event.payload);

        // Build subject using Subject type: {prefix}.{graph_type}.{aggregate_id}
        let graph_type_str = format!("{:?}", graph_type).to_lowercase();
        let subject = build_entity_subject(
            &self.config.subject_prefix,
            &graph_type_str,
            &event.aggregate_id.to_string(),
        ).map_err(|e| NatsError::JetStreamError(e))?;
        
        // For compatibility, also store string representations
        let event_type_str = format!("{:?}", event_type).to_lowercase();
        let graph_type_str = format!("{:?}", graph_type).to_lowercase();
        
        // Create headers
        let headers = EventHeaders {
            aggregate_id: event.aggregate_id,
            event_type: event_type_str.clone(),
            graph_type: graph_type_str.clone(),
            correlation_id: event.correlation_id,
            causation_id: event.causation_id,
        };
        
        // Create envelope
        let envelope = EventEnvelope {
            event: event.clone(),
            cid: cid.map(|c| c.to_string()),
            sequence: 0, // Will be set by JetStream
            headers,
        };
        
        // Serialize envelope
        let payload = serde_json::to_vec(&envelope)
            .map_err(|e| NatsError::SerializationError(e.to_string()))?;
        
        // Create NATS headers
        let mut nats_headers = async_nats::HeaderMap::new();
        nats_headers.insert("Cim-Event-Id", event.event_id.to_string());
        nats_headers.insert("Cim-Aggregate-Id", event.aggregate_id.to_string());
        nats_headers.insert("Cim-Event-Type", event_type_str.clone());
        nats_headers.insert("Cim-Graph-Type", graph_type_str);
        nats_headers.insert("Cim-Correlation-Id", event.correlation_id.to_string());
        
        if let Some(causation_id) = event.causation_id {
            nats_headers.insert("Cim-Causation-Id", causation_id.to_string());
        }
        
        if self.config.enable_dedup {
            nats_headers.insert("Nats-Msg-Id", event.event_id.to_string());
        }
        
        // Publish with headers
        let ack = self.jetstream
            .publish_with_headers(subject, nats_headers, payload.into())
            .await
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?
            .await
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;
        
        Ok(ack.sequence)
    }
    
    /// Fetch events for an aggregate
    pub async fn fetch_events(&self, aggregate_id: Uuid) -> Result<Vec<GraphEvent>> {
        let stream = self.stream.read().await;
        let stream = stream.as_ref()
            .ok_or_else(|| NatsError::StreamNotFound(self.config.stream_name.clone()))?;
        
        // Create consumer for this aggregate using SubjectPattern for wildcard
        let consumer_name = format!("cim-graph-{}", aggregate_id);
        let filter_subject = build_aggregate_filter_pattern(
            &self.config.subject_prefix,
            &aggregate_id.to_string(),
        );
        
        let consumer: PullConsumer = stream
            .get_or_create_consumer(&consumer_name, jetstream::consumer::pull::Config {
                durable_name: Some(consumer_name.clone()),
                filter_subjects: vec![filter_subject],
                ..Default::default()
            })
            .await
            .map_err(|e| NatsError::ConsumerError(e.to_string()))?;
        
        // Fetch all messages
        let mut events = Vec::new();
        let messages = consumer.fetch()
            .max_messages(1000)
            .messages()
            .await
            .map_err(|e| NatsError::ConsumerError(e.to_string()))?;
        
        let messages: Vec<_> = messages.try_collect().await
            .map_err(|e| NatsError::ConsumerError(e.to_string()))?;
        
        for message in messages {
            // Deserialize envelope
            if let Ok(envelope) = serde_json::from_slice::<EventEnvelope>(&message.payload) {
                events.push(envelope.event);
            }
            
            // Acknowledge message
            let _ = message.ack().await;
        }
        
        Ok(events)
    }
    
    /// Fetch events by correlation ID
    ///
    /// Uses SubjectSegment validation to ensure the correlation ID is valid
    /// for use in NATS subject routing.
    pub async fn fetch_by_correlation(&self, correlation_id: Uuid) -> Result<Vec<GraphEvent>> {
        // Validate correlation ID as a valid subject segment
        let correlation_segments = build_correlation_segments(correlation_id);
        if correlation_segments.is_empty() {
            return Err(NatsError::ConsumerError(
                format!("Invalid correlation ID format: {}", correlation_id)
            ).into());
        }
        let validated_corr_id = correlation_segments[0].as_str().to_string();

        // This requires scanning all events and filtering
        // In production, you'd want to use a separate stream or index
        let stream = self.stream.read().await;
        let stream = stream.as_ref()
            .ok_or_else(|| NatsError::StreamNotFound(self.config.stream_name.clone()))?;

        // Create ephemeral consumer
        let consumer: PullConsumer = stream
            .create_consumer(jetstream::consumer::pull::Config {
                ..Default::default()
            })
            .await
            .map_err(|e| NatsError::ConsumerError(e.to_string()))?;

        let mut events = Vec::new();
        let messages = consumer.fetch()
            .max_messages(1000)
            .messages()
            .await
            .map_err(|e| NatsError::ConsumerError(e.to_string()))?;

        let messages: Vec<_> = messages.try_collect().await
            .map_err(|e| NatsError::ConsumerError(e.to_string()))?;

        for message in messages {
            // Check correlation ID in headers using validated segment
            if let Some(corr_id) = message.headers
                .as_ref()
                .and_then(|h| h.get("Cim-Correlation-Id"))
                .and_then(|value| std::str::from_utf8(value.as_ref()).ok())
            {
                if corr_id == validated_corr_id {
                    if let Ok(envelope) = serde_json::from_slice::<EventEnvelope>(&message.payload) {
                        events.push(envelope.event);
                    }
                }
            }

            let _ = message.ack().await;
        }

        Ok(events)
    }
    
    /// Subscribe to events for real-time updates
    pub async fn subscribe_to_aggregate(&self, aggregate_id: Uuid) -> Result<EventSubscription> {
        // Use SubjectPattern wildcard to subscribe to all event types for this aggregate
        let filter_subject = build_aggregate_filter_pattern(
            &self.config.subject_prefix,
            &aggregate_id.to_string(),
        );
        
        let subscriber = self.client
            .subscribe(filter_subject)
            .await
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;
        
        Ok(EventSubscription {
            subscriber,
            aggregate_id,
        })
    }
    
    /// Subscribe to events for a specific graph type and aggregate
    pub async fn subscribe_to_graph_type(
        &self,
        graph_type: SubjectGraphType,
        aggregate_id: Uuid,
    ) -> Result<EventSubscription> {
        // Build SubjectPattern for graph type with wildcard for all aggregates
        let graph_type_str = format!("{:?}", graph_type).to_lowercase();
        let filter_subject = build_graph_type_filter_pattern(
            &self.config.subject_prefix,
            &graph_type_str,
        );
        
        let subscriber = self.client
            .subscribe(filter_subject)
            .await
            .map_err(|e| NatsError::SubscriptionError(e.to_string()))?;
        
        Ok(EventSubscription {
            subscriber,
            aggregate_id,
        })
    }
    
    /// Create a durable consumer for event replay
    pub async fn create_replay_consumer(
        &self,
        consumer_name: &str,
        start_sequence: Option<u64>,
    ) -> Result<ReplayConsumer> {
        let stream = self.stream.read().await;
        let stream = stream.as_ref()
            .ok_or_else(|| NatsError::StreamNotFound(self.config.stream_name.clone()))?;
        
        let mut config = jetstream::consumer::pull::Config {
            durable_name: Some(consumer_name.to_string()),
            ..Default::default()
        };
        
        if let Some(seq) = start_sequence {
            config.deliver_policy = jetstream::consumer::DeliverPolicy::ByStartSequence {
                start_sequence: seq,
            };
        }
        
        let consumer = stream
            .create_consumer(config)
            .await
            .map_err(|e| NatsError::ConsumerError(e.to_string()))?;
        
        Ok(ReplayConsumer { consumer })
    }
}

/// Subscription to aggregate events
#[derive(Debug)]
pub struct EventSubscription {
    subscriber: async_nats::Subscriber,
    aggregate_id: Uuid,
}

impl EventSubscription {
    /// Get the aggregate ID this subscription is for
    pub fn aggregate_id(&self) -> Uuid {
        self.aggregate_id
    }
    
    /// Get the next event
    pub async fn next(&mut self) -> Result<Option<GraphEvent>> {
        if let Some(message) = self.subscriber.next().await {
            if let Ok(envelope) = serde_json::from_slice::<EventEnvelope>(&message.payload) {
                return Ok(Some(envelope.event));
            }
        }
        Ok(None)
    }
}

/// Consumer for replaying events
#[derive(Debug)]
pub struct ReplayConsumer {
    consumer: PullConsumer,
}

impl ReplayConsumer {
    /// Fetch a batch of events
    pub async fn fetch_batch(&self, max_messages: usize) -> Result<Vec<GraphEvent>> {
        let mut events = Vec::new();
        let messages = self.consumer
            .fetch()
            .max_messages(max_messages)
            .messages()
            .await
            .map_err(|e| NatsError::ConsumerError(e.to_string()))?;
        
        let messages: Vec<_> = messages.try_collect().await
            .map_err(|e| NatsError::ConsumerError(e.to_string()))?;
        
        for message in messages {
            if let Ok(envelope) = serde_json::from_slice::<EventEnvelope>(&message.payload) {
                events.push(envelope.event);
            }
            let _ = message.ack().await;
        }
        
        Ok(events)
    }
}

/// Trait for async event store operations
#[async_trait]
pub trait AsyncEventStore: Send + Sync {
    /// Publish an event
    async fn publish(&self, event: GraphEvent) -> Result<u64>;
    
    /// Fetch events for an aggregate
    async fn fetch_aggregate_events(&self, aggregate_id: Uuid) -> Result<Vec<GraphEvent>>;
    
    /// Subscribe to events
    async fn subscribe(&self, aggregate_id: Uuid) -> Result<Box<dyn EventStream>>;
}

/// Trait for event streams
#[async_trait]
pub trait EventStream: Send + Sync {
    /// Get next event
    async fn next(&mut self) -> Result<Option<GraphEvent>>;
}

#[async_trait]
impl AsyncEventStore for JetStreamEventStore {
    async fn publish(&self, event: GraphEvent) -> Result<u64> {
        self.publish_event(event, None).await
    }
    
    async fn fetch_aggregate_events(&self, aggregate_id: Uuid) -> Result<Vec<GraphEvent>> {
        self.fetch_events(aggregate_id).await
    }
    
    async fn subscribe(&self, aggregate_id: Uuid) -> Result<Box<dyn EventStream>> {
        let subscription = self.subscribe_to_aggregate(aggregate_id).await?;
        Ok(Box::new(subscription))
    }
}

#[async_trait]
impl EventStream for EventSubscription {
    async fn next(&mut self) -> Result<Option<GraphEvent>> {
        self.next().await
    }
}

// ============================================================================
// Subject Building Helpers using CIM Domain Subject Algebra
// ============================================================================

/// Build a concrete subject for entity events using Subject type.
///
/// Creates a validated subject like `prefix.graph_type.entity_id`
fn build_entity_subject(prefix: &str, graph_type: &str, entity_id: &str) -> std::result::Result<String, String> {
    let mut segments = Vec::new();

    // Parse prefix into segments
    for part in prefix.split('.') {
        if !part.is_empty() {
            segments.push(
                SubjectSegment::new(part.to_string())
                    .map_err(|e| format!("Invalid prefix segment '{}': {}", part, e))?
            );
        }
    }

    // Add graph type segment
    segments.push(
        SubjectSegment::new(graph_type.to_string())
            .map_err(|e| format!("Invalid graph type '{}': {}", graph_type, e))?
    );

    // Add entity ID segment
    segments.push(
        SubjectSegment::new(entity_id.to_string())
            .map_err(|e| format!("Invalid entity ID '{}': {}", entity_id, e))?
    );

    Subject::from_segments(segments)
        .map(|s| s.to_string())
        .map_err(|e| format!("Failed to build subject: {}", e))
}

/// Build a wildcard pattern for stream subscription using SubjectPattern.
///
/// Creates patterns like `prefix.>` for multi-level wildcards
fn build_wildcard_pattern(prefix: &str, wildcard: &str) -> String {
    let pattern_str = format!("{}.{}", prefix, wildcard);
    // Validate the pattern using SubjectPattern
    match SubjectPattern::parse(&pattern_str) {
        Ok(pattern) => pattern.to_string(),
        Err(_) => pattern_str, // Fallback to string if validation fails
    }
}

/// Build a filter pattern for aggregate events using SubjectPattern.
///
/// Creates patterns like `prefix.*.aggregate_id` for single-level wildcards
fn build_aggregate_filter_pattern(prefix: &str, aggregate_id: &str) -> String {
    let pattern_str = format!("{}.*.{}", prefix, aggregate_id);
    // Validate using SubjectPattern
    match SubjectPattern::parse(&pattern_str) {
        Ok(pattern) => pattern.to_string(),
        Err(_) => pattern_str,
    }
}

/// Build a filter pattern for graph type events using SubjectPattern.
///
/// Creates patterns like `prefix.graph_type.*` for subscribing to all entities of a type
fn build_graph_type_filter_pattern(prefix: &str, graph_type: &str) -> String {
    let pattern_str = format!("{}.{}.*", prefix, graph_type);
    // Validate using SubjectPattern
    match SubjectPattern::parse(&pattern_str) {
        Ok(pattern) => pattern.to_string(),
        Err(_) => pattern_str,
    }
}

/// Build a subject for correlation-based queries using SubjectSegment validation.
///
/// Returns segments that can be used for header-based filtering
fn build_correlation_segments(correlation_id: Uuid) -> Vec<SubjectSegment> {
    // Use SubjectSegment to validate the correlation ID format
    if let Ok(segment) = SubjectSegment::new(correlation_id.to_string()) {
        vec![segment]
    } else {
        vec![]
    }
}

// ============================================================================
// Event Type Determination
// ============================================================================

/// Determine event type and graph type from payload
fn determine_event_type(payload: &EventPayload) -> (EventType, SubjectGraphType) {
    match payload {
        EventPayload::Generic(_) => (EventType::Updated, SubjectGraphType::Composed),
        EventPayload::Ipld(p) => {
            use crate::events::IpldPayload::*;
            match p {
                CidAdded { .. } => (EventType::NodeAdded, SubjectGraphType::Ipld),
                CidLinkAdded { .. } => (EventType::EdgeAdded, SubjectGraphType::Ipld),
                CidPinned { .. } | CidUnpinned { .. } => (EventType::Updated, SubjectGraphType::Ipld),
            }
        }
        EventPayload::Context(p) => {
            use crate::events::ContextPayload::*;
            match p {
                BoundedContextCreated { .. } => (EventType::Created, SubjectGraphType::Context),
                AggregateAdded { .. } | EntityAdded { .. } => (EventType::NodeAdded, SubjectGraphType::Context),
                ValueObjectAttached { .. } | RelationshipEstablished { .. } => (EventType::Updated, SubjectGraphType::Context),
            }
        }
        EventPayload::Workflow(p) => {
            use crate::events::WorkflowPayload::*;
            match p {
                WorkflowDefined { .. } => (EventType::Created, SubjectGraphType::Workflow),
                StateAdded { .. } => (EventType::NodeAdded, SubjectGraphType::Workflow),
                TransitionAdded { .. } => (EventType::EdgeAdded, SubjectGraphType::Workflow),
                StateTransitioned { .. } => (EventType::StateChanged, SubjectGraphType::Workflow),
                _ => (EventType::Updated, SubjectGraphType::Workflow),
            }
        }
        EventPayload::Concept(p) => {
            use crate::events::ConceptPayload::*;
            match p {
                ConceptDefined { .. } => (EventType::Created, SubjectGraphType::Concept),
                RelationAdded { .. } => (EventType::EdgeAdded, SubjectGraphType::Concept),
                PropertiesAdded { .. } | PropertyInferred { .. } => (EventType::Updated, SubjectGraphType::Concept),
            }
        }
        EventPayload::Composed(p) => {
            use crate::events::ComposedPayload::*;
            match p {
                SubGraphAdded { .. } => (EventType::NodeAdded, SubjectGraphType::Composed),
                CrossGraphLinkCreated { .. } => (EventType::EdgeAdded, SubjectGraphType::Composed),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Subject Building Tests - Exercise Subject, SubjectSegment, SubjectPattern
    // ========================================================================

    #[test]
    fn test_build_entity_subject() {
        // Test building entity subjects using Subject and SubjectSegment
        let subject = build_entity_subject("local.cim", "workflow", "12345").unwrap();
        assert_eq!(subject, "local.cim.workflow.12345");

        // Test with single-part prefix
        let subject = build_entity_subject("cim", "ipld", "abc123").unwrap();
        assert_eq!(subject, "cim.ipld.abc123");
    }

    #[test]
    fn test_build_entity_subject_with_uuid() {
        // Test that UUID-formatted entity IDs work correctly
        let uuid = Uuid::new_v4();
        let subject = build_entity_subject("local.cim", "context", &uuid.to_string()).unwrap();
        assert!(subject.starts_with("local.cim.context."));
        assert!(subject.contains(&uuid.to_string()));
    }

    #[test]
    fn test_build_wildcard_pattern() {
        // Test building wildcard patterns using SubjectPattern
        let pattern = build_wildcard_pattern("local.cim", ">");
        assert_eq!(pattern, "local.cim.>");

        let pattern = build_wildcard_pattern("events", "*");
        assert_eq!(pattern, "events.*");
    }

    #[test]
    fn test_build_aggregate_filter_pattern() {
        // Test aggregate filter patterns using SubjectPattern
        let pattern = build_aggregate_filter_pattern("local.cim", "12345");
        assert_eq!(pattern, "local.cim.*.12345");

        let uuid = Uuid::new_v4();
        let pattern = build_aggregate_filter_pattern("cim.graph", &uuid.to_string());
        assert!(pattern.contains(".*.")); // Contains single-level wildcard
        assert!(pattern.ends_with(&uuid.to_string()));
    }

    #[test]
    fn test_build_graph_type_filter_pattern() {
        // Test graph type filter patterns using SubjectPattern
        let pattern = build_graph_type_filter_pattern("local.cim", "workflow");
        assert_eq!(pattern, "local.cim.workflow.*");

        let pattern = build_graph_type_filter_pattern("cim", "ipld");
        assert_eq!(pattern, "cim.ipld.*");
    }

    #[test]
    fn test_build_correlation_segments() {
        // Test SubjectSegment validation for correlation IDs
        let corr_id = Uuid::new_v4();
        let segments = build_correlation_segments(corr_id);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].as_str(), corr_id.to_string());
    }

    #[test]
    fn test_subject_types_integration() {
        // Integration test: verify Subject, SubjectSegment, SubjectPattern work together
        let prefix = "local.cim";
        let graph_type = "workflow";
        let entity_id = Uuid::new_v4().to_string();

        // Build concrete subject
        let subject = build_entity_subject(prefix, graph_type, &entity_id).unwrap();

        // Build patterns that would match this subject
        let all_pattern = build_wildcard_pattern(prefix, ">");
        let agg_pattern = build_aggregate_filter_pattern(prefix, &entity_id);
        let type_pattern = build_graph_type_filter_pattern(prefix, graph_type);

        // Verify patterns are syntactically valid by parsing them
        assert!(SubjectPattern::parse(&all_pattern).is_ok());
        assert!(SubjectPattern::parse(&agg_pattern).is_ok());
        assert!(SubjectPattern::parse(&type_pattern).is_ok());

        // The concrete subject should be parseable as a Subject
        assert!(subject.contains(prefix));
        assert!(subject.contains(graph_type));
        assert!(subject.contains(&entity_id));
    }

    // ========================================================================
    // Integration Tests (Require NATS Server)
    // ========================================================================

    /// Create a unique test config to avoid conflicts with existing streams
    fn test_config() -> JetStreamConfig {
        let test_id = Uuid::new_v4().to_string()[..8].to_string();
        JetStreamConfig {
            server_url: "localhost:4222".to_string(),
            stream_name: format!("CIM_GRAPH_TEST_{}", test_id),
            subject_prefix: format!("test.cim.graph.{}", test_id),
            max_age_secs: 3600, // 1 hour for tests
            enable_dedup: true,
        }
    }

    #[tokio::test]
    #[ignore] // Requires NATS server
    async fn test_jetstream_connection() {
        let config = test_config();
        let store = JetStreamEventStore::new(config).await;
        assert!(store.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires NATS server
    async fn test_publish_and_fetch() {
        use crate::events::GenericPayload;

        let config = test_config();
        let store = JetStreamEventStore::new(config).await.unwrap();

        let aggregate_id = Uuid::new_v4();
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Generic(GenericPayload {
                event_type: "TestEvent".to_string(),
                data: serde_json::json!({ "test": true }),
            }),
        };

        // Publish event
        let seq = store.publish_event(event.clone(), None).await.unwrap();
        assert!(seq > 0);

        // Fetch events
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let events = store.fetch_events(aggregate_id).await.unwrap();
        assert!(!events.is_empty());
        assert_eq!(events[0].event_id, event.event_id);
    }

    #[tokio::test]
    #[ignore] // Requires NATS server
    async fn test_subscribe_to_aggregate() {
        use crate::events::GenericPayload;

        let config = test_config();
        let store = JetStreamEventStore::new(config).await.unwrap();

        let aggregate_id = Uuid::new_v4();

        // Subscribe before publishing
        let subscription = store.subscribe_to_aggregate(aggregate_id).await.unwrap();
        assert_eq!(subscription.aggregate_id(), aggregate_id);

        // Publish an event
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Generic(GenericPayload {
                event_type: "SubscriptionTest".to_string(),
                data: serde_json::json!({ "subscribed": true }),
            }),
        };

        let seq = store.publish_event(event.clone(), None).await.unwrap();
        assert!(seq > 0);

        // Note: Core NATS subscriptions may not receive JetStream messages
        // This test verifies subscription setup works
    }

    #[tokio::test]
    #[ignore] // Requires NATS server
    async fn test_correlation_id_fetch() {
        use crate::events::GenericPayload;

        let config = test_config();
        let store = JetStreamEventStore::new(config).await.unwrap();

        let correlation_id = Uuid::new_v4();
        let aggregate_id = Uuid::new_v4();

        // Publish multiple events with same correlation ID
        for i in 0..3 {
            let event = GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id,
                correlation_id,
                causation_id: None,
                payload: EventPayload::Generic(GenericPayload {
                    event_type: format!("CorrelatedEvent{}", i),
                    data: serde_json::json!({ "index": i }),
                }),
            };
            store.publish_event(event, None).await.unwrap();
        }

        // Small delay for message delivery
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Fetch by correlation ID
        let events = store.fetch_by_correlation(correlation_id).await.unwrap();
        assert_eq!(events.len(), 3);
        for event in &events {
            assert_eq!(event.correlation_id, correlation_id);
        }
    }

    #[tokio::test]
    #[ignore] // Requires NATS server
    async fn test_replay_consumer() {
        use crate::events::GenericPayload;

        let config = test_config();
        let store = JetStreamEventStore::new(config).await.unwrap();

        let aggregate_id = Uuid::new_v4();

        // Publish several events
        for i in 0..5 {
            let event = GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id,
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Generic(GenericPayload {
                    event_type: format!("ReplayEvent{}", i),
                    data: serde_json::json!({ "sequence": i }),
                }),
            };
            store.publish_event(event, None).await.unwrap();
        }

        // Small delay
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Create replay consumer from beginning
        let consumer_name = format!("replay-test-{}", Uuid::new_v4());
        let replay_consumer = store.create_replay_consumer(&consumer_name, None).await.unwrap();

        // Fetch batch
        let events = replay_consumer.fetch_batch(10).await.unwrap();
        assert_eq!(events.len(), 5);
    }
}
