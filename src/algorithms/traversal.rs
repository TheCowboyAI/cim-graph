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

    fn create_linear_graph() -> TestProjection {
        // Creates: A -> B -> C
        let mut projection = create_empty_projection();

        let node_a = WorkflowNode::new("A", WorkflowNodeType::Start);
        let node_b = WorkflowNode::state("B", "Middle");
        let node_c = WorkflowNode::new("C", WorkflowNodeType::End);

        projection.nodes.insert("A".to_string(), node_a);
        projection.nodes.insert("B".to_string(), node_b);
        projection.nodes.insert("C".to_string(), node_c);

        let edge_ab = WorkflowEdge::transition("e1", "A", "B");
        let edge_bc = WorkflowEdge::transition("e2", "B", "C");

        projection.edges.insert("e1".to_string(), edge_ab);
        projection.edges.insert("e2".to_string(), edge_bc);

        projection.adjacency.insert("A".to_string(), vec!["B".to_string()]);
        projection.adjacency.insert("B".to_string(), vec!["C".to_string()]);
        projection.adjacency.insert("C".to_string(), vec![]);

        projection
    }

    fn create_branching_graph() -> TestProjection {
        // Creates:
        //     B
        //   /   \
        // A       D
        //   \   /
        //     C
        let mut projection = create_empty_projection();

        let node_a = WorkflowNode::new("A", WorkflowNodeType::Start);
        let node_b = WorkflowNode::state("B", "Branch1");
        let node_c = WorkflowNode::state("C", "Branch2");
        let node_d = WorkflowNode::new("D", WorkflowNodeType::End);

        projection.nodes.insert("A".to_string(), node_a);
        projection.nodes.insert("B".to_string(), node_b);
        projection.nodes.insert("C".to_string(), node_c);
        projection.nodes.insert("D".to_string(), node_d);

        let edge_ab = WorkflowEdge::transition("e1", "A", "B");
        let edge_ac = WorkflowEdge::transition("e2", "A", "C");
        let edge_bd = WorkflowEdge::transition("e3", "B", "D");
        let edge_cd = WorkflowEdge::transition("e4", "C", "D");

        projection.edges.insert("e1".to_string(), edge_ab);
        projection.edges.insert("e2".to_string(), edge_ac);
        projection.edges.insert("e3".to_string(), edge_bd);
        projection.edges.insert("e4".to_string(), edge_cd);

        projection.adjacency.insert("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        projection.adjacency.insert("B".to_string(), vec!["D".to_string()]);
        projection.adjacency.insert("C".to_string(), vec!["D".to_string()]);
        projection.adjacency.insert("D".to_string(), vec![]);

        projection
    }

    fn create_disconnected_graph() -> TestProjection {
        // Creates: A -> B   C -> D (two disconnected components)
        let mut projection = create_empty_projection();

        let node_a = WorkflowNode::new("A", WorkflowNodeType::Start);
        let node_b = WorkflowNode::state("B", "End1");
        let node_c = WorkflowNode::state("C", "Start2");
        let node_d = WorkflowNode::new("D", WorkflowNodeType::End);

        projection.nodes.insert("A".to_string(), node_a);
        projection.nodes.insert("B".to_string(), node_b);
        projection.nodes.insert("C".to_string(), node_c);
        projection.nodes.insert("D".to_string(), node_d);

        let edge_ab = WorkflowEdge::transition("e1", "A", "B");
        let edge_cd = WorkflowEdge::transition("e2", "C", "D");

        projection.edges.insert("e1".to_string(), edge_ab);
        projection.edges.insert("e2".to_string(), edge_cd);

        projection.adjacency.insert("A".to_string(), vec!["B".to_string()]);
        projection.adjacency.insert("B".to_string(), vec![]);
        projection.adjacency.insert("C".to_string(), vec!["D".to_string()]);
        projection.adjacency.insert("D".to_string(), vec![]);

        projection
    }

    // ========== BFS Tests ==========

    #[test]
    fn test_bfs_linear_graph() {
        let projection = create_linear_graph();
        let result = bfs(&projection, "A").unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "A");
        assert_eq!(result[1], "B");
        assert_eq!(result[2], "C");
    }

    #[test]
    fn test_bfs_branching_graph() {
        let projection = create_branching_graph();
        let result = bfs(&projection, "A").unwrap();

        assert_eq!(result.len(), 4);
        assert_eq!(result[0], "A");
        // B and C can be in either order at level 1
        assert!(result[1] == "B" || result[1] == "C");
        assert!(result[2] == "B" || result[2] == "C");
        assert_ne!(result[1], result[2]);
        assert_eq!(result[3], "D");
    }

    #[test]
    fn test_bfs_from_middle() {
        let projection = create_linear_graph();
        let result = bfs(&projection, "B").unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "B");
        assert_eq!(result[1], "C");
    }

    #[test]
    fn test_bfs_node_not_found() {
        let projection = create_linear_graph();
        let result = bfs(&projection, "X");

        assert!(result.is_err());
        match result.unwrap_err() {
            GraphError::NodeNotFound(id) => assert_eq!(id, "X"),
            _ => panic!("Expected NodeNotFound error"),
        }
    }

    #[test]
    fn test_bfs_single_node() {
        let mut projection = create_empty_projection();
        let node = WorkflowNode::new("A", WorkflowNodeType::Start);
        projection.nodes.insert("A".to_string(), node);
        projection.adjacency.insert("A".to_string(), vec![]);

        let result = bfs(&projection, "A").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "A");
    }

    #[test]
    fn test_bfs_disconnected_graph() {
        let projection = create_disconnected_graph();

        // BFS from A should only find A and B
        let result_a = bfs(&projection, "A").unwrap();
        assert_eq!(result_a.len(), 2);
        assert!(result_a.contains(&"A".to_string()));
        assert!(result_a.contains(&"B".to_string()));

        // BFS from C should only find C and D
        let result_c = bfs(&projection, "C").unwrap();
        assert_eq!(result_c.len(), 2);
        assert!(result_c.contains(&"C".to_string()));
        assert!(result_c.contains(&"D".to_string()));
    }

    // ========== DFS Tests ==========

    #[test]
    fn test_dfs_linear_graph() {
        let projection = create_linear_graph();
        let result = dfs(&projection, "A").unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "A");
        assert_eq!(result[1], "B");
        assert_eq!(result[2], "C");
    }

    #[test]
    fn test_dfs_branching_graph() {
        let projection = create_branching_graph();
        let result = dfs(&projection, "A").unwrap();

        assert_eq!(result.len(), 4);
        assert_eq!(result[0], "A");
        // DFS will go deep first, so either B->D or C->D path
        assert!(result.contains(&"A".to_string()));
        assert!(result.contains(&"B".to_string()));
        assert!(result.contains(&"C".to_string()));
        assert!(result.contains(&"D".to_string()));
    }

    #[test]
    fn test_dfs_from_middle() {
        let projection = create_linear_graph();
        let result = dfs(&projection, "B").unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "B");
        assert_eq!(result[1], "C");
    }

    #[test]
    fn test_dfs_node_not_found() {
        let projection = create_linear_graph();
        let result = dfs(&projection, "Z");

        assert!(result.is_err());
        match result.unwrap_err() {
            GraphError::NodeNotFound(id) => assert_eq!(id, "Z"),
            _ => panic!("Expected NodeNotFound error"),
        }
    }

    #[test]
    fn test_dfs_single_node() {
        let mut projection = create_empty_projection();
        let node = WorkflowNode::new("X", WorkflowNodeType::Start);
        projection.nodes.insert("X".to_string(), node);
        projection.adjacency.insert("X".to_string(), vec![]);

        let result = dfs(&projection, "X").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "X");
    }

    #[test]
    fn test_dfs_disconnected_graph() {
        let projection = create_disconnected_graph();

        // DFS from A should only find A and B
        let result_a = dfs(&projection, "A").unwrap();
        assert_eq!(result_a.len(), 2);
        assert!(result_a.contains(&"A".to_string()));
        assert!(result_a.contains(&"B".to_string()));
    }

    // ========== Topological Sort Tests ==========

    #[test]
    fn test_topological_sort_linear() {
        let projection = create_linear_graph();
        let result = topological_sort(&projection).unwrap();

        assert_eq!(result.len(), 3);
        // A must come before B, B must come before C
        let pos_a = result.iter().position(|x| x == "A").unwrap();
        let pos_b = result.iter().position(|x| x == "B").unwrap();
        let pos_c = result.iter().position(|x| x == "C").unwrap();

        assert!(pos_a < pos_b);
        assert!(pos_b < pos_c);
    }

    #[test]
    fn test_topological_sort_branching() {
        let projection = create_branching_graph();
        let result = topological_sort(&projection).unwrap();

        assert_eq!(result.len(), 4);

        // A must come before B and C
        let pos_a = result.iter().position(|x| x == "A").unwrap();
        let pos_b = result.iter().position(|x| x == "B").unwrap();
        let pos_c = result.iter().position(|x| x == "C").unwrap();
        let pos_d = result.iter().position(|x| x == "D").unwrap();

        assert!(pos_a < pos_b);
        assert!(pos_a < pos_c);
        // B and C must come before D
        assert!(pos_b < pos_d);
        assert!(pos_c < pos_d);
    }

    #[test]
    fn test_topological_sort_empty_graph() {
        let projection = create_empty_projection();
        let result = topological_sort(&projection).unwrap();

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_topological_sort_single_node() {
        let mut projection = create_empty_projection();
        let node = WorkflowNode::new("A", WorkflowNodeType::Start);
        projection.nodes.insert("A".to_string(), node);
        projection.adjacency.insert("A".to_string(), vec![]);

        let result = topological_sort(&projection).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "A");
    }

    #[test]
    fn test_topological_sort_disconnected() {
        let projection = create_disconnected_graph();
        let result = topological_sort(&projection).unwrap();

        assert_eq!(result.len(), 4);

        // A must come before B
        let pos_a = result.iter().position(|x| x == "A").unwrap();
        let pos_b = result.iter().position(|x| x == "B").unwrap();
        assert!(pos_a < pos_b);

        // C must come before D
        let pos_c = result.iter().position(|x| x == "C").unwrap();
        let pos_d = result.iter().position(|x| x == "D").unwrap();
        assert!(pos_c < pos_d);
    }

    // ========== Additional Coverage Tests ==========

    #[test]
    fn test_bfs_empty_graph() {
        let projection = create_empty_projection();
        let result = bfs(&projection, "A");
        assert!(result.is_err());
    }

    #[test]
    fn test_dfs_empty_graph() {
        let projection = create_empty_projection();
        let result = dfs(&projection, "A");
        assert!(result.is_err());
    }

    #[test]
    fn test_bfs_leaf_node() {
        let projection = create_linear_graph();
        // Starting from end node C
        let result = bfs(&projection, "C").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "C");
    }

    #[test]
    fn test_dfs_leaf_node() {
        let projection = create_linear_graph();
        // Starting from end node C
        let result = dfs(&projection, "C").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "C");
    }

    #[test]
    fn test_bfs_traversal_order() {
        // Test that BFS visits nodes in level order
        // Diamond graph: A -> B, A -> C, B -> D, C -> D
        let projection = create_branching_graph();
        let result = bfs(&projection, "A").unwrap();

        assert_eq!(result.len(), 4);
        assert_eq!(result[0], "A"); // Level 0
        // B and C at level 1 (order depends on adjacency)
        assert!(result[1] == "B" || result[1] == "C");
        assert!(result[2] == "B" || result[2] == "C");
        assert_eq!(result[3], "D"); // Level 2
    }

    #[test]
    fn test_dfs_traversal_order() {
        // DFS goes deep first
        let projection = create_linear_graph();
        let result = dfs(&projection, "A").unwrap();

        // In a linear graph, DFS and BFS produce same result
        assert_eq!(result, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_bfs_with_cycle() {
        // Create a graph with a cycle: A -> B -> C -> A
        let mut projection = create_empty_projection();

        for id in ["A", "B", "C"] {
            let node = WorkflowNode::new(id, WorkflowNodeType::Start);
            projection.nodes.insert(id.to_string(), node);
        }

        projection.adjacency.insert("A".to_string(), vec!["B".to_string()]);
        projection.adjacency.insert("B".to_string(), vec!["C".to_string()]);
        projection.adjacency.insert("C".to_string(), vec!["A".to_string()]);

        let result = bfs(&projection, "A").unwrap();

        // Should visit each node exactly once despite cycle
        assert_eq!(result.len(), 3);
        let mut sorted = result.clone();
        sorted.sort();
        assert_eq!(sorted, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_dfs_with_cycle() {
        // Create a graph with a cycle
        let mut projection = create_empty_projection();

        for id in ["A", "B", "C"] {
            let node = WorkflowNode::new(id, WorkflowNodeType::Start);
            projection.nodes.insert(id.to_string(), node);
        }

        projection.adjacency.insert("A".to_string(), vec!["B".to_string()]);
        projection.adjacency.insert("B".to_string(), vec!["C".to_string()]);
        projection.adjacency.insert("C".to_string(), vec!["A".to_string()]);

        let result = dfs(&projection, "A").unwrap();

        // Should visit each node exactly once
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_topological_sort_with_cycle_fails() {
        // Create a graph with a cycle: A -> B -> C -> A
        let mut projection = create_empty_projection();

        for id in ["A", "B", "C"] {
            let node = WorkflowNode::new(id, WorkflowNodeType::Start);
            projection.nodes.insert(id.to_string(), node);
        }

        projection.adjacency.insert("A".to_string(), vec!["B".to_string()]);
        projection.adjacency.insert("B".to_string(), vec!["C".to_string()]);
        projection.adjacency.insert("C".to_string(), vec!["A".to_string()]);

        let result = topological_sort(&projection);
        assert!(result.is_err());

        match result.unwrap_err() {
            crate::error::GraphError::InvalidOperation(msg) => {
                assert!(msg.contains("cycles") || msg.contains("cycle"));
            }
            _ => panic!("Expected InvalidOperation error"),
        }
    }

    #[test]
    fn test_topological_sort_complex_dag() {
        // Create a more complex DAG:
        //     B -> D
        //   /   \
        // A       F
        //   \   /
        //     C -> E
        let mut projection = create_empty_projection();

        for id in ["A", "B", "C", "D", "E", "F"] {
            let node = WorkflowNode::new(id, WorkflowNodeType::Start);
            projection.nodes.insert(id.to_string(), node);
        }

        projection.adjacency.insert("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        projection.adjacency.insert("B".to_string(), vec!["D".to_string(), "F".to_string()]);
        projection.adjacency.insert("C".to_string(), vec!["E".to_string(), "F".to_string()]);
        projection.adjacency.insert("D".to_string(), vec![]);
        projection.adjacency.insert("E".to_string(), vec![]);
        projection.adjacency.insert("F".to_string(), vec![]);

        let result = topological_sort(&projection).unwrap();
        assert_eq!(result.len(), 6);

        // A must come before B and C
        let pos_a = result.iter().position(|x| x == "A").unwrap();
        let pos_b = result.iter().position(|x| x == "B").unwrap();
        let pos_c = result.iter().position(|x| x == "C").unwrap();
        assert!(pos_a < pos_b);
        assert!(pos_a < pos_c);

        // B must come before D and F
        let pos_d = result.iter().position(|x| x == "D").unwrap();
        let pos_f = result.iter().position(|x| x == "F").unwrap();
        assert!(pos_b < pos_d);
        assert!(pos_b < pos_f);

        // C must come before E and F
        let pos_e = result.iter().position(|x| x == "E").unwrap();
        assert!(pos_c < pos_e);
        assert!(pos_c < pos_f);
    }

    #[test]
    fn test_bfs_wide_graph() {
        // A -> B, C, D, E, F (5 neighbors)
        let mut projection = create_empty_projection();

        for id in ["A", "B", "C", "D", "E", "F"] {
            let node = WorkflowNode::new(id, WorkflowNodeType::Start);
            projection.nodes.insert(id.to_string(), node);
        }

        projection.adjacency.insert("A".to_string(),
            vec!["B".to_string(), "C".to_string(), "D".to_string(), "E".to_string(), "F".to_string()]);
        projection.adjacency.insert("B".to_string(), vec![]);
        projection.adjacency.insert("C".to_string(), vec![]);
        projection.adjacency.insert("D".to_string(), vec![]);
        projection.adjacency.insert("E".to_string(), vec![]);
        projection.adjacency.insert("F".to_string(), vec![]);

        let result = bfs(&projection, "A").unwrap();
        assert_eq!(result.len(), 6);
        assert_eq!(result[0], "A");
    }

    #[test]
    fn test_dfs_deep_graph() {
        // A -> B -> C -> D -> E -> F (deep chain)
        let ids = vec!["A", "B", "C", "D", "E", "F"];
        let mut projection = create_empty_projection();

        for id in &ids {
            let node = WorkflowNode::new(*id, WorkflowNodeType::Start);
            projection.nodes.insert(id.to_string(), node);
        }

        for i in 0..ids.len() - 1 {
            projection.adjacency.insert(ids[i].to_string(), vec![ids[i + 1].to_string()]);
        }
        projection.adjacency.insert("F".to_string(), vec![]);

        let result = dfs(&projection, "A").unwrap();
        assert_eq!(result, ids.iter().map(|s| s.to_string()).collect::<Vec<_>>());
    }

    #[test]
    fn test_topological_sort_parallel_chains() {
        // Two parallel chains: A -> B -> C and D -> E -> F
        let mut projection = create_empty_projection();

        for id in ["A", "B", "C", "D", "E", "F"] {
            let node = WorkflowNode::new(id, WorkflowNodeType::Start);
            projection.nodes.insert(id.to_string(), node);
        }

        projection.adjacency.insert("A".to_string(), vec!["B".to_string()]);
        projection.adjacency.insert("B".to_string(), vec!["C".to_string()]);
        projection.adjacency.insert("C".to_string(), vec![]);
        projection.adjacency.insert("D".to_string(), vec!["E".to_string()]);
        projection.adjacency.insert("E".to_string(), vec!["F".to_string()]);
        projection.adjacency.insert("F".to_string(), vec![]);

        let result = topological_sort(&projection).unwrap();
        assert_eq!(result.len(), 6);

        // Verify ordering within each chain
        let pos_a = result.iter().position(|x| x == "A").unwrap();
        let pos_b = result.iter().position(|x| x == "B").unwrap();
        let pos_c = result.iter().position(|x| x == "C").unwrap();
        assert!(pos_a < pos_b && pos_b < pos_c);

        let pos_d = result.iter().position(|x| x == "D").unwrap();
        let pos_e = result.iter().position(|x| x == "E").unwrap();
        let pos_f = result.iter().position(|x| x == "F").unwrap();
        assert!(pos_d < pos_e && pos_e < pos_f);
    }
}