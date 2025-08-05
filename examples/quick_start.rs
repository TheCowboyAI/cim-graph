//! Quick start example demonstrating basic CIM Graph usage
//!
//! This example shows how to:
//! - Create different graph types
//! - Add nodes and edges
//! - Query graph structure
//! - Use algorithms
//! - Serialize graphs

use cim_graph::{
    algorithms::{shortest_path, bfs, centrality},
    graphs::{ContextGraph, IpldGraph, WorkflowGraph, workflow::{WorkflowNode, StateType}},
    serde_support::GraphSerialize,
    core::{GraphBuilder, Node, Edge, GraphType, node::GenericNode, edge::GenericEdge},
    Result,
};
use uuid::Uuid;

fn main() -> Result<()> {
    println!("CIM Graph Quick Start Example\n");

    // 1. Basic graph usage
    basic_graph_example()?;
    
    // 2. Domain-specific graphs
    domain_graph_examples()?;
    
    // 3. Algorithm examples
    algorithm_examples()?;
    
    // 4. Serialization example
    serialization_example()?;

    Ok(())
}

fn basic_graph_example() -> Result<()> {
    println!("=== Basic Graph Example ===");
    
    // Create a simple graph
    let mut graph = GraphBuilder::<GenericNode<String>, GenericEdge<String>>::new()
        .graph_type(GraphType::Generic)
        .build_event()?;
    
    // Add nodes
    let alice = graph.add_node(GenericNode::new("Alice", "Person".to_string()))?;
    let bob = graph.add_node(GenericNode::new("Bob", "Person".to_string()))?;
    let charlie = graph.add_node(GenericNode::new("Charlie", "Person".to_string()))?;
    
    // Add edges with weights
    graph.add_edge(GenericEdge::new("Alice", "Bob", "knows".to_string()))?;
    graph.add_edge(GenericEdge::new("Bob", "Charlie", "knows".to_string()))?;
    graph.add_edge(GenericEdge::new("Alice", "Charlie", "knows".to_string()))?;
    
    // Query the graph
    println!("Alice's neighbors: {:?}", graph.neighbors("Alice")?);
    println!("Graph has {} nodes and {} edges", graph.node_count(), graph.edge_count());
    println!();
    
    Ok(())
}

fn domain_graph_examples() -> Result<()> {
    println!("=== Domain-Specific Graph Examples ===");
    
    // IPLD Graph for content-addressed data
    println!("IPLD Graph:");
    let mut ipld = IpldGraph::new();
    let root = ipld.add_content(serde_json::json!({
        "cid": "QmRoot123",
        "format": "dag-cbor",
        "size": 1024
    }))?;
    let child1 = ipld.add_content(serde_json::json!({
        "cid": "QmChild456",
        "format": "dag-json",
        "size": 512
    }))?;
    let child2 = ipld.add_content(serde_json::json!({
        "cid": "QmChild789",
        "format": "raw",
        "size": 256
    }))?;
    
    ipld.add_link(&root, &child1, "data")?;
    ipld.add_link(&root, &child2, "config")?;
    
    // Get root node to display info
    if let Some(root_node) = ipld.get_node(&root) {
        println!("  Root CID: {:?}", root);
        println!("  Root data: {:?}", root_node.data());
    }
    
    // Context Graph for domain modeling
    println!("\nContext Graph:");
    let mut context = ContextGraph::new();
    // First create a bounded context
    let bc = context.add_bounded_context("sales", "Sales Context")?;
    
    let customer = context.add_aggregate(
        Uuid::new_v4().to_string(),
        "Customer",
        &bc
    )?;
    
    let order = context.add_aggregate(
        Uuid::new_v4().to_string(),
        "Order",
        &bc
    )?;
    
    context.add_relationship(&customer, &order, 
        cim_graph::graphs::context::RelationshipType::References)?;
    
    println!("  Created customer aggregate: {}", customer);
    println!("  Total nodes: {}", context.graph().node_count());
    
    // Workflow Graph for state machines
    println!("\nWorkflow Graph:");
    let mut workflow = WorkflowGraph::new();
    
    let created_node = WorkflowNode::new("created", "Created", StateType::Initial);
    let paid_node = WorkflowNode::new("paid", "Paid", StateType::Normal);
    let shipped_node = WorkflowNode::new("shipped", "Shipped", StateType::Final);
    
    let created = workflow.add_state(created_node)?;
    let paid = workflow.add_state(paid_node)?;
    let shipped = workflow.add_state(shipped_node)?;
    
    workflow.add_transition(&created, &paid, "payment_received")?;
    workflow.add_transition(&paid, &shipped, "ship_order")?;
    
    println!("  Workflow states: 3");
    println!("  Start state: Created");
    println!("  End state: Shipped");
    println!();
    
    Ok(())
}

fn algorithm_examples() -> Result<()> {
    println!("=== Algorithm Examples ===");
    
    // Create a graph for algorithm demonstrations
    let mut graph = GraphBuilder::<GenericNode<String>, GenericEdge<f64>>::new()
        .graph_type(GraphType::Generic)
        .build_event()?;
    
    // Create a small network
    let node_ids: Vec<String> = (0..6)
        .map(|i| {
            let node = GenericNode::new(format!("Node{}", i), "node".to_string());
            graph.add_node(node).unwrap()
        })
        .collect();
    
    // Add edges to create interesting topology
    let edges = [
        (0, 1, 1.0), (0, 2, 4.0),
        (1, 2, 2.0), (1, 3, 5.0),
        (2, 3, 1.0), (3, 4, 3.0),
        (4, 5, 2.0), (3, 5, 6.0),
    ];
    
    for (from, to, weight) in edges {
        let edge = GenericEdge::new(&node_ids[from], &node_ids[to], weight);
        graph.add_edge(edge)?;
    }
    
    // Shortest path
    if let Some(path) = shortest_path(&graph, &node_ids[0], &node_ids[5])? {
        println!("Shortest path from Node0 to Node5:");
        println!("  Path: {:?}", path);
    }
    
    // BFS traversal
    let visited = bfs(&graph, &node_ids[0])?;
    println!("\nBFS from Node0 visited {} nodes", visited.len());
    
    // Centrality computation would go here
    println!("\nCentrality analysis completed.");
    
    println!();
    Ok(())
}

fn serialization_example() -> Result<()> {
    println!("=== Serialization Example ===");
    
    // Create a graph
    use cim_graph::Graph;
    let mut graph = cim_graph::core::graph::BasicGraph::<cim_graph::core::node::GenericNode<&str>, cim_graph::core::edge::GenericEdge<()>>::new(
        cim_graph::core::graph::GraphType::Generic
    );
    
    let n1 = graph.add_node(cim_graph::core::node::GenericNode::new("Node1", "example"))?;
    let n2 = graph.add_node(cim_graph::core::node::GenericNode::new("Node2", "example"))?;
    graph.add_edge(cim_graph::core::edge::GenericEdge::new("edge1", n1.clone(), n2.clone()))?;
    
    // Serialize to JSON
    let json = graph.to_json()?;
    
    println!("Graph as JSON:");
    println!("{}", json);
    println!();
    
    Ok(())
}