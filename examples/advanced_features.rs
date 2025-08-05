//! Advanced features example for CIM Graph
//!
//! This example demonstrates:
//! - Graph composition
//! - Event handling
//! - Custom algorithms
//! - Performance optimization
//! - Error handling patterns

use cim_graph::{
    algorithms::{shortest_path, centrality},
    core::{EventGraph, GraphEvent, GenericNode, GenericEdge},
    graphs::{ComposedGraph, ContextGraph, IpldGraph, WorkflowGraph},
    GraphBuilder, Node, Edge, Result, GraphError,
};
use std::collections::HashMap;
use std::time::Instant;
use uuid::Uuid;

fn main() -> Result<()> {
    println!("CIM Graph Advanced Features Example\n");

    // 1. Event-driven graph
    event_driven_example()?;
    
    // 2. Graph composition
    graph_composition_example()?;
    
    // 3. Performance patterns
    performance_example()?;
    
    // 4. Error handling patterns
    error_handling_example()?;

    Ok(())
}

fn event_driven_example() -> Result<()> {
    println!("=== Event-Driven Graph Example ===");
    
    // Create an event-aware graph
    let mut graph = EventGraph::new();
    
    // Subscribe to events
    let mut event_count = 0;
    graph.subscribe(move |event: &GraphEvent| {
        match event {
            GraphEvent::NodeAdded { id, node_type, timestamp } => {
                println!("  [{}] Node added: {} (type: {})", 
                    timestamp.format("%H:%M:%S"), id, node_type);
            }
            GraphEvent::EdgeAdded { from, to, edge_type, timestamp } => {
                println!("  [{}] Edge added: {} -> {} (type: {})", 
                    timestamp.format("%H:%M:%S"), from, to, edge_type);
            }
            _ => {}
        }
        event_count += 1;
    });
    
    // Perform operations that emit events
    let n1 = graph.add_node(GenericNode::new("EventNode1", "event"))?;
    let n2 = graph.add_node(GenericNode::new("EventNode2", "event"))?;
    graph.add_edge(GenericEdge::new("triggers", &n1, &n2))?;
    
    // Access event history
    println!("\nTotal events recorded: {}", graph.events().len());
    
    // Create snapshot for replay
    let snapshot = graph.snapshot();
    println!("Snapshot created with {} events", snapshot.event_count());
    
    println!();
    Ok(())
}

fn graph_composition_example() -> Result<()> {
    println!("=== Graph Composition Example ===");
    
    // Create different domain graphs
    let mut ipld = IpldGraph::new();
    let data_cid = ipld.add_content(serde_json::json!({
        "cid": "QmData123",
        "format": "dag-json",
        "size": 2048
    }))?;
    
    let mut context = ContextGraph::new();
    let bounded_context = context.add_bounded_context("business", "Business Context")?;
    let entity = context.add_aggregate(
        Uuid::new_v4().to_string(),
        "Document",
        bounded_context
    )?;
            "ipld_cid": "QmData123"
        })
    )?;
    
    let mut workflow = WorkflowGraph::new("document_workflow");
    let draft = workflow.add_state("Draft", 
        cim_graph::graphs::workflow::StateType::Initial)?;
    let published = workflow.add_state("Published",
        cim_graph::graphs::workflow::StateType::Final)?;
    workflow.add_transition(draft, published, "publish", None)?;
    
    // Compose graphs with mappings
    let composed = ComposedGraph::new()
        .add_graph("storage", ipld)
        .add_graph("domain", context)
        .add_graph("process", workflow)
        .with_mapping("storage", "domain", |storage_node, domain_node| {
            // Map IPLD CIDs to domain entities
            if let Some(cid) = storage_node.data.get("cid") {
                if let Some(stored_cid) = domain_node.data.get("ipld_cid") {
                    return cid == stored_cid;
                }
            }
            false
        })
        .build()?;
    
    // Query across composed graphs
    println!("Composed graph contains:");
    println!("  {} storage nodes", composed.nodes_in_graph("storage").len());
    println!("  {} domain entities", composed.nodes_in_graph("domain").len());
    println!("  {} workflow states", composed.nodes_in_graph("process").len());
    
    // Cross-graph query example
    let cross_refs = composed.find_cross_references()?;
    println!("  {} cross-graph references", cross_refs.len());
    
    println!();
    Ok(())
}

fn performance_example() -> Result<()> {
    println!("=== Performance Optimization Example ===");
    
    // Pre-allocate for known size
    let node_count = 1000;
    let edge_count = 5000;
    
    let start = Instant::now();
    
    let mut graph = GraphBuilder::new()
        .with_capacity(node_count, edge_count)
        .build();
    
    // Batch node creation
    let nodes: Vec<_> = (0..node_count)
        .map(|i| GenericNode::new(format!("Node{}", i), "benchmark"))
        .collect();
    
    let node_ids = graph.add_nodes_batch(&nodes)?;
    
    // Batch edge creation
    let mut edges = Vec::with_capacity(edge_count);
    for i in 0..edge_count {
        let from = node_ids[i % node_count];
        let to = node_ids[(i + 1) % node_count];
        edges.push((from, to, Edge::with_weight((i % 10) as f64)));
    }
    
    graph.add_edges_batch(edges)?;
    
    let build_time = start.elapsed();
    println!("Built graph with {} nodes and {} edges in {:?}", 
        node_count, edge_count, build_time);
    
    // Benchmark algorithms
    let algo_start = Instant::now();
    let centralities = centrality(&graph)?;
    let centrality_time = algo_start.elapsed();
    
    println!("Centrality computed in {:?}", centrality_time);
    
    // Find top nodes
    let mut ranked: Vec<_> = centralities.into_iter().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    println!("Top 3 nodes by Centrality:");
    for (node_id, score) in ranked.iter().take(3) {
        let node = graph.get_node(*node_id).unwrap();
        println!("  {}: {:.4}", node.data(), score);
    }
    
    println!();
    Ok(())
}

fn error_handling_example() -> Result<()> {
    println!("=== Error Handling Patterns ===");
    
    let mut graph = GraphBuilder::new().build();
    
    // Pattern 1: Graceful fallback
    let result = safe_add_edge(&mut graph, "NonExistent1", "NonExistent2");
    match result {
        Ok(_) => println!("Edge added successfully"),
        Err(e) => println!("Gracefully handled: {}", e),
    }
    
    // Pattern 2: Error context
    let node = graph.add_node(GenericNode::new("Test", "error_demo"))?;
    let result = graph.get_node(node)
        .ok_or_else(|| GraphError::NodeNotFound(node.to_string()))
        .map(|n| n.data().to_string());
    
    println!("Node data with context: {:?}", result);
    
    // Pattern 3: Transaction-like operations
    let transaction_result = transactional_update(&mut graph);
    match transaction_result {
        Ok(count) => println!("Transaction succeeded: {} operations", count),
        Err(e) => println!("Transaction rolled back: {}", e),
    }
    
    // Pattern 4: Custom error types
    match validate_graph_constraints(&graph) {
        Ok(()) => println!("Graph validation passed"),
        Err(ConstraintError::TooManyNodes(n)) => {
            println!("Constraint violation: too many nodes ({})", n)
        }
        Err(ConstraintError::InvalidTopology(msg)) => {
            println!("Constraint violation: {}", msg)
        }
    }
    
    println!();
    Ok(())
}

// Helper functions demonstrating error handling patterns

fn safe_add_edge(graph: &mut impl Graph, from_name: &str, to_name: &str) -> Result<()> {
    // Find or create nodes
    let from = graph.nodes()
        .find(|n| n.data() == from_name)
        .map(|n| n.id())
        .ok_or_else(|| GraphError::NodeNotFound(from_name.to_string()))?;
    
    let to = graph.nodes()
        .find(|n| n.data() == to_name)
        .map(|n| n.id())
        .ok_or_else(|| GraphError::NodeNotFound(to_name.to_string()))?;
    
    graph.add_edge(GenericEdge::new("safe", &from, &to))
}

fn transactional_update(graph: &mut impl Graph) -> Result<usize> {
    let checkpoint = graph.node_count();
    let mut operations = 0;
    
    // Try to perform multiple operations
    let result: Result<()> = (|| {
        for i in 0..5 {
            graph.add_node(GenericNode::new(format!("Transaction{}", i), "tx"))?;
            operations += 1;
            
            // Simulate potential failure
            if i == 3 {
                return Err(GraphError::InvalidOperation("Simulated failure".to_string()));
            }
        }
        Ok(())
    })();
    
    match result {
        Ok(()) => Ok(operations),
        Err(e) => {
            // Rollback by removing added nodes
            while graph.node_count() > checkpoint {
                // In real implementation, would track and remove specific nodes
            }
            Err(e)
        }
    }
}

#[derive(Debug)]
enum ConstraintError {
    TooManyNodes(usize),
    InvalidTopology(String),
}

fn validate_graph_constraints(graph: &impl Graph) -> std::result::Result<(), ConstraintError> {
    const MAX_NODES: usize = 10000;
    
    if graph.node_count() > MAX_NODES {
        return Err(ConstraintError::TooManyNodes(graph.node_count()));
    }
    
    // Check for other constraints...
    
    Ok(())
}

// Trait to demonstrate working with generic graphs
trait Graph {
    fn add_node(&mut self, node: Node<String>) -> Result<cim_graph::core::NodeId>;
    fn add_edge(&mut self, from: cim_graph::core::NodeId, to: cim_graph::core::NodeId, edge: Edge<String>) -> Result<cim_graph::core::EdgeId>;
    fn get_node(&self, id: cim_graph::core::NodeId) -> Option<&Node<String>>;
    fn nodes(&self) -> Box<dyn Iterator<Item = &Node<String>> + '_>;
    fn node_count(&self) -> usize;
}