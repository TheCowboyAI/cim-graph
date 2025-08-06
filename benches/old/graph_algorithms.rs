use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use cim_graph::algorithms::{shortest_path, all_paths, dfs, bfs, topological_sort, centrality, clustering_coefficient};
use cim_graph::core::{EventGraph, GenericEdge, GenericNode, GraphType};

/// Create a graph with a specific structure for benchmarking
fn create_benchmark_graph(nodes: usize, edge_factor: f64) -> (EventGraph<GenericNode<String>, GenericEdge<String>>, Vec<String>) {
    let mut graph = EventGraph::<GenericNode<String>, GenericEdge<String>>::new(GraphType::Generic);
    let mut node_ids = Vec::new();
    
    // Create nodes
    for i in 0..nodes {
        let node = GenericNode::new(format!("node_{}", i), format!("Node {}", i));
        let node_id = graph.add_node(node).unwrap();
        node_ids.push(node_id);
    }
    
    // Create edges based on edge_factor
    let num_edges = (nodes as f64 * edge_factor) as usize;
    for i in 0..num_edges {
        let from = i % nodes;
        let to = (i * 7 + 3) % nodes; // Use prime numbers for better distribution
        
        if from != to {
            let edge = GenericEdge::new(
                node_ids[from].clone(),
                node_ids[to].clone(),
                "connects".to_string(),
            );
            graph.add_edge(edge).ok();
        }
    }
    
    (graph, node_ids)
}

/// Create a directed acyclic graph (DAG) for specific algorithms
fn create_dag(layers: usize, nodes_per_layer: usize) -> (EventGraph<GenericNode<String>, GenericEdge<String>>, Vec<String>) {
    let mut graph = EventGraph::<GenericNode<String>, GenericEdge<String>>::new(GraphType::Generic);
    let mut node_ids = Vec::new();
    let mut layer_nodes = Vec::new();
    
    // Create nodes layer by layer
    for layer in 0..layers {
        let mut current_layer = Vec::new();
        for n in 0..nodes_per_layer {
            let node = GenericNode::new(
                format!("node_{}_{}", layer, n),
                format!("Layer {} Node {}", layer, n),
            );
            let node_id = graph.add_node(node).unwrap();
            node_ids.push(node_id.clone());
            current_layer.push(node_id);
        }
        layer_nodes.push(current_layer);
    }
    
    // Connect layers
    for layer in 0..layers - 1 {
        for from_idx in 0..nodes_per_layer {
            for to_idx in 0..nodes_per_layer {
                if (from_idx + to_idx) % 3 != 0 { // Skip some connections
                    let edge = GenericEdge::new(
                        layer_nodes[layer][from_idx].clone(),
                        layer_nodes[layer + 1][to_idx].clone(),
                        "flows_to".to_string(),
                    );
                    graph.add_edge(edge).ok();
                }
            }
        }
    }
    
    (graph, node_ids)
}

/// Benchmark pathfinding algorithms
fn bench_pathfinding(c: &mut Criterion) {
    let mut group = c.benchmark_group("pathfinding");
    
    // Shortest path
    for size in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("shortest_path", size),
            size,
            |b, &size| {
                let (graph, node_ids) = create_benchmark_graph(size, 2.5);
                let start = &node_ids[0];
                let end = &node_ids[size - 1];
                
                b.iter(|| black_box(shortest_path(&graph, start, end)));
            },
        );
    }
    
    
    // All paths
    for size in [10, 20, 30].iter() { // Smaller sizes due to exponential complexity
        group.bench_with_input(
            BenchmarkId::new("all_paths", size),
            size,
            |b, &size| {
                let (graph, node_ids) = create_benchmark_graph(size, 1.5); // Fewer edges
                let start = &node_ids[0];
                let end = &node_ids[size - 1];
                
                b.iter(|| {
                    let paths = all_paths(&graph, start, end, 10).unwrap_or_default();
                    black_box(paths.len())
                });
            },
        );
    }
    
    // Has path
    for size in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("has_path", size),
            size,
            |b, &size| {
                let (graph, node_ids) = create_benchmark_graph(size, 2.0);
                let start = &node_ids[0];
                let end = &node_ids[size - 1];
                
                // Use shortest_path to check if path exists
                b.iter(|| {
                    let path = shortest_path(&graph, start, end).unwrap_or(None);
                    black_box(path.is_some())
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark traversal algorithms
fn bench_traversal(c: &mut Criterion) {
    let mut group = c.benchmark_group("traversal");
    
    // Breadth-first search
    for size in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("bfs", size),
            size,
            |b, &size| {
                let (graph, node_ids) = create_benchmark_graph(size, 3.0);
                let start = &node_ids[0];
                
                b.iter(|| {
                    let visited = bfs(&graph, start).unwrap_or_default();
                    black_box(visited.len())
                });
            },
        );
    }
    
    // Depth-first search
    for size in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("dfs", size),
            size,
            |b, &size| {
                let (graph, node_ids) = create_benchmark_graph(size, 3.0);
                let start = &node_ids[0];
                
                b.iter(|| {
                    let visited = dfs(&graph, start).unwrap_or_default();
                    black_box(visited.len())
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark graph metrics
fn bench_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("metrics");
    
    // Degree centrality
    for size in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("centrality", size),
            size,
            |b, &size| {
                let (graph, _) = create_benchmark_graph(size, 3.0);
                
                b.iter(|| {
                    let centrality = centrality(&graph).unwrap_or_default();
                    black_box(centrality.len())
                });
            },
        );
    }
    
    // Clustering coefficient
    for size in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("clustering_coefficient", size),
            size,
            |b, &size| {
                let (graph, _) = create_benchmark_graph(size, 4.0);
                
                b.iter(|| black_box(clustering_coefficient(&graph).unwrap_or_default()));
            },
        );
    }
    
    group.finish();
}

/// Benchmark graph analysis algorithms
fn bench_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("analysis");
    
    // Topological sort
    for size in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("topological_sort", size),
            size,
            |b, &size| {
                let (graph, _) = create_dag(size / 10 + 1, 10);
                
                b.iter(|| {
                    let sorted = topological_sort(&graph).unwrap_or_default();
                    black_box(sorted.len())
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark complex graph operations
fn bench_complex_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_operations");
    
    // Graph creation with various densities
    for (size, density) in [(50, 1.5), (100, 2.0), (200, 2.5), (500, 3.0)].iter() {
        group.bench_with_input(
            BenchmarkId::new("graph_creation", format!("{}_{}", size, density)),
            &(*size, *density),
            |b, &(size, density)| {
                b.iter(|| {
                    let (graph, _) = create_benchmark_graph(size, density);
                    black_box(graph.node_count())
                });
            },
        );
    }
    
    // Full graph analysis pipeline
    for size in [20, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("full_analysis", size),
            size,
            |b, &size| {
                let (graph, node_ids) = create_benchmark_graph(size, 2.5);
                
                b.iter(|| {
                    // Perform multiple analyses
                    let degree_cent = centrality(&graph).unwrap_or_default();
                    let clustering = clustering_coefficient(&graph).unwrap_or_default();
                    
                    // Do some path finding
                    let start = &node_ids[0];
                    let end = &node_ids[size - 1];
                    let shortest = shortest_path(&graph, start, end).unwrap_or(None);
                    
                    // Do traversals
                    let dfs_result = dfs(&graph, start).unwrap_or_default();
                    let bfs_result = bfs(&graph, start).unwrap_or_default();
                    
                    black_box((
                        degree_cent.len(),
                        clustering,
                        shortest.is_some(),
                        dfs_result.len(),
                        bfs_result.len(),
                    ))
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_pathfinding,
    bench_traversal,
    bench_metrics,
    bench_analysis,
    bench_complex_operations
);
criterion_main!(benches);