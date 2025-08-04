//! Acceptance tests for US-004: Query Graph Structure

use cim_graph::core::{GenericEdge, GenericNode};
use cim_graph::GraphBuilder;

type TestNode = GenericNode<String>;
type TestEdge = GenericEdge<i32>;

#[test]
fn test_ac_004_1_neighbors_directed() {
    // Given: a directed graph with connected nodes
    let mut graph = GraphBuilder::<TestNode, TestEdge>::new()
        .build_event()
        .expect("Failed to create graph");

    // Create a simple directed graph: n1 -> n2 -> n3
    //                                      \-> n4
    graph.add_node(TestNode::new("n1", "Node 1".to_string())).unwrap();
    graph.add_node(TestNode::new("n2", "Node 2".to_string())).unwrap();
    graph.add_node(TestNode::new("n3", "Node 3".to_string())).unwrap();
    graph.add_node(TestNode::new("n4", "Node 4".to_string())).unwrap();

    graph.add_edge(TestEdge::with_id("e1", "n1", "n2", 10)).unwrap();
    graph.add_edge(TestEdge::with_id("e2", "n2", "n3", 20)).unwrap();
    graph.add_edge(TestEdge::with_id("e3", "n2", "n4", 30)).unwrap();

    // When: I query neighbors
    let n1_neighbors = graph.neighbors("n1").expect("Failed to get neighbors");
    let n2_neighbors = graph.neighbors("n2").expect("Failed to get neighbors");
    let n3_neighbors = graph.neighbors("n3").expect("Failed to get neighbors");

    // Then: outgoing neighbors are returned
    assert_eq!(n1_neighbors, vec!["n2"]);
    assert_eq!(n2_neighbors.len(), 2);
    assert!(n2_neighbors.contains(&"n3".to_string()));
    assert!(n2_neighbors.contains(&"n4".to_string()));
    assert_eq!(n3_neighbors.len(), 0); // n3 has no outgoing edges
}

#[test]
fn test_ac_004_2_path_query() {
    // Given: a graph with nodes and edges
    let mut graph = GraphBuilder::<TestNode, TestEdge>::new()
        .build_event()
        .expect("Failed to create graph");

    // Create a diamond-shaped graph:
    //     n1
    //    /  \
    //   n2  n3
    //    \  /
    //     n4
    graph.add_node(TestNode::new("n1", "Start".to_string())).unwrap();
    graph.add_node(TestNode::new("n2", "Left".to_string())).unwrap();
    graph.add_node(TestNode::new("n3", "Right".to_string())).unwrap();
    graph.add_node(TestNode::new("n4", "End".to_string())).unwrap();

    graph.add_edge(TestEdge::with_id("e1", "n1", "n2", 10)).unwrap();
    graph.add_edge(TestEdge::with_id("e2", "n1", "n3", 20)).unwrap();
    graph.add_edge(TestEdge::with_id("e3", "n2", "n4", 30)).unwrap();
    graph.add_edge(TestEdge::with_id("e4", "n3", "n4", 40)).unwrap();

    // When: I query for paths
    // Note: This is testing basic connectivity via neighbors
    let n1_reachable = graph.neighbors("n1").unwrap();
    let n2_reachable = graph.neighbors("n2").unwrap();
    let n3_reachable = graph.neighbors("n3").unwrap();

    // Then: paths are discoverable
    assert_eq!(n1_reachable.len(), 2); // Can reach n2 and n3
    assert_eq!(n2_reachable, vec!["n4"]);
    assert_eq!(n3_reachable, vec!["n4"]);
}

#[test]
fn test_query_node_degree() {
    // Given: a graph with various node connections
    let mut graph = GraphBuilder::<TestNode, TestEdge>::new()
        .build_event()
        .expect("Failed to create graph");

    // Create a hub-and-spoke pattern
    graph.add_node(TestNode::new("hub", "Central Hub".to_string())).unwrap();
    graph.add_node(TestNode::new("s1", "Spoke 1".to_string())).unwrap();
    graph.add_node(TestNode::new("s2", "Spoke 2".to_string())).unwrap();
    graph.add_node(TestNode::new("s3", "Spoke 3".to_string())).unwrap();

    // Hub has edges to all spokes
    graph.add_edge(TestEdge::with_id("e1", "hub", "s1", 10)).unwrap();
    graph.add_edge(TestEdge::with_id("e2", "hub", "s2", 20)).unwrap();
    graph.add_edge(TestEdge::with_id("e3", "hub", "s3", 30)).unwrap();
    
    // Some spokes connect back
    graph.add_edge(TestEdge::with_id("e4", "s1", "hub", 40)).unwrap();
    graph.add_edge(TestEdge::with_id("e5", "s2", "hub", 50)).unwrap();

    // When: I query node degrees (out-degree)
    let hub_degree = graph.degree("hub").expect("Failed to get degree");
    let s1_degree = graph.degree("s1").expect("Failed to get degree");
    let s3_degree = graph.degree("s3").expect("Failed to get degree");

    // Then: out-degrees are correct
    assert_eq!(hub_degree, 3); // 3 outgoing edges
    assert_eq!(s1_degree, 1); // 1 outgoing edge
    assert_eq!(s3_degree, 0); // 0 outgoing edges
}

#[test]
fn test_query_graph_statistics() {
    // Given: a populated graph
    let mut graph = GraphBuilder::<TestNode, TestEdge>::new()
        .name("Test Statistics Graph")
        .build_event()
        .expect("Failed to create graph");

    // Add nodes
    for i in 1..=10 {
        graph.add_node(TestNode::new(format!("n{}", i), format!("Node {}", i))).unwrap();
    }

    // Add edges in a ring pattern
    for i in 1..10 {
        graph.add_edge(TestEdge::with_id(
            format!("e{}", i),
            format!("n{}", i),
            format!("n{}", i + 1),
            i as i32,
        )).unwrap();
    }
    // Complete the ring
    graph.add_edge(TestEdge::with_id("e10", "n10", "n1", 10)).unwrap();

    // When: I query graph statistics
    let node_count = graph.node_count();
    let edge_count = graph.edge_count();
    let node_ids = graph.node_ids();
    let edge_ids = graph.edge_ids();

    // Then: statistics are accurate
    assert_eq!(node_count, 10);
    assert_eq!(edge_count, 10);
    assert_eq!(node_ids.len(), 10);
    assert_eq!(edge_ids.len(), 10);
    
    // Verify all nodes exist
    for i in 1..=10 {
        assert!(graph.contains_node(&format!("n{}", i)));
    }
}

#[test]
fn test_query_edges_between_nodes() {
    // Given: a graph with multiple edges between same nodes
    let mut graph = GraphBuilder::<TestNode, TestEdge>::new()
        .build_event()
        .expect("Failed to create graph");

    graph.add_node(TestNode::new("a", "Node A".to_string())).unwrap();
    graph.add_node(TestNode::new("b", "Node B".to_string())).unwrap();

    // Add multiple edges from a to b
    graph.add_edge(TestEdge::with_id("e1", "a", "b", 10)).unwrap();
    graph.add_edge(TestEdge::with_id("e2", "a", "b", 20)).unwrap();
    graph.add_edge(TestEdge::with_id("e3", "a", "b", 30)).unwrap();
    
    // Add edge from b to a
    graph.add_edge(TestEdge::with_id("e4", "b", "a", 40)).unwrap();

    // When: I query edges between nodes
    let edges_a_to_b = graph.edges_between("a", "b");
    let edges_b_to_a = graph.edges_between("b", "a");

    // Then: all edges in the specified direction are returned
    assert_eq!(edges_a_to_b.len(), 3);
    assert_eq!(edges_b_to_a.len(), 1);
    
    // Verify edge data
    let weights: Vec<i32> = edges_a_to_b.iter().map(|e| *e.data()).collect();
    assert!(weights.contains(&10));
    assert!(weights.contains(&20));
    assert!(weights.contains(&30));
}

#[test]
fn test_query_predecessors() {
    // Given: a directed graph
    let mut graph = GraphBuilder::<TestNode, TestEdge>::new()
        .build_event()
        .expect("Failed to create graph");

    // Create graph: n1 -> n2 <- n3
    //                     |
    //                     v
    //                     n4
    graph.add_node(TestNode::new("n1", "Node 1".to_string())).unwrap();
    graph.add_node(TestNode::new("n2", "Node 2".to_string())).unwrap();
    graph.add_node(TestNode::new("n3", "Node 3".to_string())).unwrap();
    graph.add_node(TestNode::new("n4", "Node 4".to_string())).unwrap();

    graph.add_edge(TestEdge::with_id("e1", "n1", "n2", 10)).unwrap();
    graph.add_edge(TestEdge::with_id("e2", "n3", "n2", 20)).unwrap();
    graph.add_edge(TestEdge::with_id("e3", "n2", "n4", 30)).unwrap();

    // When: I query predecessors
    let n2_predecessors = graph.predecessors("n2").expect("Failed to get predecessors");
    let n4_predecessors = graph.predecessors("n4").expect("Failed to get predecessors");
    let n1_predecessors = graph.predecessors("n1").expect("Failed to get predecessors");

    // Then: incoming neighbors are returned
    assert_eq!(n2_predecessors.len(), 2);
    assert!(n2_predecessors.contains(&"n1".to_string()));
    assert!(n2_predecessors.contains(&"n3".to_string()));
    assert_eq!(n4_predecessors, vec!["n2"]);
    assert_eq!(n1_predecessors.len(), 0); // n1 has no incoming edges
}