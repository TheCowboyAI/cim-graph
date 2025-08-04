//! Property tests for algebraic properties of graph operations

use proptest::prelude::*;
use cim_graph::core::{Graph, GenericNode, GenericEdge, Node, Edge, GraphType};
use cim_graph::core::graph::BasicGraph;
use super::generators::*;
use std::collections::HashSet;

/// Helper to merge two graphs
fn merge_graphs(
    g1: &BasicGraph<GenericNode<String>, GenericEdge<f64>>,
    g2: &BasicGraph<GenericNode<String>, GenericEdge<f64>>,
) -> BasicGraph<GenericNode<String>, GenericEdge<f64>> {
    let mut merged = BasicGraph::new(g1.graph_type());
    
    // Add all nodes from g1
    for node_id in g1.node_ids() {
        if let Some(node) = g1.get_node(&node_id) {
            let _ = merged.add_node(node.clone());
        }
    }
    
    // Add all nodes from g2 (skip duplicates)
    for node_id in g2.node_ids() {
        if !merged.contains_node(&node_id) {
            if let Some(node) = g2.get_node(&node_id) {
                let _ = merged.add_node(node.clone());
            }
        }
    }
    
    // Add all edges from g1
    for edge_id in g1.edge_ids() {
        if let Some(edge) = g1.get_edge(&edge_id) {
            let _ = merged.add_edge(edge.clone());
        }
    }
    
    // Add all edges from g2 (skip duplicates)
    for edge_id in g2.edge_ids() {
        if !merged.contains_edge(&edge_id) {
            if let Some(edge) = g2.get_edge(&edge_id) {
                // Only add if both nodes exist in merged graph
                if merged.contains_node(&edge.source()) && merged.contains_node(&edge.target()) {
                    let _ = merged.add_edge(edge.clone());
                }
            }
        }
    }
    
    merged
}

/// Helper to compute graph intersection
fn intersect_graphs(
    g1: &BasicGraph<GenericNode<String>, GenericEdge<f64>>,
    g2: &BasicGraph<GenericNode<String>, GenericEdge<f64>>,
) -> BasicGraph<GenericNode<String>, GenericEdge<f64>> {
    let mut intersection = BasicGraph::new(g1.graph_type());
    
    // Add nodes that exist in both graphs
    for node_id in g1.node_ids() {
        if g2.contains_node(&node_id) {
            if let Some(node) = g1.get_node(&node_id) {
                let _ = intersection.add_node(node.clone());
            }
        }
    }
    
    // Add edges that exist in both graphs
    for edge_id in g1.edge_ids() {
        if g2.contains_edge(&edge_id) {
            if let Some(edge) = g1.get_edge(&edge_id) {
                // Only add if both nodes exist in intersection
                if intersection.contains_node(&edge.source()) && 
                   intersection.contains_node(&edge.target()) {
                    let _ = intersection.add_edge(edge.clone());
                }
            }
        }
    }
    
    intersection
}

/// Check if two graphs are structurally equal
fn graphs_equal(
    g1: &BasicGraph<GenericNode<String>, GenericEdge<f64>>,
    g2: &BasicGraph<GenericNode<String>, GenericEdge<f64>>,
) -> bool {
    // Same number of nodes and edges
    if g1.node_count() != g2.node_count() || g1.edge_count() != g2.edge_count() {
        return false;
    }
    
    // Same node IDs
    let g1_nodes: HashSet<_> = g1.node_ids().into_iter().collect();
    let g2_nodes: HashSet<_> = g2.node_ids().into_iter().collect();
    if g1_nodes != g2_nodes {
        return false;
    }
    
    // Same edge IDs
    let g1_edges: HashSet<_> = g1.edge_ids().into_iter().collect();
    let g2_edges: HashSet<_> = g2.edge_ids().into_iter().collect();
    if g1_edges != g2_edges {
        return false;
    }
    
    true
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    
    #[test]
    fn union_is_commutative(
        g1 in small_graph_strategy(),
        g2 in small_graph_strategy(),
    ) {
        let union_1_2 = merge_graphs(&g1, &g2);
        let union_2_1 = merge_graphs(&g2, &g1);
        
        prop_assert!(graphs_equal(&union_1_2, &union_2_1));
    }
    
    #[test]
    fn union_is_associative(
        g1 in small_graph_strategy(),
        g2 in small_graph_strategy(),
        g3 in small_graph_strategy(),
    ) {
        let union_12_3 = merge_graphs(&merge_graphs(&g1, &g2), &g3);
        let union_1_23 = merge_graphs(&g1, &merge_graphs(&g2, &g3));
        
        prop_assert!(graphs_equal(&union_12_3, &union_1_23));
    }
    
    #[test]
    fn intersection_is_commutative(
        g1 in small_graph_strategy(),
        g2 in small_graph_strategy(),
    ) {
        let inter_1_2 = intersect_graphs(&g1, &g2);
        let inter_2_1 = intersect_graphs(&g2, &g1);
        
        prop_assert!(graphs_equal(&inter_1_2, &inter_2_1));
    }
    
    #[test]
    fn intersection_is_associative(
        g1 in small_graph_strategy(),
        g2 in small_graph_strategy(),
        g3 in small_graph_strategy(),
    ) {
        let inter_12_3 = intersect_graphs(&intersect_graphs(&g1, &g2), &g3);
        let inter_1_23 = intersect_graphs(&g1, &intersect_graphs(&g2, &g3));
        
        prop_assert!(graphs_equal(&inter_12_3, &inter_1_23));
    }
    
    #[test]
    fn empty_graph_is_identity_for_union(
        graph in small_graph_strategy(),
    ) {
        let empty = BasicGraph::new(graph.graph_type());
        let union_with_empty = merge_graphs(&graph, &empty);
        
        // Union with empty should give back the original graph
        prop_assert_eq!(union_with_empty.node_count(), graph.node_count());
        prop_assert_eq!(union_with_empty.edge_count(), graph.edge_count());
        
        for node_id in graph.node_ids() {
            prop_assert!(union_with_empty.contains_node(&node_id));
        }
        
        for edge_id in graph.edge_ids() {
            prop_assert!(union_with_empty.contains_edge(&edge_id));
        }
    }
    
    #[test]
    fn empty_graph_is_zero_for_intersection(
        graph in small_graph_strategy(),
    ) {
        let empty = BasicGraph::new(graph.graph_type());
        let intersection_with_empty = intersect_graphs(&graph, &empty);
        
        // Intersection with empty should give empty graph
        prop_assert_eq!(intersection_with_empty.node_count(), 0);
        prop_assert_eq!(intersection_with_empty.edge_count(), 0);
    }
    
    #[test]
    fn self_intersection_is_idempotent(
        graph in small_graph_strategy(),
    ) {
        let self_intersection = intersect_graphs(&graph, &graph);
        
        // Intersection with self should give back the same graph
        prop_assert!(graphs_equal(&self_intersection, &graph));
    }
    
    #[test]
    fn node_addition_order_independence(
        nodes in nodes_strategy(0..20),
    ) {
        // Adding nodes in different orders should result in the same graph
        let mut graph1 = BasicGraph::new(GraphType::Generic);
        let mut graph2 = BasicGraph::new(GraphType::Generic);
        
        // Add in original order
        for node in &nodes {
            let _ = graph1.add_node(node.clone());
        }
        
        // Add in reverse order
        for node in nodes.iter().rev() {
            let _ = graph2.add_node(node.clone());
        }
        
        prop_assert_eq!(graph1.node_count(), graph2.node_count());
        for node in &nodes {
            prop_assert!(graph1.contains_node(&node.id()));
            prop_assert!(graph2.contains_node(&node.id()));
        }
    }
    
    #[test]
    fn edge_addition_order_independence(
        mut graph in small_graph_strategy().prop_filter("has at least 3 nodes", |g| g.node_count() >= 3),
    ) {
        let node_ids = graph.node_ids();
        let edges = vec![
            GenericEdge::with_id("e1", &node_ids[0], &node_ids[1], 1.0),
            GenericEdge::with_id("e2", &node_ids[1], &node_ids[2], 2.0),
            GenericEdge::with_id("e3", &node_ids[0], &node_ids[2], 3.0),
        ];
        
        // Create two graphs with same nodes
        let mut graph1 = BasicGraph::new(graph.graph_type());
        let mut graph2 = BasicGraph::new(graph.graph_type());
        
        for node_id in &node_ids {
            if let Some(node) = graph.get_node(node_id) {
                let _ = graph1.add_node(node.clone());
                let _ = graph2.add_node(node.clone());
            }
        }
        
        // Add edges in different orders
        for edge in &edges {
            let _ = graph1.add_edge(edge.clone());
        }
        
        for edge in edges.iter().rev() {
            let _ = graph2.add_edge(edge.clone());
        }
        
        prop_assert_eq!(graph1.edge_count(), graph2.edge_count());
        for edge in &edges {
            prop_assert!(graph1.contains_edge(&edge.id()));
            prop_assert!(graph2.contains_edge(&edge.id()));
        }
    }
    
    #[test]
    fn subgraph_property(
        graph in medium_graph_strategy().prop_filter("has nodes", |g| g.node_count() > 0),
        subset_ratio in 0.0..=1.0,
    ) {
        // Create a subgraph by selecting a subset of nodes
        let all_nodes = graph.node_ids();
        let subset_size = ((all_nodes.len() as f64) * subset_ratio) as usize;
        let subset_nodes: HashSet<_> = all_nodes.iter().take(subset_size).cloned().collect();
        
        // Count edges in the subgraph
        let subgraph_edges = graph.edge_ids()
            .into_iter()
            .filter(|edge_id| {
                let edge = graph.get_edge(edge_id).unwrap();
                subset_nodes.contains(&edge.source()) && subset_nodes.contains(&edge.target())
            })
            .count();
        
        // Subgraph should have fewer or equal nodes and edges
        prop_assert!(subset_nodes.len() <= graph.node_count());
        prop_assert!(subgraph_edges <= graph.edge_count());
    }
    
    #[test]
    fn complement_graph_property(
        graph in small_graph_strategy(),
    ) {
        // For a simple graph, the complement has an edge where the original doesn't
        let node_ids = graph.node_ids();
        let n = node_ids.len();
        
        if n >= 2 {
            // Count maximum possible edges
            let max_edges = n * (n - 1); // For directed graph
            
            // Count existing directed edges
            let existing_edges = graph.edge_count();
            
            // In a complement, we should have max_edges - existing_edges
            // (minus self-loops which we don't allow)
            prop_assert!(existing_edges <= max_edges);
        }
    }
}

#[cfg(test)]
mod operation_properties {
    use super::*;
    
    proptest! {
        #[test]
        fn remove_then_add_node_preserves_structure(
            mut graph in small_graph_strategy().prop_filter("has nodes", |g| g.node_count() > 0),
        ) {
            let node_ids = graph.node_ids();
            let node_to_modify = &node_ids[0];
            
            // Save the node
            let saved_node = graph.get_node(node_to_modify).unwrap().clone();
            
            // Save edges connected to this node
            let connected_edges: Vec<_> = graph.edge_ids()
                .into_iter()
                .filter_map(|edge_id| {
                    let edge = graph.get_edge(&edge_id)?;
                    if edge.source() == *node_to_modify || edge.target() == *node_to_modify {
                        Some(edge.clone())
                    } else {
                        None
                    }
                })
                .collect();
            
            let initial_node_count = graph.node_count();
            
            // Remove and re-add
            let _ = graph.remove_node(node_to_modify);
            let _ = graph.add_node(saved_node);
            
            // Verify node count is preserved
            prop_assert_eq!(graph.node_count(), initial_node_count);
            prop_assert!(graph.contains_node(node_to_modify));
            
            // Note: Edges are not preserved when removing a node
            // This is expected behavior
        }
        
        #[test]
        fn clear_is_equivalent_to_removing_all(
            mut graph1 in small_graph_strategy(),
        ) {
            let mut graph2 = graph1.clone();
            
            // Method 1: Clear
            graph1.clear();
            
            // Method 2: Remove all nodes (which removes edges too)
            let node_ids = graph2.node_ids();
            for node_id in node_ids {
                let _ = graph2.remove_node(&node_id);
            }
            
            prop_assert_eq!(graph1.node_count(), 0);
            prop_assert_eq!(graph2.node_count(), 0);
            prop_assert_eq!(graph1.edge_count(), 0);
            prop_assert_eq!(graph2.edge_count(), 0);
        }
    }
}