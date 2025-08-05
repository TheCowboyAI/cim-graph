//! Event sourcing system for graphs with correlation, causation, and deterministic ordering
//!
//! This module provides a proper event sourcing implementation that includes:
//! - Correlation IDs for tracking related events
//! - Causation IDs for tracking event chains
//! - Deterministic event ordering
//! - Event versioning and replay capabilities

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use crate::core::GraphType;

/// Event metadata that provides correlation, causation, and ordering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Unique event ID
    pub event_id: Uuid,
    
    /// Aggregate ID this event belongs to
    pub aggregate_id: Uuid,
    
    /// Correlation ID - groups related events across aggregates
    pub correlation_id: Uuid,
    
    /// Causation ID - the event that caused this event
    pub causation_id: Option<Uuid>,
    
    /// Event version number for ordering within aggregate
    pub version: u64,
    
    /// Timestamp when event occurred
    pub occurred_at: DateTime<Utc>,
    
    /// User/system that triggered the event
    pub triggered_by: String,
    
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl EventMetadata {
    /// Create new event metadata
    pub fn new(aggregate_id: Uuid, correlation_id: Uuid, version: u64, triggered_by: String) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id,
            causation_id: None,
            version,
            occurred_at: Utc::now(),
            triggered_by,
            metadata: HashMap::new(),
        }
    }
    
    /// Create metadata for a caused event
    pub fn with_causation(mut self, causation_id: Uuid) -> Self {
        self.causation_id = Some(causation_id);
        self
    }
    
    /// Add custom metadata
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Graph domain events with full event sourcing support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEvent {
    /// Event metadata
    pub metadata: EventMetadata,
    
    /// Event payload
    pub payload: GraphEventPayload,
}

/// The actual event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphEventPayload {
    /// Graph was created
    GraphCreated {
        graph_type: GraphType,
        name: Option<String>,
        description: Option<String>,
    },
    
    /// Node was added
    NodeAdded {
        node_id: String,
        node_type: String,
        data: serde_json::Value,
    },
    
    /// Edge was added
    EdgeAdded {
        edge_id: String,
        source_id: String,
        target_id: String,
        edge_type: String,
        data: serde_json::Value,
    },
    
    /// Node was removed
    NodeRemoved {
        node_id: String,
    },
    
    /// Edge was removed
    EdgeRemoved {
        edge_id: String,
    },
    
    /// Metadata updated
    MetadataUpdated {
        field: String,
        old_value: Option<serde_json::Value>,
        new_value: Option<serde_json::Value>,
    },
    
    /// Graph cleared
    GraphCleared,
}

/// Event store for persisting and querying events
pub trait EventStore {
    /// Append events to the store
    fn append(&mut self, events: Vec<GraphEvent>) -> Result<(), String>;
    
    /// Get all events for an aggregate
    fn get_events(&self, aggregate_id: Uuid) -> Vec<GraphEvent>;
    
    /// Get events by correlation ID
    fn get_correlated_events(&self, correlation_id: Uuid) -> Vec<GraphEvent>;
    
    /// Get event causation chain
    fn get_causation_chain(&self, event_id: Uuid) -> Vec<GraphEvent>;
    
    /// Get events in a specific version range
    fn get_events_in_range(&self, aggregate_id: Uuid, from_version: u64, to_version: u64) -> Vec<GraphEvent>;
}

/// In-memory event store for testing
#[derive(Debug, Default)]
pub struct MemoryEventStore {
    events: Vec<GraphEvent>,
    by_aggregate: HashMap<Uuid, Vec<GraphEvent>>,
    by_correlation: HashMap<Uuid, Vec<GraphEvent>>,
    by_event_id: HashMap<Uuid, GraphEvent>,
}

impl EventStore for MemoryEventStore {
    fn append(&mut self, events: Vec<GraphEvent>) -> Result<(), String> {
        for event in events {
            // Store by aggregate ID
            self.by_aggregate
                .entry(event.metadata.aggregate_id)
                .or_default()
                .push(event.clone());
            
            // Store by correlation ID
            self.by_correlation
                .entry(event.metadata.correlation_id)
                .or_default()
                .push(event.clone());
            
            // Store by event ID
            self.by_event_id.insert(event.metadata.event_id, event.clone());
            
            // Store in main list
            self.events.push(event);
        }
        Ok(())
    }
    
    fn get_events(&self, aggregate_id: Uuid) -> Vec<GraphEvent> {
        self.by_aggregate
            .get(&aggregate_id)
            .cloned()
            .unwrap_or_default()
    }
    
    fn get_correlated_events(&self, correlation_id: Uuid) -> Vec<GraphEvent> {
        self.by_correlation
            .get(&correlation_id)
            .cloned()
            .unwrap_or_default()
    }
    
    fn get_causation_chain(&self, event_id: Uuid) -> Vec<GraphEvent> {
        let mut chain = Vec::new();
        let mut current_id = Some(event_id);
        
        while let Some(id) = current_id {
            if let Some(event) = self.by_event_id.get(&id) {
                chain.push(event.clone());
                current_id = event.metadata.causation_id;
            } else {
                break;
            }
        }
        
        chain.reverse(); // Return in chronological order
        chain
    }
    
    fn get_events_in_range(&self, aggregate_id: Uuid, from_version: u64, to_version: u64) -> Vec<GraphEvent> {
        self.get_events(aggregate_id)
            .into_iter()
            .filter(|e| e.metadata.version >= from_version && e.metadata.version <= to_version)
            .collect()
    }
}

/// Event-sourced graph aggregate
#[derive(Debug)]
pub struct GraphAggregate {
    id: Uuid,
    graph_type: GraphType,
    version: u64,
    node_count: usize,
    edge_count: usize,
    name: Option<String>,
    description: Option<String>,
}

impl GraphAggregate {
    /// Create new aggregate
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            graph_type: GraphType::Generic,
            version: 0,
            node_count: 0,
            edge_count: 0,
            name: None,
            description: None,
        }
    }
    
    /// Replay events to rebuild state
    pub fn replay(id: Uuid, events: Vec<GraphEvent>) -> Self {
        let mut aggregate = Self::new(id);
        for event in events {
            aggregate.apply(&event);
        }
        aggregate
    }
    
    /// Apply event to update state
    pub fn apply(&mut self, event: &GraphEvent) {
        self.version = event.metadata.version;
        
        match &event.payload {
            GraphEventPayload::GraphCreated { graph_type, name, description } => {
                self.graph_type = *graph_type;
                self.name = name.clone();
                self.description = description.clone();
            }
            GraphEventPayload::NodeAdded { .. } => {
                self.node_count += 1;
            }
            GraphEventPayload::NodeRemoved { .. } => {
                self.node_count = self.node_count.saturating_sub(1);
            }
            GraphEventPayload::EdgeAdded { .. } => {
                self.edge_count += 1;
            }
            GraphEventPayload::EdgeRemoved { .. } => {
                self.edge_count = self.edge_count.saturating_sub(1);
            }
            GraphEventPayload::GraphCleared => {
                self.node_count = 0;
                self.edge_count = 0;
            }
            GraphEventPayload::MetadataUpdated { .. } => {
                // Handle metadata updates if needed
            }
        }
    }
    
    /// Get the current version
    pub fn version(&self) -> u64 {
        self.version
    }
    
    /// Get node count
    pub fn node_count(&self) -> usize {
        self.node_count
    }
    
    /// Get edge count  
    pub fn edge_count(&self) -> usize {
        self.edge_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_correlation_and_causation() {
        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        
        // Create initial event
        let create_metadata = EventMetadata::new(
            aggregate_id,
            correlation_id,
            1,
            "test-user".to_string()
        );
        
        let create_event = GraphEvent {
            metadata: create_metadata.clone(),
            payload: GraphEventPayload::GraphCreated {
                graph_type: GraphType::WorkflowGraph,
                name: Some("Test Workflow".to_string()),
                description: None,
            },
        };
        
        // Create correlated event with causation
        let add_node_metadata = EventMetadata::new(
            aggregate_id,
            correlation_id,
            2,
            "test-user".to_string()
        ).with_causation(create_metadata.event_id);
        
        let add_node_event = GraphEvent {
            metadata: add_node_metadata.clone(),
            payload: GraphEventPayload::NodeAdded {
                node_id: "start".to_string(),
                node_type: "StartNode".to_string(),
                data: serde_json::json!({"label": "Start"}),
            },
        };
        
        // Test event store
        let mut store = MemoryEventStore::default();
        store.append(vec![create_event.clone(), add_node_event.clone()]).unwrap();
        
        // Test correlation
        let correlated = store.get_correlated_events(correlation_id);
        assert_eq!(correlated.len(), 2);
        assert_eq!(correlated[0].metadata.version, 1);
        assert_eq!(correlated[1].metadata.version, 2);
        
        // Test causation chain
        let chain = store.get_causation_chain(add_node_metadata.event_id);
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0].metadata.event_id, create_metadata.event_id);
        assert_eq!(chain[1].metadata.event_id, add_node_metadata.event_id);
    }
    
    #[test]
    fn test_aggregate_replay() {
        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        
        let events = vec![
            GraphEvent {
                metadata: EventMetadata::new(aggregate_id, correlation_id, 1, "test".to_string()),
                payload: GraphEventPayload::GraphCreated {
                    graph_type: GraphType::Generic,
                    name: Some("Test".to_string()),
                    description: None,
                },
            },
            GraphEvent {
                metadata: EventMetadata::new(aggregate_id, correlation_id, 2, "test".to_string()),
                payload: GraphEventPayload::NodeAdded {
                    node_id: "n1".to_string(),
                    node_type: "Node".to_string(),
                    data: serde_json::json!({}),
                },
            },
            GraphEvent {
                metadata: EventMetadata::new(aggregate_id, correlation_id, 3, "test".to_string()),
                payload: GraphEventPayload::NodeAdded {
                    node_id: "n2".to_string(),
                    node_type: "Node".to_string(),
                    data: serde_json::json!({}),
                },
            },
            GraphEvent {
                metadata: EventMetadata::new(aggregate_id, correlation_id, 4, "test".to_string()),
                payload: GraphEventPayload::EdgeAdded {
                    edge_id: "e1".to_string(),
                    source_id: "n1".to_string(),
                    target_id: "n2".to_string(),
                    edge_type: "Edge".to_string(),
                    data: serde_json::json!({}),
                },
            },
        ];
        
        let aggregate = GraphAggregate::replay(aggregate_id, events);
        assert_eq!(aggregate.version, 4);
        assert_eq!(aggregate.node_count, 2);
        assert_eq!(aggregate.edge_count, 1);
        assert_eq!(aggregate.name, Some("Test".to_string()));
    }
    
    #[test]
    fn test_deterministic_event_ordering() {
        let aggregate_id = Uuid::new_v4();
        let mut store = MemoryEventStore::default();
        
        // Create events with specific versions
        let events: Vec<_> = (1..=5).map(|version| {
            GraphEvent {
                metadata: EventMetadata::new(
                    aggregate_id,
                    Uuid::new_v4(),
                    version,
                    "test".to_string()
                ),
                payload: GraphEventPayload::NodeAdded {
                    node_id: format!("node-{}", version),
                    node_type: "Node".to_string(),
                    data: serde_json::json!({"version": version}),
                },
            }
        }).collect();
        
        store.append(events).unwrap();
        
        // Test version range queries
        let range = store.get_events_in_range(aggregate_id, 2, 4);
        assert_eq!(range.len(), 3);
        assert_eq!(range[0].metadata.version, 2);
        assert_eq!(range[1].metadata.version, 3);
        assert_eq!(range[2].metadata.version, 4);
        
        // Verify deterministic ordering
        let all_events = store.get_events(aggregate_id);
        for i in 0..all_events.len() - 1 {
            assert!(all_events[i].metadata.version < all_events[i + 1].metadata.version);
            assert!(all_events[i].metadata.occurred_at <= all_events[i + 1].metadata.occurred_at);
        }
    }
}