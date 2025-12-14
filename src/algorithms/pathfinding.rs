//! Pathfinding algorithms for graph projections

use crate::core::{GraphProjection, Node};
use crate::error::{GraphError, Result};
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::cmp::Ordering;

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

// ============================================================================
// A* Algorithm Implementation
// ============================================================================

/// State for A* priority queue
#[derive(Clone, Eq, PartialEq)]
struct AStarState {
    cost: u64,
    node: String,
}

impl Ord for AStarState {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap behavior
        other.cost.cmp(&self.cost)
            .then_with(|| self.node.cmp(&other.node))
    }
}

impl PartialOrd for AStarState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A* pathfinding algorithm with heuristic
///
/// Finds the shortest path between two nodes using the A* algorithm.
/// The heuristic function provides an estimate of the cost from any node to the goal.
///
/// # Arguments
/// * `projection` - The graph projection to search
/// * `from` - Starting node ID
/// * `to` - Target node ID
/// * `heuristic` - Function estimating cost from node to goal
/// * `edge_weight` - Function returning edge weight (defaults to 1 if None)
///
/// # Returns
/// * `Ok(Some(path))` - The shortest path found
/// * `Ok(None)` - No path exists
/// * `Err(error)` - Node not found error
pub fn a_star<P, H, W>(
    projection: &P,
    from: &str,
    to: &str,
    heuristic: H,
    edge_weight: Option<W>,
) -> Result<Option<Vec<String>>>
where
    P: GraphProjection,
    H: Fn(&str, &str) -> u64,
    W: Fn(&str, &str) -> u64,
{
    // Validate nodes exist
    if projection.get_node(from).is_none() {
        return Err(GraphError::NodeNotFound(from.to_string()));
    }
    if projection.get_node(to).is_none() {
        return Err(GraphError::NodeNotFound(to.to_string()));
    }

    if from == to {
        return Ok(Some(vec![from.to_string()]));
    }

    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<String, String> = HashMap::new();
    let mut g_score: HashMap<String, u64> = HashMap::new();
    let mut f_score: HashMap<String, u64> = HashMap::new();
    let mut closed_set: HashSet<String> = HashSet::new();

    g_score.insert(from.to_string(), 0);
    f_score.insert(from.to_string(), heuristic(from, to));

    open_set.push(AStarState {
        cost: heuristic(from, to),
        node: from.to_string(),
    });

    while let Some(AStarState { node: current, .. }) = open_set.pop() {
        if current == to {
            // Reconstruct path
            let mut path = vec![current];
            let mut node = &path[0];
            while let Some(parent) = came_from.get(node) {
                path.push(parent.clone());
                node = parent;
            }
            path.reverse();
            return Ok(Some(path));
        }

        if closed_set.contains(&current) {
            continue;
        }
        closed_set.insert(current.clone());

        let current_g = *g_score.get(&current).unwrap_or(&u64::MAX);

        for neighbor in projection.neighbors(&current) {
            if closed_set.contains(neighbor) {
                continue;
            }

            let weight = edge_weight
                .as_ref()
                .map(|w| w(&current, neighbor))
                .unwrap_or(1);

            let tentative_g = current_g.saturating_add(weight);

            if tentative_g < *g_score.get(neighbor).unwrap_or(&u64::MAX) {
                came_from.insert(neighbor.to_string(), current.clone());
                g_score.insert(neighbor.to_string(), tentative_g);
                let h = heuristic(neighbor, to);
                let f = tentative_g.saturating_add(h);
                f_score.insert(neighbor.to_string(), f);

                open_set.push(AStarState {
                    cost: f,
                    node: neighbor.to_string(),
                });
            }
        }
    }

    Ok(None)
}

/// A* with uniform cost (edge weights all equal to 1)
pub fn a_star_uniform<P: GraphProjection>(
    projection: &P,
    from: &str,
    to: &str,
) -> Result<Option<Vec<String>>> {
    // Use a constant heuristic of 0 (becomes Dijkstra)
    a_star(projection, from, to, |_, _| 0, Option::<fn(&str, &str) -> u64>::None)
}

// ============================================================================
// Dijkstra's Algorithm Implementation
// ============================================================================

/// Dijkstra's algorithm for weighted shortest path
///
/// Finds the shortest path using edge weights.
///
/// # Arguments
/// * `projection` - The graph projection to search
/// * `from` - Starting node ID
/// * `to` - Target node ID
/// * `edge_weight` - Function returning edge weight for (source, target)
pub fn dijkstra<P, W>(
    projection: &P,
    from: &str,
    to: &str,
    edge_weight: W,
) -> Result<Option<(Vec<String>, u64)>>
where
    P: GraphProjection,
    W: Fn(&str, &str) -> u64,
{
    // Validate nodes exist
    if projection.get_node(from).is_none() {
        return Err(GraphError::NodeNotFound(from.to_string()));
    }
    if projection.get_node(to).is_none() {
        return Err(GraphError::NodeNotFound(to.to_string()));
    }

    if from == to {
        return Ok(Some((vec![from.to_string()], 0)));
    }

    let mut dist: HashMap<String, u64> = HashMap::new();
    let mut prev: HashMap<String, String> = HashMap::new();
    let mut heap = BinaryHeap::new();

    dist.insert(from.to_string(), 0);
    heap.push(AStarState {
        cost: 0,
        node: from.to_string(),
    });

    while let Some(AStarState { cost, node }) = heap.pop() {
        if node == to {
            // Reconstruct path
            let mut path = vec![node];
            let mut current = &path[0];
            while let Some(parent) = prev.get(current) {
                path.push(parent.clone());
                current = parent;
            }
            path.reverse();
            return Ok(Some((path, cost)));
        }

        if cost > *dist.get(&node).unwrap_or(&u64::MAX) {
            continue;
        }

        for neighbor in projection.neighbors(&node) {
            let weight = edge_weight(&node, neighbor);
            let next_cost = cost.saturating_add(weight);

            if next_cost < *dist.get(neighbor).unwrap_or(&u64::MAX) {
                dist.insert(neighbor.to_string(), next_cost);
                prev.insert(neighbor.to_string(), node.clone());
                heap.push(AStarState {
                    cost: next_cost,
                    node: neighbor.to_string(),
                });
            }
        }
    }

    Ok(None)
}

/// Dijkstra's algorithm with uniform edge weights (all equal to 1)
pub fn dijkstra_uniform<P: GraphProjection>(
    projection: &P,
    from: &str,
    to: &str,
) -> Result<Option<(Vec<String>, u64)>> {
    dijkstra(projection, from, to, |_, _| 1)
}

// ============================================================================
// Bellman-Ford Algorithm Implementation
// ============================================================================

/// Bellman-Ford algorithm for weighted shortest path
///
/// Can handle negative edge weights (unlike Dijkstra).
/// Also detects negative cycles.
///
/// # Arguments
/// * `projection` - The graph projection to search
/// * `from` - Starting node ID
/// * `edge_weight` - Function returning edge weight (can be negative as i64)
///
/// # Returns
/// * `Ok(distances)` - HashMap of shortest distances from source
/// * `Err(NegativeCycle)` - If a negative cycle is detected
pub fn bellman_ford<P, W>(
    projection: &P,
    from: &str,
    edge_weight: W,
) -> Result<HashMap<String, i64>>
where
    P: GraphProjection,
    P::Node: crate::core::Node,
    W: Fn(&str, &str) -> i64,
{
    if projection.get_node(from).is_none() {
        return Err(GraphError::NodeNotFound(from.to_string()));
    }

    let nodes: Vec<String> = projection.nodes().into_iter().map(|n| n.id()).collect();
    let node_count = nodes.len();

    let mut dist: HashMap<String, i64> = HashMap::new();
    for node in &nodes {
        dist.insert(node.clone(), i64::MAX);
    }
    dist.insert(from.to_string(), 0);

    // Relax edges V-1 times
    for _ in 0..node_count - 1 {
        for u in &nodes {
            let u_dist = *dist.get(u).unwrap_or(&i64::MAX);
            if u_dist == i64::MAX {
                continue;
            }

            for v in projection.neighbors(u) {
                let weight = edge_weight(u, v);
                let new_dist = u_dist.saturating_add(weight);

                if new_dist < *dist.get(v).unwrap_or(&i64::MAX) {
                    dist.insert(v.to_string(), new_dist);
                }
            }
        }
    }

    // Check for negative cycles
    for u in &nodes {
        let u_dist = *dist.get(u).unwrap_or(&i64::MAX);
        if u_dist == i64::MAX {
            continue;
        }

        for v in projection.neighbors(u) {
            let weight = edge_weight(u, v);
            if u_dist.saturating_add(weight) < *dist.get(v).unwrap_or(&i64::MAX) {
                return Err(GraphError::InvalidOperation(
                    "Graph contains a negative-weight cycle".to_string()
                ));
            }
        }
    }

    Ok(dist)
}

// ============================================================================
// Path Length and Distance Functions
// ============================================================================

/// Calculate the length of a path (number of edges)
pub fn path_length(path: &[String]) -> usize {
    if path.is_empty() {
        0
    } else {
        path.len() - 1
    }
}

/// Calculate the weighted cost of a path
pub fn path_cost<W>(path: &[String], edge_weight: W) -> u64
where
    W: Fn(&str, &str) -> u64,
{
    if path.len() < 2 {
        return 0;
    }

    path.windows(2)
        .map(|window| edge_weight(&window[0], &window[1]))
        .sum()
}

/// Find all shortest paths between two nodes (there may be multiple paths of same length)
pub fn all_shortest_paths<P: GraphProjection>(
    projection: &P,
    from: &str,
    to: &str,
) -> Result<Vec<Vec<String>>> {
    // First find the shortest distance
    let shortest = shortest_path(projection, from, to)?;

    match shortest {
        None => Ok(vec![]),
        Some(path) => {
            let shortest_len = path.len();

            // Find all paths with that length
            let all = all_paths(projection, from, to)?;

            Ok(all.into_iter()
                .filter(|p| p.len() == shortest_len)
                .collect())
        }
    }
}

/// Check if a path is valid in the projection
pub fn is_valid_path<P: GraphProjection>(
    projection: &P,
    path: &[String],
) -> bool {
    if path.is_empty() {
        return true;
    }

    // Check first node exists
    if projection.get_node(&path[0]).is_none() {
        return false;
    }

    // Check each edge exists
    for window in path.windows(2) {
        let neighbors = projection.neighbors(&window[0]);
        if !neighbors.contains(&window[1].as_str()) {
            return false;
        }
    }

    true
}

/// Find path with maximum length constraint
pub fn shortest_path_with_max_length<P: GraphProjection>(
    projection: &P,
    from: &str,
    to: &str,
    max_length: usize,
) -> Result<Option<Vec<String>>> {
    if projection.get_node(from).is_none() {
        return Err(GraphError::NodeNotFound(from.to_string()));
    }
    if projection.get_node(to).is_none() {
        return Err(GraphError::NodeNotFound(to.to_string()));
    }

    if from == to {
        return Ok(Some(vec![from.to_string()]));
    }

    let result = shortest_path(projection, from, to)?;

    match result {
        Some(path) if path.len() <= max_length + 1 => Ok(Some(path)),
        _ => Ok(None),
    }
}

/// Find the k shortest paths between two nodes
pub fn k_shortest_paths<P: GraphProjection>(
    projection: &P,
    from: &str,
    to: &str,
    k: usize,
) -> Result<Vec<Vec<String>>> {
    if k == 0 {
        return Ok(vec![]);
    }

    let all = all_paths(projection, from, to)?;

    let mut sorted_paths = all;
    sorted_paths.sort_by_key(|p| p.len());

    Ok(sorted_paths.into_iter().take(k).collect())
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

    // ========== A* Algorithm Tests ==========

    #[test]
    fn test_a_star_linear_graph() {
        let projection = create_linear_graph();
        let result = a_star(
            &projection,
            "A",
            "D",
            |_, _| 0, // No heuristic
            Option::<fn(&str, &str) -> u64>::None
        ).unwrap();

        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(path, vec!["A", "B", "C", "D"]);
    }

    #[test]
    fn test_a_star_with_heuristic() {
        let projection = create_diamond_graph();

        // Heuristic: estimate distance to D
        let heuristic = |node: &str, _target: &str| -> u64 {
            match node {
                "A" => 2,
                "B" => 1,
                "C" => 1,
                "D" => 0,
                _ => 0,
            }
        };

        let result = a_star(
            &projection,
            "A",
            "D",
            heuristic,
            Option::<fn(&str, &str) -> u64>::None
        ).unwrap();

        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], "A");
        assert_eq!(path[2], "D");
    }

    #[test]
    fn test_a_star_with_edge_weights() {
        let projection = create_diamond_graph();

        // Make path through B more expensive
        let weights = |from: &str, to: &str| -> u64 {
            match (from, to) {
                ("A", "B") => 10,
                ("A", "C") => 1,
                ("B", "D") => 10,
                ("C", "D") => 1,
                _ => 1,
            }
        };

        let result = a_star(
            &projection,
            "A",
            "D",
            |_, _| 0,
            Some(weights)
        ).unwrap();

        assert!(result.is_some());
        let path = result.unwrap();
        // Should prefer A -> C -> D due to weights
        assert_eq!(path, vec!["A", "C", "D"]);
    }

    #[test]
    fn test_a_star_same_node() {
        let projection = create_linear_graph();
        let result = a_star(
            &projection,
            "A",
            "A",
            |_, _| 0,
            Option::<fn(&str, &str) -> u64>::None
        ).unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap(), vec!["A"]);
    }

    #[test]
    fn test_a_star_no_path() {
        let projection = create_linear_graph();
        let result = a_star(
            &projection,
            "D",
            "A",
            |_, _| 0,
            Option::<fn(&str, &str) -> u64>::None
        ).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_a_star_source_not_found() {
        let projection = create_linear_graph();
        let result = a_star(
            &projection,
            "X",
            "A",
            |_, _| 0,
            Option::<fn(&str, &str) -> u64>::None
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_a_star_target_not_found() {
        let projection = create_linear_graph();
        let result = a_star(
            &projection,
            "A",
            "X",
            |_, _| 0,
            Option::<fn(&str, &str) -> u64>::None
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_a_star_uniform() {
        let projection = create_linear_graph();
        let result = a_star_uniform(&projection, "A", "D").unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap(), vec!["A", "B", "C", "D"]);
    }

    #[test]
    fn test_a_star_complex_graph() {
        let projection = create_complex_graph();
        let result = a_star_uniform(&projection, "A", "F").unwrap();

        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(path.len(), 4);
        assert_eq!(path[0], "A");
        assert_eq!(path[3], "F");
    }

    // ========== Dijkstra Algorithm Tests ==========

    #[test]
    fn test_dijkstra_linear_graph() {
        let projection = create_linear_graph();
        let result = dijkstra(&projection, "A", "D", |_, _| 1).unwrap();

        assert!(result.is_some());
        let (path, cost) = result.unwrap();
        assert_eq!(path, vec!["A", "B", "C", "D"]);
        assert_eq!(cost, 3);
    }

    #[test]
    fn test_dijkstra_with_weights() {
        let projection = create_diamond_graph();

        // Make path through B more expensive
        let weights = |from: &str, to: &str| -> u64 {
            match (from, to) {
                ("A", "B") => 10,
                ("A", "C") => 2,
                ("B", "D") => 10,
                ("C", "D") => 2,
                _ => 1,
            }
        };

        let result = dijkstra(&projection, "A", "D", weights).unwrap();

        assert!(result.is_some());
        let (path, cost) = result.unwrap();
        assert_eq!(path, vec!["A", "C", "D"]);
        assert_eq!(cost, 4);
    }

    #[test]
    fn test_dijkstra_same_node() {
        let projection = create_linear_graph();
        let result = dijkstra(&projection, "A", "A", |_, _| 1).unwrap();

        assert!(result.is_some());
        let (path, cost) = result.unwrap();
        assert_eq!(path, vec!["A"]);
        assert_eq!(cost, 0);
    }

    #[test]
    fn test_dijkstra_no_path() {
        let projection = create_linear_graph();
        let result = dijkstra(&projection, "D", "A", |_, _| 1).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_dijkstra_source_not_found() {
        let projection = create_linear_graph();
        let result = dijkstra(&projection, "X", "A", |_, _| 1);

        assert!(result.is_err());
    }

    #[test]
    fn test_dijkstra_target_not_found() {
        let projection = create_linear_graph();
        let result = dijkstra(&projection, "A", "X", |_, _| 1);

        assert!(result.is_err());
    }

    #[test]
    fn test_dijkstra_uniform() {
        let projection = create_complex_graph();
        let result = dijkstra_uniform(&projection, "A", "F").unwrap();

        assert!(result.is_some());
        let (path, cost) = result.unwrap();
        assert_eq!(path.len(), 4);
        assert_eq!(cost, 3);
    }

    #[test]
    fn test_dijkstra_variable_weights() {
        let projection = create_complex_graph();

        // Create weights that make one path clearly better
        let weights = |from: &str, to: &str| -> u64 {
            match (from, to) {
                ("A", "D") => 1,
                ("D", "E") => 1,
                ("E", "F") => 1,
                _ => 100,
            }
        };

        let result = dijkstra(&projection, "A", "F", weights).unwrap();

        assert!(result.is_some());
        let (path, cost) = result.unwrap();
        assert_eq!(path, vec!["A", "D", "E", "F"]);
        assert_eq!(cost, 3);
    }

    // ========== Bellman-Ford Algorithm Tests ==========

    #[test]
    fn test_bellman_ford_linear_graph() {
        let projection = create_linear_graph();
        let result = bellman_ford(&projection, "A", |_, _| 1).unwrap();

        assert_eq!(*result.get("A").unwrap(), 0);
        assert_eq!(*result.get("B").unwrap(), 1);
        assert_eq!(*result.get("C").unwrap(), 2);
        assert_eq!(*result.get("D").unwrap(), 3);
    }

    #[test]
    fn test_bellman_ford_with_negative_weights() {
        let projection = create_linear_graph();

        // Use negative weights (which Bellman-Ford can handle)
        let weights = |from: &str, to: &str| -> i64 {
            match (from, to) {
                ("A", "B") => -1,
                ("B", "C") => 2,
                ("C", "D") => -1,
                _ => 1,
            }
        };

        let result = bellman_ford(&projection, "A", weights).unwrap();

        assert_eq!(*result.get("A").unwrap(), 0);
        assert_eq!(*result.get("B").unwrap(), -1);
        assert_eq!(*result.get("C").unwrap(), 1);
        assert_eq!(*result.get("D").unwrap(), 0);
    }

    #[test]
    fn test_bellman_ford_source_not_found() {
        let projection = create_linear_graph();
        let result = bellman_ford(&projection, "X", |_, _| 1);

        assert!(result.is_err());
    }

    #[test]
    fn test_bellman_ford_unreachable_nodes() {
        // A -> B, C (disconnected)
        let mut projection = create_empty_projection();
        for id in ["A", "B", "C"] {
            let node = WorkflowNode::new(id, WorkflowNodeType::Start);
            projection.nodes.insert(id.to_string(), node);
        }

        projection.adjacency.insert("A".to_string(), vec!["B".to_string()]);
        projection.adjacency.insert("B".to_string(), vec![]);
        projection.adjacency.insert("C".to_string(), vec![]);

        let result = bellman_ford(&projection, "A", |_, _| 1).unwrap();

        assert_eq!(*result.get("A").unwrap(), 0);
        assert_eq!(*result.get("B").unwrap(), 1);
        assert_eq!(*result.get("C").unwrap(), i64::MAX); // Unreachable
    }

    // ========== Path Length and Cost Tests ==========

    #[test]
    fn test_path_length_empty() {
        let path: Vec<String> = vec![];
        assert_eq!(path_length(&path), 0);
    }

    #[test]
    fn test_path_length_single_node() {
        let path = vec!["A".to_string()];
        assert_eq!(path_length(&path), 0);
    }

    #[test]
    fn test_path_length_multiple_nodes() {
        let path = vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string()];
        assert_eq!(path_length(&path), 3);
    }

    #[test]
    fn test_path_cost_empty() {
        let path: Vec<String> = vec![];
        assert_eq!(path_cost(&path, |_, _| 1), 0);
    }

    #[test]
    fn test_path_cost_single_node() {
        let path = vec!["A".to_string()];
        assert_eq!(path_cost(&path, |_, _| 1), 0);
    }

    #[test]
    fn test_path_cost_uniform_weights() {
        let path = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        assert_eq!(path_cost(&path, |_, _| 1), 2);
    }

    #[test]
    fn test_path_cost_variable_weights() {
        let path = vec!["A".to_string(), "B".to_string(), "C".to_string()];

        let weights = |from: &str, to: &str| -> u64 {
            match (from, to) {
                ("A", "B") => 5,
                ("B", "C") => 10,
                _ => 1,
            }
        };

        assert_eq!(path_cost(&path, weights), 15);
    }

    // ========== All Shortest Paths Tests ==========

    #[test]
    fn test_all_shortest_paths_diamond() {
        let projection = create_diamond_graph();
        let result = all_shortest_paths(&projection, "A", "D").unwrap();

        // Two shortest paths of length 3
        assert_eq!(result.len(), 2);
        for path in &result {
            assert_eq!(path.len(), 3);
        }
    }

    #[test]
    fn test_all_shortest_paths_no_path() {
        let projection = create_linear_graph();
        let result = all_shortest_paths(&projection, "D", "A").unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn test_all_shortest_paths_same_node() {
        let projection = create_linear_graph();
        let result = all_shortest_paths(&projection, "A", "A").unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], vec!["A"]);
    }

    // ========== Is Valid Path Tests ==========

    #[test]
    fn test_is_valid_path_empty() {
        let projection = create_linear_graph();
        assert!(is_valid_path(&projection, &[]));
    }

    #[test]
    fn test_is_valid_path_single_node() {
        let projection = create_linear_graph();
        let path = vec!["A".to_string()];
        assert!(is_valid_path(&projection, &path));
    }

    #[test]
    fn test_is_valid_path_valid() {
        let projection = create_linear_graph();
        let path = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        assert!(is_valid_path(&projection, &path));
    }

    #[test]
    fn test_is_valid_path_invalid_edge() {
        let projection = create_linear_graph();
        // No direct edge from A to C
        let path = vec!["A".to_string(), "C".to_string()];
        assert!(!is_valid_path(&projection, &path));
    }

    #[test]
    fn test_is_valid_path_nonexistent_node() {
        let projection = create_linear_graph();
        let path = vec!["X".to_string(), "Y".to_string()];
        assert!(!is_valid_path(&projection, &path));
    }

    // ========== Shortest Path With Max Length Tests ==========

    #[test]
    fn test_shortest_path_with_max_length_within_limit() {
        let projection = create_linear_graph();
        let result = shortest_path_with_max_length(&projection, "A", "D", 5).unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 4);
    }

    #[test]
    fn test_shortest_path_with_max_length_at_limit() {
        let projection = create_linear_graph();
        // Path A->B->C->D has 3 edges, so max_length=3 should work
        let result = shortest_path_with_max_length(&projection, "A", "D", 3).unwrap();

        assert!(result.is_some());
    }

    #[test]
    fn test_shortest_path_with_max_length_exceeds_limit() {
        let projection = create_linear_graph();
        // Path A->B->C->D has 3 edges, max_length=2 should fail
        let result = shortest_path_with_max_length(&projection, "A", "D", 2).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_shortest_path_with_max_length_same_node() {
        let projection = create_linear_graph();
        let result = shortest_path_with_max_length(&projection, "A", "A", 0).unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap(), vec!["A"]);
    }

    #[test]
    fn test_shortest_path_with_max_length_source_not_found() {
        let projection = create_linear_graph();
        let result = shortest_path_with_max_length(&projection, "X", "A", 5);

        assert!(result.is_err());
    }

    // ========== K Shortest Paths Tests ==========

    #[test]
    fn test_k_shortest_paths_diamond() {
        let projection = create_diamond_graph();
        let result = k_shortest_paths(&projection, "A", "D", 2).unwrap();

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_k_shortest_paths_zero() {
        let projection = create_linear_graph();
        let result = k_shortest_paths(&projection, "A", "D", 0).unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn test_k_shortest_paths_one() {
        let projection = create_diamond_graph();
        let result = k_shortest_paths(&projection, "A", "D", 1).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 3);
    }

    #[test]
    fn test_k_shortest_paths_more_than_exist() {
        let projection = create_linear_graph();
        let result = k_shortest_paths(&projection, "A", "D", 10).unwrap();

        // Only one path exists in linear graph
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_k_shortest_paths_complex() {
        let projection = create_complex_graph();
        let result = k_shortest_paths(&projection, "A", "F", 3).unwrap();

        // Should get at least 3 paths (or all available if less)
        assert!(!result.is_empty());
        assert!(result.len() <= 3);

        // Paths should be sorted by length
        for window in result.windows(2) {
            assert!(window[0].len() <= window[1].len());
        }
    }

    #[test]
    fn test_k_shortest_paths_no_path() {
        let projection = create_linear_graph();
        let result = k_shortest_paths(&projection, "D", "A", 5).unwrap();

        assert!(result.is_empty());
    }
}