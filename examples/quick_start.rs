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
    graphs::{ContextGraph, IpldGraph, WorkflowGraph},
    serde_support::{to_json_pretty, JsonConfig},
    GraphBuilder, Node, Edge, Result,
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
    let mut graph = GraphBuilder::new()
        .with_capacity(10, 20)
        .directed(true)
        .build();
    
    // Add nodes
    let alice = graph.add_node(Node::new("Alice", "Person"))?;
    let bob = graph.add_node(Node::new("Bob", "Person"))?;
    let charlie = graph.add_node(Node::new("Charlie", "Person"))?;
    
    // Add edges with weights
    graph.add_edge(alice, bob, Edge::with_weight(1.0))?;
    graph.add_edge(bob, charlie, Edge::with_weight(2.0))?;
    graph.add_edge(alice, charlie, Edge::with_weight(5.0))?;
    
    // Query the graph
    println!("Alice's neighbors: {:?}", graph.neighbors(alice)?);
    println!("Graph has {} nodes and {} edges", graph.node_count(), graph.edge_count());
    println!();
    
    Ok(())
}

fn domain_graph_examples() -> Result<()> {
    println!("=== Domain-Specific Graph Examples ===");
    
    // IPLD Graph for content-addressed data
    println!("IPLD Graph:");
    let mut ipld = IpldGraph::new();
    let root = ipld.add_cid("QmRoot123", "dag-cbor", 1024)?;
    let child1 = ipld.add_cid("QmChild456", "dag-json", 512)?;
    let child2 = ipld.add_cid("QmChild789", "raw", 256)?;
    
    ipld.add_link(root, child1, "data", Some("/users"))?;
    ipld.add_link(root, child2, "config", None)?;
    
    println!("  Root CID: {}", ipld.get_cid(root).unwrap());
    println!("  Children: {:?}", ipld.get_children(root)?);
    
    // Context Graph for domain modeling
    println!("\nContext Graph:");
    let mut context = ContextGraph::new("ecommerce");
    let customer = context.add_aggregate(
        "Customer",
        Uuid::new_v4(),
        serde_json::json!({
            "name": "Alice Smith",
            "email": "alice@example.com"
        })
    )?;
    
    let order = context.add_aggregate(
        "Order",
        Uuid::new_v4(),
        serde_json::json!({
            "total": 150.00,
            "items": ["laptop", "mouse"]
        })
    )?;
    
    context.add_relationship(customer, order, "placed", 
        cim_graph::graphs::context::Cardinality::OneToMany)?;
    
    println!("  Bounded context: {}", context.bounded_context());
    println!("  Aggregates: {} total", context.list_aggregates().len());
    
    // Workflow Graph for state machines
    println!("\nWorkflow Graph:");
    let mut workflow = WorkflowGraph::new("order_processing");
    
    let created = workflow.add_state("Created", 
        cim_graph::graphs::workflow::StateType::Start)?;
    let paid = workflow.add_state("Paid", 
        cim_graph::graphs::workflow::StateType::Regular)?;
    let shipped = workflow.add_state("Shipped", 
        cim_graph::graphs::workflow::StateType::End)?;
    
    workflow.add_transition(created, paid, "payment_received", None)?;
    workflow.add_transition(paid, shipped, "ship_order", None)?;
    
    println!("  Workflow states: 3");
    println!("  Start state: Created");
    println!("  End state: Shipped");
    println!();
    
    Ok(())
}

fn algorithm_examples() -> Result<()> {
    println!("=== Algorithm Examples ===");
    
    // Create a graph for algorithm demonstrations
    let mut graph = GraphBuilder::new().build();
    
    // Create a small network
    let nodes: Vec<_> = (0..6)
        .map(|i| graph.add_node(Node::new(format!("Node{}", i), "node")).unwrap())
        .collect();
    
    // Add edges to create interesting topology
    let edges = [
        (0, 1, 1.0), (0, 2, 4.0),
        (1, 2, 2.0), (1, 3, 5.0),
        (2, 3, 1.0), (3, 4, 3.0),
        (4, 5, 2.0), (3, 5, 6.0),
    ];
    
    for (from, to, weight) in edges {
        graph.add_edge(nodes[from], nodes[to], Edge::with_weight(weight))?;
    }
    
    // Shortest path
    if let Some((distance, path)) = shortest_path(&graph, nodes[0], nodes[5])? {
        println!("Shortest path from Node0 to Node5:");
        println!("  Distance: {}", distance);
        println!("  Path: {:?}", path.iter()
            .map(|&id| graph.get_node(id).unwrap().data())
            .collect::<Vec<_>>());
    }
    
    // BFS traversal
    let visited = bfs(&graph, nodes[0])?;
    println!("\nBFS from Node0 visited {} nodes", visited.len());
    
    // Centrality
    let centralities = centrality(&graph)?;
    println!("\nNode centralities:");
    for (node_id, score) in centralities.iter().take(3) {
        let node = graph.get_node(*node_id).unwrap();
        println!("  {}: {:.3}", node.data(), score);
    }
    
    println!();
    Ok(())
}

fn serialization_example() -> Result<()> {
    println!("=== Serialization Example ===");
    
    // Create a graph
    let mut graph = GraphBuilder::new()
        .with_metadata("name", serde_json::json!("Example Graph"))
        .with_metadata("version", serde_json::json!("1.0"))
        .build();
    
    let n1 = graph.add_node(Node::new("Node1", "example"))?;
    let n2 = graph.add_node(Node::new("Node2", "example"))?;
    graph.add_edge(n1, n2, Edge::new("connects"))?;
    
    // Serialize to JSON
    let config = JsonConfig::new()
        .with_indent(2)
        .with_sort_keys(true);
    
    let json = to_json_pretty(&graph, config)?;
    
    println!("Graph as JSON:");
    println!("{}", json);
    println!();
    
    Ok(())
}