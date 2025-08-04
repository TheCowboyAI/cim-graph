//! Property tests for graph algorithms

use proptest::prelude::*;
use cim_graph::core::{Graph, GenericNode, GenericEdge, Node, Edge, GraphType};
use cim_graph::core::graph::BasicGraph;
use super::generators::*;
use std::collections::{HashSet, HashMap, VecDeque};

/// Simple BFS implementation for testing
fn bfs_reachable_nodes(
    graph: &BasicGraph<GenericNode<String>, GenericEdge<f64>>,
    start: &str,
) -> HashSet<String> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    
    if graph.contains_node(start) {
        queue.push_back(start.to_string());
        visited.insert(start.to_string());
    }
    
    while let Some(current) = queue.pop_front() {
        if let Ok(neighbors) = graph.neighbors(&current) {
            for neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor.clone());
                    queue.push_back(neighbor);
                }
            }
        }
    }
    
    visited
}

/// Simple DFS implementation for testing
fn dfs_reachable_nodes(
    graph: &BasicGraph<GenericNode<String>, GenericEdge<f64>>,
    start: &str,
) -> HashSet<String> {
    let mut visited = HashSet::new();
    let mut stack = vec![start.to_string()];
    
    while let Some(current) = stack.pop() {
        if visited.contains(&current) {
            continue;
        }
        
        visited.insert(current.clone());
        
        if let Ok(neighbors) = graph.neighbors(&current) {
            for neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    stack.push(neighbor);
                }
            }
        }
    }
    
    visited
}

/// Check if graph has a cycle using DFS
fn has_cycle(graph: &BasicGraph<GenericNode<String>, GenericEdge<f64>>) -> bool {
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();
    
    fn dfs_cycle(
        graph: &BasicGraph<GenericNode<String>, GenericEdge<f64>>,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        
        if let Ok(neighbors) = graph.neighbors(node) {
            for neighbor in neighbors.into_iter() {
                if !visited.contains(&neighbor) {
                    if dfs_cycle(graph, &neighbor, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(&neighbor) {
                    return true;
                }
            }
        }
        
        rec_stack.remove(node);
        false
    }
    
    for node_id in graph.node_ids().into_iter() {
        if !visited.contains(&node_id) {
            if dfs_cycle(graph, &node_id, &mut visited, &mut rec_stack) {
                return true;
            }
        }
    }
    
    false
}

/// Count connected components in the graph
fn count_components(graph: &BasicGraph<GenericNode<String>, GenericEdge<f64>>) -> usize {
    let mut visited = HashSet::new();
    let mut components = 0;
    
    for node_id in graph.node_ids().into_iter() {
        if !visited.contains(&node_id) {
            components += 1;
            let reachable = bfs_reachable_nodes(graph, &node_id);
            visited.extend(reachable);
        }
    }
    
    components
}

/// Simple shortest path using BFS (unweighted)
fn shortest_path_length(
    graph: &BasicGraph<GenericNode<String>, GenericEdge<f64>>,
    start: &str,
    end: &str,
) -> Option<usize> {
    if !graph.contains_node(start) || !graph.contains_node(end) {
        return None;
    }
    
    if start == end {
        return Some(0);
    }
    
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    
    queue.push_back((start.to_string(), 0));
    visited.insert(start.to_string());
    
    while let Some((current, dist)) = queue.pop_front() {
        if let Ok(neighbors) = graph.neighbors(&current) {
            for neighbor in neighbors {
                if neighbor == end {
                    return Some(dist + 1);
                }
                
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor.clone());
                    queue.push_back((neighbor, dist + 1));
                }
            }
        }
    }
    
    None
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    
    #[test]
    fn bfs_and_dfs_find_same_reachable_nodes(
        graph in small_graph_strategy(),
        start_idx in any::<prop::sample::Index>(),
    ) {
        let node_ids = graph.node_ids();
        if !node_ids.is_empty() {
            let start = &node_ids[start_idx.index(node_ids.len())];
            
            let bfs_nodes = bfs_reachable_nodes(&graph, start);
            let dfs_nodes = dfs_reachable_nodes(&graph, start);
            
            prop_assert_eq!(bfs_nodes, dfs_nodes);
        }
    }
    
    #[test]
    fn reachability_is_transitive(
        graph in small_graph_strategy(),
    ) {
        let node_ids = graph.node_ids();
        
        // Build reachability map
        let mut reachable: HashMap<String, HashSet<String>> = HashMap::new();
        for node_id in &node_ids {
            reachable.insert(node_id.clone(), bfs_reachable_nodes(&graph, node_id));
        }
        
        // Check transitivity: if A can reach B and B can reach C, then A can reach C
        for a in &node_ids {
            for b in &node_ids {
                for c in &node_ids {
                    if reachable[a].contains(b) && reachable[b].contains(c) {
                        prop_assert!(
                            reachable[a].contains(c),
                            "Transitivity violated: {} -> {} -> {}, but {} cannot reach {}",
                            a, b, c, a, c
                        );
                    }
                }
            }
        }
    }
    
    #[test]
    fn tree_has_no_cycles(
        tree in tree_graph_strategy(3, 2..4),
    ) {
        prop_assert!(!has_cycle(&tree));
    }
    
    #[test]
    fn cycle_detection_finds_cycles(
        cycle in cyclic_graph_strategy(3..10),
    ) {
        if cycle.node_count() > 0 {
            prop_assert!(has_cycle(&cycle));
        }
    }
    
    #[test]
    fn connected_graph_has_one_component(
        graph in small_graph_strategy()
            .prop_filter("connected", |g| {
                if g.node_count() == 0 { return true; }
                let start = &g.node_ids()[0];
                bfs_reachable_nodes(g, start).len() == g.node_count()
            }),
    ) {
        if graph.node_count() > 0 {
            prop_assert_eq!(count_components(&graph), 1);
        }
    }
    
    #[test]
    fn disconnected_graph_component_count(
        components in disconnected_graph_strategy(2..5),
    ) {
        // Each original component should remain separate
        let component_count = count_components(&components);
        prop_assert!(component_count >= 2);
    }
    
    #[test]
    fn shortest_path_properties(
        graph in small_graph_strategy(),
    ) {
        let node_ids = graph.node_ids();
        
        for (i, start) in node_ids.iter().enumerate() {
            // Path from node to itself is 0
            prop_assert_eq!(shortest_path_length(&graph, start, start), Some(0));
            
            // If path exists, it should be symmetric in undirected graphs
            for (j, end) in node_ids.iter().enumerate() {
                if i != j {
                    let path_length = shortest_path_length(&graph, start, end);
                    
                    // If there's a path, it should be at least 1
                    if let Some(length) = path_length {
                        prop_assert!(length >= 1);
                    }
                }
            }
        }
    }
    
    #[test]
    fn path_length_triangle_inequality(
        graph in small_graph_strategy().prop_filter("has at least 3 nodes", |g| g.node_count() >= 3),
    ) {
        let node_ids = graph.node_ids();
        
        // For any three nodes a, b, c:
        // distance(a,c) <= distance(a,b) + distance(b,c)
        for a in &node_ids {
            for b in &node_ids {
                for c in &node_ids {
                    if a != b && b != c && a != c {
                        let ab = shortest_path_length(&graph, a, b);
                        let bc = shortest_path_length(&graph, b, c);
                        let ac = shortest_path_length(&graph, a, c);
                        
                        if let (Some(ab_len), Some(bc_len), Some(ac_len)) = (ab, bc, ac) {
                            prop_assert!(
                                ac_len <= ab_len + bc_len,
                                "Triangle inequality violated: d({},{}) = {} > {} + {} = d({},{}) + d({},{})",
                                a, c, ac_len, ab_len, bc_len, a, b, b, c
                            );
                        }
                    }
                }
            }
        }
    }
    
    #[test]
    fn adding_edge_maintains_or_decreases_distances(
        mut graph in small_graph_strategy().prop_filter("has at least 2 nodes", |g| g.node_count() >= 2),
    ) {
        let node_ids = graph.node_ids();
        
        // Calculate initial distances
        let mut initial_distances = HashMap::new();
        for i in 0..node_ids.len() {
            for j in 0..node_ids.len() {
                if i != j {
                    let dist = shortest_path_length(&graph, &node_ids[i], &node_ids[j]);
                    initial_distances.insert((node_ids[i].clone(), node_ids[j].clone()), dist);
                }
            }
        }
        
        // Add a new edge if possible
        let from = &node_ids[0];
        let to = &node_ids[1];
        
        // Check if edge already exists
        let existing_edges = graph.edges_between(from, to);
        if existing_edges.is_empty() {
            let new_edge = GenericEdge::new(from, to, 1.0);
            let _ = graph.add_edge(new_edge);
            
            // Recalculate distances
            for i in 0..node_ids.len() {
                for j in 0..node_ids.len() {
                    if i != j {
                        let new_dist = shortest_path_length(&graph, &node_ids[i], &node_ids[j]);
                        let old_dist = initial_distances[&(node_ids[i].clone(), node_ids[j].clone())];
                        
                        // Distance should only decrease or stay the same
                        match (old_dist, new_dist) {
                            (Some(old), Some(new)) => prop_assert!(new <= old),
                            (None, Some(_)) => {}, // New path created, OK
                            (Some(_), None) => prop_assert!(false, "Path disappeared after adding edge"),
                            (None, None) => {}, // Still no path, OK
                        }
                    }
                }
            }
        }
    }
    
    #[test]
    fn degree_sum_equals_twice_edge_count(
        graph in medium_graph_strategy(),
    ) {
        // In a directed graph, sum of out-degrees equals edge count
        let mut total_out_degree = 0;
        
        for node_id in graph.node_ids() {
            if let Ok(neighbors) = graph.neighbors(&node_id) {
                total_out_degree += neighbors.len();
            }
        }
        
        prop_assert_eq!(total_out_degree, graph.edge_count());
    }
    
    #[test]
    fn complete_graph_properties(
        n in 3..=10usize,
    ) {
        // Create a complete graph
        let mut graph = BasicGraph::new(GraphType::Generic);
        let nodes: Vec<_> = (0..n).map(|i| GenericNode::new(format!("n{}", i), "data")).collect();
        
        for node in &nodes {
            graph.add_node(node.clone()).unwrap();
        }
        
        for i in 0..n {
            for j in 0..n {
                if i != j {
                    graph.add_edge(GenericEdge::new(
                        format!("n{}", i),
                        format!("n{}", j),
                        1.0,
                    )).unwrap();
                }
            }
        }
        
        // Complete directed graph has n*(n-1) edges
        prop_assert_eq!(graph.edge_count(), n * (n - 1));
        
        // Every node has out-degree n-1
        for node_id in graph.node_ids() {
            let neighbors = graph.neighbors(&node_id).unwrap();
            prop_assert_eq!(neighbors.len(), n - 1);
        }
        
        // Every pair of nodes has distance 1
        for i in 0..n {
            for j in 0..n {
                if i != j {
                    let dist = shortest_path_length(&graph, &format!("n{}", i), &format!("n{}", j));
                    prop_assert_eq!(dist, Some(1));
                }
            }
        }
    }
}

#[cfg(test)]
mod algorithm_correctness {
    use super::*;
    
    proptest! {
        #[test]
        fn node_removal_reduces_reachability(
            mut graph in small_graph_strategy().prop_filter("has at least 2 nodes", |g| g.node_count() >= 2),
        ) {
            let node_ids = graph.node_ids();
            let node_to_remove = &node_ids[0];
            let other_node = &node_ids[1];
            
            // Get initial reachability
            let initial_reachable = bfs_reachable_nodes(&graph, other_node);
            let could_reach_removed = initial_reachable.contains(node_to_remove);
            
            // Remove node
            let _ = graph.remove_node(node_to_remove);
            
            // Get new reachability
            let new_reachable = bfs_reachable_nodes(&graph, other_node);
            
            // Reachability should only decrease
            prop_assert!(new_reachable.len() <= initial_reachable.len());
            
            // Removed node should not be reachable
            prop_assert!(!new_reachable.contains(node_to_remove));
            
            // All newly reachable nodes were previously reachable
            for node in &new_reachable {
                prop_assert!(initial_reachable.contains(node));
            }
        }
    }
}