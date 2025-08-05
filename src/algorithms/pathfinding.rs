//! Pathfinding algorithms for graph projections

use crate::core::GraphProjection;
use crate::error::{GraphError, Result};
use std::collections::{HashMap, VecDeque};

/// Find shortest path between two nodes using BFS
pub fn shortest_path<P: GraphProjection>(
    projection: &P,
    from: &str,
    to: &str,
) -> Result<Option<Vec<String>>> {
    // Validate nodes exist
    if projection.get_node(from).is_none() {
        return Err(GraphError::NodeNotFound(from.to_string()));
    }
    if projection.get_node(to).is_none() {
        return Err(GraphError::NodeNotFound(to.to_string()));
    }
    
    // BFS to find shortest path
    let mut queue = VecDeque::new();
    let mut visited = HashMap::new();
    let mut parent: HashMap<String, String> = HashMap::new();
    
    queue.push_back(from);
    visited.insert(from, true);
    
    while let Some(current) = queue.pop_front() {
        if current == to {
            // Reconstruct path
            let mut path = Vec::new();
            let mut node = to;
            
            while node != from {
                path.push(node.to_string());
                node = parent.get(node).unwrap().as_str();
            }
            path.push(from.to_string());
            path.reverse();
            
            return Ok(Some(path));
        }
        
        for neighbor in projection.neighbors(current) {
            if !visited.contains_key(neighbor) {
                visited.insert(neighbor, true);
                parent.insert(neighbor.to_string(), current.to_string());
                queue.push_back(neighbor);
            }
        }
    }
    
    Ok(None)
}

/// Find all paths between two nodes
pub fn all_paths<P: GraphProjection>(
    projection: &P,
    from: &str,
    to: &str,
) -> Result<Vec<Vec<String>>> {
    // Validate nodes exist
    if projection.get_node(from).is_none() {
        return Err(GraphError::NodeNotFound(from.to_string()));
    }
    if projection.get_node(to).is_none() {
        return Err(GraphError::NodeNotFound(to.to_string()));
    }
    
    let mut all_paths = Vec::new();
    let mut current_path = vec![from.to_string()];
    let mut visited: HashMap<String, bool> = HashMap::new();
    visited.insert(from.to_string(), true);
    
    find_all_paths_dfs(
        projection,
        from,
        to,
        &mut visited,
        &mut current_path,
        &mut all_paths,
    );
    
    Ok(all_paths)
}

fn find_all_paths_dfs<P: GraphProjection>(
    projection: &P,
    current: &str,
    target: &str,
    visited: &mut HashMap<String, bool>,
    current_path: &mut Vec<String>,
    all_paths: &mut Vec<Vec<String>>,
) {
    if current == target {
        all_paths.push(current_path.clone());
        return;
    }
    
    for neighbor in projection.neighbors(current) {
        if !visited.contains_key(neighbor) {
            visited.insert(neighbor.to_string(), true);
            current_path.push(neighbor.to_string());
            
            find_all_paths_dfs(projection, neighbor, target, visited, current_path, all_paths);
            
            current_path.pop();
            visited.remove(neighbor);
        }
    }
}