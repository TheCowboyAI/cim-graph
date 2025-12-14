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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphs::workflow::{WorkflowNode, WorkflowEdge, WorkflowNodeType};
    use crate::core::projection_engine::GenericGraphProjection;
    use crate::core::GraphType;
    use uuid::Uuid;

    type TestProjection = GenericGraphProjection<WorkflowNode, WorkflowEdge>;

    fn create_empty_projection() -> TestProjection {
        GenericGraphProjection::new(Uuid::new_v4(), GraphType::Generic)
    }

    fn create_star_graph() -> TestProjection {
        // Creates a star graph: A is the center, connected to B, C, D, E
        // A -> B, A -> C, A -> D, A -> E
        let mut projection = create_empty_projection();

        for id in ["A", "B", "C", "D", "E"] {
            let node = WorkflowNode::new(id, WorkflowNodeType::Start);
            projection.nodes.insert(id.to_string(), node);
        }

        projection.adjacency.insert("A".to_string(), vec![
            "B".to_string(), "C".to_string(), "D".to_string(), "E".to_string()
        ]);
        projection.adjacency.insert("B".to_string(), vec![]);
        projection.adjacency.insert("C".to_string(), vec![]);
        projection.adjacency.insert("D".to_string(), vec![]);
        projection.adjacency.insert("E".to_string(), vec![]);

        projection
    }

    fn create_linear_graph() -> TestProjection {
        // Creates: A -> B -> C -> D
        let mut projection = create_empty_projection();

        for id in ["A", "B", "C", "D"] {
            let node = WorkflowNode::new(id, WorkflowNodeType::Start);
            projection.nodes.insert(id.to_string(), node);
        }

        projection.adjacency.insert("A".to_string(), vec!["B".to_string()]);
        projection.adjacency.insert("B".to_string(), vec!["C".to_string()]);
        projection.adjacency.insert("C".to_string(), vec!["D".to_string()]);
        projection.adjacency.insert("D".to_string(), vec![]);

        projection
    }

    fn create_complete_triangle() -> TestProjection {
        // Creates a complete triangle: A <-> B <-> C <-> A
        let mut projection = create_empty_projection();

        for id in ["A", "B", "C"] {
            let node = WorkflowNode::new(id, WorkflowNodeType::Start);
            projection.nodes.insert(id.to_string(), node);
        }

        // Create edges for complete graph
        let edge_ab = WorkflowEdge::transition("e1", "A", "B");
        let edge_bc = WorkflowEdge::transition("e2", "B", "C");
        let edge_ca = WorkflowEdge::transition("e3", "C", "A");
        let edge_ba = WorkflowEdge::transition("e4", "B", "A");
        let edge_cb = WorkflowEdge::transition("e5", "C", "B");
        let edge_ac = WorkflowEdge::transition("e6", "A", "C");

        projection.edges.insert("e1".to_string(), edge_ab);
        projection.edges.insert("e2".to_string(), edge_bc);
        projection.edges.insert("e3".to_string(), edge_ca);
        projection.edges.insert("e4".to_string(), edge_ba);
        projection.edges.insert("e5".to_string(), edge_cb);
        projection.edges.insert("e6".to_string(), edge_ac);

        projection.adjacency.insert("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        projection.adjacency.insert("B".to_string(), vec!["A".to_string(), "C".to_string()]);
        projection.adjacency.insert("C".to_string(), vec!["A".to_string(), "B".to_string()]);

        projection
    }

    fn create_single_node() -> TestProjection {
        let mut projection = create_empty_projection();
        let node = WorkflowNode::new("A", WorkflowNodeType::Start);
        projection.nodes.insert("A".to_string(), node);
        projection.adjacency.insert("A".to_string(), vec![]);
        projection
    }

    // ========== Centrality Tests ==========

    #[test]
    fn test_centrality_star_graph() {
        let projection = create_star_graph();
        let result = centrality(&projection).unwrap();

        // A has the highest degree (4 neighbors)
        // B, C, D, E have degree 0 (no outgoing edges)
        assert!(result.contains_key("A"));
        assert_eq!(*result.get("A").unwrap(), 1.0); // Normalized to 1.0 (max)

        for id in ["B", "C", "D", "E"] {
            assert!(result.contains_key(id));
            assert_eq!(*result.get(id).unwrap(), 0.0);
        }
    }

    #[test]
    fn test_centrality_linear_graph() {
        let projection = create_linear_graph();
        let result = centrality(&projection).unwrap();

        // Each node (except D) has exactly 1 neighbor
        // So normalized centrality should be 1.0 for A, B, C and 0.0 for D
        assert!(result.contains_key("A"));
        assert!(result.contains_key("B"));
        assert!(result.contains_key("C"));
        assert!(result.contains_key("D"));

        // A, B, C all have degree 1 (normalized to 1.0)
        assert_eq!(*result.get("A").unwrap(), 1.0);
        assert_eq!(*result.get("B").unwrap(), 1.0);
        assert_eq!(*result.get("C").unwrap(), 1.0);
        // D has degree 0
        assert_eq!(*result.get("D").unwrap(), 0.0);
    }

    #[test]
    fn test_centrality_empty_graph() {
        let projection = create_empty_projection();
        let result = centrality(&projection).unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn test_centrality_single_node() {
        let projection = create_single_node();
        let result = centrality(&projection).unwrap();

        assert_eq!(result.len(), 1);
        assert!(result.contains_key("A"));
        // Single node with no neighbors has degree 0
        // When max is 0, no normalization happens (or all are 0)
        assert_eq!(*result.get("A").unwrap(), 0.0);
    }

    #[test]
    fn test_centrality_complete_triangle() {
        let projection = create_complete_triangle();
        let result = centrality(&projection).unwrap();

        // All nodes in complete graph have same degree
        assert_eq!(result.len(), 3);

        // All should have normalized centrality of 1.0 (all are equal and max)
        for id in ["A", "B", "C"] {
            assert!(result.contains_key(id));
            assert_eq!(*result.get(id).unwrap(), 1.0);
        }
    }

    // ========== Clustering Coefficient Tests ==========

    #[test]
    fn test_clustering_coefficient_complete_triangle() {
        let projection = create_complete_triangle();
        let result = clustering_coefficient(&projection).unwrap();

        // In a complete triangle, all possible edges between neighbors exist
        // So clustering coefficient should be 1.0
        assert!((result - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_clustering_coefficient_star_graph() {
        let projection = create_star_graph();
        let result = clustering_coefficient(&projection).unwrap();

        // In a star graph, the center node has 4 neighbors
        // but there are no edges between the neighbors
        // Leaf nodes have 0 neighbors (degree < 2), so they don't count
        // Center has clustering = 0 (no edges between neighbors)
        // But actually center has neighbors B, C, D, E with no edges between them
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_clustering_coefficient_empty_graph() {
        let projection = create_empty_projection();
        let result = clustering_coefficient(&projection).unwrap();

        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_clustering_coefficient_single_node() {
        let projection = create_single_node();
        let result = clustering_coefficient(&projection).unwrap();

        // Single node has no neighbors, so coefficient is 0
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_clustering_coefficient_linear_graph() {
        let projection = create_linear_graph();
        let result = clustering_coefficient(&projection).unwrap();

        // Linear graph: no node has 2+ neighbors (all have 1 or 0)
        // So clustering coefficient is 0
        assert_eq!(result, 0.0);
    }
}