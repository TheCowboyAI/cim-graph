//! Tests for core module - Event-driven tests only

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::core::event::*;
    use crate::core::node::GenericNode;
    use crate::core::edge::GenericEdge;
    use crate::core::graph::{GraphType, GraphId};
    
    #[test]
    fn test_memory_event_handler() {
        let handler = MemoryEventHandler::new();
        
        // Test initial state
        assert!(handler.events().is_empty());
        
        // Test handling events
        let graph_id = GraphId::new();
        let event1 = GraphEvent::GraphCreated {
            graph_id: graph_id.clone(),
            graph_type: GraphType::Generic,
        };
        handler.handle_event(&event1);
        
        let event2 = GraphEvent::NodeAdded {
            graph_id: graph_id.clone(),
            node_id: "node1".to_string(),
        };
        handler.handle_event(&event2);
        
        // Test events are stored
        let events = handler.events();
        assert_eq!(events.len(), 2);
        
        // Test clear
        handler.clear();
        assert!(handler.events().is_empty());
    }
    
    #[test]
    fn test_all_graph_events() {
        let handler = MemoryEventHandler::new();
        let graph_id = GraphId::new();
        
        // Test all event types
        let events = vec![
            GraphEvent::GraphCreated {
                graph_id: graph_id.clone(),
                graph_type: GraphType::WorkflowGraph,
            },
            GraphEvent::NodeAdded {
                graph_id: graph_id.clone(),
                node_id: "node1".to_string(),
            },
            GraphEvent::NodeRemoved {
                graph_id: graph_id.clone(),
                node_id: "node1".to_string(),
            },
            GraphEvent::EdgeAdded {
                graph_id: graph_id.clone(),
                edge_id: "edge1".to_string(),
                source: "node1".to_string(),
                target: "node2".to_string(),
            },
            GraphEvent::EdgeRemoved {
                graph_id: graph_id.clone(),
                edge_id: "edge1".to_string(),
            },
            GraphEvent::GraphCleared {
                graph_id: graph_id.clone(),
            },
            GraphEvent::MetadataUpdated {
                graph_id: graph_id.clone(),
                field: "name".to_string(),
                old_value: None,
                new_value: Some(serde_json::Value::String("New Name".to_string())),
            },
        ];
        
        // Handle all events
        for event in &events {
            handler.handle_event(event);
        }
        
        // Verify all events were captured
        let captured = handler.events();
        assert_eq!(captured.len(), events.len());
    }
    
    #[test]
    fn test_generic_node() {
        let node = GenericNode::new("id1", "test data");
        assert_eq!(node.id(), "id1");
        assert_eq!(node.data(), &"test data");
        
        // Test with different data type
        let node2 = GenericNode::new("id2", 42);
        assert_eq!(node2.id(), "id2");
        assert_eq!(*node2.data(), 42);
    }
    
    #[test]
    fn test_generic_edge() {
        let edge = GenericEdge::new("src", "tgt", 1.5);
        assert_eq!(edge.source(), "src");
        assert_eq!(edge.target(), "tgt");
        assert_eq!(*edge.data(), 1.5);
        
        // Test with different data type
        let edge2 = GenericEdge::new("src2", "tgt2", 2.0);
        assert_eq!(edge2.source(), "src2");
        assert_eq!(edge2.target(), "tgt2");
    }
    
    #[test]
    fn test_graph_type_enum() {
        // Test GraphType serialization
        let types = vec![
            GraphType::Generic,
            GraphType::IpldGraph,
            GraphType::ContextGraph,
            GraphType::WorkflowGraph,
            GraphType::ConceptGraph,
            GraphType::ComposedGraph,
        ];
        
        for graph_type in types {
            let serialized = serde_json::to_string(&graph_type).unwrap();
            let deserialized: GraphType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(graph_type, deserialized);
        }
    }
    
    #[test]
    fn test_graph_id() {
        // Test GraphId creation and uniqueness
        let id1 = GraphId::new();
        let id2 = GraphId::new();
        
        assert_ne!(id1, id2);
        assert_eq!(id1, id1.clone());
        
        // Test conversion to/from string
        let id_str = id1.to_string();
        let parsed = GraphId::from(id_str.clone());
        assert_eq!(id1.to_string(), parsed.to_string());
    }
    
    // Note: Direct mutation tests have been removed as the system is now event-driven only
    // All state changes must go through events, not direct mutations
}