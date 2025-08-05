//! NATS JetStream event store implementation
//!
//! Provides persistent event storage using NATS JetStream

use crate::error::{GraphError, Result};
use crate::events::{GraphEvent, EventPayload, build_event_subject, build_graph_subscription, GraphType as SubjectGraphType, EventType};
use crate::core::ipld_chain::Cid;
use async_nats::jetstream::{self, consumer::PullConsumer, stream::Stream};
use futures::StreamExt;
use async_trait::async_trait;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// NATS-specific errors
#[derive(Debug, thiserror::Error)]
pub enum NatsError {
    #[error("NATS connection error: {0}")]
    ConnectionError(String),
    
    #[error("JetStream error: {0}")]
    JetStreamError(String),
    
    #[error("Stream not found: {0}")]
    StreamNotFound(String),
    
    #[error("Consumer error: {0}")]
    ConsumerError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
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
            subject_prefix: "cim.graph".to_string(),
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
        let subjects = vec![format!("{}.*.*", self.config.subject_prefix)];
        
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
        
        // Build subject using cim-subject
        let subject = build_event_subject(graph_type, event.aggregate_id, event_type);
        
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
        
        // Create consumer for this aggregate
        let consumer_name = format!("cim-graph-{}", aggregate_id);
        let filter_subject = format!("{}.*.{}", self.config.subject_prefix, aggregate_id);
        
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
        let mut messages = consumer.fetch()
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
    pub async fn fetch_by_correlation(&self, correlation_id: Uuid) -> Result<Vec<GraphEvent>> {
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
        let mut messages = consumer.fetch()
            .max_messages(1000)
            .messages()
            .await
            .map_err(|e| NatsError::ConsumerError(e.to_string()))?;
        
        let messages: Vec<_> = messages.try_collect().await
            .map_err(|e| NatsError::ConsumerError(e.to_string()))?;
        
        for message in messages {
            // Check correlation ID in headers
            if let Some(corr_id) = message.headers
                .as_ref()
                .and_then(|h| h.get("Cim-Correlation-Id"))
                .and_then(|value| std::str::from_utf8(value.as_ref()).ok())
                .and_then(|s| Uuid::parse_str(s).ok())
            {
                if corr_id == correlation_id {
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
        // Use wildcard to subscribe to all event types for this aggregate
        let filter_subject = build_graph_subscription(SubjectGraphType::Composed, aggregate_id);
        
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
        let filter_subject = build_graph_subscription(graph_type, aggregate_id);
        
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
pub struct EventSubscription {
    subscriber: async_nats::Subscriber,
    aggregate_id: Uuid,
}

impl EventSubscription {
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
pub struct ReplayConsumer {
    consumer: PullConsumer,
}

impl ReplayConsumer {
    /// Fetch a batch of events
    pub async fn fetch_batch(&self, max_messages: usize) -> Result<Vec<GraphEvent>> {
        let mut events = Vec::new();
        let mut messages = self.consumer
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
    
    #[tokio::test]
    #[ignore] // Requires NATS server
    async fn test_jetstream_connection() {
        let config = JetStreamConfig::default();
        let store = JetStreamEventStore::new(config).await;
        assert!(store.is_ok());
    }
    
    #[tokio::test]
    #[ignore] // Requires NATS server
    async fn test_publish_and_fetch() {
        use crate::events::GenericPayload;
        
        let config = JetStreamConfig::default();
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
}