//! Acceptance tests for US-002: Add Nodes to Graph

use cim_graph::core::{GenericEdge, GenericNode};
use cim_graph::{Graph, GraphBuilder, GraphError, Node};

type TestNode = GenericNode<String>;
type TestEdge = GenericEdge<()>;

#[test]
fn test_ac_002_1_add_node_to_empty_graph() {
    // Given: an empty graph
    let mut graph = GraphBuilder::<TestNode, TestEdge>::new()
        .build()
        .expect("Failed to create graph");

    assert_eq!(graph.node_count(), 0);

    // When: I add a node
    let node = TestNode::new("node1", "test_data".to_string());
    let node_id = graph.add_node(node).expect("Failed to add node");

    // Then: the node is stored with a unique ID
    assert_eq!(node_id, "node1");
    assert_eq!(graph.node_count(), 1);
    assert!(graph.contains_node("node1"));

    // And: I can retrieve the node
    let retrieved = graph.get_node("node1").expect("Node should exist");
    assert_eq!(retrieved.id(), "node1");
    assert_eq!(retrieved.data(), &"test_data".to_string());
}

#[test]
fn test_ac_002_2_add_duplicate_node_fails() {
    // Given: a graph with a node
    let mut graph = GraphBuilder::<TestNode, TestEdge>::new()
        .build()
        .expect("Failed to create graph");

    let node1 = TestNode::new("duplicate_id", "first".to_string());
    graph.add_node(node1).expect("Failed to add first node");

    // When: I add a node with duplicate ID
    let node2 = TestNode::new("duplicate_id", "second".to_string());
    let result = graph.add_node(node2);

    // Then: the operation fails with appropriate error
    assert!(matches!(result, Err(GraphError::DuplicateNode(_))));
    assert_eq!(graph.node_count(), 1); // Still only one node
}

#[test]
fn test_add_multiple_nodes() {
    // Test adding multiple nodes
    let mut graph = GraphBuilder::<TestNode, TestEdge>::new()
        .build()
        .expect("Failed to create graph");

    // Add several nodes
    for i in 0..5 {
        let node = TestNode::new(format!("node{}", i), format!("data{}", i));
        graph.add_node(node).expect("Failed to add node");
    }

    // Verify all nodes were added
    assert_eq!(graph.node_count(), 5);
    for i in 0..5 {
        assert!(graph.contains_node(&format!("node{}", i)));
    }

    // Verify node IDs are returned correctly
    let node_ids = graph.node_ids();
    assert_eq!(node_ids.len(), 5);
}

#[test]
fn test_remove_node() {
    // Test node removal
    let mut graph = GraphBuilder::<TestNode, TestEdge>::new()
        .build()
        .expect("Failed to create graph");

    // Add a node
    let node = TestNode::new("to_remove", "temporary".to_string());
    graph.add_node(node).expect("Failed to add node");
    assert_eq!(graph.node_count(), 1);

    // Remove the node
    let removed = graph
        .remove_node("to_remove")
        .expect("Failed to remove node");
    assert_eq!(removed.id(), "to_remove");
    assert_eq!(removed.data(), &"temporary".to_string());

    // Verify node is gone
    assert_eq!(graph.node_count(), 0);
    assert!(!graph.contains_node("to_remove"));
}

#[test]
fn test_remove_nonexistent_node_fails() {
    // Test removing a node that doesn't exist
    let mut graph = GraphBuilder::<TestNode, TestEdge>::new()
        .build()
        .expect("Failed to create graph");

    let result = graph.remove_node("nonexistent");
    assert!(matches!(result, Err(GraphError::NodeNotFound(_))));
}

#[test]
fn test_get_node_operations() {
    // Test getting nodes
    let mut graph = GraphBuilder::<TestNode, TestEdge>::new()
        .build()
        .expect("Failed to create graph");

    // Add a node
    let node = TestNode::new("test_node", "original_data".to_string());
    graph.add_node(node).expect("Failed to add node");

    // Test immutable get
    let node_ref = graph.get_node("test_node").expect("Node should exist");
    assert_eq!(node_ref.data(), &"original_data".to_string());

    // Test mutable get
    let node_mut = graph.get_node_mut("test_node").expect("Node should exist");
    *node_mut.data_mut() = "modified_data".to_string();

    // Verify modification
    let node_ref = graph.get_node("test_node").expect("Node should exist");
    assert_eq!(node_ref.data(), &"modified_data".to_string());
}

#[test]
fn test_graph_clear() {
    // Test clearing all nodes
    let mut graph = GraphBuilder::<TestNode, TestEdge>::new()
        .build()
        .expect("Failed to create graph");

    // Add multiple nodes
    for i in 0..10 {
        let node = TestNode::new(format!("node{}", i), format!("data{}", i));
        graph.add_node(node).expect("Failed to add node");
    }
    assert_eq!(graph.node_count(), 10);

    // Clear the graph
    graph.clear();

    // Verify graph is empty
    assert_eq!(graph.node_count(), 0);
    assert_eq!(graph.node_ids().len(), 0);
}
