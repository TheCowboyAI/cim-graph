//! Graph traversal algorithms for projections

use crate::core::{GraphProjection, Node};
use crate::error::{GraphError, Result};
use std::collections::{HashMap, HashSet, VecDeque};

/// Breadth-first search traversal
pub fn bfs<P: GraphProjection>(projection: &P, start: &str) -> Result<Vec<String>> {
    if projection.get_node(start).is_none() {
        return Err(GraphError::NodeNotFound(start.to_string()));
    }
    
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut result = Vec::new();
    
    queue.push_back(start);
    visited.insert(start);
    
    while let Some(current) = queue.pop_front() {
        result.push(current.to_string());
        
        for neighbor in projection.neighbors(current) {
            if !visited.contains(neighbor) {
                visited.insert(neighbor);
                queue.push_back(neighbor);
            }
        }
    }
    
    Ok(result)
}

/// Depth-first search traversal
pub fn dfs<P: GraphProjection>(projection: &P, start: &str) -> Result<Vec<String>> {
    if projection.get_node(start).is_none() {
        return Err(GraphError::NodeNotFound(start.to_string()));
    }
    
    let mut visited = HashSet::new();
    let mut result = Vec::new();
    
    dfs_helper(projection, start, &mut visited, &mut result);
    
    Ok(result)
}

fn dfs_helper<P: GraphProjection>(
    projection: &P,
    current: &str,
    visited: &mut HashSet<String>,
    result: &mut Vec<String>,
) {
    visited.insert(current.to_string());
    result.push(current.to_string());
    
    let neighbors = projection.neighbors(current);
    for neighbor in neighbors {
        if !visited.contains(neighbor) {
            dfs_helper(projection, neighbor, visited, result);
        }
    }
}

/// Topological sort (for DAGs)
pub fn topological_sort<P: GraphProjection>(projection: &P) -> Result<Vec<String>> 
where
    P::Node: Node,
{
    let nodes = projection.nodes();
    let mut in_degree = HashMap::new();
    let mut result = Vec::new();
    
    // Calculate in-degrees
    for node in &nodes {
        let node_id = node.id();
        in_degree.entry(node_id.clone()).or_insert(0);
        
        for neighbor in projection.neighbors(&node_id) {
            *in_degree.entry(neighbor.to_string()).or_insert(0) += 1;
        }
    }
    
    // Find nodes with no incoming edges
    let mut queue = VecDeque::new();
    for (node_id, &degree) in &in_degree {
        if degree == 0 {
            queue.push_back(node_id.clone());
        }
    }
    
    // Process nodes
    while let Some(current) = queue.pop_front() {
        result.push(current.clone());
        
        for neighbor in projection.neighbors(&current) {
            if let Some(degree) = in_degree.get_mut(neighbor) {
                *degree -= 1;
                if *degree == 0 {
                    queue.push_back(neighbor.to_string());
                }
            }
        }
    }
    
    // Check for cycles
    if result.len() != nodes.len() {
        return Err(GraphError::InvalidOperation(
            "Graph contains cycles - cannot perform topological sort".to_string(),
        ));
    }
    
    Ok(result)
}