//! Direct tests for event module

#[cfg(test)]
mod tests {
    use super::super::event::*;
    use super::super::graph::{GraphId, GraphType};
    
    #[test]
    fn test_event_module_coverage() {
        // Test MemoryEventHandler directly
        let handler = MemoryEventHandler::new();
        assert!(handler.events().is_empty());
        
        let graph_id = GraphId::new();
        
        // Test all event variants
        let events = vec![
            GraphEvent::GraphCreated {
                graph_id: graph_id.clone(),
                graph_type: GraphType::Generic,
            },
            GraphEvent::NodeAdded {
                graph_id: graph_id.clone(),
                node_id: "n1".to_string(),
            },
            GraphEvent::NodeRemoved {
                graph_id: graph_id.clone(),
                node_id: "n1".to_string(),
            },
            GraphEvent::EdgeAdded {
                graph_id: graph_id.clone(),
                edge_id: "e1".to_string(),
                source: "n1".to_string(),
                target: "n2".to_string(),
            },
            GraphEvent::EdgeRemoved {
                graph_id: graph_id.clone(),
                edge_id: "e1".to_string(),
            },
            GraphEvent::GraphCleared {
                graph_id: graph_id.clone(),
            },
            GraphEvent::MetadataUpdated {
                graph_id: graph_id.clone(),
                field: "name".to_string(),
                old_value: None,
                new_value: Some(serde_json::json!("New Name")),
            },
        ];
        
        // Handle each event
        for event in &events {
            handler.handle_event(event);
        }
        
        // Verify all stored
        let stored = handler.events();
        assert_eq!(stored.len(), events.len());
        
        // Test clear
        handler.clear();
        assert!(handler.events().is_empty());
    }
}