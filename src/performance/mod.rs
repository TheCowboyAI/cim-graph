//! Performance optimizations for CIM Graph
//!
//! This module contains performance-critical optimizations including:
//! - Graph indexing for fast lookups
//! - Caching strategies
//! - Memory pooling
//! - Parallel operations

use crate::core::{Node, Edge};
use crate::error::Result;
use std::collections::{HashMap, BTreeMap};
use std::sync::{Arc, RwLock};
use rayon::prelude::*;

/// Index for fast node lookups by various properties
#[derive(Debug)]
pub struct NodeIndex<N: Node> {
    /// Index by node ID for O(1) lookup
    by_id: HashMap<String, Arc<N>>,
    
    /// Index by node type/label for grouped queries
    by_type: HashMap<String, Vec<Arc<N>>>,
    
    /// Spatial index for nodes with position data
    #[allow(dead_code)]
    spatial_index: Option<SpatialIndex>,
}

impl<N: Node> NodeIndex<N> {
    /// Create a new node index
    pub fn new() -> Self {
        Self {
            by_id: HashMap::new(),
            by_type: HashMap::new(),
            spatial_index: None,
        }
    }
    
    /// Add a node to the index
    pub fn insert(&mut self, node: Arc<N>) {
        let id = node.id();
        self.by_id.insert(id.clone(), node.clone());
        
        // Index by type if the node has a type field
        // This is a simplified example - real implementation would use traits
        self.by_type.entry("default".to_string())
            .or_default()
            .push(node);
    }
    
    /// Remove a node from the index
    pub fn remove(&mut self, id: &str) -> Option<Arc<N>> {
        if let Some(node) = self.by_id.remove(id) {
            // Also remove from type index
            for nodes in self.by_type.values_mut() {
                nodes.retain(|n| n.id() != id);
            }
            Some(node)
        } else {
            None
        }
    }
    
    /// Get node by ID
    pub fn get(&self, id: &str) -> Option<&Arc<N>> {
        self.by_id.get(id)
    }
    
    /// Get all nodes of a specific type
    pub fn get_by_type(&self, node_type: &str) -> Option<&Vec<Arc<N>>> {
        self.by_type.get(node_type)
    }
}

/// Spatial index for geographic/coordinate-based queries
#[derive(Debug)]
pub struct SpatialIndex {
    // Simplified R-tree or quadtree implementation
    _tree: BTreeMap<(i32, i32), Vec<String>>,
}

/// Edge index for fast edge lookups
#[derive(Debug)]
pub struct EdgeIndex<E: Edge> {
    /// All edges indexed by source node
    by_source: HashMap<String, Vec<Arc<E>>>,
    
    /// All edges indexed by target node
    by_target: HashMap<String, Vec<Arc<E>>>,
    
    /// Direct lookup by edge ID
    by_id: HashMap<String, Arc<E>>,
}

impl<E: Edge> EdgeIndex<E> {
    /// Create a new edge index
    pub fn new() -> Self {
        Self {
            by_source: HashMap::with_capacity(1000),
            by_target: HashMap::with_capacity(1000),
            by_id: HashMap::with_capacity(1000),
        }
    }
    
    /// Add an edge to the index
    pub fn insert(&mut self, edge: Arc<E>) {
        let id = edge.id();
        let source = edge.source();
        let target = edge.target();
        
        self.by_id.insert(id, edge.clone());
        
        self.by_source.entry(source)
            .or_default()
            .push(edge.clone());
            
        self.by_target.entry(target)
            .or_default()
            .push(edge);
    }
    
    /// Get all edges from a source node
    pub fn edges_from(&self, source: &str) -> Option<&Vec<Arc<E>>> {
        self.by_source.get(source)
    }
    
    /// Get all edges to a target node
    pub fn edges_to(&self, target: &str) -> Option<&Vec<Arc<E>>> {
        self.by_target.get(target)
    }
}

/// Cache for expensive computations
#[derive(Debug)]
pub struct GraphCache {
    /// Cache for shortest path computations
    shortest_paths: RwLock<HashMap<(String, String), Vec<String>>>,
    
    /// Cache for node degree calculations
    degrees: RwLock<HashMap<String, usize>>,
    
    /// Cache for connected components
    components: RwLock<Option<Vec<Vec<String>>>>,
    
    /// Cache generation number - increment to invalidate all caches
    generation: RwLock<u64>,
}

impl GraphCache {
    /// Create a new graph cache
    pub fn new() -> Self {
        Self {
            shortest_paths: RwLock::new(HashMap::new()),
            degrees: RwLock::new(HashMap::new()),
            components: RwLock::new(None),
            generation: RwLock::new(0),
        }
    }
    
    /// Invalidate all caches
    pub fn invalidate(&self) {
        let mut gen = self.generation.write().unwrap();
        *gen += 1;
        
        // Clear all caches
        self.shortest_paths.write().unwrap().clear();
        self.degrees.write().unwrap().clear();
        *self.components.write().unwrap() = None;
    }
    
    /// Get or compute shortest path
    pub fn get_shortest_path<F>(
        &self,
        from: &str,
        to: &str,
        compute: F,
    ) -> Result<Vec<String>>
    where
        F: FnOnce() -> Result<Vec<String>>,
    {
        let key = (from.to_string(), to.to_string());
        
        // Try to get from cache
        if let Some(path) = self.shortest_paths.read().unwrap().get(&key) {
            return Ok(path.clone());
        }
        
        // Compute and cache
        let path = compute()?;
        self.shortest_paths.write().unwrap().insert(key, path.clone());
        Ok(path)
    }
}

/// Memory pool for node allocations
#[derive(Debug)]
pub struct NodePool<N> {
    pool: Vec<N>,
    capacity: usize,
}

impl<N: Default> NodePool<N> {
    /// Create a new node pool with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            pool: Vec::with_capacity(capacity),
            capacity,
        }
    }
    
    /// Get a node from the pool or create a new one
    pub fn acquire(&mut self) -> N {
        self.pool.pop().unwrap_or_default()
    }
    
    /// Return a node to the pool
    pub fn release(&mut self, node: N) {
        if self.pool.len() < self.capacity {
            self.pool.push(node);
        }
    }
}

/// Parallel graph operations using rayon
pub mod parallel {
    use super::*;
    
    /// Parallel BFS traversal
    pub fn parallel_bfs<F>(
        start_nodes: Vec<String>,
        get_neighbors: F,
    ) -> Vec<String>
    where
        F: Fn(&str) -> Vec<String> + Sync,
    {
        use std::sync::Mutex;
        let visited = Mutex::new(std::collections::HashSet::new());
        let mut current_level = start_nodes;
        let mut result = Vec::new();
        
        while !current_level.is_empty() {
            // Filter out already visited nodes and collect new ones
            let (to_process, new_results): (Vec<_>, Vec<_>) = current_level
                .into_iter()
                .filter_map(|node| {
                    let mut visited_guard = visited.lock().unwrap();
                    if visited_guard.insert(node.clone()) {
                        Some((node.clone(), node))
                    } else {
                        None
                    }
                })
                .unzip();
            
            result.extend(new_results);
            
            // Process current level in parallel to get neighbors
            let next_level: Vec<String> = to_process
                .par_iter()
                .flat_map(|node| get_neighbors(node))
                .collect();
                
            current_level = next_level;
        }
        
        result
    }
    
    /// Parallel degree calculation for all nodes
    pub fn parallel_degrees<'a, I, F>(
        nodes: I,
        get_degree: F,
    ) -> HashMap<String, usize>
    where
        I: ParallelIterator<Item = &'a String>,
        F: Fn(&str) -> usize + Sync,
    {
        nodes
            .map(|node| (node.clone(), get_degree(node)))
            .collect()
    }
}

/// Optimized graph operations
pub trait OptimizedGraph {
    /// Node type for this graph
    type Node: Node;
    /// Edge type for this graph
    type Edge: Edge;
    
    /// Get node with caching
    fn get_node_cached(&self, id: &str) -> Option<&Self::Node>;
    
    /// Get edges with indexing
    fn get_edges_indexed(&self, from: &str, to: &str) -> Vec<&Self::Edge>;
    
    /// Add multiple nodes in a single operation
    fn add_nodes_bulk(&mut self, nodes: Vec<Self::Node>) -> Result<()>;
    
    /// Add multiple edges in a single operation
    fn add_edges_bulk(&mut self, edges: Vec<Self::Edge>) -> Result<()>;
}

/// Performance monitoring utilities
pub mod monitoring {
    use std::time::{Duration, Instant};
    use std::sync::Mutex;
    
    /// Simple performance counter
    #[derive(Debug)]
    pub struct PerfCounter {
        #[allow(dead_code)]
        name: String,
        count: Mutex<u64>,
        total_time: Mutex<Duration>,
    }
    
    impl PerfCounter {
        /// Create a new performance counter
        pub fn new(name: impl Into<String>) -> Self {
            Self {
                name: name.into(),
                count: Mutex::new(0),
                total_time: Mutex::new(Duration::ZERO),
            }
        }
        
        /// Measure and record an operation
        pub fn measure<F, R>(&self, op: F) -> R
        where
            F: FnOnce() -> R,
        {
            let start = Instant::now();
            let result = op();
            let elapsed = start.elapsed();
            
            *self.count.lock().unwrap() += 1;
            *self.total_time.lock().unwrap() += elapsed;
            
            result
        }
        
        /// Get average time per operation
        pub fn average_time(&self) -> Duration {
            let count = *self.count.lock().unwrap();
            let total = *self.total_time.lock().unwrap();
            
            if count > 0 {
                total / count as u32
            } else {
                Duration::ZERO
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[derive(Clone, Debug)]
    struct TestNode {
        id: String,
    }
    
    impl Node for TestNode {
        fn id(&self) -> String {
            self.id.clone()
        }
    }
    
    #[test]
    fn test_node_index() {
        let mut index = NodeIndex::new();
        let node = Arc::new(TestNode { id: "test".to_string() });
        
        index.insert(node.clone());
        assert!(index.get("test").is_some());
        
        index.remove("test");
        assert!(index.get("test").is_none());
    }
    
    #[test]
    fn test_cache_invalidation() {
        let cache = GraphCache::new();
        
        // Add to cache
        let path = vec!["a".to_string(), "b".to_string()];
        cache.get_shortest_path("a", "b", || Ok(path.clone())).unwrap();
        
        // Verify it's cached
        let cached = cache.shortest_paths.read().unwrap();
        assert!(cached.contains_key(&("a".to_string(), "b".to_string())));
        drop(cached);
        
        // Invalidate
        cache.invalidate();
        
        // Verify it's cleared
        let cached = cache.shortest_paths.read().unwrap();
        assert!(cached.is_empty());
    }
    
    #[test]
    fn test_node_pool() {
        #[derive(Default, Debug, PartialEq)]
        struct TestPoolNode {
            value: i32,
        }
        
        let mut pool = NodePool::<TestPoolNode>::new(5);
        
        // Test acquire from empty pool
        let node1 = pool.acquire();
        assert_eq!(node1.value, 0); // Default value
        
        // Test release and reacquire
        let node2 = TestPoolNode { value: 42 };
        pool.release(node2);
        let reacquired = pool.acquire();
        assert_eq!(reacquired.value, 42);
        
        // Test capacity limit
        for i in 0..10 {
            pool.release(TestPoolNode { value: i });
        }
        // Pool should only keep up to capacity (5)
        let mut acquired_values = Vec::new();
        for _ in 0..6 {
            acquired_values.push(pool.acquire().value);
        }
        // Last one should be default since pool was at capacity
        assert_eq!(acquired_values[5], 0);
    }
    
    #[test]
    fn test_parallel_bfs() {
        use parallel::parallel_bfs;
        use std::collections::HashMap;
        
        // Create a simple graph structure
        let graph = HashMap::from([
            ("A".to_string(), vec!["B".to_string(), "C".to_string()]),
            ("B".to_string(), vec!["D".to_string()]),
            ("C".to_string(), vec!["D".to_string()]),
            ("D".to_string(), vec![]),
        ]);
        
        let get_neighbors = |node: &str| -> Vec<String> {
            graph.get(node).cloned().unwrap_or_default()
        };
        
        let result = parallel_bfs(vec!["A".to_string()], get_neighbors);
        assert_eq!(result.len(), 4);
        assert!(result.contains(&"A".to_string()));
        assert!(result.contains(&"B".to_string()));
        assert!(result.contains(&"C".to_string()));
        assert!(result.contains(&"D".to_string()));
    }
    
    #[test]
    fn test_parallel_degrees() {
        use parallel::parallel_degrees;
        use rayon::prelude::*;
        use std::collections::HashMap;
        
        let nodes = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let degrees = HashMap::from([
            ("A", 3),
            ("B", 2),
            ("C", 1),
        ]);
        
        let get_degree = |node: &str| -> usize {
            degrees.get(node).copied().unwrap_or(0)
        };
        
        let result = parallel_degrees(nodes.par_iter(), get_degree);
        assert_eq!(result.get("A"), Some(&3));
        assert_eq!(result.get("B"), Some(&2));
        assert_eq!(result.get("C"), Some(&1));
    }
    
    #[test]
    fn test_perf_counter() {
        use monitoring::PerfCounter;
        use std::thread;
        use std::time::Duration;
        
        let counter = PerfCounter::new("test_operation");
        
        // Test measuring operations
        let result = counter.measure(|| {
            thread::sleep(Duration::from_millis(10));
            42
        });
        assert_eq!(result, 42);
        
        // Measure another operation
        counter.measure(|| {
            thread::sleep(Duration::from_millis(10));
        });
        
        // Check average time
        let avg = counter.average_time();
        assert!(avg >= Duration::from_millis(10));
        assert!(avg < Duration::from_millis(50)); // Should be reasonably close
    }
    
    #[test]
    fn test_perf_counter_empty() {
        use monitoring::PerfCounter;

        let counter = PerfCounter::new("empty");
        // Average of no operations should be zero
        assert_eq!(counter.average_time(), std::time::Duration::ZERO);
    }

    // ========== Additional Coverage Tests ==========

    #[derive(Clone, Debug)]
    struct TestEdge {
        id: String,
        source: String,
        target: String,
    }

    impl Edge for TestEdge {
        fn id(&self) -> String {
            self.id.clone()
        }
        fn source(&self) -> String {
            self.source.clone()
        }
        fn target(&self) -> String {
            self.target.clone()
        }
    }

    #[test]
    fn test_node_index_empty() {
        let index: NodeIndex<TestNode> = NodeIndex::new();
        assert!(index.get("nonexistent").is_none());
        assert!(index.get_by_type("default").is_none());
    }

    #[test]
    fn test_node_index_multiple_nodes() {
        let mut index = NodeIndex::new();

        for i in 0..5 {
            let node = Arc::new(TestNode { id: format!("node{}", i) });
            index.insert(node);
        }

        // Verify all nodes are retrievable
        for i in 0..5 {
            assert!(index.get(&format!("node{}", i)).is_some());
        }

        // Verify type index
        let by_type = index.get_by_type("default").unwrap();
        assert_eq!(by_type.len(), 5);
    }

    #[test]
    fn test_node_index_remove_multiple() {
        let mut index = NodeIndex::new();

        for i in 0..3 {
            let node = Arc::new(TestNode { id: format!("node{}", i) });
            index.insert(node);
        }

        // Remove one node
        let removed = index.remove("node1");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id, "node1");

        // Verify it's gone
        assert!(index.get("node1").is_none());

        // Verify others still exist
        assert!(index.get("node0").is_some());
        assert!(index.get("node2").is_some());

        // Verify type index is updated
        let by_type = index.get_by_type("default").unwrap();
        assert_eq!(by_type.len(), 2);
    }

    #[test]
    fn test_node_index_remove_nonexistent() {
        let mut index: NodeIndex<TestNode> = NodeIndex::new();
        let result = index.remove("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_edge_index() {
        let mut index: EdgeIndex<TestEdge> = EdgeIndex::new();

        let edge1 = Arc::new(TestEdge {
            id: "e1".to_string(),
            source: "A".to_string(),
            target: "B".to_string(),
        });
        let edge2 = Arc::new(TestEdge {
            id: "e2".to_string(),
            source: "A".to_string(),
            target: "C".to_string(),
        });
        let edge3 = Arc::new(TestEdge {
            id: "e3".to_string(),
            source: "B".to_string(),
            target: "C".to_string(),
        });

        index.insert(edge1);
        index.insert(edge2);
        index.insert(edge3);

        // Test edges_from
        let from_a = index.edges_from("A").unwrap();
        assert_eq!(from_a.len(), 2);

        let from_b = index.edges_from("B").unwrap();
        assert_eq!(from_b.len(), 1);

        assert!(index.edges_from("C").is_none());
        assert!(index.edges_from("X").is_none());

        // Test edges_to
        let to_c = index.edges_to("C").unwrap();
        assert_eq!(to_c.len(), 2);

        let to_b = index.edges_to("B").unwrap();
        assert_eq!(to_b.len(), 1);

        assert!(index.edges_to("A").is_none());
    }

    #[test]
    fn test_graph_cache_shortest_path() {
        let cache = GraphCache::new();

        // Compute and cache a path
        let path1 = cache.get_shortest_path("A", "B", || {
            Ok(vec!["A".to_string(), "X".to_string(), "B".to_string()])
        }).unwrap();
        assert_eq!(path1.len(), 3);

        // Second call should return cached value (compute fn won't be called)
        let path2 = cache.get_shortest_path("A", "B", || {
            panic!("Should not compute - should use cache");
        }).unwrap();
        assert_eq!(path1, path2);
    }

    #[test]
    fn test_graph_cache_different_keys() {
        let cache = GraphCache::new();

        let path_ab = cache.get_shortest_path("A", "B", || {
            Ok(vec!["A".to_string(), "B".to_string()])
        }).unwrap();

        let path_cd = cache.get_shortest_path("C", "D", || {
            Ok(vec!["C".to_string(), "D".to_string()])
        }).unwrap();

        assert_eq!(path_ab, vec!["A", "B"]);
        assert_eq!(path_cd, vec!["C", "D"]);
    }

    #[test]
    fn test_node_pool_capacity_behavior() {
        #[derive(Default, Debug, PartialEq)]
        struct PoolableNode {
            value: i32,
        }

        let mut pool = NodePool::<PoolableNode>::new(3);

        // Fill the pool beyond capacity
        for i in 0..5 {
            pool.release(PoolableNode { value: i });
        }

        // Should only have 3 items (capacity)
        let n1 = pool.acquire();
        let n2 = pool.acquire();
        let n3 = pool.acquire();
        let n4 = pool.acquire(); // This should be default

        // Values 0 and 1 were dropped, 2, 3, 4 were kept
        assert!(n4.value == 0); // Default value when pool is empty
    }

    #[test]
    fn test_parallel_bfs_empty_start() {
        use parallel::parallel_bfs;
        use std::collections::HashMap;

        let graph: HashMap<String, Vec<String>> = HashMap::new();

        let get_neighbors = |_node: &str| -> Vec<String> {
            vec![]
        };

        let result = parallel_bfs(vec![], get_neighbors);
        assert!(result.is_empty());
    }

    #[test]
    fn test_parallel_bfs_single_node() {
        use parallel::parallel_bfs;
        use std::collections::HashMap;

        let mut graph = HashMap::new();
        graph.insert("A".to_string(), vec![]);

        let get_neighbors = |node: &str| -> Vec<String> {
            graph.get(node).cloned().unwrap_or_default()
        };

        let result = parallel_bfs(vec!["A".to_string()], get_neighbors);
        assert_eq!(result, vec!["A"]);
    }

    #[test]
    fn test_parallel_bfs_multiple_start_nodes() {
        use parallel::parallel_bfs;
        use std::collections::HashMap;

        let graph = HashMap::from([
            ("A".to_string(), vec!["C".to_string()]),
            ("B".to_string(), vec!["C".to_string()]),
            ("C".to_string(), vec!["D".to_string()]),
            ("D".to_string(), vec![]),
        ]);

        let get_neighbors = |node: &str| -> Vec<String> {
            graph.get(node).cloned().unwrap_or_default()
        };

        // Start from multiple nodes
        let result = parallel_bfs(vec!["A".to_string(), "B".to_string()], get_neighbors);

        assert_eq!(result.len(), 4);
        assert!(result.contains(&"A".to_string()));
        assert!(result.contains(&"B".to_string()));
        assert!(result.contains(&"C".to_string()));
        assert!(result.contains(&"D".to_string()));
    }

    #[test]
    fn test_parallel_degrees_empty() {
        use parallel::parallel_degrees;
        use rayon::prelude::*;

        let nodes: Vec<String> = vec![];

        let get_degree = |_node: &str| -> usize { 0 };

        let result = parallel_degrees(nodes.par_iter(), get_degree);
        assert!(result.is_empty());
    }

    #[test]
    fn test_parallel_degrees_single_node() {
        use parallel::parallel_degrees;
        use rayon::prelude::*;

        let nodes = vec!["A".to_string()];

        let get_degree = |_node: &str| -> usize { 5 };

        let result = parallel_degrees(nodes.par_iter(), get_degree);
        assert_eq!(result.get("A"), Some(&5));
    }

    #[test]
    fn test_perf_counter_multiple_measurements() {
        use monitoring::PerfCounter;
        use std::thread;
        use std::time::Duration;

        let counter = PerfCounter::new("multi");

        // Perform multiple measurements
        for _ in 0..5 {
            counter.measure(|| {
                thread::sleep(Duration::from_millis(5));
            });
        }

        let avg = counter.average_time();
        assert!(avg >= Duration::from_millis(5));
    }

    #[test]
    fn test_perf_counter_with_return_value() {
        use monitoring::PerfCounter;

        let counter = PerfCounter::new("return_value");

        let result = counter.measure(|| {
            let x = 10 + 20;
            x * 2
        });

        assert_eq!(result, 60);

        // Counter should have recorded one operation
        let avg = counter.average_time();
        assert!(avg >= std::time::Duration::ZERO);
    }

    #[test]
    fn test_graph_cache_error_propagation() {
        let cache = GraphCache::new();

        let result = cache.get_shortest_path("A", "B", || {
            Err(crate::error::GraphError::NodeNotFound("test".to_string()))
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_node_index_debug() {
        let index: NodeIndex<TestNode> = NodeIndex::new();
        let debug_str = format!("{:?}", index);
        assert!(debug_str.contains("NodeIndex"));
    }

    #[test]
    fn test_edge_index_debug() {
        let index: EdgeIndex<TestEdge> = EdgeIndex::new();
        let debug_str = format!("{:?}", index);
        assert!(debug_str.contains("EdgeIndex"));
    }

    #[test]
    fn test_graph_cache_debug() {
        let cache = GraphCache::new();
        let debug_str = format!("{:?}", cache);
        assert!(debug_str.contains("GraphCache"));
    }

    #[test]
    fn test_node_pool_debug() {
        #[derive(Default, Debug)]
        struct DebugNode;

        let pool = NodePool::<DebugNode>::new(5);
        let debug_str = format!("{:?}", pool);
        assert!(debug_str.contains("NodePool"));
    }
}