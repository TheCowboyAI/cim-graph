//! Graph metrics algorithms
//!
//! Provides algorithms for computing various graph metrics

use crate::core::{EventGraph, Node, Edge};
use crate::error::Result;
use std::collections::{HashMap, HashSet};

/// Compute degree centrality for all nodes
pub fn degree_centrality<N: Node, E: Edge>(
    graph: &EventGraph<N, E>,
) -> Result<HashMap<String, f64>> {
    let mut centrality = HashMap::new();
    let node_count = graph.node_count() as f64;
    
    if node_count <= 1.0 {
        // For graphs with 0 or 1 nodes, centrality is not well-defined
        for node_id in graph.node_ids() {
            centrality.insert(node_id, 0.0);
        }
        return Ok(centrality);
    }
    
    for node_id in graph.node_ids() {
        let degree = graph.degree(&node_id)? as f64;
        // Normalize by the maximum possible degree (n-1)
        centrality.insert(node_id, degree / (node_count - 1.0));
    }
    
    Ok(centrality)
}

/// Alias for degree_centrality (to match the public API)
pub fn centrality<N: Node, E: Edge>(
    graph: &EventGraph<N, E>,
) -> Result<HashMap<String, f64>> {
    degree_centrality(graph)
}

/// Compute clustering coefficient for a specific node
pub fn node_clustering_coefficient<N: Node, E: Edge>(
    graph: &EventGraph<N, E>,
    node_id: &str,
) -> Result<f64> {
    let neighbors = graph.neighbors(node_id)?;
    let neighbor_count = neighbors.len();
    
    if neighbor_count < 2 {
        // Clustering coefficient is 0 for nodes with less than 2 neighbors
        return Ok(0.0);
    }
    
    // Count edges between neighbors
    let mut edge_count = 0;
    let neighbor_set: HashSet<_> = neighbors.iter().cloned().collect();
    
    for neighbor in &neighbors {
        if let Ok(second_neighbors) = graph.neighbors(neighbor) {
            for second_neighbor in second_neighbors {
                if neighbor_set.contains(&second_neighbor) {
                    edge_count += 1;
                }
            }
        }
    }
    
    // In a directed graph, we counted each edge once
    // Maximum possible edges between k neighbors is k*(k-1)
    let max_edges = neighbor_count * (neighbor_count - 1);
    
    Ok(edge_count as f64 / max_edges as f64)
}

/// Compute average clustering coefficient for the entire graph
pub fn clustering_coefficient<N: Node, E: Edge>(
    graph: &EventGraph<N, E>,
) -> Result<f64> {
    let node_ids = graph.node_ids();
    
    if node_ids.is_empty() {
        return Ok(0.0);
    }
    
    let mut total = 0.0;
    let mut count = 0;
    
    for node_id in node_ids {
        let coeff = node_clustering_coefficient(graph, &node_id)?;
        total += coeff;
        count += 1;
    }
    
    Ok(total / count as f64)
}

/// Compute the density of the graph
pub fn graph_density<N: Node, E: Edge>(
    graph: &EventGraph<N, E>,
) -> Result<f64> {
    let node_count = graph.node_count() as f64;
    let edge_count = graph.edge_count() as f64;
    
    if node_count <= 1.0 {
        return Ok(0.0);
    }
    
    // For directed graphs: density = edges / (nodes * (nodes - 1))
    // For undirected graphs: density = 2 * edges / (nodes * (nodes - 1))
    // We assume directed graphs here
    let max_edges = node_count * (node_count - 1.0);
    
    Ok(edge_count / max_edges)
}

/// Find connected components using DFS
pub fn connected_components<N: Node, E: Edge>(
    graph: &EventGraph<N, E>,
) -> Result<Vec<Vec<String>>> {
    let mut visited = HashSet::new();
    let mut components = Vec::new();
    
    for node_id in graph.node_ids() {
        if !visited.contains(&node_id) {
            let mut component = Vec::new();
            dfs_component(graph, &node_id, &mut visited, &mut component)?;
            components.push(component);
        }
    }
    
    Ok(components)
}

/// Helper function for connected components
fn dfs_component<N: Node, E: Edge>(
    graph: &EventGraph<N, E>,
    start: &str,
    visited: &mut HashSet<String>,
    component: &mut Vec<String>,
) -> Result<()> {
    visited.insert(start.to_string());
    component.push(start.to_string());
    
    if let Ok(neighbors) = graph.neighbors(start) {
        for neighbor in neighbors {
            if !visited.contains(&neighbor) {
                dfs_component(graph, &neighbor, visited, component)?;
            }
        }
    }
    
    // Also check predecessors for undirected connectivity
    if let Ok(predecessors) = graph.predecessors(start) {
        for predecessor in predecessors {
            if !visited.contains(&predecessor) {
                dfs_component(graph, &predecessor, visited, component)?;
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{GraphBuilder, GraphType};
    use crate::core::node::GenericNode;
    use crate::core::edge::GenericEdge;
    
    #[test]
    fn test_degree_centrality() {
        let mut graph = GraphBuilder::new()
            .graph_type(GraphType::Generic)
            .build_event::<GenericNode<&'static str>, GenericEdge<f64>>()
            .unwrap();
            
        // Create a star graph: A is connected to B, C, D
        graph.add_node(GenericNode::new("A", "data")).unwrap();
        graph.add_node(GenericNode::new("B", "data")).unwrap();
        graph.add_node(GenericNode::new("C", "data")).unwrap();
        graph.add_node(GenericNode::new("D", "data")).unwrap();
        
        graph.add_edge(GenericEdge::new("A", "B", 1.0)).unwrap();
        graph.add_edge(GenericEdge::new("A", "C", 1.0)).unwrap();
        graph.add_edge(GenericEdge::new("A", "D", 1.0)).unwrap();
        
        let centrality = degree_centrality(&graph).unwrap();
        
        // A has degree 3, normalized by (n-1) = 3
        assert_eq!(centrality["A"], 1.0);
        
        // B, C, D each have degree 0 (no outgoing edges)
        assert_eq!(centrality["B"], 0.0);
    }
    
    #[test]
    fn test_clustering_coefficient() {
        let mut graph = GraphBuilder::new()
            .graph_type(GraphType::Generic)
            .build_event::<GenericNode<&'static str>, GenericEdge<f64>>()
            .unwrap();
            
        // Create a triangle: A -> B, A -> C, B -> C
        graph.add_node(GenericNode::new("A", "data")).unwrap();
        graph.add_node(GenericNode::new("B", "data")).unwrap();
        graph.add_node(GenericNode::new("C", "data")).unwrap();
        
        graph.add_edge(GenericEdge::new("A", "B", 1.0)).unwrap();
        graph.add_edge(GenericEdge::new("A", "C", 1.0)).unwrap();
        graph.add_edge(GenericEdge::new("B", "C", 1.0)).unwrap();
        
        // A's neighbors (B, C) are connected
        let coeff = node_clustering_coefficient(&graph, "A").unwrap();
        assert_eq!(coeff, 1.0);
    }
}