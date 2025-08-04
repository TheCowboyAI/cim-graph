//! Graph traversal algorithms
//!
//! Provides various ways to traverse graphs

use crate::core::{EventGraph, Node, Edge};
use crate::error::{GraphError, Result};
use std::collections::{HashSet, VecDeque, HashMap};

/// Depth-first search traversal
pub fn dfs<N: Node, E: Edge>(
    graph: &EventGraph<N, E>,
    start: &str,
) -> Result<Vec<String>> {
    if !graph.contains_node(start) {
        return Err(GraphError::NodeNotFound(start.to_string()));
    }
    
    let mut visited = HashSet::new();
    let mut result = Vec::new();
    let mut stack = vec![start.to_string()];
    
    while let Some(node) = stack.pop() {
        if visited.insert(node.clone()) {
            result.push(node.clone());
            
            if let Ok(neighbors) = graph.neighbors(&node) {
                for neighbor in neighbors.into_iter().rev() {
                    if !visited.contains(&neighbor) {
                        stack.push(neighbor);
                    }
                }
            }
        }
    }
    
    Ok(result)
}

/// Breadth-first search traversal
pub fn bfs<N: Node, E: Edge>(
    graph: &EventGraph<N, E>,
    start: &str,
) -> Result<Vec<String>> {
    if !graph.contains_node(start) {
        return Err(GraphError::NodeNotFound(start.to_string()));
    }
    
    let mut visited = HashSet::new();
    let mut result = Vec::new();
    let mut queue = VecDeque::new();
    
    visited.insert(start.to_string());
    queue.push_back(start.to_string());
    
    while let Some(node) = queue.pop_front() {
        result.push(node.clone());
        
        if let Ok(neighbors) = graph.neighbors(&node) {
            for neighbor in neighbors {
                if visited.insert(neighbor.clone()) {
                    queue.push_back(neighbor);
                }
            }
        }
    }
    
    Ok(result)
}

/// Topological sort using Kahn's algorithm
pub fn topological_sort<N: Node, E: Edge>(
    graph: &EventGraph<N, E>,
) -> Result<Vec<String>> {
    // Calculate in-degrees
    let mut in_degrees: HashMap<String, usize> = HashMap::new();
    let nodes = graph.node_ids();
    
    // Initialize all nodes with 0 in-degree
    for node in &nodes {
        in_degrees.insert(node.clone(), 0);
    }
    
    // Count in-degrees
    for node in &nodes {
        if let Ok(neighbors) = graph.neighbors(node) {
            for neighbor in neighbors {
                *in_degrees.get_mut(&neighbor).unwrap() += 1;
            }
        }
    }
    
    // Find nodes with no incoming edges
    let mut queue: VecDeque<String> = nodes.iter()
        .filter(|&node| in_degrees[node] == 0)
        .cloned()
        .collect();
    
    let mut result = Vec::new();
    
    // Process nodes
    while let Some(node) = queue.pop_front() {
        result.push(node.clone());
        
        if let Ok(neighbors) = graph.neighbors(&node) {
            for neighbor in neighbors {
                let degree = in_degrees.get_mut(&neighbor).unwrap();
                *degree -= 1;
                
                if *degree == 0 {
                    queue.push_back(neighbor);
                }
            }
        }
    }
    
    // Check if all nodes were processed (no cycles)
    if result.len() != nodes.len() {
        return Err(GraphError::InvalidOperation("Graph contains cycles".to_string()));
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{GraphBuilder, GraphType};
    use crate::core::graph::{BasicNode, BasicEdge};
    
    #[test]
    fn test_dfs() {
        let mut graph = GraphBuilder::new()
            .graph_type(GraphType::BasicGraph)
            .build_event::<BasicNode, BasicEdge>()
            .unwrap();
            
        // Create a simple graph
        graph.add_node(BasicNode::new("A")).unwrap();
        graph.add_node(BasicNode::new("B")).unwrap();
        graph.add_node(BasicNode::new("C")).unwrap();
        
        graph.add_edge(BasicEdge::new("A", "B")).unwrap();
        graph.add_edge(BasicEdge::new("A", "C")).unwrap();
        
        let result = dfs(&graph, "A").unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "A");
    }
    
    #[test]
    fn test_bfs() {
        let mut graph = GraphBuilder::new()
            .graph_type(GraphType::BasicGraph)
            .build_event::<BasicNode, BasicEdge>()
            .unwrap();
            
        // Create a simple graph
        graph.add_node(BasicNode::new("A")).unwrap();
        graph.add_node(BasicNode::new("B")).unwrap();
        graph.add_node(BasicNode::new("C")).unwrap();
        
        graph.add_edge(BasicEdge::new("A", "B")).unwrap();
        graph.add_edge(BasicEdge::new("A", "C")).unwrap();
        
        let result = bfs(&graph, "A").unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "A");
        // B and C should be at the same level
        assert!(result[1] == "B" || result[1] == "C");
        assert!(result[2] == "B" || result[2] == "C");
    }
}