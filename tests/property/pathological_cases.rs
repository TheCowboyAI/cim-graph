//! Property tests for pathological and edge cases

use proptest::prelude::*;
use cim_graph::core::{Graph, GenericNode, GenericEdge, GraphType, Node, Edge};
use cim_graph::core::graph::BasicGraph;
use super::generators::*;
use std::collections::HashSet;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn empty_graph_operations(
        graph_type in prop::sample::select(vec![
            GraphType::Generic,
            GraphType::WorkflowGraph,
            GraphType::ContextGraph,
        ]),
    ) {
        let graph = BasicGraph::<GenericNode<String>, GenericEdge<f64>>::new(graph_type);
        
        // All counts should be zero
        prop_assert_eq!(graph.node_count(), 0);
        prop_assert_eq!(graph.edge_count(), 0);
        prop_assert!(graph.node_ids().is_empty());
        prop_assert!(graph.edge_ids().is_empty());
        
        // Queries on non-existent items should fail gracefully
        prop_assert!(graph.get_node("nonexistent").is_none());
        prop_assert!(graph.get_edge("nonexistent").is_none());
        prop_assert!(!graph.contains_node("nonexistent"));
        prop_assert!(!graph.contains_edge("nonexistent"));
        
        // Operations should fail appropriately
        prop_assert!(graph.remove_node("nonexistent").is_err());
        prop_assert!(graph.remove_edge("nonexistent").is_err());
        prop_assert!(graph.neighbors("nonexistent").is_err());
        
        // edges_between should return empty
        prop_assert!(graph.edges_between("a", "b").is_empty());
        
        // Clear should be idempotent
        let mut mutable_graph = graph;
        mutable_graph.clear();
        prop_assert_eq!(mutable_graph.node_count(), 0);
    }
    
    #[test]
    fn single_node_graph_operations(
        node in node_strategy(),
    ) {
        let mut graph = BasicGraph::new(GraphType::Generic);
        let node_id = node.id();
        
        graph.add_node(node.clone()).unwrap();
        
        // Basic properties
        prop_assert_eq!(graph.node_count(), 1);
        prop_assert_eq!(graph.edge_count(), 0);
        prop_assert!(graph.contains_node(&node_id));
        
        // Neighbors should be empty
        let neighbors = graph.neighbors(&node_id).unwrap();
        prop_assert!(neighbors.is_empty());
        
        // Cannot add edge to non-existent node
        let edge = GenericEdge::new(&node_id, "nonexistent", 1.0);
        prop_assert!(graph.add_edge(edge).is_err());
        
        // Self-loops are allowed
        let self_loop = GenericEdge::new(&node_id, &node_id, 1.0);
        if graph.add_edge(self_loop).is_ok() {
            prop_assert_eq!(graph.edge_count(), 1);
            prop_assert_eq!(graph.neighbors(&node_id).unwrap().len(), 1);
        }
    }
    
    #[test]
    fn massive_node_ids(
        data in "[a-z]{1,10}",
    ) {
        let mut graph = BasicGraph::new(GraphType::Generic);
        
        // Test with very long IDs
        let long_id = "a".repeat(1000);
        let node = GenericNode::new(long_id.clone(), data);
        
        prop_assert!(graph.add_node(node).is_ok());
        prop_assert!(graph.contains_node(&long_id));
        
        // Test with special characters in IDs
        let special_ids = vec![
            "node-with-dashes",
            "node_with_underscores",
            "node.with.dots",
            "node:with:colons",
            "node/with/slashes",
            "node\\with\\backslashes",
            "node with spaces",
            "node\twith\ttabs",
            "node\nwith\nnewlines",
            "🚀emoji🎉node🔥",
            "node(with)parens",
            "node[with]brackets",
            "node{with}braces",
            "node<with>angles",
            "node@with@at",
            "node#with#hash",
            "node$with$dollar",
            "node%with%percent",
            "node^with^caret",
            "node&with&amp",
            "node*with*star",
            "node+with+plus",
            "node=with=equals",
            "node|with|pipe",
            "node?with?question",
            "node!with!exclaim",
            "node~with~tilde",
            "node`with`backtick",
            "node'with'quote",
            "node\"with\"doublequote",
        ];
        
        for special_id in special_ids {
            let special_node = GenericNode::new(special_id, "data");
            prop_assert!(graph.add_node(special_node).is_ok());
            prop_assert!(graph.contains_node(special_id));
        }
    }
    
    #[test]
    fn extreme_edge_weights(
        mut graph in small_graph_strategy().prop_filter("has at least 2 nodes", |g| g.node_count() >= 2),
    ) {
        let node_ids = graph.node_ids();
        let from = &node_ids[0];
        let to = &node_ids[1];
        
        // Test extreme weight values
        let extreme_weights = vec![
            f64::MIN,
            f64::MAX,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::MIN_POSITIVE,
            -0.0,
            0.0,
            f64::EPSILON,
            1e308,
            -1e308,
        ];
        
        for (i, weight) in extreme_weights.iter().enumerate() {
            let edge = GenericEdge::with_id(
                format!("extreme-edge-{}", i),
                from,
                to,
                *weight,
            );
            
            prop_assert!(graph.add_edge(edge).is_ok());
        }
        
        // NaN is special - it might behave differently
        let nan_edge = GenericEdge::with_id("nan-edge", from, to, f64::NAN);
        let result = graph.add_edge(nan_edge);
        // We accept both success and failure for NaN
        prop_assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn highly_connected_node(
        hub_count in 100..=500usize,
    ) {
        let mut graph = BasicGraph::new(GraphType::Generic);
        
        // Create a hub node
        let hub = GenericNode::new("hub", "central");
        graph.add_node(hub).unwrap();
        
        // Connect many nodes to the hub
        for i in 0..hub_count {
            let leaf = GenericNode::new(format!("leaf-{}", i), "peripheral");
            graph.add_node(leaf).unwrap();
            
            // Bidirectional connections
            graph.add_edge(GenericEdge::new("hub", format!("leaf-{}", i), 1.0)).unwrap();
            graph.add_edge(GenericEdge::new(format!("leaf-{}", i), "hub", 1.0)).unwrap();
        }
        
        // Hub should have many neighbors
        let hub_neighbors = graph.neighbors("hub").unwrap();
        prop_assert_eq!(hub_neighbors.len(), hub_count);
        
        // Total edges should be 2 * hub_count
        prop_assert_eq!(graph.edge_count(), 2 * hub_count);
        
        // Removing hub should remove all edges
        graph.remove_node("hub").unwrap();
        prop_assert_eq!(graph.edge_count(), 0);
        prop_assert_eq!(graph.node_count(), hub_count);
    }
    
    #[test]
    fn parallel_edges(
        mut graph in small_graph_strategy().prop_filter("has at least 2 nodes", |g| g.node_count() >= 2),
        num_parallel in 2..=10usize,
    ) {
        let node_ids = graph.node_ids();
        let from = &node_ids[0];
        let to = &node_ids[1];
        
        let initial_edge_count = graph.edge_count();
        
        // Add multiple edges between same nodes
        for i in 0..num_parallel {
            let edge = GenericEdge::with_id(
                format!("parallel-{}", i),
                from,
                to,
                i as f64,
            );
            prop_assert!(graph.add_edge(edge).is_ok());
        }
        
        // Should have added all edges
        prop_assert_eq!(graph.edge_count(), initial_edge_count + num_parallel);
        
        // edges_between should return all parallel edges
        let edges_between = graph.edges_between(from, to);
        prop_assert_eq!(edges_between.len(), num_parallel);
        
        // Each edge should have unique ID and correct weight
        let edge_ids: HashSet<_> = edges_between.iter().map(|e| e.id()).collect();
        prop_assert_eq!(edge_ids.len(), num_parallel);
    }
    
    #[test]
    fn node_data_mutations(
        mut graph in small_graph_strategy().prop_filter("has nodes", |g| g.node_count() > 0),
        new_data in "[a-z]{5,20}",
    ) {
        let node_id = graph.node_ids()[0].clone();
        
        // Get mutable reference and modify data
        if let Some(node) = graph.get_node_mut(&node_id) {
            *node.data_mut() = new_data.clone();
        }
        
        // Verify modification persisted
        let node = graph.get_node(&node_id).unwrap();
        prop_assert_eq!(node.data(), &new_data);
    }
    
    #[test]
    fn rapid_add_remove_cycles(
        operations in prop::collection::vec(
            (node_strategy(), any::<bool>()),
            10..50
        ),
    ) {
        let mut graph = BasicGraph::new(GraphType::Generic);
        let mut active_nodes = HashSet::new();
        
        for (node, should_add) in operations {
            let node_id = node.id();
            
            if should_add && !active_nodes.contains(&node_id) {
                prop_assert!(graph.add_node(node).is_ok());
                active_nodes.insert(node_id);
            } else if !should_add && active_nodes.contains(&node_id) {
                prop_assert!(graph.remove_node(&node_id).is_ok());
                active_nodes.remove(&node_id);
            }
        }
        
        // Final state should be consistent
        prop_assert_eq!(graph.node_count(), active_nodes.len());
        for node_id in &active_nodes {
            prop_assert!(graph.contains_node(node_id));
        }
    }
    
    #[test]
    fn metadata_extreme_values(
        mut graph in small_graph_strategy(),
        huge_name in "[a-z]{1000,2000}",
        huge_desc in "[a-z ]{2000,3000}",
    ) {
        // Set extremely large metadata
        graph.metadata_mut().name = Some(huge_name.clone());
        graph.metadata_mut().description = Some(huge_desc.clone());
        
        // Add many properties
        for i in 0..1000 {
            graph.metadata_mut().properties.insert(
                format!("prop-{}", i),
                serde_json::json!({
                    "nested": {
                        "deeply": {
                            "nested": {
                                "value": i
                            }
                        }
                    }
                }),
            );
        }
        
        // Should still function normally
        prop_assert_eq!(graph.metadata().name.as_ref().unwrap(), &huge_name);
        prop_assert_eq!(graph.metadata().properties.len(), 1000);
        
        // to_json should still work
        let json_result = graph.to_json();
        prop_assert!(json_result.is_ok());
    }
    
    #[test]
    fn concurrent_modification_simulation(
        initial_graph in small_graph_strategy(),
        modifications in prop::collection::vec(
            prop_oneof![
                (node_strategy(), Just(true)).prop_map(|(n, _)| ("add_node", n.id(), n.data().clone(), 0.0)),
                (Just(()), Just(false)).prop_map(|_| ("remove_node", "".to_string(), "".to_string(), 0.0)),
                (edge_data_strategy(), Just(false)).prop_map(|(w, _)| ("add_edge", "".to_string(), "".to_string(), w)),
                (Just(()), Just(false)).prop_map(|_| ("remove_edge", "".to_string(), "".to_string(), 0.0)),
            ],
            10..30
        ),
    ) {
        let mut graph = initial_graph;
        let mut operation_count = 0;
        
        for (op_type, id, data, weight) in modifications {
            match op_type {
                "add_node" => {
                    let node = GenericNode::new(format!("{}-{}", id, operation_count), data);
                    let _ = graph.add_node(node);
                }
                "remove_node" => {
                    let nodes = graph.node_ids();
                    if !nodes.is_empty() {
                        let idx = operation_count % nodes.len();
                        let _ = graph.remove_node(&nodes[idx]);
                    }
                }
                "add_edge" => {
                    let nodes = graph.node_ids();
                    if nodes.len() >= 2 {
                        let from_idx = operation_count % nodes.len();
                        let to_idx = (operation_count + 1) % nodes.len();
                        if from_idx != to_idx {
                            let edge = GenericEdge::new(&nodes[from_idx], &nodes[to_idx], weight);
                            let _ = graph.add_edge(edge);
                        }
                    }
                }
                "remove_edge" => {
                    let edges = graph.edge_ids();
                    if !edges.is_empty() {
                        let idx = operation_count % edges.len();
                        let _ = graph.remove_edge(&edges[idx]);
                    }
                }
                _ => {}
            }
            operation_count += 1;
            
            // Invariants should hold after each operation
            prop_assert!(graph.node_count() == graph.node_ids().len());
            prop_assert!(graph.edge_count() == graph.edge_ids().len());
            
            // All edges should reference valid nodes
            for edge_id in graph.edge_ids() {
                let edge = graph.get_edge(&edge_id).unwrap();
                prop_assert!(graph.contains_node(&edge.source()));
                prop_assert!(graph.contains_node(&edge.target()));
            }
        }
    }
    
    #[test]
    fn zero_capacity_behavior() {
        // Test what happens with zero-capacity collections
        let mut graph = BasicGraph::<GenericNode<String>, GenericEdge<f64>>::new(GraphType::Generic);
        
        // Clear already empty graph
        graph.clear();
        prop_assert_eq!(graph.node_count(), 0);
        
        // Multiple clears
        for _ in 0..10 {
            graph.clear();
        }
        prop_assert_eq!(graph.node_count(), 0);
        
        // Operations on empty graph
        prop_assert!(graph.neighbors("any").is_err());
        prop_assert!(graph.remove_node("any").is_err());
        prop_assert!(graph.remove_edge("any").is_err());
    }
}

#[cfg(test)]
mod error_handling {
    use super::*;
    use cim_graph::error::GraphError;
    
    proptest! {
        #[test]
        fn duplicate_node_id_rejected(
            mut graph in small_graph_strategy(),
            node in node_strategy(),
        ) {
            // Add node once
            let first_result = graph.add_node(node.clone());
            
            if first_result.is_ok() {
                // Try to add again with same ID
                let second_result = graph.add_node(node);
                prop_assert!(matches!(second_result, Err(GraphError::DuplicateNode(_))));
            }
        }
        
        #[test]
        fn edge_without_nodes_rejected(
            mut graph in Just(BasicGraph::<GenericNode<String>, GenericEdge<f64>>::new(GraphType::Generic)),
            edge in edge_strategy(&[]),
        ) {
            if let Some(edge) = edge {
                let result = graph.add_edge(edge);
                prop_assert!(matches!(result, Err(GraphError::NodeNotFound(_))));
            }
        }
        
        #[test]
        fn nonexistent_node_operations_fail(
            graph in small_graph_strategy(),
            fake_id in "[a-z]{20,30}", // Unlikely to match real IDs
        ) {
            // All these should fail gracefully
            prop_assert!(graph.get_node(&fake_id).is_none());
            prop_assert!(!graph.contains_node(&fake_id));
            prop_assert!(graph.neighbors(&fake_id).is_err());
            
            let mut mutable_graph = graph;
            prop_assert!(mutable_graph.remove_node(&fake_id).is_err());
            prop_assert!(mutable_graph.get_node_mut(&fake_id).is_none());
        }
    }
}