//! Event system for graph operations

use crate::core::{GraphId, GraphType};
use serde::{Deserialize, Serialize};

/// Events emitted by graph operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphEvent {
    /// Graph was created
    GraphCreated {
        /// Unique identifier for the graph
        graph_id: GraphId,
        /// Type of graph being created (Generic, Hierarchical, etc.)
        graph_type: GraphType,
    },
    
    /// Node was added
    NodeAdded {
        /// Graph that the node is being added to
        graph_id: GraphId,
        /// Unique identifier for the node within the graph
        node_id: String,
    },
    
    /// Node was removed
    NodeRemoved {
        /// Graph that the node is being removed from
        graph_id: GraphId,
        /// Identifier of the node to remove
        node_id: String,
    },
    
    /// Edge was added
    EdgeAdded {
        /// Graph that the edge is being added to
        graph_id: GraphId,
        /// Unique identifier for the edge
        edge_id: String,
        /// Source node ID for the edge
        source: String,
        /// Target node ID for the edge
        target: String,
    },
    
    /// Edge was removed
    EdgeRemoved {
        /// Graph that the edge is being removed from
        graph_id: GraphId,
        /// Identifier of the edge to remove
        edge_id: String,
    },
    
    /// Graph was cleared
    GraphCleared {
        /// Graph that is being cleared of all nodes and edges
        graph_id: GraphId,
    },
    
    /// Metadata was updated
    MetadataUpdated {
        /// Graph whose metadata is being updated
        graph_id: GraphId,
        /// Name of the metadata field being updated
        field: String,
        /// Previous value of the field (if any)
        old_value: Option<serde_json::Value>,
        /// New value of the field (if any)
        new_value: Option<serde_json::Value>,
    }
}

/// Trait for types that can handle graph events
pub trait EventHandler: Send + Sync {
    /// Handle a graph event
    fn handle_event(&self, event: &GraphEvent);
}

/// Simple event handler that stores events in memory
#[derive(Debug, Default)]
pub struct MemoryEventHandler {
    /// In-memory storage of all handled events
    events: std::sync::Mutex<Vec<GraphEvent>>,
}

impl MemoryEventHandler {
    /// Create a new memory event handler
    pub fn new() -> Self {
        Self {
            events: std::sync::Mutex::new(Vec::new()),
        }
    }
    
    /// Get all stored events
    pub fn events(&self) -> Vec<GraphEvent> {
        self.events.lock().unwrap().clone()
    }
    
    /// Clear all stored events
    pub fn clear(&self) {
        self.events.lock().unwrap().clear();
    }
}

impl EventHandler for MemoryEventHandler {
    fn handle_event(&self, event: &GraphEvent) {
        self.events.lock().unwrap().push(event.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_graph_id() -> GraphId {
        GraphId(Uuid::new_v4())
    }

    // ========== GraphEvent Tests ==========

    #[test]
    fn test_graph_event_graph_created() {
        let graph_id = create_test_graph_id();
        let event = GraphEvent::GraphCreated {
            graph_id: graph_id.clone(),
            graph_type: GraphType::Generic,
        };

        match event {
            GraphEvent::GraphCreated { graph_id: gid, graph_type } => {
                assert_eq!(gid, graph_id);
                assert!(matches!(graph_type, GraphType::Generic));
            }
            _ => panic!("Expected GraphCreated event"),
        }
    }

    #[test]
    fn test_graph_event_node_added() {
        let graph_id = create_test_graph_id();
        let event = GraphEvent::NodeAdded {
            graph_id: graph_id.clone(),
            node_id: "test_node".to_string(),
        };

        match event {
            GraphEvent::NodeAdded { graph_id: gid, node_id } => {
                assert_eq!(gid, graph_id);
                assert_eq!(node_id, "test_node");
            }
            _ => panic!("Expected NodeAdded event"),
        }
    }

    #[test]
    fn test_graph_event_node_removed() {
        let graph_id = create_test_graph_id();
        let event = GraphEvent::NodeRemoved {
            graph_id: graph_id.clone(),
            node_id: "removed_node".to_string(),
        };

        match event {
            GraphEvent::NodeRemoved { graph_id: gid, node_id } => {
                assert_eq!(gid, graph_id);
                assert_eq!(node_id, "removed_node");
            }
            _ => panic!("Expected NodeRemoved event"),
        }
    }

    #[test]
    fn test_graph_event_edge_added() {
        let graph_id = create_test_graph_id();
        let event = GraphEvent::EdgeAdded {
            graph_id: graph_id.clone(),
            edge_id: "edge_1".to_string(),
            source: "node_a".to_string(),
            target: "node_b".to_string(),
        };

        match event {
            GraphEvent::EdgeAdded { graph_id: gid, edge_id, source, target } => {
                assert_eq!(gid, graph_id);
                assert_eq!(edge_id, "edge_1");
                assert_eq!(source, "node_a");
                assert_eq!(target, "node_b");
            }
            _ => panic!("Expected EdgeAdded event"),
        }
    }

    #[test]
    fn test_graph_event_edge_removed() {
        let graph_id = create_test_graph_id();
        let event = GraphEvent::EdgeRemoved {
            graph_id: graph_id.clone(),
            edge_id: "edge_to_remove".to_string(),
        };

        match event {
            GraphEvent::EdgeRemoved { graph_id: gid, edge_id } => {
                assert_eq!(gid, graph_id);
                assert_eq!(edge_id, "edge_to_remove");
            }
            _ => panic!("Expected EdgeRemoved event"),
        }
    }

    #[test]
    fn test_graph_event_graph_cleared() {
        let graph_id = create_test_graph_id();
        let event = GraphEvent::GraphCleared {
            graph_id: graph_id.clone(),
        };

        match event {
            GraphEvent::GraphCleared { graph_id: gid } => {
                assert_eq!(gid, graph_id);
            }
            _ => panic!("Expected GraphCleared event"),
        }
    }

    #[test]
    fn test_graph_event_metadata_updated() {
        let graph_id = create_test_graph_id();
        let old_value = Some(serde_json::json!("old"));
        let new_value = Some(serde_json::json!("new"));

        let event = GraphEvent::MetadataUpdated {
            graph_id: graph_id.clone(),
            field: "name".to_string(),
            old_value: old_value.clone(),
            new_value: new_value.clone(),
        };

        match event {
            GraphEvent::MetadataUpdated { graph_id: gid, field, old_value: ov, new_value: nv } => {
                assert_eq!(gid, graph_id);
                assert_eq!(field, "name");
                assert_eq!(ov, old_value);
                assert_eq!(nv, new_value);
            }
            _ => panic!("Expected MetadataUpdated event"),
        }
    }

    #[test]
    fn test_graph_event_metadata_updated_with_none() {
        let graph_id = create_test_graph_id();

        let event = GraphEvent::MetadataUpdated {
            graph_id: graph_id.clone(),
            field: "description".to_string(),
            old_value: None,
            new_value: Some(serde_json::json!("New description")),
        };

        match event {
            GraphEvent::MetadataUpdated { old_value, new_value, .. } => {
                assert!(old_value.is_none());
                assert!(new_value.is_some());
            }
            _ => panic!("Expected MetadataUpdated event"),
        }
    }

    // ========== GraphEvent Clone Tests ==========

    #[test]
    fn test_graph_event_clone() {
        let graph_id = create_test_graph_id();
        let event = GraphEvent::NodeAdded {
            graph_id: graph_id.clone(),
            node_id: "cloned_node".to_string(),
        };

        let cloned = event.clone();

        match (event, cloned) {
            (
                GraphEvent::NodeAdded { graph_id: g1, node_id: n1 },
                GraphEvent::NodeAdded { graph_id: g2, node_id: n2 }
            ) => {
                assert_eq!(g1, g2);
                assert_eq!(n1, n2);
            }
            _ => panic!("Clone failed"),
        }
    }

    // ========== GraphEvent Serialization Tests ==========

    #[test]
    fn test_graph_event_serialization() {
        let graph_id = create_test_graph_id();
        let event = GraphEvent::EdgeAdded {
            graph_id: graph_id.clone(),
            edge_id: "e1".to_string(),
            source: "a".to_string(),
            target: "b".to_string(),
        };

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: GraphEvent = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            GraphEvent::EdgeAdded { edge_id, source, target, .. } => {
                assert_eq!(edge_id, "e1");
                assert_eq!(source, "a");
                assert_eq!(target, "b");
            }
            _ => panic!("Deserialization failed"),
        }
    }

    #[test]
    fn test_graph_event_all_variants_serialization() {
        let graph_id = create_test_graph_id();

        let events = vec![
            GraphEvent::GraphCreated {
                graph_id: graph_id.clone(),
                graph_type: GraphType::WorkflowGraph,
            },
            GraphEvent::NodeAdded {
                graph_id: graph_id.clone(),
                node_id: "n1".to_string(),
            },
            GraphEvent::NodeRemoved {
                graph_id: graph_id.clone(),
                node_id: "n2".to_string(),
            },
            GraphEvent::EdgeAdded {
                graph_id: graph_id.clone(),
                edge_id: "e1".to_string(),
                source: "s".to_string(),
                target: "t".to_string(),
            },
            GraphEvent::EdgeRemoved {
                graph_id: graph_id.clone(),
                edge_id: "e2".to_string(),
            },
            GraphEvent::GraphCleared {
                graph_id: graph_id.clone(),
            },
            GraphEvent::MetadataUpdated {
                graph_id: graph_id.clone(),
                field: "f".to_string(),
                old_value: None,
                new_value: Some(serde_json::json!(42)),
            },
        ];

        for event in events {
            let serialized = serde_json::to_string(&event).unwrap();
            let deserialized: GraphEvent = serde_json::from_str(&serialized).unwrap();
            // Verify round-trip doesn't panic
            let _ = format!("{:?}", deserialized);
        }
    }

    // ========== MemoryEventHandler Tests ==========

    #[test]
    fn test_memory_event_handler_new() {
        let handler = MemoryEventHandler::new();
        assert!(handler.events().is_empty());
    }

    #[test]
    fn test_memory_event_handler_default() {
        let handler = MemoryEventHandler::default();
        assert!(handler.events().is_empty());
    }

    #[test]
    fn test_memory_event_handler_handle_single_event() {
        let handler = MemoryEventHandler::new();
        let graph_id = create_test_graph_id();

        let event = GraphEvent::NodeAdded {
            graph_id,
            node_id: "test".to_string(),
        };

        handler.handle_event(&event);

        let events = handler.events();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_memory_event_handler_handle_multiple_events() {
        let handler = MemoryEventHandler::new();
        let graph_id = create_test_graph_id();

        for i in 0..5 {
            let event = GraphEvent::NodeAdded {
                graph_id: graph_id.clone(),
                node_id: format!("node_{}", i),
            };
            handler.handle_event(&event);
        }

        let events = handler.events();
        assert_eq!(events.len(), 5);
    }

    #[test]
    fn test_memory_event_handler_clear() {
        let handler = MemoryEventHandler::new();
        let graph_id = create_test_graph_id();

        handler.handle_event(&GraphEvent::NodeAdded {
            graph_id: graph_id.clone(),
            node_id: "n1".to_string(),
        });
        handler.handle_event(&GraphEvent::NodeAdded {
            graph_id,
            node_id: "n2".to_string(),
        });

        assert_eq!(handler.events().len(), 2);

        handler.clear();

        assert!(handler.events().is_empty());
    }

    #[test]
    fn test_memory_event_handler_events_returns_clone() {
        let handler = MemoryEventHandler::new();
        let graph_id = create_test_graph_id();

        handler.handle_event(&GraphEvent::NodeAdded {
            graph_id,
            node_id: "test".to_string(),
        });

        let events1 = handler.events();
        let events2 = handler.events();

        // Both should be equal but different vectors
        assert_eq!(events1.len(), events2.len());
    }

    #[test]
    fn test_memory_event_handler_preserves_event_order() {
        let handler = MemoryEventHandler::new();
        let graph_id = create_test_graph_id();

        let node_ids = vec!["first", "second", "third", "fourth"];
        for node_id in &node_ids {
            handler.handle_event(&GraphEvent::NodeAdded {
                graph_id: graph_id.clone(),
                node_id: node_id.to_string(),
            });
        }

        let events = handler.events();
        assert_eq!(events.len(), 4);

        for (i, event) in events.iter().enumerate() {
            match event {
                GraphEvent::NodeAdded { node_id, .. } => {
                    assert_eq!(node_id, node_ids[i]);
                }
                _ => panic!("Unexpected event type"),
            }
        }
    }

    #[test]
    fn test_memory_event_handler_mixed_event_types() {
        let handler = MemoryEventHandler::new();
        let graph_id = create_test_graph_id();

        handler.handle_event(&GraphEvent::GraphCreated {
            graph_id: graph_id.clone(),
            graph_type: GraphType::Generic,
        });
        handler.handle_event(&GraphEvent::NodeAdded {
            graph_id: graph_id.clone(),
            node_id: "n1".to_string(),
        });
        handler.handle_event(&GraphEvent::EdgeAdded {
            graph_id: graph_id.clone(),
            edge_id: "e1".to_string(),
            source: "n1".to_string(),
            target: "n2".to_string(),
        });
        handler.handle_event(&GraphEvent::MetadataUpdated {
            graph_id,
            field: "name".to_string(),
            old_value: None,
            new_value: Some(serde_json::json!("Test Graph")),
        });

        let events = handler.events();
        assert_eq!(events.len(), 4);

        assert!(matches!(events[0], GraphEvent::GraphCreated { .. }));
        assert!(matches!(events[1], GraphEvent::NodeAdded { .. }));
        assert!(matches!(events[2], GraphEvent::EdgeAdded { .. }));
        assert!(matches!(events[3], GraphEvent::MetadataUpdated { .. }));
    }

    #[test]
    fn test_memory_event_handler_debug() {
        let handler = MemoryEventHandler::new();
        let debug_str = format!("{:?}", handler);
        assert!(debug_str.contains("MemoryEventHandler"));
    }

    #[test]
    fn test_memory_event_handler_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let handler = Arc::new(MemoryEventHandler::new());
        let graph_id = create_test_graph_id();

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let handler = Arc::clone(&handler);
                let graph_id = graph_id.clone();
                thread::spawn(move || {
                    handler.handle_event(&GraphEvent::NodeAdded {
                        graph_id,
                        node_id: format!("node_{}", i),
                    });
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // All events should be recorded
        assert_eq!(handler.events().len(), 10);
    }

    // ========== GraphEvent Debug Tests ==========

    #[test]
    fn test_graph_event_debug() {
        let graph_id = create_test_graph_id();
        let event = GraphEvent::NodeAdded {
            graph_id,
            node_id: "debug_test".to_string(),
        };

        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("NodeAdded"));
        assert!(debug_str.contains("debug_test"));
    }
}