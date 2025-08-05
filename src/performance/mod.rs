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
pub struct SpatialIndex {
    // Simplified R-tree or quadtree implementation
    _tree: BTreeMap<(i32, i32), Vec<String>>,
}

/// Edge index for fast edge lookups
pub struct EdgeIndex<E: Edge> {
    /// All edges indexed by source node
    by_source: HashMap<String, Vec<Arc<E>>>,
    
    /// All edges indexed by target node
    by_target: HashMap<String, Vec<Arc<E>>>,
    
    /// Direct lookup by edge ID
    by_id: HashMap<String, Arc<E>>,
}

impl<E: Edge> EdgeIndex<E> {
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
pub struct NodePool<N> {
    pool: Vec<N>,
    capacity: usize,
}

impl<N: Default> NodePool<N> {
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
    type Node: Node;
    type Edge: Edge;
    
    /// Get node with caching
    fn get_node_cached(&self, id: &str) -> Option<&Self::Node>;
    
    /// Get edges with indexing
    fn get_edges_indexed(&self, from: &str, to: &str) -> Vec<&Self::Edge>;
    
    /// Bulk operations for better performance
    fn add_nodes_bulk(&mut self, nodes: Vec<Self::Node>) -> Result<()>;
    fn add_edges_bulk(&mut self, edges: Vec<Self::Edge>) -> Result<()>;
}

/// Performance monitoring utilities
pub mod monitoring {
    use std::time::{Duration, Instant};
    use std::sync::Mutex;
    
    /// Simple performance counter
    pub struct PerfCounter {
        #[allow(dead_code)]
        name: String,
        count: Mutex<u64>,
        total_time: Mutex<Duration>,
    }
    
    impl PerfCounter {
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
}