//! Property tests for graph invariants

use proptest::prelude::*;
use cim_graph::core::{Graph, GenericNode, GenericEdge, Node, Edge};
use cim_graph::core::graph::BasicGraph;
use super::generators::*;
use std::collections::HashSet;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn adding_node_increases_count_by_one(
        mut graph in small_graph_strategy(),
        new_node in node_strategy(),
    ) {
        // Ensure the node ID is unique
        let mut node_id = new_node.id();
        let mut counter = 0;
        while graph.contains_node(&node_id) {
            counter += 1;
            node_id = format!("{}-{}", new_node.id(), counter);
        }
        let unique_node = GenericNode::new(node_id, new_node.data().clone());
        
        let initial_count = graph.node_count();
        let result = graph.add_node(unique_node.clone());
        
        prop_assert!(result.is_ok());
        prop_assert_eq!(graph.node_count(), initial_count + 1);
        prop_assert!(graph.contains_node(&unique_node.id()));
    }
    
    #[test]
    fn removing_node_decreases_count_by_one(
        mut graph in small_graph_strategy().prop_filter("has nodes", |g| g.node_count() > 0),
    ) {
        let node_ids = graph.node_ids();
        let node_to_remove = &node_ids[0];
        let initial_count = graph.node_count();
        
        let result = graph.remove_node(node_to_remove);
        
        prop_assert!(result.is_ok());
        prop_assert_eq!(graph.node_count(), initial_count - 1);
        prop_assert!(!graph.contains_node(node_to_remove));
    }
    
    #[test]
    fn removing_node_removes_all_its_edges(
        mut graph in small_graph_strategy().prop_filter("has nodes", |g| g.node_count() > 0),
    ) {
        let node_ids = graph.node_ids();
        let node_to_remove = &node_ids[0];
        
        // Collect edges connected to this node
        let connected_edges: Vec<String> = graph.edge_ids()
            .into_iter()
            .filter(|edge_id| {
                let edge = graph.get_edge(edge_id).unwrap();
                edge.source() == *node_to_remove || edge.target() == *node_to_remove
            })
            .collect();
        
        // Remove the node
        let result = graph.remove_node(node_to_remove);
        prop_assert!(result.is_ok());
        
        // Verify all connected edges are gone
        for edge_id in connected_edges {
            prop_assert!(!graph.contains_edge(&edge_id));
        }
        
        // Verify no remaining edges reference the removed node
        for edge_id in graph.edge_ids() {
            let edge = graph.get_edge(&edge_id).unwrap();
            prop_assert_ne!(edge.source(), *node_to_remove);
            prop_assert_ne!(edge.target(), *node_to_remove);
        }
    }
    
    #[test]
    fn adding_edge_increases_count_by_one(
        mut graph in small_graph_strategy().prop_filter("has at least 2 nodes", |g| g.node_count() >= 2),
    ) {
        let node_ids = graph.node_ids();
        let from = &node_ids[0];
        let to = &node_ids[1];
        
        let initial_count = graph.edge_count();
        let edge = GenericEdge::new(from, to, 1.0);
        let edge_id = edge.id();
        
        // Only add if this exact edge doesn't exist
        if !graph.contains_edge(&edge_id) {
            let result = graph.add_edge(edge);
            prop_assert!(result.is_ok());
            prop_assert_eq!(graph.edge_count(), initial_count + 1);
            prop_assert!(graph.contains_edge(&edge_id));
        }
    }
    
    #[test]
    fn removing_edge_decreases_count_by_one(
        mut graph in small_graph_strategy().prop_filter("has edges", |g| g.edge_count() > 0),
    ) {
        let edge_ids = graph.edge_ids();
        let edge_to_remove = &edge_ids[0];
        let initial_count = graph.edge_count();
        
        let result = graph.remove_edge(edge_to_remove);
        
        prop_assert!(result.is_ok());
        prop_assert_eq!(graph.edge_count(), initial_count - 1);
        prop_assert!(!graph.contains_edge(edge_to_remove));
    }
    
    #[test]
    fn node_ids_are_unique(graph in medium_graph_strategy()) {
        let node_ids = graph.node_ids();
        let unique_ids: HashSet<_> = node_ids.iter().cloned().collect();
        prop_assert_eq!(node_ids.len(), unique_ids.len());
    }
    
    #[test]
    fn edge_ids_are_unique(graph in medium_graph_strategy()) {
        let edge_ids = graph.edge_ids();
        let unique_ids: HashSet<_> = edge_ids.iter().cloned().collect();
        prop_assert_eq!(edge_ids.len(), unique_ids.len());
    }
    
    #[test]
    fn all_edges_reference_existing_nodes(graph in medium_graph_strategy()) {
        for edge_id in graph.edge_ids() {
            let edge = graph.get_edge(&edge_id).unwrap();
            prop_assert!(graph.contains_node(&edge.source()));
            prop_assert!(graph.contains_node(&edge.target()));
        }
    }
    
    #[test]
    fn neighbors_returns_valid_node_ids(
        graph in small_graph_strategy().prop_filter("has nodes", |g| g.node_count() > 0),
    ) {
        let node_ids = graph.node_ids();
        let node_id = &node_ids[0];
        
        let neighbors_result = graph.neighbors(node_id);
        prop_assert!(neighbors_result.is_ok());
        
        let neighbors = neighbors_result.unwrap();
        for neighbor_id in neighbors {
            prop_assert!(graph.contains_node(&neighbor_id));
        }
    }
    
    #[test]
    fn clear_removes_all_nodes_and_edges(mut graph in medium_graph_strategy()) {
        graph.clear();
        
        prop_assert_eq!(graph.node_count(), 0);
        prop_assert_eq!(graph.edge_count(), 0);
        prop_assert!(graph.node_ids().is_empty());
        prop_assert!(graph.edge_ids().is_empty());
    }
    
    #[test]
    fn duplicate_node_error(
        mut graph in small_graph_strategy(),
        node in node_strategy(),
    ) {
        // First add should succeed
        let first_result = graph.add_node(node.clone());
        
        if first_result.is_ok() {
            // Second add with same ID should fail
            let duplicate_result = graph.add_node(node);
            prop_assert!(duplicate_result.is_err());
        }
    }
    
    #[test]
    fn edge_requires_existing_nodes(
        mut graph in small_graph_strategy(),
        edge in prop::option::of(edge_strategy(&[])).prop_filter_map("valid edge", |e| e),
    ) {
        // If neither node exists, adding edge should fail
        if !graph.contains_node(&edge.source()) && !graph.contains_node(&edge.target()) {
            let result = graph.add_edge(edge);
            prop_assert!(result.is_err());
        }
    }
    
    #[test]
    fn metadata_updates_on_modification(mut graph in small_graph_strategy()) {
        let initial_updated_at = graph.metadata().updated_at;
        
        // Sleep a tiny bit to ensure time difference
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        // Any modification should update the timestamp
        if graph.node_count() > 0 {
            let node_id = graph.node_ids()[0].clone();
            let _ = graph.remove_node(&node_id);
            prop_assert!(graph.metadata().updated_at > initial_updated_at);
        } else {
            let node = GenericNode::new("test", "data");
            let _ = graph.add_node(node);
            prop_assert!(graph.metadata().updated_at > initial_updated_at);
        }
    }
    
    #[test]
    fn graph_maintains_adjacency_consistency(graph in medium_graph_strategy()) {
        // For each edge, verify neighbors list is consistent
        for edge_id in graph.edge_ids() {
            let edge = graph.get_edge(&edge_id).unwrap();
            let source_neighbors = graph.neighbors(&edge.source()).unwrap();
            
            // The target should be in source's neighbors
            prop_assert!(source_neighbors.contains(&edge.target()));
        }
    }
    
    #[test]
    fn edges_between_returns_correct_edges(graph in small_graph_strategy()) {
        for edge_id in graph.edge_ids() {
            let edge = graph.get_edge(&edge_id).unwrap();
            let edges_between = graph.edges_between(&edge.source(), &edge.target());
            
            // The edge should be in the edges_between result
            prop_assert!(edges_between.iter().any(|e| e.id() == edge_id));
        }
    }
}

#[cfg(test)]
mod tree_invariants {
    use super::*;
    
    proptest! {
        #[test]
        fn tree_has_no_cycles(tree in tree_graph_strategy(3, 2..4)) {
            // A tree with n nodes should have exactly n-1 edges
            if tree.node_count() > 0 {
                prop_assert_eq!(tree.edge_count(), tree.node_count() - 1);
            }
            
            // Verify no node has multiple paths to root
            // (This is a simplified check - a full cycle detection would be more complex)
            let mut visited = HashSet::new();
            let mut queue = vec![];
            
            if let Some(root_id) = tree.node_ids().first() {
                queue.push(root_id.clone());
                
                while let Some(current) = queue.pop() {
                    if visited.contains(&current) {
                        // Found a cycle
                        prop_assert!(false, "Tree contains a cycle");
                    }
                    visited.insert(current.clone());
                    
                    if let Ok(neighbors) = tree.neighbors(&current) {
                        queue.extend(neighbors);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod cycle_invariants {
    use super::*;
    
    proptest! {
        #[test]
        fn cyclic_graph_has_correct_structure(cycle in cyclic_graph_strategy(3..10)) {
            let node_count = cycle.node_count();
            let edge_count = cycle.edge_count();
            
            if node_count > 0 {
                // A cycle should have exactly n edges for n nodes
                prop_assert_eq!(edge_count, node_count);
                
                // Each node should have exactly one outgoing edge
                for node_id in cycle.node_ids() {
                    let neighbors = cycle.neighbors(&node_id).unwrap();
                    prop_assert_eq!(neighbors.len(), 1);
                }
            }
        }
    }
}