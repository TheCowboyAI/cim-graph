//! Tests for core module

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::core::event::*;
    use crate::core::node::GenericNode;
    use crate::core::edge::GenericEdge;
    use crate::core::graph::{BasicGraph, GraphType};
    use std::sync::{Arc, Mutex};
    
    #[test]
    fn test_graph_events() {
        // Test event creation and handler
        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();
        
        // Create a closure that implements the handler logic
        let handler_fn = move |event: &GraphEvent| {
            events_clone.lock().unwrap().push(event.clone());
        };
        
        // Wrap it in a struct that implements EventHandler
        struct TestHandler<F> {
            handler: F,
        }
        
        impl<F> EventHandler for TestHandler<F>
        where
            F: Fn(&GraphEvent) + Send + Sync,
        {
            fn handle_event(&self, event: &GraphEvent) {
                (self.handler)(event);
            }
        }
        
        let handler = Arc::new(TestHandler { handler: handler_fn });
        
        let mut graph = GraphBuilder::<GenericNode<&str>, GenericEdge<()>>::new()
            .add_handler(handler)
            .build_event()
            .unwrap();
            
        // Add node should trigger NodeAdded event
        let node_id = graph.add_node(GenericNode::new("test", "data")).unwrap();
        
        // Add edge should trigger EdgeAdded event
        let node2_id = graph.add_node(GenericNode::new("test2", "data2")).unwrap();
        graph.add_edge(GenericEdge::new(&node_id, &node2_id, ())).unwrap();
        
        // Check events were captured
        let captured_events = events.lock().unwrap();
        assert_eq!(captured_events.len(), 3); // NodeAdded + NodeAdded + EdgeAdded
        
        match &captured_events[0] {
            GraphEvent::NodeAdded { node_id: id, .. } => assert_eq!(id, &node_id),
            _ => panic!("Expected NodeAdded event"),
        }
        
        match &captured_events[2] {
            GraphEvent::EdgeAdded { source, target, .. } => {
                assert_eq!(source, &node_id);
                assert_eq!(target, &node2_id);
            }
            _ => panic!("Expected EdgeAdded event"),
        }
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
    fn test_basic_graph() {
        let mut graph = BasicGraph::<GenericNode<&str>, GenericEdge<()>>::new(GraphType::Generic);
        
        // Test metadata
        assert_eq!(graph.graph_type(), GraphType::Generic);
        graph.metadata_mut().name = Some("Test Graph".to_string());
        assert_eq!(graph.metadata().name, Some("Test Graph".to_string()));
        
        // Test node operations
        let n1 = graph.add_node(GenericNode::new("n1", "data1")).unwrap();
        let n2 = graph.add_node(GenericNode::new("n2", "data2")).unwrap();
        
        assert_eq!(graph.node_count(), 2);
        assert!(graph.contains_node(&n1));
        assert!(graph.get_node(&n1).is_some());
        
        // Test edge operations
        let e1 = graph.add_edge(GenericEdge::new(&n1, &n2, ())).unwrap();
        assert_eq!(graph.edge_count(), 1);
        assert!(graph.contains_edge(&e1));
        assert!(graph.get_edge(&e1).is_some());
        
        // Test neighbors
        let neighbors = graph.neighbors(&n1).unwrap();
        assert_eq!(neighbors, vec![n2.clone()]);
        
        // Test edge count
        assert_eq!(graph.edge_count(), 1);
        
        // Test node removal
        graph.remove_node(&n1).unwrap();
        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.edge_count(), 0); // Edge should be removed too
        
        // Test error cases
        assert!(graph.get_node("nonexistent").is_none());
        assert!(graph.remove_node("nonexistent").is_err());
    }
    
    #[test]
    fn test_graph_builder_with_handler() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();
        
        // Create a closure that implements the handler logic
        let handler_fn = move |event: &GraphEvent| {
            events_clone.lock().unwrap().push(event.clone());
        };
        
        // Wrap it in a struct that implements EventHandler
        struct TestHandler<F> {
            handler: F,
        }
        
        impl<F> EventHandler for TestHandler<F>
        where
            F: Fn(&GraphEvent) + Send + Sync,
        {
            fn handle_event(&self, event: &GraphEvent) {
                (self.handler)(event);
            }
        }
        
        let handler = Arc::new(TestHandler { handler: handler_fn });
        
        let graph = GraphBuilder::<GenericNode<&str>, GenericEdge<()>>::new()
            .graph_type(GraphType::WorkflowGraph)
            .name("Test Workflow")
            .description("A test workflow graph")
            .add_handler(handler)
            .build()
            .unwrap();
            
        assert_eq!(graph.graph_type(), GraphType::WorkflowGraph);
        assert_eq!(graph.metadata().name, Some("Test Workflow".to_string()));
        assert_eq!(graph.metadata().description, Some("A test workflow graph".to_string()));
    }
    
    #[test]
    fn test_from_serde_json_error() {
        use crate::error::GraphError;
        let json_err = serde_json::from_str::<String>("invalid json").unwrap_err();
        let graph_err: GraphError = json_err.into();
        assert!(matches!(graph_err, GraphError::SerializationError(_)));
    }
}