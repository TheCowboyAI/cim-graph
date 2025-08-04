use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use cim_graph::core::{EventGraph, GenericEdge, GenericNode, GraphType};
use cim_graph::graphs::{
    ConceptGraph, ContextGraph, IpldGraph, WorkflowGraph,
};
use std::collections::HashMap;
use uuid::Uuid;

/// Create a populated graph for serialization benchmarks
fn create_populated_graph(nodes: usize, edges: usize) -> EventGraph<GenericNode<String>, GenericEdge<String>> {
    let mut graph = EventGraph::<GenericNode<String>, GenericEdge<String>>::new(GraphType::Generic);
    let mut node_ids = Vec::new();
    
    // Add nodes with various properties
    for i in 0..nodes {
        let mut node = GenericNode::new(format!("node_{}", i), format!("Node {}", i));
        
        // Add various properties to make serialization more realistic
        node.set_property("index", i.to_string());
        node.set_property("timestamp", chrono::Utc::now().to_rfc3339());
        node.set_property("category", format!("category_{}", i % 5));
        node.set_property("data", format!("{{\"value\": {}, \"active\": {}}}", i * 10, i % 2 == 0));
        
        let node_id = graph.add_node(node).unwrap();
        node_ids.push(node_id);
    }
    
    // Add edges with properties
    for i in 0..edges {
        let from_idx = i % nodes;
        let to_idx = (i * 3 + 1) % nodes;
        
        if from_idx != to_idx {
            let mut edge = GenericEdge::new(
                node_ids[from_idx].clone(),
                node_ids[to_idx].clone(),
                format!("edge_type_{}", i % 3),
            );
            
            // Add edge properties
            edge.set_property("weight", (i as f64 / 10.0).to_string());
            edge.set_property("created", chrono::Utc::now().to_rfc3339());
            
            graph.add_edge(edge).ok();
        }
    }
    
    graph
}

/// Benchmark core graph serialization
fn bench_graph_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_serialization");
    
    // Serialize to JSON
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("to_json", size),
            size,
            |b, &size| {
                let graph = create_populated_graph(size, size * 2);
                
                b.iter(|| {
                    let json = serde_json::to_string(&graph).unwrap();
                    black_box(json.len())
                });
            },
        );
    }
    
    // Serialize to pretty JSON
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("to_json_pretty", size),
            size,
            |b, &size| {
                let graph = create_populated_graph(size, size * 2);
                
                b.iter(|| {
                    let json = serde_json::to_string_pretty(&graph).unwrap();
                    black_box(json.len())
                });
            },
        );
    }
    
    // Deserialize from JSON
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("from_json", size),
            size,
            |b, &size| {
                let graph = create_populated_graph(size, size * 2);
                let json = serde_json::to_string(&graph).unwrap();
                
                b.iter(|| {
                    let deserialized: EventGraph<GenericNode<String>, GenericEdge<String>> = serde_json::from_str(&json).unwrap();
                    black_box(deserialized.node_count())
                });
            },
        );
    }
    
    // Serialize to binary (using JSON as bytes for now)
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("to_bytes", size),
            size,
            |b, &size| {
                let graph = create_populated_graph(size, size * 2);
                
                b.iter(|| {
                    let json = serde_json::to_vec(&graph).unwrap();
                    black_box(json.len())
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark IPLD graph serialization
fn bench_ipld_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("ipld_serialization");
    
    for size in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("serialize", size),
            size,
            |b, &size| {
                let mut graph = IpldGraph::new();
                
                // Add blocks
                for i in 0..size {
                    let cid = format!("Qm{}", Uuid::new_v4().to_string().replace("-", ""));
                    let mut data = HashMap::new();
                    data.insert("index".to_string(), i.to_string());
                    data.insert("content".to_string(), format!("Block content for {}", i));
                    data.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());
                    graph.add_block(cid, data).unwrap();
                }
                
                // Add links
                let blocks: Vec<_> = graph.blocks().map(|b| b.0.clone()).collect();
                for i in 0..blocks.len().saturating_sub(1) {
                    graph.link_blocks(
                        blocks[i].clone(),
                        blocks[i + 1].clone(),
                        "next".to_string(),
                    ).ok();
                }
                
                b.iter(|| {
                    let json = serde_json::to_string(&graph).unwrap();
                    black_box(json.len())
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark Context graph serialization
fn bench_context_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("context_serialization");
    
    for size in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("serialize", size),
            size,
            |b, &size| {
                let mut graph = ContextGraph::new();
                
                // Add contexts
                for i in 0..size {
                    let id = format!("context_{}", i);
                    let mut properties = HashMap::new();
                    properties.insert("domain".to_string(), format!("domain_{}", i % 5));
                    properties.insert("version".to_string(), "1.0".to_string());
                    properties.insert("metadata".to_string(), serde_json::json!({
                        "created": chrono::Utc::now().to_rfc3339(),
                        "tags": vec![format!("tag_{}", i % 3), format!("tag_{}", i % 7)],
                        "priority": i % 10
                    }).to_string());
                    
                    graph.add_context(id, format!("Context_{}", i), properties).unwrap();
                }
                
                // Add relationships
                let contexts: Vec<_> = graph.contexts().map(|c| c.0.clone()).collect();
                for i in 0..contexts.len() / 2 {
                    let j = (i * 2 + 1) % contexts.len();
                    graph.add_relationship(
                        contexts[i].clone(),
                        contexts[j].clone(),
                        "relates_to".to_string(),
                        HashMap::new(),
                    ).ok();
                }
                
                b.iter(|| {
                    let json = serde_json::to_string(&graph).unwrap();
                    black_box(json.len())
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark Workflow graph serialization
fn bench_workflow_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("workflow_serialization");
    
    for size in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("serialize", size),
            size,
            |b, &size| {
                let mut graph = WorkflowGraph::new();
                
                // Create workflow tasks
                let mut task_ids = Vec::new();
                for i in 0..size {
                    let task_id = format!("task_{}", i);
                    let mut config = HashMap::new();
                    config.insert("type".to_string(), format!("task_type_{}", i % 4));
                    config.insert("priority".to_string(), (i % 3).to_string());
                    config.insert("timeout".to_string(), "3600".to_string());
                    config.insert("retry_count".to_string(), "3".to_string());
                    config.insert("config".to_string(), serde_json::json!({
                        "input": format!("input_{}", i),
                        "output": format!("output_{}", i),
                        "parameters": {
                            "threads": 4,
                            "memory": "2GB"
                        }
                    }).to_string());
                    
                    graph.add_task(task_id.clone(), format!("Task_{}", i), config).unwrap();
                    task_ids.push(task_id);
                }
                
                // Create dependencies
                for i in 0..task_ids.len() - 1 {
                    graph.add_dependency(
                        task_ids[i].clone(),
                        task_ids[i + 1].clone(),
                        "depends_on".to_string(),
                    ).unwrap();
                    
                    // Add some parallel branches
                    if i % 3 == 0 && i + 3 < task_ids.len() {
                        graph.add_dependency(
                            task_ids[i].clone(),
                            task_ids[i + 3].clone(),
                            "parallel".to_string(),
                        ).ok();
                    }
                }
                
                b.iter(|| {
                    let json = serde_json::to_string(&graph).unwrap();
                    black_box(json.len())
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark Concept graph serialization
fn bench_concept_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("concept_serialization");
    
    for size in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("serialize", size),
            size,
            |b, &size| {
                let mut graph = ConceptGraph::new();
                
                // Create concepts
                let mut concept_ids = Vec::new();
                for i in 0..size {
                    let concept_id = format!("concept_{}", i);
                    let attributes = serde_json::json!({
                        "category": format!("category_{}", i % 7),
                        "domain": format!("domain_{}", i % 3),
                        "definition": format!("This is the definition for concept {}", i),
                        "metadata": {
                            "created": chrono::Utc::now().to_rfc3339(),
                            "version": "1.0",
                            "tags": vec![format!("tag_{}", i % 5), format!("tag_{}", i % 11)],
                            "references": (0..3).map(|j| format!("ref_{}_{}", i, j)).collect::<Vec<_>>()
                        }
                    });
                    
                    graph.add_concept(
                        &concept_id,
                        &format!("Concept_{}", i),
                        attributes,
                    ).unwrap();
                    concept_ids.push(concept_id);
                }
                
                // Create relations
                for i in 0..concept_ids.len() {
                    // Create hierarchy
                    if i > 0 {
                        let parent_idx = (i - 1) / 2;
                        use cim_graph::graphs::concept::SemanticRelation;
                        
                        graph.add_relation(
                            &concept_ids[parent_idx],
                            &concept_ids[i],
                            SemanticRelation::SubClassOf,
                        ).ok();
                    }
                    
                    // Create cross-references
                    if i % 5 == 0 && i + 7 < concept_ids.len() {
                        use cim_graph::graphs::concept::SemanticRelation;
                        
                        graph.add_relation(
                            &concept_ids[i],
                            &concept_ids[i + 7],
                            SemanticRelation::SimilarTo,
                        ).ok();
                    }
                }
                
                b.iter(|| {
                    let json = serde_json::to_string(&graph).unwrap();
                    black_box(json.len())
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark round-trip serialization
fn bench_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip");
    
    // Core graph round-trip
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("core_graph", size),
            size,
            |b, &size| {
                let graph = create_populated_graph(size, size * 2);
                
                b.iter(|| {
                    let json = serde_json::to_string(&graph).unwrap();
                    let deserialized: EventGraph<GenericNode<String>, GenericEdge<String>> = serde_json::from_str(&json).unwrap();
                    black_box((json.len(), deserialized.node_count()))
                });
            },
        );
    }
    
    // Complex graph with nested data
    group.bench_function("complex_nested", |b| {
        b.iter_batched(
            || {
                let mut graph = EventGraph::<GenericNode<String>, GenericEdge<String>>::new(GraphType::Generic);
                
                // Add nodes with deeply nested data
                for i in 0..50 {
                    let mut node = GenericNode::new(format!("node_{}", i), format!("Node {}", i));
                    
                    let nested_data = serde_json::json!({
                        "level1": {
                            "level2": {
                                "level3": {
                                    "values": (0..10).collect::<Vec<_>>(),
                                    "metadata": {
                                        "created": chrono::Utc::now().to_rfc3339(),
                                        "tags": vec!["tag1", "tag2", "tag3"]
                                    }
                                }
                            }
                        }
                    });
                    
                    node.set_property("nested", nested_data.to_string());
                    graph.add_node(node).unwrap();
                }
                
                graph
            },
            |graph| {
                let json = serde_json::to_string(&graph).unwrap();
                let deserialized: EventGraph<GenericNode<String>, GenericEdge<String>> = serde_json::from_str(&json).unwrap();
                black_box((json.len(), deserialized.node_count()))
            },
            BatchSize::SmallInput,
        );
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_graph_serialization,
    bench_ipld_serialization,
    bench_context_serialization,
    bench_workflow_serialization,
    bench_concept_serialization,
    bench_roundtrip
);
criterion_main!(benches);