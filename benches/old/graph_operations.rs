use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use cim_graph::core::{EventGraph, GenericEdge, GenericNode, GraphType};
use uuid::Uuid;

/// Benchmark adding nodes to a graph
fn bench_add_node(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_node");
    
    for size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_batched(
                || {
                    let mut graph = EventGraph::<GenericNode<String>, GenericEdge<String>>::new(GraphType::Generic);
                    // Pre-populate graph to measure performance at different sizes
                    for i in 0..size {
                        let node = GenericNode::new(format!("node_{}", i), format!("Node {}", i));
                        graph.add_node(node).unwrap();
                    }
                    graph
                },
                |mut graph| {
                    let node = GenericNode::new(
                        format!("new_node_{}", Uuid::new_v4()),
                        "New Node".to_string(),
                    );
                    black_box(graph.add_node(node))
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

/// Benchmark adding edges to a graph
fn bench_add_edge(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_edge");
    
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_batched(
                || {
                    let mut graph = EventGraph::<GenericNode<String>, GenericEdge<String>>::new(GraphType::Generic);
                    let mut node_ids = Vec::new();
                    
                    // Create nodes
                    for i in 0..size {
                        let node = GenericNode::new(format!("node_{}", i), format!("Node {}", i));
                        let node_id = graph.add_node(node).unwrap();
                        node_ids.push(node_id);
                    }
                    
                    // Add some edges (50% connectivity)
                    for i in 0..size / 2 {
                        for j in (i + 1)..size.min(i + 10) {
                            let edge = GenericEdge::new(
                                node_ids[i].clone(),
                                node_ids[j].clone(),
                                "connects_to".to_string(),
                            );
                            graph.add_edge(edge).ok();
                        }
                    }
                    
                    (graph, node_ids)
                },
                |(mut graph, node_ids)| {
                    if node_ids.len() >= 2 {
                        let edge = GenericEdge::new(
                            node_ids[0].clone(),
                            node_ids[1].clone(),
                            "new_edge".to_string(),
                        );
                        black_box(graph.add_edge(edge))
                    } else {
                        black_box(Ok(String::new()))
                    }
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

/// Benchmark getting nodes from a graph
fn bench_get_node(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_node");
    
    for size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut graph = EventGraph::<GenericNode<String>, GenericEdge<String>>::new(GraphType::Generic);
            let mut node_ids = Vec::new();
            
            for i in 0..size {
                let node = GenericNode::new(format!("node_{}", i), format!("Node {}", i));
                let node_id = graph.add_node(node).unwrap();
                node_ids.push(node_id);
            }
            
            let target_id = &node_ids[size / 2]; // Get middle node
            
            b.iter(|| black_box(graph.get_node(target_id)));
        });
    }
    group.finish();
}

/// Benchmark getting edges from a graph
fn bench_get_edges(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_edges");
    
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut graph = EventGraph::<GenericNode<String>, GenericEdge<String>>::new(GraphType::Generic);
            let mut node_ids = Vec::new();
            
            // Create nodes
            for i in 0..size {
                let node = GenericNode::new(format!("node_{}", i), format!("Node {}", i));
                let node_id = graph.add_node(node).unwrap();
                node_ids.push(node_id);
            }
            
            // Add edges (full connectivity would be too much, so we limit connections)
            for i in 0..size {
                for j in (i + 1)..size.min(i + 5) {
                    let edge = GenericEdge::new(
                        node_ids[i].clone(),
                        node_ids[j].clone(),
                        "connects_to".to_string(),
                    );
                    graph.add_edge(edge).ok();
                }
            }
            
            let target_id = &node_ids[size / 2];
            
            b.iter(|| {
                let edges = graph.edges_between(target_id, target_id);
                black_box(edges.len())
            });
        });
    }
    group.finish();
}

/// Benchmark node removal
fn bench_remove_node(c: &mut Criterion) {
    let mut group = c.benchmark_group("remove_node");
    
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_batched(
                || {
                    let mut graph = EventGraph::<GenericNode<String>, GenericEdge<String>>::new(GraphType::Generic);
                    let mut node_ids = Vec::new();
                    
                    for i in 0..size {
                        let node = GenericNode::new(format!("node_{}", i), format!("Node {}", i));
                        let node_id = graph.add_node(node).unwrap();
                        node_ids.push(node_id);
                    }
                    
                    (graph, node_ids)
                },
                |(mut graph, node_ids)| {
                    if !node_ids.is_empty() {
                        black_box(graph.remove_node(&node_ids[0]))
                    } else {
                        black_box(Err(cim_graph::error::GraphError::NodeNotFound("empty".to_string())))
                    }
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

/// Benchmark edge removal
fn bench_remove_edge(c: &mut Criterion) {
    let mut group = c.benchmark_group("remove_edge");
    
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_batched(
                || {
                    let mut graph = EventGraph::<GenericNode<String>, GenericEdge<String>>::new(GraphType::Generic);
                    let mut node_ids = Vec::new();
                    let mut edge_ids = Vec::new();
                    
                    // Create nodes
                    for i in 0..size {
                        let node = GenericNode::new(format!("node_{}", i), format!("Node {}", i));
                        let node_id = graph.add_node(node).unwrap();
                        node_ids.push(node_id);
                    }
                    
                    // Add edges
                    for i in 0..(size as usize).saturating_sub(1) {
                        let edge = GenericEdge::new(
                            node_ids[i].clone(),
                            node_ids[i + 1].clone(),
                            "connects_to".to_string(),
                        );
                        if let Ok(edge_id) = graph.add_edge(edge) {
                            edge_ids.push(edge_id);
                        }
                    }
                    
                    (graph, edge_ids)
                },
                |(mut graph, edge_ids)| {
                    if !edge_ids.is_empty() {
                        black_box(graph.remove_edge(&edge_ids[0]))
                    } else {
                        black_box(Err(cim_graph::error::GraphError::EdgeNotFound("empty".to_string())))
                    }
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

/// Benchmark getting all nodes
fn bench_get_all_nodes(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_all_nodes");
    
    for size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut graph = EventGraph::<GenericNode<String>, GenericEdge<String>>::new(GraphType::Generic);
            
            for i in 0..size {
                let node = GenericNode::new(format!("node_{}", i), format!("Node {}", i));
                graph.add_node(node);
            }
            
            b.iter(|| {
                let nodes = graph.node_ids();
                black_box(nodes.len())
            });
        });
    }
    group.finish();
}

/// Benchmark getting all edges
fn bench_get_all_edges(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_all_edges");
    
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut graph = EventGraph::<GenericNode<String>, GenericEdge<String>>::new(GraphType::Generic);
            let mut node_ids = Vec::new();
            
            // Create nodes
            for i in 0..size {
                let node = GenericNode::new(format!("node_{}", i), format!("Node {}", i));
                let node_id = graph.add_node(node).unwrap();
                node_ids.push(node_id);
            }
            
            // Add edges (creating a ring + some cross connections)
            for i in 0..size {
                // Ring connection
                let edge = GenericEdge::new(
                    node_ids[i].clone(),
                    node_ids[(i + 1) % size].clone(),
                    "next".to_string(),
                );
                graph.add_edge(edge).ok();
                
                // Cross connection
                if i + 2 < size {
                    let edge = GenericEdge::new(
                        node_ids[i].clone(),
                        node_ids[i + 2].clone(),
                        "skip".to_string(),
                    );
                    graph.add_edge(edge).ok();
                }
            }
            
            b.iter(|| {
                let edges = graph.edge_ids();
                black_box(edges.len())
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_add_node,
    bench_add_edge,
    bench_get_node,
    bench_get_edges,
    bench_remove_node,
    bench_remove_edge,
    bench_get_all_nodes,
    bench_get_all_edges
);
criterion_main!(benches);