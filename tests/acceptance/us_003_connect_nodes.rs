//! Acceptance tests for US-003: Connect Nodes with Edges

use cim_graph::core::{GenericEdge, GenericNode};
use cim_graph::{Edge, GraphBuilder, GraphError};

type TestNode = GenericNode<String>;
type TestEdge = GenericEdge<i32>;

#[test]
fn test_ac_003_1_add_edge_between_nodes() {
    // Given: a graph with two nodes
    let mut graph = GraphBuilder::<TestNode, TestEdge>::new()
        .build_event()
        .expect("Failed to create graph");

    let node1 = TestNode::new("n1", "Node 1".to_string());
    let node2 = TestNode::new("n2", "Node 2".to_string());
    graph.add_node(node1).expect("Failed to add node1");
    graph.add_node(node2).expect("Failed to add node2");

    // When: I add an edge between them
    let edge = TestEdge::with_id("e1", "n1", "n2", 10);
    let edge_id = graph.add_edge(edge).expect("Failed to add edge");

    // Then: the edge is stored with unique ID
    assert_eq!(edge_id, "e1");
    assert_eq!(graph.edge_count(), 1);
    assert!(graph.contains_edge("e1"));

    // And: I can retrieve the edge
    let retrieved = graph.get_edge("e1").expect("Edge should exist");
    assert_eq!(retrieved.id(), "e1");
    assert_eq!(retrieved.source(), "n1");
    assert_eq!(retrieved.target(), "n2");
    assert_eq!(retrieved.data(), &10);
}

#[test]
fn test_ac_003_2_add_multiple_edges_same_nodes() {
    // Given: a graph with two nodes
    let mut graph = GraphBuilder::<TestNode, TestEdge>::new()
        .build_event()
        .expect("Failed to create graph");

    graph.add_node(TestNode::new("n1", "Node 1".to_string())).unwrap();
    graph.add_node(TestNode::new("n2", "Node 2".to_string())).unwrap();

    // When: I add multiple edges between the same nodes
    let edge1 = TestEdge::with_id("e1", "n1", "n2", 10);
    let edge2 = TestEdge::with_id("e2", "n1", "n2", 20);
    let edge3 = TestEdge::with_id("e3", "n2", "n1", 30); // Reverse direction

    graph.add_edge(edge1).expect("Failed to add edge1");
    graph.add_edge(edge2).expect("Failed to add edge2");
    graph.add_edge(edge3).expect("Failed to add edge3");

    // Then: all edges are maintained independently
    assert_eq!(graph.edge_count(), 3);
    
    // And: I can query edges by direction
    let forward_edges = graph.edges_between("n1", "n2");
    assert_eq!(forward_edges.len(), 2);
    
    let reverse_edges = graph.edges_between("n2", "n1");
    assert_eq!(reverse_edges.len(), 1);
}

#[test]
fn test_ac_003_3_add_edge_missing_node_fails() {
    // Given: a graph with only one node
    let mut graph = GraphBuilder::<TestNode, TestEdge>::new()
        .build_event()
        .expect("Failed to create graph");

    graph.add_node(TestNode::new("n1", "Node 1".to_string())).unwrap();

    // When: I add an edge referencing a non-existent node
    let edge = TestEdge::with_id("e1", "n1", "n2", 10);
    let result = graph.add_edge(edge);

    // Then: the operation fails with appropriate error
    assert!(matches!(result, Err(GraphError::NodeNotFound(_))));
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn test_ac_003_4_remove_edge() {
    // Given: a graph with connected nodes
    let mut graph = GraphBuilder::<TestNode, TestEdge>::new()
        .build_event()
        .expect("Failed to create graph");

    graph.add_node(TestNode::new("n1", "Node 1".to_string())).unwrap();
    graph.add_node(TestNode::new("n2", "Node 2".to_string())).unwrap();
    
    let edge = TestEdge::with_id("e1", "n1", "n2", 10);
    graph.add_edge(edge).expect("Failed to add edge");
    assert_eq!(graph.edge_count(), 1);

    // When: I remove the edge
    let removed = graph.remove_edge("e1").expect("Failed to remove edge");

    // Then: the edge is removed
    assert_eq!(removed.id(), "e1");
    assert_eq!(graph.edge_count(), 0);
    assert!(!graph.contains_edge("e1"));
}

#[test]
fn test_ac_003_5_duplicate_edge_id_fails() {
    // Given: a graph with an edge
    let mut graph = GraphBuilder::<TestNode, TestEdge>::new()
        .build_event()
        .expect("Failed to create graph");

    graph.add_node(TestNode::new("n1", "Node 1".to_string())).unwrap();
    graph.add_node(TestNode::new("n2", "Node 2".to_string())).unwrap();
    graph.add_node(TestNode::new("n3", "Node 3".to_string())).unwrap();
    
    let edge1 = TestEdge::with_id("duplicate_id", "n1", "n2", 10);
    graph.add_edge(edge1).expect("Failed to add first edge");

    // When: I add another edge with the same ID
    let edge2 = TestEdge::with_id("duplicate_id", "n2", "n3", 20);
    let result = graph.add_edge(edge2);

    // Then: the operation fails
    assert!(matches!(result, Err(GraphError::DuplicateEdge { .. })));
    assert_eq!(graph.edge_count(), 1);
}

#[test]
fn test_ac_003_6_remove_node_removes_edges() {
    // Given: a graph with connected nodes
    let mut graph = GraphBuilder::<TestNode, TestEdge>::new()
        .build_event()
        .expect("Failed to create graph");

    graph.add_node(TestNode::new("n1", "Node 1".to_string())).unwrap();
    graph.add_node(TestNode::new("n2", "Node 2".to_string())).unwrap();
    graph.add_node(TestNode::new("n3", "Node 3".to_string())).unwrap();
    
    // Add edges: n1 -> n2, n2 -> n3, n3 -> n1
    graph.add_edge(TestEdge::with_id("e1", "n1", "n2", 10)).unwrap();
    graph.add_edge(TestEdge::with_id("e2", "n2", "n3", 20)).unwrap();
    graph.add_edge(TestEdge::with_id("e3", "n3", "n1", 30)).unwrap();
    assert_eq!(graph.edge_count(), 3);

    // When: I remove a node
    graph.remove_node("n2").expect("Failed to remove node");

    // Then: all edges connected to that node are removed
    assert_eq!(graph.edge_count(), 1); // Only e3 (n3 -> n1) remains
    assert!(!graph.contains_edge("e1"));
    assert!(!graph.contains_edge("e2"));
    assert!(graph.contains_edge("e3"));
}

#[test]
fn test_ac_003_7_self_loops_allowed() {
    // Given: a graph with a node
    let mut graph = GraphBuilder::<TestNode, TestEdge>::new()
        .build_event()
        .expect("Failed to create graph");

    graph.add_node(TestNode::new("n1", "Node 1".to_string())).unwrap();

    // When: I add a self-loop edge
    let edge = TestEdge::with_id("self_loop", "n1", "n1", 42);
    let edge_id = graph.add_edge(edge).expect("Failed to add self-loop");

    // Then: the self-loop is allowed
    assert_eq!(edge_id, "self_loop");
    assert_eq!(graph.edge_count(), 1);
    
    let retrieved = graph.get_edge("self_loop").unwrap();
    assert_eq!(retrieved.source(), "n1");
    assert_eq!(retrieved.target(), "n1");
}