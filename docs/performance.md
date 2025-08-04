# Performance Guide

This guide covers performance optimization strategies for CIM Graph.

## Table of Contents

1. [Performance Features](#performance-features)
2. [Optimization Techniques](#optimization-techniques)
3. [Benchmarking](#benchmarking)
4. [Best Practices](#best-practices)
5. [Common Pitfalls](#common-pitfalls)

## Performance Features

### Graph Indexing

CIM Graph provides built-in indexing for fast lookups:

```rust
use cim_graph::performance::{NodeIndex, EdgeIndex};

// Create indexes for a graph
let mut node_index = NodeIndex::new();
let mut edge_index = EdgeIndex::new();

// Index nodes for O(1) lookup
node_index.insert(Arc::new(node));

// Index edges by source/target
edge_index.insert(Arc::new(edge));

// Fast lookups
let node = node_index.get("node_id");
let edges = edge_index.edges_from("source_id");
```

### Caching

The library includes a caching layer for expensive computations:

```rust
use cim_graph::performance::GraphCache;

let cache = GraphCache::new();

// Cache shortest paths
let path = cache.get_shortest_path("A", "B", || {
    // Expensive computation only runs if not cached
    algorithms::shortest_path(&graph, "A", "B")
})?;

// Invalidate cache when graph changes
cache.invalidate();
```

### Parallel Operations

Use parallel algorithms for large graphs:

```rust
use cim_graph::performance::parallel;

// Parallel BFS traversal
let visited = parallel::parallel_bfs(
    vec!["start".to_string()],
    |node| graph.neighbors(node).unwrap_or_default()
);

// Parallel degree calculation
let degrees = parallel::parallel_degrees(
    node_ids.par_iter(),
    |node| graph.degree(node)
);
```

### Memory Pooling

Reduce allocations with object pools:

```rust
use cim_graph::performance::NodePool;

let mut pool = NodePool::new(1000);

// Acquire from pool
let mut node = pool.acquire();
// ... use node ...

// Return to pool
pool.release(node);
```

## Optimization Techniques

### 1. Bulk Operations

Always prefer bulk operations over individual ones:

```rust
// Bad: Individual operations
for node in nodes {
    graph.add_node(node)?;
}

// Good: Bulk operation
graph.add_nodes_bulk(nodes)?;
```

### 2. Pre-allocate Capacity

When creating graphs with known sizes:

```rust
let graph = GraphBuilder::new()
    .with_capacity(1000, 5000)  // nodes, edges
    .build();
```

### 3. Use Appropriate Graph Types

Choose the right graph type for your use case:

- **IpldGraph**: Best for content-addressed data
- **WorkflowGraph**: Optimized for state machines
- **ContextGraph**: Efficient for domain modeling
- **ConceptGraph**: Designed for semantic reasoning

### 4. Lazy Evaluation

Defer expensive computations:

```rust
// Use iterators instead of collecting
let high_degree_nodes = graph.nodes()
    .filter(|n| graph.degree(n) > 100)
    .take(10);  // Only process what you need
```

### 5. Index Strategic Properties

Create custom indexes for frequently queried properties:

```rust
// Index nodes by custom property
let mut by_color: HashMap<Color, Vec<NodeId>> = HashMap::new();
for node in graph.nodes() {
    by_color.entry(node.color()).or_default().push(node.id());
}
```

## Benchmarking

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench graph_operations

# Compare with baseline
cargo bench -- --baseline main
```

### Creating Custom Benchmarks

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_add_nodes(c: &mut Criterion) {
    c.bench_function("add 1000 nodes", |b| {
        b.iter(|| {
            let mut graph = WorkflowGraph::new();
            for i in 0..1000 {
                let node = WorkflowNode::new(&format!("n{}", i), "Node", StateType::Normal);
                black_box(graph.add_state(node));
            }
        });
    });
}

criterion_group!(benches, bench_add_nodes);
criterion_main!(benches);
```

### Performance Monitoring

Use the built-in performance counters:

```rust
use cim_graph::performance::monitoring::PerfCounter;

let counter = PerfCounter::new("graph_operations");

let result = counter.measure(|| {
    // Operation to measure
    graph.add_node(node)
});

println!("Average time: {:?}", counter.average_time());
```

## Best Practices

### 1. Minimize Graph Modifications

Graph modifications invalidate caches and indexes:

```rust
// Bad: Many small modifications
for (from, to) in edges {
    graph.add_edge(from, to)?;
    run_algorithm(&graph);  // Cache invalidated each time
}

// Good: Batch modifications
graph.add_edges_bulk(edges)?;
run_algorithm(&graph);  // Cache used efficiently
```

### 2. Use References When Possible

Avoid cloning large structures:

```rust
// Bad: Cloning nodes
let nodes: Vec<Node> = graph.nodes().cloned().collect();

// Good: Using references
let nodes: Vec<&Node> = graph.nodes().collect();
```

### 3. Profile Before Optimizing

Use profiling tools to identify bottlenecks:

```bash
# CPU profiling
cargo install flamegraph
cargo flamegraph --bench graph_operations

# Memory profiling
cargo install cargo-profiling
cargo profiling callgrind --bench graph_operations
```

### 4. Consider Graph Sparsity

Different algorithms perform better on sparse vs dense graphs:

```rust
let density = graph.edge_count() as f64 / 
    (graph.node_count() * (graph.node_count() - 1)) as f64;

if density < 0.1 {
    // Use algorithms optimized for sparse graphs
} else {
    // Use algorithms optimized for dense graphs
}
```

### 5. Tune Algorithm Parameters

Many algorithms have tunable parameters:

```rust
// For BFS on large graphs, use parallel version
if graph.node_count() > 10_000 {
    parallel::parallel_bfs(start_nodes, get_neighbors)
} else {
    algorithms::bfs(&graph, start)
}
```

## Common Pitfalls

### 1. Not Using Indexes

```rust
// Bad: O(n) lookup
let node = graph.nodes().find(|n| n.id() == "target");

// Good: O(1) with index
let node = node_index.get("target");
```

### 2. Excessive String Allocations

```rust
// Bad: Creating strings in hot loops
for i in 0..1000000 {
    let id = format!("node_{}", i);  // Allocation!
    graph.get_node(&id);
}

// Good: Pre-compute or use &str
let ids: Vec<String> = (0..1000000).map(|i| format!("node_{}", i)).collect();
for id in &ids {
    graph.get_node(id);
}
```

### 3. Not Leveraging Parallelism

```rust
// Bad: Sequential processing of independent operations
let results: Vec<_> = nodes.iter()
    .map(|node| expensive_computation(node))
    .collect();

// Good: Parallel processing
use rayon::prelude::*;
let results: Vec<_> = nodes.par_iter()
    .map(|node| expensive_computation(node))
    .collect();
```

### 4. Ignoring Cache Locality

```rust
// Bad: Random access pattern
for _ in 0..1000 {
    let random_node = nodes.choose(&mut rng).unwrap();
    process_node(random_node);
}

// Good: Sequential access
for node in &nodes {
    process_node(node);
}
```

### 5. Over-engineering

Don't optimize prematurely:

```rust
// Often unnecessary
let mut node_cache = LruCache::new(10000);
let mut edge_cache = LruCache::new(50000);
let mut path_cache = HashMap::new();

// Start simple
let graph = WorkflowGraph::new();
// Optimize only after profiling shows it's needed
```

## Performance Checklist

Before deploying to production:

- [ ] Run benchmarks and compare with baseline
- [ ] Profile CPU usage for hot paths
- [ ] Check memory usage and allocations
- [ ] Test with realistic graph sizes
- [ ] Enable parallel algorithms for large graphs
- [ ] Configure appropriate cache sizes
- [ ] Set capacity hints when known
- [ ] Use bulk operations where possible
- [ ] Minimize string allocations in loops
- [ ] Consider graph-specific optimizations

## Advanced Optimizations

### Custom Memory Allocators

For extreme performance, consider custom allocators:

```rust
// Use jemalloc for better multi-threaded performance
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;
```

### SIMD Operations

For numerical computations on node/edge weights:

```rust
// Future: SIMD support for batch operations
// graph.update_weights_simd(&weights);
```

### GPU Acceleration

For massive graphs, consider GPU acceleration:

```rust
// Future: GPU support for algorithms
// let result = graph.pagerank_gpu()?;
```