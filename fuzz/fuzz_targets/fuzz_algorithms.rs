#![no_main]

use libfuzzer_sys::fuzz_target;
use cim_graph::graphs::workflow::{WorkflowGraph, WorkflowNode, StateType};
use cim_graph::algorithms;

// Fuzz graph algorithms
fuzz_target!(|data: &[u8]| {
    if data.len() < 10 {
        return;
    }
    
    // Build a graph from fuzz input
    let mut graph = WorkflowGraph::new();
    
    // Create nodes
    let node_count = (data[0] % 20) + 1;
    for i in 0..node_count {
        let id = format!("n{}", i);
        let node = WorkflowNode::new(&id, &id, StateType::Normal);
        let _ = graph.add_state(node);
    }
    
    // Create edges based on fuzz data
    let mut idx = 1;
    while idx + 1 < data.len() && idx < 50 {
        let from = format!("n{}", data[idx] % node_count);
        let to = format!("n{}", data[idx + 1] % node_count);
        let _ = graph.add_transition(&from, &to, "edge");
        idx += 2;
    }
    
    // Test pathfinding algorithms
    if node_count > 1 {
        let start = "n0";
        let end = format!("n{}", node_count - 1);
        
        // Shortest path should not panic
        if let Ok(path) = algorithms::shortest_path(graph.graph(), start, &end) {
            if let Some(path) = path {
                // Verify path is valid
                assert!(!path.is_empty());
                assert_eq!(path[0], start);
                assert_eq!(path[path.len() - 1], end);
                
                // Verify each edge in path exists
                for i in 0..path.len() - 1 {
                    let edges = graph.graph().edges_between(&path[i], &path[i + 1]);
                    assert!(!edges.is_empty());
                }
            }
        }
        
        // All paths should not panic
        if let Ok(paths) = algorithms::all_paths(graph.graph(), start, &end, 10) {
            for path in paths {
                assert!(!path.is_empty());
                assert_eq!(path[0], start);
                assert_eq!(path[path.len() - 1], end);
            }
        }
    }
    
    // Test traversal algorithms
    if node_count > 0 {
        let start = "n0";
        
        // DFS should not panic
        if let Ok(visited) = algorithms::dfs(graph.graph(), start) {
            // All visited nodes should exist
            for node in visited {
                assert!(graph.get_state(&node).is_some());
            }
        }
        
        // BFS should not panic
        if let Ok(visited) = algorithms::bfs(graph.graph(), start) {
            // All visited nodes should exist
            for node in visited {
                assert!(graph.get_state(&node).is_some());
            }
        }
    }
    
    // Test topological sort
    if let Ok(sorted) = algorithms::topological_sort(graph.graph()) {
        // All nodes should be in the sorted order
        let sorted_set: std::collections::HashSet<_> = sorted.iter().collect();
        for i in 0..node_count {
            let node = format!("n{}", i);
            if graph.get_state(&node).is_some() {
                assert!(sorted_set.contains(&node));
            }
        }
    }
    
    // Test metrics
    if let Ok(centrality) = algorithms::centrality(graph.graph()) {
        // Centrality values should be non-negative
        for (_, value) in centrality {
            assert!(value >= 0.0);
            assert!(value.is_finite());
        }
    }
    
    if let Ok(clustering) = algorithms::clustering_coefficient(graph.graph()) {
        // Clustering coefficient should be between 0 and 1
        assert!(clustering >= 0.0);
        assert!(clustering <= 1.0);
        assert!(clustering.is_finite());
    }
});

// Fuzz with specific graph patterns
fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    
    let mut graph = WorkflowGraph::new();
    
    match data[0] % 5 {
        0 => {
            // Linear chain
            let length = (data.get(1).unwrap_or(&10) % 20) + 2;
            for i in 0..length {
                let id = format!("chain_{}", i);
                let _ = graph.add_state(WorkflowNode::new(&id, &id, StateType::Normal));
                if i > 0 {
                    let from = format!("chain_{}", i - 1);
                    let _ = graph.add_transition(&from, &id, "next");
                }
            }
        }
        1 => {
            // Star graph
            let center = "center";
            let _ = graph.add_state(WorkflowNode::new(center, center, StateType::Normal));
            let spokes = (data.get(1).unwrap_or(&10) % 20) + 1;
            for i in 0..spokes {
                let spoke = format!("spoke_{}", i);
                let _ = graph.add_state(WorkflowNode::new(&spoke, &spoke, StateType::Normal));
                let _ = graph.add_transition(center, &spoke, "out");
            }
        }
        2 => {
            // Complete graph
            let size = (data.get(1).unwrap_or(&5) % 10) + 2;
            for i in 0..size {
                let id = format!("complete_{}", i);
                let _ = graph.add_state(WorkflowNode::new(&id, &id, StateType::Normal));
            }
            for i in 0..size {
                for j in 0..size {
                    if i != j {
                        let from = format!("complete_{}", i);
                        let to = format!("complete_{}", j);
                        let _ = graph.add_transition(&from, &to, "edge");
                    }
                }
            }
        }
        3 => {
            // Binary tree
            let depth = (data.get(1).unwrap_or(&4) % 5) + 1;
            let _ = graph.add_state(WorkflowNode::new("root", "root", StateType::Normal));
            let mut queue = vec!["root".to_string()];
            let mut level = 0;
            while level < depth && !queue.is_empty() {
                let mut next_queue = Vec::new();
                for parent in queue {
                    for i in 0..2 {
                        let child = format!("{}_child_{}", parent, i);
                        let _ = graph.add_state(WorkflowNode::new(&child, &child, StateType::Normal));
                        let _ = graph.add_transition(&parent, &child, "child");
                        next_queue.push(child);
                    }
                }
                queue = next_queue;
                level += 1;
            }
        }
        4 => {
            // Disconnected components
            let components = (data.get(1).unwrap_or(&3) % 5) + 1;
            for c in 0..components {
                let size = (data.get(2 + c).unwrap_or(&3) % 5) + 2;
                for i in 0..size {
                    let id = format!("comp{}_node{}", c, i);
                    let _ = graph.add_state(WorkflowNode::new(&id, &id, StateType::Normal));
                    if i > 0 {
                        let from = format!("comp{}_node{}", c, i - 1);
                        let _ = graph.add_transition(&from, &id, "edge");
                    }
                }
            }
        }
        _ => {}
    }
    
    // Run all algorithms on the generated graph - they should handle any structure
    let node_ids: Vec<_> = graph.graph().node_ids();
    if !node_ids.is_empty() {
        let start = &node_ids[0];
        let _ = algorithms::dfs(graph.graph(), start);
        let _ = algorithms::bfs(graph.graph(), start);
        let _ = algorithms::topological_sort(graph.graph());
        let _ = algorithms::centrality(graph.graph());
        let _ = algorithms::clustering_coefficient(graph.graph());
        
        if node_ids.len() > 1 {
            let end = &node_ids[node_ids.len() - 1];
            let _ = algorithms::shortest_path(graph.graph(), start, end);
            let _ = algorithms::all_paths(graph.graph(), start, end, 5);
        }
    }
});