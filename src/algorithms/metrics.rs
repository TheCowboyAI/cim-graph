//! Graph metrics algorithms for projections

use crate::core::GraphProjection;
use crate::{Node, error::Result};
use std::collections::HashMap;

/// Calculate centrality measures for nodes
pub fn centrality<P: GraphProjection>(projection: &P) -> Result<HashMap<String, f64>> 
where
    P::Node: Node,
{
    let mut centrality_scores = HashMap::new();
    
    // Simple degree centrality
    for node in projection.nodes() {
        let node_id = node.id();
        let degree = projection.neighbors(&node_id).len() as f64;
        centrality_scores.insert(node_id, degree);
    }
    
    // Normalize by max degree
    if let Some(&max_degree) = centrality_scores.values().max_by(|a, b| a.partial_cmp(b).unwrap()) {
        if max_degree > 0.0 {
            for score in centrality_scores.values_mut() {
                *score /= max_degree;
            }
        }
    }
    
    Ok(centrality_scores)
}

/// Calculate clustering coefficient
pub fn clustering_coefficient<P: GraphProjection>(projection: &P) -> Result<f64> 
where
    P::Node: Node,
{
    let nodes = projection.nodes();
    if nodes.is_empty() {
        return Ok(0.0);
    }
    
    let mut total_coefficient = 0.0;
    let mut count = 0;
    
    for node in nodes {
        let node_id = node.id();
        let neighbors = projection.neighbors(&node_id);
        
        if neighbors.len() < 2 {
            continue;
        }
        
        // Count edges between neighbors
        let mut edge_count = 0;
        for i in 0..neighbors.len() {
            for j in (i + 1)..neighbors.len() {
                if !projection.edges_between(neighbors[i], neighbors[j]).is_empty() {
                    edge_count += 1;
                }
            }
        }
        
        // Calculate local clustering coefficient
        let possible_edges = (neighbors.len() * (neighbors.len() - 1)) / 2;
        if possible_edges > 0 {
            let local_coefficient = edge_count as f64 / possible_edges as f64;
            total_coefficient += local_coefficient;
            count += 1;
        }
    }
    
    if count > 0 {
        Ok(total_coefficient / count as f64)
    } else {
        Ok(0.0)
    }
}