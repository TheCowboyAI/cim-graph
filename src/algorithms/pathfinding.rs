//! Pathfinding algorithms
//!
//! Provides algorithms for finding paths in graphs

use crate::core::{EventGraph, Node, Edge};
use crate::error::Result;
use std::collections::{HashMap, HashSet, BinaryHeap};
use std::cmp::Ordering;

/// Find shortest path between two nodes using Dijkstra's algorithm
pub fn shortest_path<N: Node, E: Edge>(
    graph: &EventGraph<N, E>,
    start: &str,
    end: &str,
) -> Result<Option<Vec<String>>> {
    // Check nodes exist
    if !graph.contains_node(start) || !graph.contains_node(end) {
        return Ok(None);
    }
    
    if start == end {
        return Ok(Some(vec![start.to_string()]));
    }
    
    // Initialize distances and previous nodes
    let mut distances: HashMap<String, f64> = HashMap::new();
    let mut previous: HashMap<String, String> = HashMap::new();
    let mut heap = BinaryHeap::new();
    
    // Start with the source node
    distances.insert(start.to_string(), 0.0);
    heap.push(State { cost: 0.0, node: start.to_string() });
    
    // Process nodes
    while let Some(State { cost, node }) = heap.pop() {
        // Found the target
        if node == end {
            let mut path = Vec::new();
            let mut current = end.to_string();
            
            while current != start {
                path.push(current.clone());
                match previous.get(&current) {
                    Some(prev) => current = prev.clone(),
                    None => return Ok(None),
                }
            }
            
            path.push(start.to_string());
            path.reverse();
            return Ok(Some(path));
        }
        
        // Skip if we've found a better path
        if distances.get(&node).map_or(false, |&d| cost > d) {
            continue;
        }
        
        // Check neighbors
        if let Ok(neighbors) = graph.neighbors(&node) {
            for neighbor in neighbors {
                let next_cost = cost + 1.0; // Unit weight for now
                
                if next_cost < *distances.get(&neighbor).unwrap_or(&f64::INFINITY) {
                    distances.insert(neighbor.clone(), next_cost);
                    previous.insert(neighbor.clone(), node.clone());
                    heap.push(State { cost: next_cost, node: neighbor });
                }
            }
        }
    }
    
    Ok(None)
}

/// Find all paths between two nodes up to a maximum length
pub fn all_paths<N: Node, E: Edge>(
    graph: &EventGraph<N, E>,
    start: &str,
    end: &str,
    max_length: usize,
) -> Result<Vec<Vec<String>>> {
    if !graph.contains_node(start) || !graph.contains_node(end) {
        return Ok(vec![]);
    }
    
    let mut all_paths = Vec::new();
    let mut current_path = vec![start.to_string()];
    let mut visited = HashSet::new();
    visited.insert(start.to_string());
    
    find_paths_dfs(
        graph,
        start,
        end,
        &mut current_path,
        &mut visited,
        &mut all_paths,
        max_length,
    )?;
    
    Ok(all_paths)
}

/// Helper function for DFS path finding
fn find_paths_dfs<N: Node, E: Edge>(
    graph: &EventGraph<N, E>,
    current: &str,
    target: &str,
    path: &mut Vec<String>,
    visited: &mut HashSet<String>,
    all_paths: &mut Vec<Vec<String>>,
    max_length: usize,
) -> Result<()> {
    if current == target {
        all_paths.push(path.clone());
        return Ok(());
    }
    
    if path.len() >= max_length {
        return Ok(());
    }
    
    if let Ok(neighbors) = graph.neighbors(current) {
        for neighbor in neighbors {
            if !visited.contains(&neighbor) {
                visited.insert(neighbor.clone());
                path.push(neighbor.clone());
                
                find_paths_dfs(graph, &neighbor, target, path, visited, all_paths, max_length)?;
                
                path.pop();
                visited.remove(&neighbor);
            }
        }
    }
    
    Ok(())
}

/// State for Dijkstra's algorithm
#[derive(Clone, PartialEq)]
struct State {
    cost: f64,
    node: String,
}

impl Eq for State {}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order for min-heap
        other.cost.partial_cmp(&self.cost).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{GraphBuilder, GraphType};
    use crate::core::graph::BasicNode;
    use crate::core::graph::BasicEdge;
    
    #[test]
    fn test_shortest_path() {
        let mut graph = GraphBuilder::new()
            .graph_type(GraphType::BasicGraph)
            .build_event::<BasicNode, BasicEdge>()
            .unwrap();
            
        // Create a simple graph: A -> B -> C
        //                         \-> D -> C
        graph.add_node(BasicNode::new("A")).unwrap();
        graph.add_node(BasicNode::new("B")).unwrap();
        graph.add_node(BasicNode::new("C")).unwrap();
        graph.add_node(BasicNode::new("D")).unwrap();
        
        graph.add_edge(BasicEdge::new("A", "B")).unwrap();
        graph.add_edge(BasicEdge::new("B", "C")).unwrap();
        graph.add_edge(BasicEdge::new("A", "D")).unwrap();
        graph.add_edge(BasicEdge::new("D", "C")).unwrap();
        
        // Find shortest path
        let path = shortest_path(&graph, "A", "C").unwrap();
        assert!(path.is_some());
        
        let path = path.unwrap();
        assert_eq!(path.len(), 3); // A -> B -> C or A -> D -> C
        assert_eq!(path[0], "A");
        assert_eq!(path[2], "C");
    }
}