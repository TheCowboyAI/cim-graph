# CIM Graph Benchmarks

This directory contains performance benchmarks for the CIM Graph library.

## Available Benchmarks

### 1. Graph Operations (`graph_operations.rs`)
Benchmarks core graph operations across different graph sizes:
- **add_node**: Adding nodes to graphs of various sizes
- **add_edge**: Adding edges to graphs with different node counts
- **get_node**: Node lookup performance
- **get_edges**: Edge retrieval performance  
- **remove_node**: Node removal with connected edges
- **remove_edge**: Edge removal performance
- **get_all_nodes**: Full node iteration
- **get_all_edges**: Full edge iteration

### 2. Graph Types (`graph_types.rs`)
Benchmarks operations specific to each graph type:
- **IPLD Graph**: Block addition, linking, and retrieval
- **Context Graph**: Context and relationship management
- **Workflow Graph**: Task creation and dependency tracking
- **Concept Graph**: Concept hierarchy and semantic relations
- **Composed Graph**: Mixed-type node and edge operations

### 3. Graph Algorithms (`graph_algorithms.rs`)
Benchmarks graph algorithm performance:
- **Pathfinding**: Shortest path and all paths algorithms
- **Traversal**: BFS and DFS traversal
- **Metrics**: Centrality and clustering coefficient calculations
- **Analysis**: Topological sorting
- **Complex Operations**: Combined algorithm pipelines

### 4. Serialization (`serialization.rs`)
Benchmarks serialization/deserialization performance:
- **JSON serialization**: To/from JSON for all graph types
- **Pretty JSON**: Formatted JSON output
- **Binary serialization**: Compact binary format
- **Round-trip**: Full serialization and deserialization cycles

## Running Benchmarks

Run all benchmarks:
```bash
cargo bench
```

Run specific benchmark suite:
```bash
cargo bench graph_operations
cargo bench graph_types
cargo bench graph_algorithms
cargo bench serialization
```

Run specific benchmark:
```bash
cargo bench add_node
cargo bench shortest_path
```

## Benchmark Configuration

The benchmarks test with various graph sizes to measure scalability:
- Small graphs: 10 nodes
- Medium graphs: 100-500 nodes  
- Large graphs: 1000-10000 nodes

Edge density is controlled by edge factors:
- Sparse: 1.5x edges to nodes
- Normal: 2-3x edges to nodes
- Dense: 4x+ edges to nodes

## Interpreting Results

Criterion generates detailed HTML reports in `target/criterion/` with:
- Performance over time
- Statistical analysis
- Comparison between runs
- Outlier detection

Key metrics to watch:
- **Throughput**: Operations per second
- **Latency**: Time per operation
- **Scalability**: Performance vs graph size
- **Memory**: Peak memory usage (when profiling)

## Adding New Benchmarks

To add a new benchmark:

1. Add benchmark function to appropriate file
2. Use criterion's benchmark groups for organization
3. Test multiple input sizes for scalability analysis
4. Use `black_box` to prevent compiler optimizations
5. Add description to this README

Example:
```rust
fn bench_new_operation(c: &mut Criterion) {
    let mut group = c.benchmark_group("new_operation");
    
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, &size| {
                // Setup
                let graph = create_graph(size);
                
                // Benchmark
                b.iter(|| {
                    black_box(graph.new_operation())
                });
            },
        );
    }
    
    group.finish();
}
```