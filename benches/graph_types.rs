use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use cim_graph::graphs::{
    ComposedGraph, ConceptGraph, ContextGraph, IpldGraph, WorkflowGraph,
};
use std::collections::HashMap;
use uuid::Uuid;
use serde_json;

/// Benchmark IPLD graph operations
fn bench_ipld_graph(c: &mut Criterion) {
    let mut group = c.benchmark_group("ipld_graph");

    // Add block benchmark
    group.bench_function("add_block", |b| {
        b.iter_batched(
            || IpldGraph::new(),
            |mut graph| {
                let cid = format!("Qm{}", Uuid::new_v4().to_string().replace("-", ""));
                let mut data = HashMap::new();
                data.insert("type".to_string(), "test_block".to_string());
                data.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());
                black_box(graph.add_block(cid, data))
            },
            BatchSize::SmallInput,
        );
    });

    // Link blocks benchmark
    group.bench_function("link_blocks", |b| {
        b.iter_batched(
            || {
                let mut graph = IpldGraph::new();
                let cid1 = format!("Qm{}", Uuid::new_v4().to_string().replace("-", ""));
                let cid2 = format!("Qm{}", Uuid::new_v4().to_string().replace("-", ""));
                
                let mut data1 = HashMap::new();
                data1.insert("type".to_string(), "block1".to_string());
                graph.add_block(cid1.clone(), data1).unwrap();
                
                let mut data2 = HashMap::new();
                data2.insert("type".to_string(), "block2".to_string());
                graph.add_block(cid2.clone(), data2).unwrap();
                
                (graph, cid1, cid2)
            },
            |(mut graph, cid1, cid2)| {
                black_box(graph.link_blocks(cid1, cid2, "references".to_string()))
            },
            BatchSize::SmallInput,
        );
    });

    // Get block benchmark with different graph sizes
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("get_block", size),
            size,
            |b, &size| {
                let mut graph = IpldGraph::new();
                let mut cids = Vec::new();
                
                for i in 0..size {
                    let cid = format!("Qm{}{}", i, Uuid::new_v4().to_string().replace("-", ""));
                    let mut data = HashMap::new();
                    data.insert("index".to_string(), i.to_string());
                    graph.add_block(cid.clone(), data).unwrap();
                    cids.push(cid);
                }
                
                let target_cid = &cids[size / 2];
                
                b.iter(|| black_box(graph.get_block(target_cid)));
            },
        );
    }

    group.finish();
}

/// Benchmark Context graph operations
fn bench_context_graph(c: &mut Criterion) {
    let mut group = c.benchmark_group("context_graph");

    // Add context benchmark
    group.bench_function("add_context", |b| {
        b.iter_batched(
            || ContextGraph::new(),
            |mut graph| {
                let id = Uuid::new_v4().to_string();
                let mut properties = HashMap::new();
                properties.insert("domain".to_string(), "test".to_string());
                properties.insert("version".to_string(), "1.0".to_string());
                black_box(graph.add_context(id, "TestContext".to_string(), properties))
            },
            BatchSize::SmallInput,
        );
    });

    // Add relationship benchmark
    group.bench_function("add_relationship", |b| {
        b.iter_batched(
            || {
                let mut graph = ContextGraph::new();
                let id1 = Uuid::new_v4().to_string();
                let id2 = Uuid::new_v4().to_string();
                
                graph.add_context(id1.clone(), "Context1".to_string(), HashMap::new()).unwrap();
                graph.add_context(id2.clone(), "Context2".to_string(), HashMap::new()).unwrap();
                
                (graph, id1, id2)
            },
            |(mut graph, id1, id2)| {
                black_box(graph.add_relationship(
                    id1,
                    id2,
                    "relates_to".to_string(),
                    HashMap::new(),
                ))
            },
            BatchSize::SmallInput,
        );
    });

    // Get context benchmark with different sizes
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("get_context", size),
            size,
            |b, &size| {
                let mut graph = ContextGraph::new();
                let mut ids = Vec::new();
                
                for i in 0..size {
                    let id = format!("context_{}", i);
                    let mut properties = HashMap::new();
                    properties.insert("index".to_string(), i.to_string());
                    graph.add_context(id.clone(), format!("Context{}", i), properties).unwrap();
                    ids.push(id);
                }
                
                let target_id = &ids[size / 2];
                
                b.iter(|| black_box(graph.get_context(target_id)));
            },
        );
    }

    group.finish();
}

/// Benchmark Workflow graph operations
fn bench_workflow_graph(c: &mut Criterion) {
    let mut group = c.benchmark_group("workflow_graph");

    // Add task benchmark
    group.bench_function("add_task", |b| {
        b.iter_batched(
            || WorkflowGraph::new(),
            |mut graph| {
                let task_id = Uuid::new_v4().to_string();
                let mut config = HashMap::new();
                config.insert("priority".to_string(), "high".to_string());
                config.insert("timeout".to_string(), "3600".to_string());
                black_box(graph.add_task(task_id, "ProcessData".to_string(), config))
            },
            BatchSize::SmallInput,
        );
    });

    // Add dependency benchmark
    group.bench_function("add_dependency", |b| {
        b.iter_batched(
            || {
                let mut graph = WorkflowGraph::new();
                let task1 = Uuid::new_v4().to_string();
                let task2 = Uuid::new_v4().to_string();
                
                graph.add_task(task1.clone(), "Task1".to_string(), HashMap::new()).unwrap();
                graph.add_task(task2.clone(), "Task2".to_string(), HashMap::new()).unwrap();
                
                (graph, task1, task2)
            },
            |(mut graph, from, to)| {
                black_box(graph.add_dependency(from, to, "depends_on".to_string()))
            },
            BatchSize::SmallInput,
        );
    });

    // Complex workflow creation benchmark
    for size in [5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::new("create_workflow", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut graph = WorkflowGraph::new();
                    let mut task_ids = Vec::new();
                    
                    // Create tasks
                    for i in 0..size {
                        let task_id = format!("task_{}", i);
                        let mut config = HashMap::new();
                        config.insert("step".to_string(), i.to_string());
                        graph.add_task(task_id.clone(), format!("Step{}", i), config).unwrap();
                        task_ids.push(task_id);
                    }
                    
                    // Create linear dependencies
                    for i in 0..size - 1 {
                        graph.add_dependency(
                            task_ids[i].clone(),
                            task_ids[i + 1].clone(),
                            "next".to_string(),
                        ).unwrap();
                    }
                    
                    // Add some parallel branches
                    if size > 4 {
                        for i in (0..size - 2).step_by(3) {
                            graph.add_dependency(
                                task_ids[i].clone(),
                                task_ids[i + 2].clone(),
                                "parallel".to_string(),
                            ).ok();
                        }
                    }
                    
                    black_box(graph)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark Concept graph operations
fn bench_concept_graph(c: &mut Criterion) {
    let mut group = c.benchmark_group("concept_graph");

    // Add concept benchmark
    group.bench_function("add_concept", |b| {
        b.iter_batched(
            || ConceptGraph::new(),
            |mut graph| {
                let concept_id = Uuid::new_v4().to_string();
                let properties = serde_json::json!({
                    "category": "entity",
                    "domain": "test"
                });
                black_box(graph.add_concept(
                    &concept_id,
                    "TestConcept",
                    properties,
                ))
            },
            BatchSize::SmallInput,
        );
    });

    // Add relation benchmark
    group.bench_function("add_relation", |b| {
        b.iter_batched(
            || {
                let mut graph = ConceptGraph::new();
                let concept1 = Uuid::new_v4().to_string();
                let concept2 = Uuid::new_v4().to_string();
                
                graph.add_concept(&concept1, "Concept1", serde_json::json!({})).unwrap();
                graph.add_concept(&concept2, "Concept2", serde_json::json!({})).unwrap();
                
                (graph, concept1, concept2)
            },
            |(mut graph, from, to)| {
                use cim_graph::graphs::concept::SemanticRelation;
                black_box(graph.add_relation(
                    &from,
                    &to,
                    SemanticRelation::SubClassOf,
                ))
            },
            BatchSize::SmallInput,
        );
    });

    // Concept hierarchy benchmark
    for depth in [3, 5, 7, 10].iter() {
        group.bench_with_input(
            BenchmarkId::new("create_hierarchy", depth),
            depth,
            |b, &depth| {
                b.iter(|| {
                    let mut graph = ConceptGraph::new();
                    let mut level_ids = vec![vec![]];
                    
                    // Create root
                    let root_id = "root".to_string();
                    graph.add_concept(&root_id, "Root", serde_json::json!({})).unwrap();
                    level_ids[0].push(root_id);
                    
                    // Create hierarchy
                    for level in 1..=depth {
                        let mut current_level = Vec::new();
                        let nodes_at_level = 2_usize.pow(level as u32).min(100);
                        
                        for i in 0..nodes_at_level {
                            let concept_id = format!("concept_{}_{}", level, i);
                            let attrs = serde_json::json!({
                                "level": level
                            });
                            
                            graph.add_concept(
                                &concept_id,
                                &format!("Concept_{}_{}", level, i),
                                attrs,
                            ).unwrap();
                            
                            // Link to parent
                            let parent_idx = i / 2;
                            if parent_idx < level_ids[level - 1].len() {
                                use cim_graph::graphs::concept::SemanticRelation;
                                graph.add_relation(
                                    &level_ids[level - 1][parent_idx],
                                    &concept_id,
                                    SemanticRelation::SubClassOf,
                                ).unwrap();
                            }
                            
                            current_level.push(concept_id);
                        }
                        
                        level_ids.push(current_level);
                    }
                    
                    black_box(graph)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark Composed graph operations
fn bench_composed_graph(c: &mut Criterion) {
    let mut group = c.benchmark_group("composed_graph");

    // Add composed node benchmark
    group.bench_function("add_composed_node", |b| {
        b.iter_batched(
            || ComposedGraph::new(),
            |mut graph| {
                use cim_graph::graphs::composed::ComposedNode;
                let node_id = Uuid::new_v4().to_string();
                let node = ComposedNode::workflow(node_id, "Task".to_string());
                black_box(graph.add_node(node))
            },
            BatchSize::SmallInput,
        );
    });

    // Add composed edge benchmark
    group.bench_function("add_composed_edge", |b| {
        b.iter_batched(
            || {
                use cim_graph::graphs::composed::ComposedNode;
                let mut graph = ComposedGraph::new();
                let node1 = ComposedNode::context("ctx1".to_string(), "Context1".to_string());
                let node2 = ComposedNode::workflow("wf1".to_string(), "Workflow1".to_string());
                graph.add_node(node1).unwrap();
                graph.add_node(node2).unwrap();
                graph
            },
            |mut graph| {
                use cim_graph::graphs::composed::ComposedEdge;
                let edge = ComposedEdge::new(
                    "ctx1".to_string(),
                    "wf1".to_string(),
                    "triggers".to_string(),
                );
                black_box(graph.add_edge(edge))
            },
            BatchSize::SmallInput,
        );
    });

    // Complex composed graph benchmark
    for num_nodes in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("create_composed", num_nodes),
            num_nodes,
            |b, &num_nodes| {
                b.iter(|| {
                    use cim_graph::graphs::composed::{ComposedNode, ComposedEdge};
                    let mut graph = ComposedGraph::new();
                    let mut node_ids = Vec::new();
                    
                    // Add various types of nodes
                    for i in 0..num_nodes {
                        let node_id = format!("node_{}", i);
                        
                        let node = match i % 4 {
                            0 => ComposedNode::ipld(node_id.clone(), format!("Block_{}", i)),
                            1 => ComposedNode::context(node_id.clone(), format!("Context_{}", i)),
                            2 => ComposedNode::workflow(node_id.clone(), format!("Task_{}", i)),
                            _ => ComposedNode::concept(node_id.clone(), format!("Concept_{}", i)),
                        };
                        
                        graph.add_node(node).unwrap();
                        node_ids.push(node_id);
                    }
                    
                    // Connect nodes with edges
                    for i in 0..num_nodes.saturating_sub(1) {
                        let edge = ComposedEdge::new(
                            node_ids[i].clone(),
                            node_ids[i + 1].clone(),
                            "connects".to_string(),
                        );
                        graph.add_edge(edge).ok();
                    }
                    
                    black_box(graph)
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_ipld_graph,
    bench_context_graph,
    bench_workflow_graph,
    bench_concept_graph,
    bench_composed_graph
);
criterion_main!(benches);