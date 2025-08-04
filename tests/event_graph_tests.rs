//! Tests for the event-driven graph implementation

use cim_graph::core::{GenericEdge, GenericNode, GraphEvent, MemoryEventHandler};
use cim_graph::{Edge, GraphBuilder, Node};
use std::sync::Arc;

type TestNode = GenericNode<String>;
type TestEdge = GenericEdge<i32>;

#[test]
fn test_event_graph_creation() {
    let handler = Arc::new(MemoryEventHandler::new());
    let graph = GraphBuilder::<TestNode, TestEdge>::new()
        .name("Test Event Graph")
        .add_handler(handler.clone())
        .build_event()
        .expect("Failed to create event graph");
    
    assert_eq!(graph.node_count(), 0);
    assert_eq!(graph.edge_count(), 0);
    assert_eq!(graph.metadata().name, Some("Test Event Graph".to_string()));
}

#[test]
fn test_node_events() {
    let handler = Arc::new(MemoryEventHandler::new());
    let mut graph = GraphBuilder::<TestNode, TestEdge>::new()
        .add_handler(handler.clone())
        .build_event()
        .expect("Failed to create event graph");
    
    // Add a node
    let node = TestNode::new("n1", "Node 1".to_string());
    let node_id = graph.add_node(node).expect("Failed to add node");
    
    // Check event was emitted
    let events = handler.events();
    assert_eq!(events.len(), 1);
    
    match &events[0] {
        GraphEvent::NodeAdded { graph_id: _, node_id: event_node_id } => {
            assert_eq!(event_node_id, &node_id);
        }
        _ => panic!("Expected NodeAdded event"),
    }
    
    // Remove the node
    handler.clear();
    graph.remove_node("n1").expect("Failed to remove node");
    
    let events = handler.events();
    assert_eq!(events.len(), 1);
    
    match &events[0] {
        GraphEvent::NodeRemoved { graph_id: _, node_id: event_node_id } => {
            assert_eq!(event_node_id, "n1");
        }
        _ => panic!("Expected NodeRemoved event"),
    }
}

#[test]
fn test_edge_events() {
    let handler = Arc::new(MemoryEventHandler::new());
    let mut graph = GraphBuilder::<TestNode, TestEdge>::new()
        .add_handler(handler.clone())
        .build_event()
        .expect("Failed to create event graph");
    
    // Add nodes
    graph.add_node(TestNode::new("n1", "Node 1".to_string())).unwrap();
    graph.add_node(TestNode::new("n2", "Node 2".to_string())).unwrap();
    
    // Clear events from node additions
    handler.clear();
    
    // Add an edge
    let edge = TestEdge::with_id("e1", "n1", "n2", 10);
    let edge_id = graph.add_edge(edge).expect("Failed to add edge");
    
    // Check event was emitted
    let events = handler.events();
    assert_eq!(events.len(), 1);
    
    match &events[0] {
        GraphEvent::EdgeAdded { graph_id: _, edge_id: event_edge_id, source, target } => {
            assert_eq!(event_edge_id, &edge_id);
            assert_eq!(source, "n1");
            assert_eq!(target, "n2");
        }
        _ => panic!("Expected EdgeAdded event"),
    }
    
    // Remove the edge
    handler.clear();
    graph.remove_edge("e1").expect("Failed to remove edge");
    
    let events = handler.events();
    assert_eq!(events.len(), 1);
    
    match &events[0] {
        GraphEvent::EdgeRemoved { graph_id: _, edge_id: event_edge_id } => {
            assert_eq!(event_edge_id, "e1");
        }
        _ => panic!("Expected EdgeRemoved event"),
    }
}

#[test]
fn test_removing_node_emits_edge_events() {
    let handler = Arc::new(MemoryEventHandler::new());
    let mut graph = GraphBuilder::<TestNode, TestEdge>::new()
        .add_handler(handler.clone())
        .build_event()
        .expect("Failed to create event graph");
    
    // Add nodes
    graph.add_node(TestNode::new("n1", "Node 1".to_string())).unwrap();
    graph.add_node(TestNode::new("n2", "Node 2".to_string())).unwrap();
    graph.add_node(TestNode::new("n3", "Node 3".to_string())).unwrap();
    
    // Add edges
    graph.add_edge(TestEdge::with_id("e1", "n1", "n2", 10)).unwrap();
    graph.add_edge(TestEdge::with_id("e2", "n2", "n3", 20)).unwrap();
    graph.add_edge(TestEdge::with_id("e3", "n3", "n2", 30)).unwrap();
    
    // Clear events
    handler.clear();
    
    // Remove node n2 (should emit events for edges e1, e2, and e3)
    graph.remove_node("n2").expect("Failed to remove node");
    
    let events = handler.events();
    
    // Should have 3 EdgeRemoved events and 1 NodeRemoved event
    let edge_removed_count = events.iter().filter(|e| matches!(e, GraphEvent::EdgeRemoved { .. })).count();
    let node_removed_count = events.iter().filter(|e| matches!(e, GraphEvent::NodeRemoved { .. })).count();
    
    assert_eq!(edge_removed_count, 3);
    assert_eq!(node_removed_count, 1);
}

#[test]
fn test_graph_queries() {
    let mut graph = GraphBuilder::<TestNode, TestEdge>::new()
        .build_event()
        .expect("Failed to create event graph");
    
    // Add nodes
    graph.add_node(TestNode::new("n1", "Node 1".to_string())).unwrap();
    graph.add_node(TestNode::new("n2", "Node 2".to_string())).unwrap();
    graph.add_node(TestNode::new("n3", "Node 3".to_string())).unwrap();
    
    // Add edges
    graph.add_edge(TestEdge::with_id("e1", "n1", "n2", 10)).unwrap();
    graph.add_edge(TestEdge::with_id("e2", "n1", "n3", 20)).unwrap();
    graph.add_edge(TestEdge::with_id("e3", "n2", "n3", 30)).unwrap();
    
    // Test neighbors
    let neighbors = graph.neighbors("n1").expect("Failed to get neighbors");
    assert_eq!(neighbors.len(), 2);
    assert!(neighbors.contains(&"n2".to_string()));
    assert!(neighbors.contains(&"n3".to_string()));
    
    // Test predecessors
    let predecessors = graph.predecessors("n3").expect("Failed to get predecessors");
    assert_eq!(predecessors.len(), 2);
    assert!(predecessors.contains(&"n1".to_string()));
    assert!(predecessors.contains(&"n2".to_string()));
    
    // Test degree
    let degree = graph.degree("n1").expect("Failed to get degree");
    assert_eq!(degree, 2);
    
    // Test edges between nodes
    let edges = graph.edges_between("n1", "n2");
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].id(), "e1");
}

#[test]
fn test_petgraph_access() {
    let mut graph = GraphBuilder::<TestNode, TestEdge>::new()
        .build_event()
        .expect("Failed to create event graph");
    
    // Add some nodes
    graph.add_node(TestNode::new("n1", "Node 1".to_string())).unwrap();
    graph.add_node(TestNode::new("n2", "Node 2".to_string())).unwrap();
    
    // Access the underlying petgraph
    let petgraph = graph.petgraph();
    assert_eq!(petgraph.node_count(), 2);
    assert_eq!(petgraph.edge_count(), 0);
    
    // Add an edge
    graph.add_edge(TestEdge::with_id("e1", "n1", "n2", 10)).unwrap();
    
    // Check petgraph was updated
    let petgraph = graph.petgraph();
    assert_eq!(petgraph.edge_count(), 1);
}