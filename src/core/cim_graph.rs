//! The CORRECT CIM graph implementation - graphs as event projections
//! 
//! In CIM, graphs are NOT mutable data structures. They are read-only
//! projections computed from event streams. This has ALWAYS been the design.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use crate::Result;

/// Graph projection - a read-only view computed from events
/// This is what a graph actually is in CIM
pub trait GraphProjection {
    /// Node type for this projection
    type Node;
    /// Edge type for this projection
    type Edge;
    
    /// Get the aggregate ID this projection represents
    fn aggregate_id(&self) -> Uuid;
    
    /// Get the current version (last event sequence number)
    fn version(&self) -> u64;
    
    /// Get a node by ID (read-only)
    fn get_node(&self, node_id: &str) -> Option<&Self::Node>;
    
    /// Get an edge by ID (read-only)
    fn get_edge(&self, edge_id: &str) -> Option<&Self::Edge>;
    
    /// Get all nodes (read-only)
    fn nodes(&self) -> Vec<&Self::Node>;
    
    /// Get all edges (read-only)
    fn edges(&self) -> Vec<&Self::Edge>;
    
    /// Get node count
    fn node_count(&self) -> usize;
    
    /// Get edge count
    fn edge_count(&self) -> usize;
    
    /// Find edges between nodes
    fn edges_between(&self, from: &str, to: &str) -> Vec<&Self::Edge>;
    
    /// Get neighbors of a node
    fn neighbors(&self, node_id: &str) -> Vec<&str>;
}

/// Events are the ONLY way to change graph state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEvent {
    /// Unique event ID
    pub event_id: Uuid,
    
    /// Aggregate this event belongs to
    pub aggregate_id: Uuid,
    
    /// Event sequence number (from NATS JetStream)
    pub sequence: u64,
    
    /// NATS subject (from cim-domain's subject module)
    pub subject: String,
    
    /// When this happened
    pub timestamp: DateTime<Utc>,
    
    /// Correlation ID for related events
    pub correlation_id: Uuid,
    
    /// Event that caused this one
    pub causation_id: Option<Uuid>,
    
    /// The actual event data
    pub data: EventData,
}

/// Event data variants - these define ALL possible state changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventData {
    /// Graph initialized
    GraphInitialized {
        /// Type of graph (e.g., "workflow", "ipld", "composed")
        graph_type: String,
        /// Initial metadata for the graph
        metadata: HashMap<String, serde_json::Value>,
    },
    
    /// Node added to graph
    NodeAdded {
        /// Unique identifier for the node
        node_id: String,
        /// Type/category of the node
        node_type: String,
        /// Additional node data
        data: serde_json::Value,
    },
    
    /// Edge added between nodes
    EdgeAdded {
        /// Unique identifier for the edge
        edge_id: String,
        /// ID of the source node
        source_id: String,
        /// ID of the target node
        target_id: String,
        /// Type/category of the edge
        edge_type: String,
        /// Additional edge data
        data: serde_json::Value,
    },
    
    /// Node removed
    NodeRemoved {
        /// ID of the node to remove
        node_id: String,
    },
    
    /// Edge removed
    EdgeRemoved {
        /// ID of the edge to remove
        edge_id: String,
    },
    
    /// Node data updated
    NodeUpdated {
        /// ID of the node to update
        node_id: String,
        /// New data for the node
        data: serde_json::Value,
    },
    
    /// Edge data updated
    EdgeUpdated {
        /// ID of the edge to update
        edge_id: String,
        /// New data for the edge
        data: serde_json::Value,
    },
}

/// Commands request state changes - they can be rejected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphCommand {
    /// Initialize a new graph
    InitializeGraph {
        /// Aggregate ID for the graph
        aggregate_id: Uuid,
        /// Type of graph to create
        graph_type: String,
        /// Initial metadata for the graph
        metadata: HashMap<String, serde_json::Value>,
    },
    
    /// Add a node
    AddNode {
        /// Graph aggregate ID
        aggregate_id: Uuid,
        /// Unique identifier for the node
        node_id: String,
        /// Type/category of the node
        node_type: String,
        /// Additional node data
        data: serde_json::Value,
    },
    
    /// Add an edge
    AddEdge {
        /// Graph aggregate ID
        aggregate_id: Uuid,
        /// Unique identifier for the edge
        edge_id: String,
        /// ID of the source node
        source_id: String,
        /// ID of the target node
        target_id: String,
        /// Type/category of the edge
        edge_type: String,
        /// Additional edge data
        data: serde_json::Value,
    },
    
    /// Remove a node
    RemoveNode {
        /// Graph aggregate ID
        aggregate_id: Uuid,
        /// ID of the node to remove
        node_id: String,
    },
    
    /// Remove an edge
    RemoveEdge {
        /// Graph aggregate ID
        aggregate_id: Uuid,
        /// ID of the edge to remove
        edge_id: String,
    },
}

/// Command handler validates commands and produces events
pub trait CommandHandler<P: GraphProjection> {
    /// Handle a command, returning events if valid
    fn handle(&self, command: GraphCommand, projection: &P) -> Result<Vec<GraphEvent>>;
}

/// Event handler processes events (projections, side effects, etc)
pub trait EventHandler {
    /// Process an event after it's committed to the stream
    fn handle(&mut self, event: &GraphEvent);
}

/// Projector builds projections from event streams
pub trait Projector<P: GraphProjection> {
    /// Build projection from events
    fn project(&self, events: Vec<GraphEvent>) -> P;

    /// Update existing projection with new event
    fn apply(&self, projection: &mut P, event: &GraphEvent);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // ========== GraphEvent Tests ==========

    fn create_test_event() -> GraphEvent {
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            sequence: 1,
            subject: "test.graph.event".to_string(),
            timestamp: Utc::now(),
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            data: EventData::GraphInitialized {
                graph_type: "test".to_string(),
                metadata: HashMap::new(),
            },
        }
    }

    #[test]
    fn test_graph_event_creation() {
        let event = create_test_event();
        assert!(!event.event_id.is_nil());
        assert!(!event.aggregate_id.is_nil());
        assert_eq!(event.sequence, 1);
        assert_eq!(event.subject, "test.graph.event");
    }

    #[test]
    fn test_graph_event_with_causation() {
        let cause_id = Uuid::new_v4();
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            sequence: 2,
            subject: "test.caused.event".to_string(),
            timestamp: Utc::now(),
            correlation_id: Uuid::new_v4(),
            causation_id: Some(cause_id),
            data: EventData::NodeAdded {
                node_id: "n1".to_string(),
                node_type: "test".to_string(),
                data: serde_json::json!({}),
            },
        };

        assert_eq!(event.causation_id, Some(cause_id));
    }

    #[test]
    fn test_graph_event_clone() {
        let event = create_test_event();
        let cloned = event.clone();

        assert_eq!(event.event_id, cloned.event_id);
        assert_eq!(event.aggregate_id, cloned.aggregate_id);
        assert_eq!(event.sequence, cloned.sequence);
        assert_eq!(event.subject, cloned.subject);
    }

    #[test]
    fn test_graph_event_serialization() {
        let event = create_test_event();
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: GraphEvent = serde_json::from_str(&serialized).unwrap();

        assert_eq!(event.event_id, deserialized.event_id);
        assert_eq!(event.aggregate_id, deserialized.aggregate_id);
        assert_eq!(event.sequence, deserialized.sequence);
    }

    #[test]
    fn test_graph_event_debug() {
        let event = create_test_event();
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("GraphEvent"));
    }

    // ========== EventData Tests ==========

    #[test]
    fn test_event_data_graph_initialized() {
        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), serde_json::json!("value"));

        let data = EventData::GraphInitialized {
            graph_type: "workflow".to_string(),
            metadata: metadata.clone(),
        };

        match data {
            EventData::GraphInitialized { graph_type, metadata: m } => {
                assert_eq!(graph_type, "workflow");
                assert_eq!(m.get("key").unwrap(), &serde_json::json!("value"));
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_event_data_node_added() {
        let data = EventData::NodeAdded {
            node_id: "node_1".to_string(),
            node_type: "entity".to_string(),
            data: serde_json::json!({"name": "Test Node"}),
        };

        match data {
            EventData::NodeAdded { node_id, node_type, data: node_data } => {
                assert_eq!(node_id, "node_1");
                assert_eq!(node_type, "entity");
                assert_eq!(node_data["name"], "Test Node");
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_event_data_edge_added() {
        let data = EventData::EdgeAdded {
            edge_id: "edge_1".to_string(),
            source_id: "node_a".to_string(),
            target_id: "node_b".to_string(),
            edge_type: "contains".to_string(),
            data: serde_json::json!({}),
        };

        match data {
            EventData::EdgeAdded { edge_id, source_id, target_id, edge_type, .. } => {
                assert_eq!(edge_id, "edge_1");
                assert_eq!(source_id, "node_a");
                assert_eq!(target_id, "node_b");
                assert_eq!(edge_type, "contains");
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_event_data_node_removed() {
        let data = EventData::NodeRemoved {
            node_id: "deleted_node".to_string(),
        };

        match data {
            EventData::NodeRemoved { node_id } => {
                assert_eq!(node_id, "deleted_node");
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_event_data_edge_removed() {
        let data = EventData::EdgeRemoved {
            edge_id: "deleted_edge".to_string(),
        };

        match data {
            EventData::EdgeRemoved { edge_id } => {
                assert_eq!(edge_id, "deleted_edge");
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_event_data_node_updated() {
        let data = EventData::NodeUpdated {
            node_id: "updated_node".to_string(),
            data: serde_json::json!({"status": "modified"}),
        };

        match data {
            EventData::NodeUpdated { node_id, data: node_data } => {
                assert_eq!(node_id, "updated_node");
                assert_eq!(node_data["status"], "modified");
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_event_data_edge_updated() {
        let data = EventData::EdgeUpdated {
            edge_id: "updated_edge".to_string(),
            data: serde_json::json!({"weight": 5}),
        };

        match data {
            EventData::EdgeUpdated { edge_id, data: edge_data } => {
                assert_eq!(edge_id, "updated_edge");
                assert_eq!(edge_data["weight"], 5);
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_event_data_all_variants_clone() {
        let variants: Vec<EventData> = vec![
            EventData::GraphInitialized {
                graph_type: "test".to_string(),
                metadata: HashMap::new(),
            },
            EventData::NodeAdded {
                node_id: "n".to_string(),
                node_type: "t".to_string(),
                data: serde_json::json!({}),
            },
            EventData::EdgeAdded {
                edge_id: "e".to_string(),
                source_id: "s".to_string(),
                target_id: "t".to_string(),
                edge_type: "et".to_string(),
                data: serde_json::json!({}),
            },
            EventData::NodeRemoved { node_id: "n".to_string() },
            EventData::EdgeRemoved { edge_id: "e".to_string() },
            EventData::NodeUpdated {
                node_id: "n".to_string(),
                data: serde_json::json!({}),
            },
            EventData::EdgeUpdated {
                edge_id: "e".to_string(),
                data: serde_json::json!({}),
            },
        ];

        for variant in variants {
            let cloned = variant.clone();
            let _ = format!("{:?}", cloned); // Verify Debug works
        }
    }

    #[test]
    fn test_event_data_serialization() {
        let data = EventData::NodeAdded {
            node_id: "serialization_test".to_string(),
            node_type: "test_type".to_string(),
            data: serde_json::json!({"nested": {"value": 42}}),
        };

        let serialized = serde_json::to_string(&data).unwrap();
        let deserialized: EventData = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            EventData::NodeAdded { node_id, node_type, data: node_data } => {
                assert_eq!(node_id, "serialization_test");
                assert_eq!(node_type, "test_type");
                assert_eq!(node_data["nested"]["value"], 42);
            }
            _ => panic!("Deserialization failed"),
        }
    }

    // ========== GraphCommand Tests ==========

    #[test]
    fn test_graph_command_initialize_graph() {
        let agg_id = Uuid::new_v4();
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), serde_json::json!("1.0"));

        let command = GraphCommand::InitializeGraph {
            aggregate_id: agg_id,
            graph_type: "workflow".to_string(),
            metadata: metadata.clone(),
        };

        match command {
            GraphCommand::InitializeGraph { aggregate_id, graph_type, metadata: m } => {
                assert_eq!(aggregate_id, agg_id);
                assert_eq!(graph_type, "workflow");
                assert_eq!(m.get("version").unwrap(), &serde_json::json!("1.0"));
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_graph_command_add_node() {
        let agg_id = Uuid::new_v4();
        let command = GraphCommand::AddNode {
            aggregate_id: agg_id,
            node_id: "new_node".to_string(),
            node_type: "entity".to_string(),
            data: serde_json::json!({"label": "New Node"}),
        };

        match command {
            GraphCommand::AddNode { aggregate_id, node_id, node_type, data } => {
                assert_eq!(aggregate_id, agg_id);
                assert_eq!(node_id, "new_node");
                assert_eq!(node_type, "entity");
                assert_eq!(data["label"], "New Node");
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_graph_command_add_edge() {
        let agg_id = Uuid::new_v4();
        let command = GraphCommand::AddEdge {
            aggregate_id: agg_id,
            edge_id: "new_edge".to_string(),
            source_id: "src".to_string(),
            target_id: "tgt".to_string(),
            edge_type: "depends_on".to_string(),
            data: serde_json::json!({"weight": 1}),
        };

        match command {
            GraphCommand::AddEdge {
                aggregate_id,
                edge_id,
                source_id,
                target_id,
                edge_type,
                data,
            } => {
                assert_eq!(aggregate_id, agg_id);
                assert_eq!(edge_id, "new_edge");
                assert_eq!(source_id, "src");
                assert_eq!(target_id, "tgt");
                assert_eq!(edge_type, "depends_on");
                assert_eq!(data["weight"], 1);
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_graph_command_remove_node() {
        let agg_id = Uuid::new_v4();
        let command = GraphCommand::RemoveNode {
            aggregate_id: agg_id,
            node_id: "node_to_remove".to_string(),
        };

        match command {
            GraphCommand::RemoveNode { aggregate_id, node_id } => {
                assert_eq!(aggregate_id, agg_id);
                assert_eq!(node_id, "node_to_remove");
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_graph_command_remove_edge() {
        let agg_id = Uuid::new_v4();
        let command = GraphCommand::RemoveEdge {
            aggregate_id: agg_id,
            edge_id: "edge_to_remove".to_string(),
        };

        match command {
            GraphCommand::RemoveEdge { aggregate_id, edge_id } => {
                assert_eq!(aggregate_id, agg_id);
                assert_eq!(edge_id, "edge_to_remove");
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_graph_command_clone() {
        let command = GraphCommand::AddNode {
            aggregate_id: Uuid::new_v4(),
            node_id: "cloned_node".to_string(),
            node_type: "test".to_string(),
            data: serde_json::json!({}),
        };

        let cloned = command.clone();

        match (command, cloned) {
            (
                GraphCommand::AddNode { node_id: n1, .. },
                GraphCommand::AddNode { node_id: n2, .. },
            ) => {
                assert_eq!(n1, n2);
            }
            _ => panic!("Clone failed"),
        }
    }

    #[test]
    fn test_graph_command_serialization() {
        let command = GraphCommand::AddEdge {
            aggregate_id: Uuid::new_v4(),
            edge_id: "ser_edge".to_string(),
            source_id: "s".to_string(),
            target_id: "t".to_string(),
            edge_type: "link".to_string(),
            data: serde_json::json!({}),
        };

        let serialized = serde_json::to_string(&command).unwrap();
        let deserialized: GraphCommand = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            GraphCommand::AddEdge { edge_id, .. } => {
                assert_eq!(edge_id, "ser_edge");
            }
            _ => panic!("Deserialization failed"),
        }
    }

    #[test]
    fn test_graph_command_all_variants_debug() {
        let commands: Vec<GraphCommand> = vec![
            GraphCommand::InitializeGraph {
                aggregate_id: Uuid::new_v4(),
                graph_type: "test".to_string(),
                metadata: HashMap::new(),
            },
            GraphCommand::AddNode {
                aggregate_id: Uuid::new_v4(),
                node_id: "n".to_string(),
                node_type: "t".to_string(),
                data: serde_json::json!({}),
            },
            GraphCommand::AddEdge {
                aggregate_id: Uuid::new_v4(),
                edge_id: "e".to_string(),
                source_id: "s".to_string(),
                target_id: "t".to_string(),
                edge_type: "et".to_string(),
                data: serde_json::json!({}),
            },
            GraphCommand::RemoveNode {
                aggregate_id: Uuid::new_v4(),
                node_id: "n".to_string(),
            },
            GraphCommand::RemoveEdge {
                aggregate_id: Uuid::new_v4(),
                edge_id: "e".to_string(),
            },
        ];

        for command in commands {
            let debug_str = format!("{:?}", command);
            assert!(!debug_str.is_empty());
        }
    }

    // ========== Integration Tests ==========

    #[test]
    fn test_event_correlation_chain() {
        let correlation_id = Uuid::new_v4();
        let event1 = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            sequence: 1,
            subject: "test.event.1".to_string(),
            timestamp: Utc::now(),
            correlation_id,
            causation_id: None,
            data: EventData::GraphInitialized {
                graph_type: "test".to_string(),
                metadata: HashMap::new(),
            },
        };

        let event2 = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: event1.aggregate_id,
            sequence: 2,
            subject: "test.event.2".to_string(),
            timestamp: Utc::now(),
            correlation_id, // Same correlation ID
            causation_id: Some(event1.event_id), // Caused by event1
            data: EventData::NodeAdded {
                node_id: "n1".to_string(),
                node_type: "test".to_string(),
                data: serde_json::json!({}),
            },
        };

        // Same correlation chain
        assert_eq!(event1.correlation_id, event2.correlation_id);
        // Causation link
        assert_eq!(event2.causation_id, Some(event1.event_id));
        // Same aggregate
        assert_eq!(event1.aggregate_id, event2.aggregate_id);
        // Increasing sequence
        assert!(event2.sequence > event1.sequence);
    }

    #[test]
    fn test_event_sequence_ordering() {
        let agg_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();

        let events: Vec<GraphEvent> = (1..=5)
            .map(|seq| GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: agg_id,
                sequence: seq,
                subject: format!("test.event.{}", seq),
                timestamp: Utc::now(),
                correlation_id,
                causation_id: None,
                data: EventData::NodeAdded {
                    node_id: format!("node_{}", seq),
                    node_type: "test".to_string(),
                    data: serde_json::json!({}),
                },
            })
            .collect();

        // Verify sequence ordering
        for i in 1..events.len() {
            assert!(events[i].sequence > events[i - 1].sequence);
        }
    }

    #[test]
    fn test_command_to_event_flow() {
        // Create a command
        let agg_id = Uuid::new_v4();
        let command = GraphCommand::AddNode {
            aggregate_id: agg_id,
            node_id: "cmd_node".to_string(),
            node_type: "entity".to_string(),
            data: serde_json::json!({"from": "command"}),
        };

        // Simulate producing an event from the command
        let event = match &command {
            GraphCommand::AddNode { aggregate_id, node_id, node_type, data } => {
                GraphEvent {
                    event_id: Uuid::new_v4(),
                    aggregate_id: *aggregate_id,
                    sequence: 1,
                    subject: "graph.node.added".to_string(),
                    timestamp: Utc::now(),
                    correlation_id: Uuid::new_v4(),
                    causation_id: None,
                    data: EventData::NodeAdded {
                        node_id: node_id.clone(),
                        node_type: node_type.clone(),
                        data: data.clone(),
                    },
                }
            }
            _ => panic!("Unexpected command type"),
        };

        // Verify the event was produced from the command
        assert_eq!(event.aggregate_id, agg_id);
        match event.data {
            EventData::NodeAdded { node_id, node_type, data } => {
                assert_eq!(node_id, "cmd_node");
                assert_eq!(node_type, "entity");
                assert_eq!(data["from"], "command");
            }
            _ => panic!("Wrong event data type"),
        }
    }

    #[test]
    fn test_graph_event_timestamp_precision() {
        let before = Utc::now();
        let event = create_test_event();
        let after = Utc::now();

        assert!(event.timestamp >= before);
        assert!(event.timestamp <= after);
    }

    #[test]
    fn test_empty_metadata() {
        let metadata: HashMap<String, serde_json::Value> = HashMap::new();
        let data = EventData::GraphInitialized {
            graph_type: "empty_metadata".to_string(),
            metadata,
        };

        match data {
            EventData::GraphInitialized { metadata, .. } => {
                assert!(metadata.is_empty());
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_complex_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("string".to_string(), serde_json::json!("value"));
        metadata.insert("number".to_string(), serde_json::json!(42));
        metadata.insert("boolean".to_string(), serde_json::json!(true));
        metadata.insert("array".to_string(), serde_json::json!([1, 2, 3]));
        metadata.insert(
            "object".to_string(),
            serde_json::json!({"nested": "value"}),
        );

        let data = EventData::GraphInitialized {
            graph_type: "complex".to_string(),
            metadata,
        };

        match data {
            EventData::GraphInitialized { metadata, .. } => {
                assert_eq!(metadata.len(), 5);
                assert_eq!(metadata["string"], "value");
                assert_eq!(metadata["number"], 42);
                assert_eq!(metadata["boolean"], true);
            }
            _ => panic!("Wrong variant"),
        }
    }
}
