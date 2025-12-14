//! Graph traversal algorithms for projections

use crate::core::{GraphProjection, Node};
use crate::error::{GraphError, Result};
use std::collections::{HashMap, HashSet, VecDeque};

// ============================================================================
// Traversal State Types
// ============================================================================

/// Result of a level-order BFS traversal
#[derive(Debug, Clone, PartialEq)]
pub struct LevelOrderResult {
    /// Nodes organized by their level (distance from start)
    pub levels: Vec<Vec<String>>,
    /// Total number of nodes visited
    pub total_nodes: usize,
    /// Maximum depth reached
    pub max_depth: usize,
}

/// Visit state for cycle detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VisitState {
    Unvisited,
    InProgress,
    Completed,
}

/// Result of connected components analysis
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectedComponentsResult {
    /// Each component as a list of node IDs
    pub components: Vec<Vec<String>>,
    /// Number of components
    pub count: usize,
    /// Size of the largest component
    pub largest_size: usize,
}

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

// ============================================================================
// Extended Traversal Algorithms
// ============================================================================

/// Depth-first search with post-order traversal
///
/// Visits children before parents (useful for dependency resolution).
pub fn dfs_postorder<P: GraphProjection>(projection: &P, start: &str) -> Result<Vec<String>> {
    if projection.get_node(start).is_none() {
        return Err(GraphError::NodeNotFound(start.to_string()));
    }

    let mut visited = HashSet::new();
    let mut result = Vec::new();

    dfs_postorder_helper(projection, start, &mut visited, &mut result);

    Ok(result)
}

fn dfs_postorder_helper<P: GraphProjection>(
    projection: &P,
    current: &str,
    visited: &mut HashSet<String>,
    result: &mut Vec<String>,
) {
    if visited.contains(current) {
        return;
    }

    visited.insert(current.to_string());

    // Visit all children first
    for neighbor in projection.neighbors(current) {
        if !visited.contains(neighbor) {
            dfs_postorder_helper(projection, neighbor, visited, result);
        }
    }

    // Then add current node (post-order)
    result.push(current.to_string());
}

/// Level-order BFS traversal that returns nodes grouped by their level
pub fn bfs_level_order<P: GraphProjection>(projection: &P, start: &str) -> Result<LevelOrderResult> {
    if projection.get_node(start).is_none() {
        return Err(GraphError::NodeNotFound(start.to_string()));
    }

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut levels: Vec<Vec<String>> = Vec::new();

    queue.push_back((start.to_string(), 0usize));
    visited.insert(start.to_string());

    while let Some((current, level)) = queue.pop_front() {
        // Ensure we have a vector for this level
        while levels.len() <= level {
            levels.push(Vec::new());
        }

        levels[level].push(current.clone());

        for neighbor in projection.neighbors(&current) {
            if !visited.contains(neighbor) {
                visited.insert(neighbor.to_string());
                queue.push_back((neighbor.to_string(), level + 1));
            }
        }
    }

    let total_nodes = levels.iter().map(|l| l.len()).sum();
    let max_depth = if levels.is_empty() { 0 } else { levels.len() - 1 };

    Ok(LevelOrderResult {
        levels,
        total_nodes,
        max_depth,
    })
}

/// Detect if graph contains a cycle
pub fn has_cycle<P: GraphProjection>(projection: &P) -> bool
where
    P::Node: Node,
{
    let nodes = projection.nodes();
    let mut state: HashMap<String, VisitState> = HashMap::new();

    for node in &nodes {
        state.insert(node.id(), VisitState::Unvisited);
    }

    for node in &nodes {
        let node_id = node.id();
        if state.get(&node_id) == Some(&VisitState::Unvisited) {
            if has_cycle_dfs(projection, &node_id, &mut state) {
                return true;
            }
        }
    }

    false
}

fn has_cycle_dfs<P: GraphProjection>(
    projection: &P,
    current: &str,
    state: &mut HashMap<String, VisitState>,
) -> bool {
    state.insert(current.to_string(), VisitState::InProgress);

    for neighbor in projection.neighbors(current) {
        match state.get(neighbor) {
            Some(VisitState::InProgress) => return true, // Back edge found
            Some(VisitState::Unvisited) => {
                if has_cycle_dfs(projection, neighbor, state) {
                    return true;
                }
            }
            _ => {}
        }
    }

    state.insert(current.to_string(), VisitState::Completed);
    false
}

/// Find all connected components in the graph (treating edges as undirected)
pub fn connected_components<P: GraphProjection>(projection: &P) -> ConnectedComponentsResult
where
    P::Node: Node,
{
    let nodes = projection.nodes();
    let mut visited = HashSet::new();
    let mut components = Vec::new();

    // Build undirected adjacency for component detection
    let mut undirected_adj: HashMap<String, HashSet<String>> = HashMap::new();

    for node in &nodes {
        let node_id = node.id();
        undirected_adj.entry(node_id.clone()).or_default();

        for neighbor in projection.neighbors(&node_id) {
            undirected_adj.entry(node_id.clone()).or_default().insert(neighbor.to_string());
            undirected_adj.entry(neighbor.to_string()).or_default().insert(node_id.clone());
        }
    }

    for node in &nodes {
        let node_id = node.id();
        if !visited.contains(&node_id) {
            let mut component = Vec::new();
            let mut queue = VecDeque::new();

            queue.push_back(node_id.clone());
            visited.insert(node_id.clone());

            while let Some(current) = queue.pop_front() {
                component.push(current.clone());

                if let Some(neighbors) = undirected_adj.get(&current) {
                    for neighbor in neighbors {
                        if !visited.contains(neighbor) {
                            visited.insert(neighbor.clone());
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }

            components.push(component);
        }
    }

    let count = components.len();
    let largest_size = components.iter().map(|c| c.len()).max().unwrap_or(0);

    ConnectedComponentsResult {
        components,
        count,
        largest_size,
    }
}

/// Check if there exists a path between two nodes
pub fn path_exists<P: GraphProjection>(projection: &P, from: &str, to: &str) -> Result<bool> {
    if projection.get_node(from).is_none() {
        return Err(GraphError::NodeNotFound(from.to_string()));
    }
    if projection.get_node(to).is_none() {
        return Err(GraphError::NodeNotFound(to.to_string()));
    }

    if from == to {
        return Ok(true);
    }

    // Use BFS to check reachability
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    queue.push_back(from);
    visited.insert(from);

    while let Some(current) = queue.pop_front() {
        for neighbor in projection.neighbors(current) {
            if neighbor == to {
                return Ok(true);
            }

            if !visited.contains(neighbor) {
                visited.insert(neighbor);
                queue.push_back(neighbor);
            }
        }
    }

    Ok(false)
}

/// Find all nodes reachable from a given node
pub fn reachable_nodes<P: GraphProjection>(projection: &P, start: &str) -> Result<HashSet<String>> {
    if projection.get_node(start).is_none() {
        return Err(GraphError::NodeNotFound(start.to_string()));
    }

    let visited = bfs(projection, start)?;
    Ok(visited.into_iter().collect())
}

/// Calculate the distance from start to all reachable nodes
pub fn distances_from<P: GraphProjection>(projection: &P, start: &str) -> Result<HashMap<String, usize>> {
    if projection.get_node(start).is_none() {
        return Err(GraphError::NodeNotFound(start.to_string()));
    }

    let mut distances = HashMap::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    distances.insert(start.to_string(), 0);
    visited.insert(start);
    queue.push_back((start, 0));

    while let Some((current, dist)) = queue.pop_front() {
        for neighbor in projection.neighbors(current) {
            if !visited.contains(neighbor) {
                visited.insert(neighbor);
                distances.insert(neighbor.to_string(), dist + 1);
                queue.push_back((neighbor, dist + 1));
            }
        }
    }

    Ok(distances)
}

/// Calculate the eccentricity of a node (maximum distance to any reachable node)
pub fn eccentricity<P: GraphProjection>(projection: &P, node: &str) -> Result<usize> {
    let distances = distances_from(projection, node)?;
    Ok(distances.values().copied().max().unwrap_or(0))
}

/// Find the diameter of the graph (maximum eccentricity)
pub fn diameter<P: GraphProjection>(projection: &P) -> Result<usize>
where
    P::Node: Node,
{
    let nodes = projection.nodes();
    if nodes.is_empty() {
        return Ok(0);
    }

    let mut max_eccentricity = 0;

    for node in &nodes {
        let ecc = eccentricity(projection, &node.id())?;
        if ecc > max_eccentricity {
            max_eccentricity = ecc;
        }
    }

    Ok(max_eccentricity)
}

/// Iterative DFS traversal (non-recursive, for large graphs)
pub fn dfs_iterative<P: GraphProjection>(projection: &P, start: &str) -> Result<Vec<String>> {
    if projection.get_node(start).is_none() {
        return Err(GraphError::NodeNotFound(start.to_string()));
    }

    let mut visited: HashSet<String> = HashSet::new();
    let mut stack = Vec::new();
    let mut result = Vec::new();

    stack.push(start.to_string());

    while let Some(current) = stack.pop() {
        if visited.contains(&current) {
            continue;
        }

        visited.insert(current.clone());
        result.push(current.clone());

        // Push neighbors in reverse order to maintain left-to-right traversal
        let neighbors: Vec<String> = projection.neighbors(&current)
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        for neighbor in neighbors.into_iter().rev() {
            if !visited.contains(&neighbor) {
                stack.push(neighbor);
            }
        }
    }

    Ok(result)
}

/// Count the number of paths between two nodes (without cycles)
pub fn count_paths<P: GraphProjection>(projection: &P, from: &str, to: &str) -> Result<usize> {
    if projection.get_node(from).is_none() {
        return Err(GraphError::NodeNotFound(from.to_string()));
    }
    if projection.get_node(to).is_none() {
        return Err(GraphError::NodeNotFound(to.to_string()));
    }

    if from == to {
        return Ok(1);
    }

    let mut visited = HashSet::new();
    Ok(count_paths_helper(projection, from, to, &mut visited))
}

fn count_paths_helper<P: GraphProjection>(
    projection: &P,
    current: &str,
    target: &str,
    visited: &mut HashSet<String>,
) -> usize {
    if current == target {
        return 1;
    }

    visited.insert(current.to_string());
    let mut count = 0;

    for neighbor in projection.neighbors(current) {
        if !visited.contains(neighbor) {
            count += count_paths_helper(projection, neighbor, target, visited);
        }
    }

    visited.remove(current);
    count
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

    // ========== DFS Post-order Tests ==========

    #[test]
    fn test_dfs_postorder_linear() {
        let projection = create_linear_graph();
        let result = dfs_postorder(&projection, "A").unwrap();

        // Post-order: children first, then parent
        // A -> B -> C becomes: C, B, A
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "C");
        assert_eq!(result[1], "B");
        assert_eq!(result[2], "A");
    }

    #[test]
    fn test_dfs_postorder_branching() {
        let projection = create_branching_graph();
        let result = dfs_postorder(&projection, "A").unwrap();

        // D must come before B and C, A must be last
        assert_eq!(result.len(), 4);
        assert_eq!(*result.last().unwrap(), "A");

        let pos_d = result.iter().position(|x| x == "D").unwrap();
        let pos_b = result.iter().position(|x| x == "B").unwrap();
        let pos_c = result.iter().position(|x| x == "C").unwrap();

        assert!(pos_d < pos_b || pos_d < pos_c);
    }

    #[test]
    fn test_dfs_postorder_single_node() {
        let mut projection = create_empty_projection();
        let node = WorkflowNode::new("A", WorkflowNodeType::Start);
        projection.nodes.insert("A".to_string(), node);
        projection.adjacency.insert("A".to_string(), vec![]);

        let result = dfs_postorder(&projection, "A").unwrap();
        assert_eq!(result, vec!["A"]);
    }

    #[test]
    fn test_dfs_postorder_not_found() {
        let projection = create_linear_graph();
        let result = dfs_postorder(&projection, "X");
        assert!(result.is_err());
    }

    // ========== BFS Level Order Tests ==========

    #[test]
    fn test_bfs_level_order_linear() {
        let projection = create_linear_graph();
        let result = bfs_level_order(&projection, "A").unwrap();

        assert_eq!(result.levels.len(), 3);
        assert_eq!(result.levels[0], vec!["A"]);
        assert_eq!(result.levels[1], vec!["B"]);
        assert_eq!(result.levels[2], vec!["C"]);
        assert_eq!(result.total_nodes, 3);
        assert_eq!(result.max_depth, 2);
    }

    #[test]
    fn test_bfs_level_order_branching() {
        let projection = create_branching_graph();
        let result = bfs_level_order(&projection, "A").unwrap();

        assert_eq!(result.levels.len(), 3);
        assert_eq!(result.levels[0], vec!["A"]);
        assert_eq!(result.levels[1].len(), 2); // B and C
        assert!(result.levels[1].contains(&"B".to_string()));
        assert!(result.levels[1].contains(&"C".to_string()));
        assert_eq!(result.levels[2], vec!["D"]);
        assert_eq!(result.total_nodes, 4);
        assert_eq!(result.max_depth, 2);
    }

    #[test]
    fn test_bfs_level_order_single_node() {
        let mut projection = create_empty_projection();
        let node = WorkflowNode::new("A", WorkflowNodeType::Start);
        projection.nodes.insert("A".to_string(), node);
        projection.adjacency.insert("A".to_string(), vec![]);

        let result = bfs_level_order(&projection, "A").unwrap();
        assert_eq!(result.levels.len(), 1);
        assert_eq!(result.total_nodes, 1);
        assert_eq!(result.max_depth, 0);
    }

    #[test]
    fn test_bfs_level_order_not_found() {
        let projection = create_linear_graph();
        let result = bfs_level_order(&projection, "X");
        assert!(result.is_err());
    }

    // ========== Has Cycle Tests ==========

    #[test]
    fn test_has_cycle_dag() {
        let projection = create_linear_graph();
        assert!(!has_cycle(&projection));
    }

    #[test]
    fn test_has_cycle_branching_dag() {
        let projection = create_branching_graph();
        assert!(!has_cycle(&projection));
    }

    #[test]
    fn test_has_cycle_with_cycle() {
        let mut projection = create_empty_projection();

        for id in ["A", "B", "C"] {
            let node = WorkflowNode::new(id, WorkflowNodeType::Start);
            projection.nodes.insert(id.to_string(), node);
        }

        projection.adjacency.insert("A".to_string(), vec!["B".to_string()]);
        projection.adjacency.insert("B".to_string(), vec!["C".to_string()]);
        projection.adjacency.insert("C".to_string(), vec!["A".to_string()]);

        assert!(has_cycle(&projection));
    }

    #[test]
    fn test_has_cycle_self_loop() {
        let mut projection = create_empty_projection();

        let node = WorkflowNode::new("A", WorkflowNodeType::Start);
        projection.nodes.insert("A".to_string(), node);
        projection.adjacency.insert("A".to_string(), vec!["A".to_string()]);

        assert!(has_cycle(&projection));
    }

    #[test]
    fn test_has_cycle_empty_graph() {
        let projection = create_empty_projection();
        assert!(!has_cycle(&projection));
    }

    // ========== Connected Components Tests ==========

    #[test]
    fn test_connected_components_single() {
        let projection = create_linear_graph();
        let result = connected_components(&projection);

        assert_eq!(result.count, 1);
        assert_eq!(result.largest_size, 3);
        assert_eq!(result.components.len(), 1);
    }

    #[test]
    fn test_connected_components_disconnected() {
        let projection = create_disconnected_graph();
        let result = connected_components(&projection);

        assert_eq!(result.count, 2);
        assert_eq!(result.largest_size, 2);
    }

    #[test]
    fn test_connected_components_empty() {
        let projection = create_empty_projection();
        let result = connected_components(&projection);

        assert_eq!(result.count, 0);
        assert_eq!(result.largest_size, 0);
    }

    #[test]
    fn test_connected_components_isolated_nodes() {
        let mut projection = create_empty_projection();

        for id in ["A", "B", "C"] {
            let node = WorkflowNode::new(id, WorkflowNodeType::Start);
            projection.nodes.insert(id.to_string(), node);
            projection.adjacency.insert(id.to_string(), vec![]);
        }

        let result = connected_components(&projection);
        assert_eq!(result.count, 3);
        assert_eq!(result.largest_size, 1);
    }

    // ========== Path Exists Tests ==========

    #[test]
    fn test_path_exists_linear() {
        let projection = create_linear_graph();

        assert!(path_exists(&projection, "A", "C").unwrap());
        assert!(path_exists(&projection, "A", "B").unwrap());
        assert!(!path_exists(&projection, "C", "A").unwrap());
    }

    #[test]
    fn test_path_exists_same_node() {
        let projection = create_linear_graph();
        assert!(path_exists(&projection, "A", "A").unwrap());
    }

    #[test]
    fn test_path_exists_disconnected() {
        let projection = create_disconnected_graph();

        assert!(path_exists(&projection, "A", "B").unwrap());
        assert!(!path_exists(&projection, "A", "D").unwrap());
    }

    #[test]
    fn test_path_exists_node_not_found() {
        let projection = create_linear_graph();

        assert!(path_exists(&projection, "X", "A").is_err());
        assert!(path_exists(&projection, "A", "X").is_err());
    }

    // ========== Reachable Nodes Tests ==========

    #[test]
    fn test_reachable_nodes_linear() {
        let projection = create_linear_graph();
        let reachable = reachable_nodes(&projection, "A").unwrap();

        assert_eq!(reachable.len(), 3);
        assert!(reachable.contains("A"));
        assert!(reachable.contains("B"));
        assert!(reachable.contains("C"));
    }

    #[test]
    fn test_reachable_nodes_from_middle() {
        let projection = create_linear_graph();
        let reachable = reachable_nodes(&projection, "B").unwrap();

        assert_eq!(reachable.len(), 2);
        assert!(reachable.contains("B"));
        assert!(reachable.contains("C"));
        assert!(!reachable.contains("A"));
    }

    #[test]
    fn test_reachable_nodes_leaf() {
        let projection = create_linear_graph();
        let reachable = reachable_nodes(&projection, "C").unwrap();

        assert_eq!(reachable.len(), 1);
        assert!(reachable.contains("C"));
    }

    #[test]
    fn test_reachable_nodes_not_found() {
        let projection = create_linear_graph();
        assert!(reachable_nodes(&projection, "X").is_err());
    }

    // ========== Distances From Tests ==========

    #[test]
    fn test_distances_from_linear() {
        let projection = create_linear_graph();
        let distances = distances_from(&projection, "A").unwrap();

        assert_eq!(*distances.get("A").unwrap(), 0);
        assert_eq!(*distances.get("B").unwrap(), 1);
        assert_eq!(*distances.get("C").unwrap(), 2);
    }

    #[test]
    fn test_distances_from_branching() {
        let projection = create_branching_graph();
        let distances = distances_from(&projection, "A").unwrap();

        assert_eq!(*distances.get("A").unwrap(), 0);
        assert_eq!(*distances.get("B").unwrap(), 1);
        assert_eq!(*distances.get("C").unwrap(), 1);
        assert_eq!(*distances.get("D").unwrap(), 2);
    }

    #[test]
    fn test_distances_from_not_found() {
        let projection = create_linear_graph();
        assert!(distances_from(&projection, "X").is_err());
    }

    // ========== Eccentricity Tests ==========

    #[test]
    fn test_eccentricity_linear() {
        let projection = create_linear_graph();

        assert_eq!(eccentricity(&projection, "A").unwrap(), 2);
        assert_eq!(eccentricity(&projection, "B").unwrap(), 1);
        assert_eq!(eccentricity(&projection, "C").unwrap(), 0);
    }

    #[test]
    fn test_eccentricity_branching() {
        let projection = create_branching_graph();

        assert_eq!(eccentricity(&projection, "A").unwrap(), 2);
    }

    #[test]
    fn test_eccentricity_not_found() {
        let projection = create_linear_graph();
        assert!(eccentricity(&projection, "X").is_err());
    }

    // ========== Diameter Tests ==========

    #[test]
    fn test_diameter_linear() {
        let projection = create_linear_graph();
        assert_eq!(diameter(&projection).unwrap(), 2);
    }

    #[test]
    fn test_diameter_branching() {
        let projection = create_branching_graph();
        assert_eq!(diameter(&projection).unwrap(), 2);
    }

    #[test]
    fn test_diameter_empty() {
        let projection = create_empty_projection();
        assert_eq!(diameter(&projection).unwrap(), 0);
    }

    #[test]
    fn test_diameter_single_node() {
        let mut projection = create_empty_projection();
        let node = WorkflowNode::new("A", WorkflowNodeType::Start);
        projection.nodes.insert("A".to_string(), node);
        projection.adjacency.insert("A".to_string(), vec![]);

        assert_eq!(diameter(&projection).unwrap(), 0);
    }

    // ========== DFS Iterative Tests ==========

    #[test]
    fn test_dfs_iterative_linear() {
        let projection = create_linear_graph();
        let result = dfs_iterative(&projection, "A").unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "A");
        assert!(result.contains(&"B".to_string()));
        assert!(result.contains(&"C".to_string()));
    }

    #[test]
    fn test_dfs_iterative_branching() {
        let projection = create_branching_graph();
        let result = dfs_iterative(&projection, "A").unwrap();

        assert_eq!(result.len(), 4);
        assert_eq!(result[0], "A");
    }

    #[test]
    fn test_dfs_iterative_not_found() {
        let projection = create_linear_graph();
        assert!(dfs_iterative(&projection, "X").is_err());
    }

    #[test]
    fn test_dfs_iterative_matches_recursive() {
        let projection = create_linear_graph();

        let iterative = dfs_iterative(&projection, "A").unwrap();
        let recursive = dfs(&projection, "A").unwrap();

        // Both should visit same nodes (order might differ slightly)
        assert_eq!(iterative.len(), recursive.len());

        let iter_set: HashSet<_> = iterative.into_iter().collect();
        let rec_set: HashSet<_> = recursive.into_iter().collect();

        assert_eq!(iter_set, rec_set);
    }

    // ========== Count Paths Tests ==========

    #[test]
    fn test_count_paths_linear() {
        let projection = create_linear_graph();
        assert_eq!(count_paths(&projection, "A", "C").unwrap(), 1);
    }

    #[test]
    fn test_count_paths_diamond() {
        let projection = create_branching_graph();
        // A -> B -> D and A -> C -> D = 2 paths
        assert_eq!(count_paths(&projection, "A", "D").unwrap(), 2);
    }

    #[test]
    fn test_count_paths_same_node() {
        let projection = create_linear_graph();
        assert_eq!(count_paths(&projection, "A", "A").unwrap(), 1);
    }

    #[test]
    fn test_count_paths_no_path() {
        let projection = create_linear_graph();
        assert_eq!(count_paths(&projection, "C", "A").unwrap(), 0);
    }

    #[test]
    fn test_count_paths_not_found() {
        let projection = create_linear_graph();

        assert!(count_paths(&projection, "X", "A").is_err());
        assert!(count_paths(&projection, "A", "X").is_err());
    }

    #[test]
    fn test_count_paths_complex() {
        // Create a graph with multiple paths:
        //     B
        //   / | \
        // A   |   D
        //   \ | /
        //     C
        let mut projection = create_empty_projection();

        for id in ["A", "B", "C", "D"] {
            let node = WorkflowNode::new(id, WorkflowNodeType::Start);
            projection.nodes.insert(id.to_string(), node);
        }

        projection.adjacency.insert("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        projection.adjacency.insert("B".to_string(), vec!["C".to_string(), "D".to_string()]);
        projection.adjacency.insert("C".to_string(), vec!["D".to_string()]);
        projection.adjacency.insert("D".to_string(), vec![]);

        // Paths from A to D:
        // A -> B -> D
        // A -> B -> C -> D
        // A -> C -> D
        assert_eq!(count_paths(&projection, "A", "D").unwrap(), 3);
    }

    // ========== Level Order Result Tests ==========

    #[test]
    fn test_level_order_result_fields() {
        let result = LevelOrderResult {
            levels: vec![
                vec!["A".to_string()],
                vec!["B".to_string(), "C".to_string()],
            ],
            total_nodes: 3,
            max_depth: 1,
        };

        assert_eq!(result.levels.len(), 2);
        assert_eq!(result.total_nodes, 3);
        assert_eq!(result.max_depth, 1);
    }

    // ========== Connected Components Result Tests ==========

    #[test]
    fn test_connected_components_result_fields() {
        let result = ConnectedComponentsResult {
            components: vec![
                vec!["A".to_string(), "B".to_string()],
                vec!["C".to_string()],
            ],
            count: 2,
            largest_size: 2,
        };

        assert_eq!(result.components.len(), 2);
        assert_eq!(result.count, 2);
        assert_eq!(result.largest_size, 2);
    }
}