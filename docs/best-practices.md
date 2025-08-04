# Best Practices Guide

This guide covers recommended patterns, performance tips, and common pitfalls when using CIM Graph.

## Table of Contents

1. [Design Patterns](#design-patterns)
2. [Performance Optimization](#performance-optimization)
3. [Error Handling](#error-handling)
4. [Testing Strategies](#testing-strategies)
5. [Graph Modeling](#graph-modeling)
6. [Memory Management](#memory-management)
7. [Concurrency](#concurrency)
8. [Common Pitfalls](#common-pitfalls)

## Design Patterns

### Builder Pattern for Complex Graphs

Use builders to construct graphs with validation:

```rust
use cim_graph::{GraphBuilder, ValidationRules};

// Good: Use builder with validation
let graph = GraphBuilder::new()
    .with_capacity(1000, 2000)  // Pre-allocate for known size
    .with_validation(ValidationRules::new()
        .require_dag(true)
        .max_node_degree(100)
        .forbid_self_loops(true))
    .with_metadata("version", "1.0")
    .with_metadata("environment", "production")
    .build()?;

// Avoid: Direct construction without validation
let graph = Graph::new(); // Missing capacity hints and validation
```

### Factory Pattern for Node Types

Create specialized node factories:

```rust
pub struct NodeFactory {
    id_generator: IdGenerator,
    default_metadata: HashMap<String, Value>,
}

impl NodeFactory {
    pub fn create_user(&mut self, name: &str, email: &str) -> UserNode {
        UserNode {
            id: self.id_generator.next(),
            name: name.to_string(),
            email: email.to_string(),
            created_at: Utc::now(),
            metadata: self.default_metadata.clone(),
        }
    }
    
    pub fn create_order(&mut self, user_id: NodeId, total: f64) -> OrderNode {
        OrderNode {
            id: self.id_generator.next(),
            user_id,
            total,
            status: OrderStatus::Pending,
            created_at: Utc::now(),
            metadata: self.default_metadata.clone(),
        }
    }
}
```

### Strategy Pattern for Graph Operations

Make operations configurable:

```rust
pub trait TraversalStrategy {
    fn should_visit(&self, node: &Node, depth: usize) -> bool;
    fn visit_order(&self) -> VisitOrder;
}

pub struct DepthLimitedStrategy {
    max_depth: usize,
}

pub struct TypeFilterStrategy {
    allowed_types: HashSet<String>,
}

// Use strategies
let strategy = DepthLimitedStrategy { max_depth: 3 };
let results = graph.traverse_with_strategy(start, strategy)?;
```

## Performance Optimization

### Pre-allocation

Always pre-allocate when you know the size:

```rust
// Good: Pre-allocate capacity
let mut graph = GraphBuilder::new()
    .with_capacity(10_000, 50_000)  // nodes, edges
    .build()?;

// For batch operations
let mut nodes = Vec::with_capacity(1000);
for data in dataset {
    nodes.push(create_node(data));
}
graph.add_nodes_batch(&nodes)?;

// Avoid: Growing dynamically
let mut graph = Graph::new();
for _ in 0..10_000 {
    graph.add_node(node)?; // Causes reallocations
}
```

### Batch Operations

Use batch operations for better performance:

```rust
// Good: Batch operations
let edges: Vec<(NodeId, NodeId, EdgeData)> = prepare_edges();
graph.add_edges_batch(edges)?;

// Good: Transaction-like batching
graph.batch_update(|batch| {
    for node in nodes {
        batch.add_node(node)?;
    }
    for edge in edges {
        batch.add_edge(edge)?;
    }
    Ok(())
})?;

// Avoid: Individual operations in loops
for (from, to, data) in edges {
    graph.add_edge(from, to, data)?; // Many individual operations
}
```

### Index Optimization

Maintain indices for frequent queries:

```rust
pub struct IndexedGraph<G: Graph> {
    graph: G,
    type_index: HashMap<String, Vec<NodeId>>,
    property_index: HashMap<(String, Value), Vec<NodeId>>,
}

impl<G: Graph> IndexedGraph<G> {
    pub fn add_node(&mut self, node: Node) -> Result<NodeId> {
        let id = self.graph.add_node(node.clone())?;
        
        // Update indices
        self.type_index
            .entry(node.node_type)
            .or_default()
            .push(id);
            
        for (key, value) in &node.properties {
            self.property_index
                .entry((key.clone(), value.clone()))
                .or_default()
                .push(id);
        }
        
        Ok(id)
    }
    
    // O(1) lookup by type
    pub fn nodes_by_type(&self, node_type: &str) -> &[NodeId] {
        self.type_index.get(node_type).map(|v| v.as_slice()).unwrap_or(&[])
    }
}
```

### Lazy Loading

Load data only when needed:

```rust
pub struct LazyNode {
    id: NodeId,
    core_data: CoreData,
    full_data: OnceCell<FullData>,
    loader: Arc<dyn DataLoader>,
}

impl LazyNode {
    pub fn get_full_data(&self) -> Result<&FullData> {
        self.full_data.get_or_try_init(|| {
            self.loader.load(self.id)
        })
    }
}
```

## Error Handling

### Use Type-Safe Errors

Define specific error types:

```rust
#[derive(thiserror::Error, Debug)]
pub enum GraphOperationError {
    #[error("Node {0} not found")]
    NodeNotFound(NodeId),
    
    #[error("Edge between {0} and {1} already exists")]
    DuplicateEdge(NodeId, NodeId),
    
    #[error("Operation would create cycle")]
    CycleDetected,
    
    #[error("Graph validation failed: {0}")]
    ValidationError(String),
}

// Use Result type alias
pub type Result<T> = std::result::Result<T, GraphOperationError>;
```

### Graceful Degradation

Handle errors without panicking:

```rust
// Good: Handle errors gracefully
pub fn find_path_safe(graph: &Graph, start: NodeId, end: NodeId) -> Option<Vec<NodeId>> {
    match dijkstra(graph, start, Some(end)) {
        Ok(paths) => paths.get(&end).map(|(_, path)| path.clone()),
        Err(e) => {
            log::warn!("Pathfinding failed: {}", e);
            None
        }
    }
}

// Avoid: Unwrapping in library code
pub fn find_path_unsafe(graph: &Graph, start: NodeId, end: NodeId) -> Vec<NodeId> {
    dijkstra(graph, start, Some(end)).unwrap()[&end].1.clone() // Can panic!
}
```

### Transaction Rollback

Implement rollback for failed operations:

```rust
pub struct TransactionalGraph<G: Graph> {
    graph: G,
    transaction_log: Vec<Operation>,
}

impl<G: Graph> TransactionalGraph<G> {
    pub fn transaction<F, R>(&mut self, f: F) -> Result<R>
    where
        F: FnOnce(&mut TransactionContext<G>) -> Result<R>,
    {
        let checkpoint = self.create_checkpoint();
        let mut ctx = TransactionContext::new(&mut self.graph, &mut self.transaction_log);
        
        match f(&mut ctx) {
            Ok(result) => {
                ctx.commit();
                Ok(result)
            }
            Err(e) => {
                self.rollback_to(checkpoint);
                Err(e)
            }
        }
    }
}
```

## Testing Strategies

### Property-Based Testing

Use property-based testing for graph invariants:

```rust
#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_add_remove_invariant(
            nodes in prop::collection::vec(any::<NodeData>(), 0..100),
            edges in prop::collection::vec(any::<(usize, usize)>(), 0..200)
        ) {
            let mut graph = Graph::new();
            let node_ids: Vec<_> = nodes.iter()
                .map(|n| graph.add_node(n.clone()).unwrap())
                .collect();
                
            // Add edges
            for (from_idx, to_idx) in edges {
                if let (Some(&from), Some(&to)) = (node_ids.get(from_idx), node_ids.get(to_idx)) {
                    graph.add_edge(from, to, EdgeData::default()).ok();
                }
            }
            
            // Property: node count should match
            assert_eq!(graph.node_count(), nodes.len());
            
            // Property: graph should be valid
            assert!(graph.validate().is_ok());
        }
    }
}
```

### Snapshot Testing

Test complex graph operations with snapshots:

```rust
#[test]
fn test_complex_algorithm() {
    let graph = create_test_graph();
    let result = complex_algorithm(&graph).unwrap();
    
    // Serialize result for snapshot
    let snapshot = serde_json::to_string_pretty(&result).unwrap();
    
    insta::assert_snapshot!(snapshot);
}
```

### Benchmark Testing

Always benchmark critical operations:

```rust
#[bench]
fn bench_dijkstra_sparse(b: &mut Bencher) {
    let graph = create_sparse_graph(1000, 2000);
    let start = NodeId::from(0);
    let end = NodeId::from(999);
    
    b.iter(|| {
        black_box(dijkstra(&graph, start, Some(end)))
    });
}
```

## Graph Modeling

### Choose the Right Graph Type

Match graph type to your domain:

```rust
// E-commerce: Use Context Graph
let mut order_graph = ContextGraph::new("orders");
order_graph.add_aggregate("Order", order_id, order_data)?;

// Workflow: Use Workflow Graph  
let mut approval_flow = WorkflowGraph::new("approval");
approval_flow.add_state("pending", StateType::Start)?;

// Knowledge: Use Concept Graph
let mut ontology = ConceptGraph::new("products");
ontology.add_concept("Electronics", dimensions)?;

// Don't: Force wrong type
let mut wrong = IpldGraph::new();
wrong.add_cid("Order123", "cbor", 0)?; // Orders aren't content-addressed!
```

### Normalize Graph Structure

Keep graphs normalized:

```rust
// Good: Normalized structure
struct NormalizedGraph {
    users: HashMap<UserId, User>,
    orders: HashMap<OrderId, Order>,
    user_orders: HashMap<UserId, Vec<OrderId>>,
}

// Avoid: Denormalized data
struct DenormalizedNode {
    user: User,
    orders: Vec<Order>, // Duplicated across nodes
}
```

## Memory Management

### Use Weak References for Cycles

Prevent memory leaks in cyclic graphs:

```rust
use std::rc::{Rc, Weak};

struct CyclicNode {
    id: NodeId,
    data: NodeData,
    parent: Option<Weak<RefCell<CyclicNode>>>,
    children: Vec<Rc<RefCell<CyclicNode>>>,
}
```

### Stream Large Graphs

Don't load entire graph into memory:

```rust
pub struct StreamingGraph {
    metadata: GraphMetadata,
    node_reader: Box<dyn NodeReader>,
    edge_reader: Box<dyn EdgeReader>,
}

impl StreamingGraph {
    pub fn nodes(&self) -> impl Iterator<Item = Result<Node>> {
        self.node_reader.read_nodes()
    }
    
    pub fn edges(&self) -> impl Iterator<Item = Result<Edge>> {
        self.edge_reader.read_edges()
    }
}
```

## Concurrency

### Read-Write Locks

Use appropriate locking strategies:

```rust
use std::sync::RwLock;

pub struct ConcurrentGraph {
    inner: RwLock<Graph>,
}

impl ConcurrentGraph {
    pub fn read_operation<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&Graph) -> Result<R>,
    {
        let graph = self.inner.read().unwrap();
        f(&*graph)
    }
    
    pub fn write_operation<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut Graph) -> Result<R>,
    {
        let mut graph = self.inner.write().unwrap();
        f(&mut *graph)
    }
}
```

### Parallel Algorithms

Use parallel iterators for independent operations:

```rust
use rayon::prelude::*;

pub fn parallel_pagerank(graph: &Graph, iterations: usize) -> HashMap<NodeId, f64> {
    let mut ranks = HashMap::new();
    let node_count = graph.node_count() as f64;
    
    // Initialize
    for node in graph.node_indices() {
        ranks.insert(node, 1.0 / node_count);
    }
    
    for _ in 0..iterations {
        let new_ranks: HashMap<_, _> = graph.node_indices()
            .par_iter()
            .map(|&node| {
                let rank = calculate_node_rank(graph, node, &ranks);
                (node, rank)
            })
            .collect();
            
        ranks = new_ranks;
    }
    
    ranks
}
```

## Common Pitfalls

### 1. Not Handling Disconnected Components

```rust
// Bad: Assumes connected graph
let path = dijkstra(&graph, start, end).unwrap(); // Panics if not connected!

// Good: Handle disconnected case
match dijkstra(&graph, start, end) {
    Ok(paths) => {
        if let Some((dist, path)) = paths.get(&end) {
            // Use path
        } else {
            // Nodes not connected
        }
    }
    Err(e) => // Handle error
}
```

### 2. Modifying During Iteration

```rust
// Bad: Modifying while iterating
for node in graph.nodes() {
    if should_remove(node) {
        graph.remove_node(node.id)?; // Iterator invalidated!
    }
}

// Good: Collect first, then modify
let to_remove: Vec<_> = graph.nodes()
    .filter(|n| should_remove(n))
    .map(|n| n.id)
    .collect();
    
for id in to_remove {
    graph.remove_node(id)?;
}
```

### 3. Ignoring Edge Direction

```rust
// Bad: Treating directed graph as undirected
let neighbors = graph.neighbors(node); // Only outgoing!

// Good: Consider both directions when needed
let outgoing = graph.neighbors_directed(node, Outgoing);
let incoming = graph.neighbors_directed(node, Incoming);
let all = outgoing.chain(incoming).collect::<HashSet<_>>();
```

### 4. Not Validating Input

```rust
// Bad: Trust all input
pub fn add_edge_unsafe(&mut self, from: NodeId, to: NodeId) {
    self.edges.push((from, to));
}

// Good: Validate input
pub fn add_edge_safe(&mut self, from: NodeId, to: NodeId) -> Result<()> {
    // Check nodes exist
    if !self.has_node(from) {
        return Err(GraphError::NodeNotFound(from));
    }
    if !self.has_node(to) {
        return Err(GraphError::NodeNotFound(to));
    }
    
    // Check for duplicate
    if self.has_edge(from, to) {
        return Err(GraphError::DuplicateEdge(from, to));
    }
    
    // Check for cycle (if DAG required)
    if self.would_create_cycle(from, to) {
        return Err(GraphError::CycleDetected);
    }
    
    self.edges.push((from, to));
    Ok(())
}
```

### 5. Memory Leaks with Callbacks

```rust
// Bad: Capturing too much in closures
let huge_data = vec![0u8; 1_000_000];
graph.subscribe(move |event| {
    // huge_data is moved here and kept alive!
    println!("Event: {:?}", event);
});

// Good: Capture only what's needed
let summary = summarize(&huge_data);
graph.subscribe(move |event| {
    println!("Event: {:?}, Context: {}", event, summary);
});
```