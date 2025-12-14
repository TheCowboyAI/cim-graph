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

    fn create_diamond_graph() -> TestProjection {
        // Creates:
        //     B
        //   /   \
        // A       D
        //   \   /
        //     C
        let mut projection = create_empty_projection();

        for id in ["A", "B", "C", "D"] {
            let node = WorkflowNode::new(id, WorkflowNodeType::Start);
            projection.nodes.insert(id.to_string(), node);
        }

        projection.adjacency.insert("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        projection.adjacency.insert("B".to_string(), vec!["D".to_string()]);
        projection.adjacency.insert("C".to_string(), vec!["D".to_string()]);
        projection.adjacency.insert("D".to_string(), vec![]);

        projection
    }

    fn create_complex_graph() -> TestProjection {
        // Creates:
        // A -> B -> C
        // |    |    |
        // v    v    v
        // D -> E -> F
        let mut projection = create_empty_projection();

        for id in ["A", "B", "C", "D", "E", "F"] {
            let node = WorkflowNode::new(id, WorkflowNodeType::Start);
            projection.nodes.insert(id.to_string(), node);
        }

        projection.adjacency.insert("A".to_string(), vec!["B".to_string(), "D".to_string()]);
        projection.adjacency.insert("B".to_string(), vec!["C".to_string(), "E".to_string()]);
        projection.adjacency.insert("C".to_string(), vec!["F".to_string()]);
        projection.adjacency.insert("D".to_string(), vec!["E".to_string()]);
        projection.adjacency.insert("E".to_string(), vec!["F".to_string()]);
        projection.adjacency.insert("F".to_string(), vec![]);

        projection
    }

    // ========== shortest_path Tests ==========

    #[test]
    fn test_shortest_path_linear_graph() {
        let projection = create_linear_graph();
        let result = shortest_path(&projection, "A", "D").unwrap();

        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(path, vec!["A", "B", "C", "D"]);
    }

    #[test]
    fn test_shortest_path_diamond_graph() {
        let projection = create_diamond_graph();
        let result = shortest_path(&projection, "A", "D").unwrap();

        assert!(result.is_some());
        let path = result.unwrap();
        // Path should be length 3 (A -> B/C -> D)
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], "A");
        assert_eq!(path[2], "D");
        assert!(path[1] == "B" || path[1] == "C");
    }

    #[test]
    fn test_shortest_path_same_node() {
        let projection = create_linear_graph();
        let result = shortest_path(&projection, "A", "A").unwrap();

        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(path, vec!["A"]);
    }

    #[test]
    fn test_shortest_path_adjacent_nodes() {
        let projection = create_linear_graph();
        let result = shortest_path(&projection, "A", "B").unwrap();

        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(path, vec!["A", "B"]);
    }

    #[test]
    fn test_shortest_path_no_path() {
        let projection = create_linear_graph();
        // D has no outgoing edges, so there's no path from D to A
        let result = shortest_path(&projection, "D", "A").unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_shortest_path_source_not_found() {
        let projection = create_linear_graph();
        let result = shortest_path(&projection, "X", "A");

        assert!(result.is_err());
        match result.unwrap_err() {
            GraphError::NodeNotFound(id) => assert_eq!(id, "X"),
            _ => panic!("Expected NodeNotFound error"),
        }
    }

    #[test]
    fn test_shortest_path_target_not_found() {
        let projection = create_linear_graph();
        let result = shortest_path(&projection, "A", "X");

        assert!(result.is_err());
        match result.unwrap_err() {
            GraphError::NodeNotFound(id) => assert_eq!(id, "X"),
            _ => panic!("Expected NodeNotFound error"),
        }
    }

    #[test]
    fn test_shortest_path_complex_graph() {
        let projection = create_complex_graph();

        // A to F: shortest path is A -> B -> C -> F (length 4)
        // or A -> B -> E -> F or A -> D -> E -> F (all length 4)
        let result = shortest_path(&projection, "A", "F").unwrap();
        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(path.len(), 4);
        assert_eq!(path[0], "A");
        assert_eq!(path[3], "F");
    }

    // ========== all_paths Tests ==========

    #[test]
    fn test_all_paths_linear_graph() {
        let projection = create_linear_graph();
        let result = all_paths(&projection, "A", "D").unwrap();

        // Only one path in a linear graph
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], vec!["A", "B", "C", "D"]);
    }

    #[test]
    fn test_all_paths_diamond_graph() {
        let projection = create_diamond_graph();
        let result = all_paths(&projection, "A", "D").unwrap();

        // Two paths: A->B->D and A->C->D
        assert_eq!(result.len(), 2);

        let path1 = vec!["A".to_string(), "B".to_string(), "D".to_string()];
        let path2 = vec!["A".to_string(), "C".to_string(), "D".to_string()];

        assert!(result.contains(&path1));
        assert!(result.contains(&path2));
    }

    #[test]
    fn test_all_paths_same_node() {
        let projection = create_linear_graph();
        let result = all_paths(&projection, "A", "A").unwrap();

        // Path from A to A is just [A]
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], vec!["A"]);
    }

    #[test]
    fn test_all_paths_no_path() {
        let projection = create_linear_graph();
        // D has no outgoing edges
        let result = all_paths(&projection, "D", "A").unwrap();

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_all_paths_source_not_found() {
        let projection = create_linear_graph();
        let result = all_paths(&projection, "X", "A");

        assert!(result.is_err());
        match result.unwrap_err() {
            GraphError::NodeNotFound(id) => assert_eq!(id, "X"),
            _ => panic!("Expected NodeNotFound error"),
        }
    }

    #[test]
    fn test_all_paths_target_not_found() {
        let projection = create_linear_graph();
        let result = all_paths(&projection, "A", "X");

        assert!(result.is_err());
        match result.unwrap_err() {
            GraphError::NodeNotFound(id) => assert_eq!(id, "X"),
            _ => panic!("Expected NodeNotFound error"),
        }
    }

    #[test]
    fn test_all_paths_complex_graph() {
        let projection = create_complex_graph();
        let result = all_paths(&projection, "A", "F").unwrap();

        // Multiple paths from A to F
        // A -> B -> C -> F
        // A -> B -> E -> F
        // A -> D -> E -> F
        assert!(result.len() >= 3);

        // All paths should start with A and end with F
        for path in &result {
            assert_eq!(path[0], "A");
            assert_eq!(*path.last().unwrap(), "F");
        }
    }

    #[test]
    fn test_all_paths_adjacent_nodes() {
        let projection = create_linear_graph();
        let result = all_paths(&projection, "A", "B").unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], vec!["A", "B"]);
    }

    // ========== Additional Coverage Tests ==========

    #[test]
    fn test_shortest_path_empty_graph() {
        let projection = create_empty_projection();
        let result = shortest_path(&projection, "A", "B");
        assert!(result.is_err());
    }

    #[test]
    fn test_shortest_path_single_node() {
        let mut projection = create_empty_projection();
        let node = WorkflowNode::new("A", WorkflowNodeType::Start);
        projection.nodes.insert("A".to_string(), node);
        projection.adjacency.insert("A".to_string(), vec![]);

        // Path to self should work
        let result = shortest_path(&projection, "A", "A").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), vec!["A"]);

        // Path to non-existent node should fail
        let result = shortest_path(&projection, "A", "B");
        assert!(result.is_err());
    }

    #[test]
    fn test_shortest_path_with_multiple_levels() {
        // Creates a graph with multiple levels:
        // A -> B -> D -> E
        //      |
        //      v
        //      C
        let mut projection = create_empty_projection();
        for id in ["A", "B", "C", "D", "E"] {
            let node = WorkflowNode::new(id, WorkflowNodeType::Start);
            projection.nodes.insert(id.to_string(), node);
        }

        projection.adjacency.insert("A".to_string(), vec!["B".to_string()]);
        projection.adjacency.insert("B".to_string(), vec!["C".to_string(), "D".to_string()]);
        projection.adjacency.insert("C".to_string(), vec![]);
        projection.adjacency.insert("D".to_string(), vec!["E".to_string()]);
        projection.adjacency.insert("E".to_string(), vec![]);

        // A to E should go through B and D
        let result = shortest_path(&projection, "A", "E").unwrap();
        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(path.len(), 4);
        assert_eq!(path[0], "A");
        assert_eq!(path[path.len() - 1], "E");
    }

    #[test]
    fn test_all_paths_with_fan_out() {
        // Creates:
        //     B -> E
        //   /
        // A -> C -> E
        //   \
        //     D -> E
        let mut projection = create_empty_projection();
        for id in ["A", "B", "C", "D", "E"] {
            let node = WorkflowNode::new(id, WorkflowNodeType::Start);
            projection.nodes.insert(id.to_string(), node);
        }

        projection.adjacency.insert("A".to_string(), vec!["B".to_string(), "C".to_string(), "D".to_string()]);
        projection.adjacency.insert("B".to_string(), vec!["E".to_string()]);
        projection.adjacency.insert("C".to_string(), vec!["E".to_string()]);
        projection.adjacency.insert("D".to_string(), vec!["E".to_string()]);
        projection.adjacency.insert("E".to_string(), vec![]);

        let result = all_paths(&projection, "A", "E").unwrap();
        assert_eq!(result.len(), 3); // A->B->E, A->C->E, A->D->E

        // All paths should be length 3
        for path in &result {
            assert_eq!(path.len(), 3);
            assert_eq!(path[0], "A");
            assert_eq!(path[2], "E");
        }
    }

    #[test]
    fn test_all_paths_empty_graph() {
        let projection = create_empty_projection();
        let result = all_paths(&projection, "A", "B");
        assert!(result.is_err());
    }

    #[test]
    fn test_all_paths_single_node_to_self() {
        let mut projection = create_empty_projection();
        let node = WorkflowNode::new("A", WorkflowNodeType::Start);
        projection.nodes.insert("A".to_string(), node);
        projection.adjacency.insert("A".to_string(), vec![]);

        let result = all_paths(&projection, "A", "A").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], vec!["A"]);
    }

    #[test]
    fn test_shortest_path_with_cycle_avoidance() {
        // Creates a graph where there's a cycle:
        // A -> B -> C -> D
        //      ^       |
        //      |_______|
        // But shortest path shouldn't loop
        let mut projection = create_empty_projection();
        for id in ["A", "B", "C", "D"] {
            let node = WorkflowNode::new(id, WorkflowNodeType::Start);
            projection.nodes.insert(id.to_string(), node);
        }

        projection.adjacency.insert("A".to_string(), vec!["B".to_string()]);
        projection.adjacency.insert("B".to_string(), vec!["C".to_string()]);
        projection.adjacency.insert("C".to_string(), vec!["D".to_string(), "B".to_string()]);
        projection.adjacency.insert("D".to_string(), vec![]);

        let result = shortest_path(&projection, "A", "D").unwrap();
        assert!(result.is_some());
        let path = result.unwrap();
        // BFS will find shortest path A->B->C->D = 4 nodes
        assert_eq!(path.len(), 4);
        assert_eq!(path, vec!["A", "B", "C", "D"]);
    }

    #[test]
    fn test_all_paths_with_partial_connectivity() {
        // A -> B
        // C -> D (disconnected from A,B)
        let mut projection = create_empty_projection();
        for id in ["A", "B", "C", "D"] {
            let node = WorkflowNode::new(id, WorkflowNodeType::Start);
            projection.nodes.insert(id.to_string(), node);
        }

        projection.adjacency.insert("A".to_string(), vec!["B".to_string()]);
        projection.adjacency.insert("B".to_string(), vec![]);
        projection.adjacency.insert("C".to_string(), vec!["D".to_string()]);
        projection.adjacency.insert("D".to_string(), vec![]);

        // No path from A to D
        let result = all_paths(&projection, "A", "D").unwrap();
        assert_eq!(result.len(), 0);

        // Path from A to B exists
        let result = all_paths(&projection, "A", "B").unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_shortest_path_bidirectional_edges() {
        // A <-> B <-> C
        let mut projection = create_empty_projection();
        for id in ["A", "B", "C"] {
            let node = WorkflowNode::new(id, WorkflowNodeType::Start);
            projection.nodes.insert(id.to_string(), node);
        }

        projection.adjacency.insert("A".to_string(), vec!["B".to_string()]);
        projection.adjacency.insert("B".to_string(), vec!["A".to_string(), "C".to_string()]);
        projection.adjacency.insert("C".to_string(), vec!["B".to_string()]);

        // Forward path
        let result = shortest_path(&projection, "A", "C").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 3); // A -> B -> C

        // Reverse path
        let result = shortest_path(&projection, "C", "A").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 3); // C -> B -> A
    }

    #[test]
    fn test_all_paths_with_long_chain() {
        // A -> B -> C -> D -> E -> F -> G
        let ids: Vec<&str> = vec!["A", "B", "C", "D", "E", "F", "G"];
        let mut projection = create_empty_projection();

        for id in &ids {
            let node = WorkflowNode::new(*id, WorkflowNodeType::Start);
            projection.nodes.insert(id.to_string(), node);
        }

        for i in 0..ids.len() - 1 {
            projection.adjacency.insert(
                ids[i].to_string(),
                vec![ids[i + 1].to_string()]
            );
        }
        projection.adjacency.insert("G".to_string(), vec![]);

        let result = all_paths(&projection, "A", "G").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 7);
    }
}