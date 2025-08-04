//! Example: Using graph algorithms
//! 
//! This example demonstrates various graph algorithms provided by CIM Graph.

use cim_graph::graphs::workflow::{WorkflowGraph, WorkflowNode, StateType};
use cim_graph::algorithms;

fn main() {
    println!("=== Graph Algorithms Example ===\n");
    
    // Create a sample graph using WorkflowGraph (which wraps EventGraph)
    let mut graph = WorkflowGraph::new();
    
    // Add nodes for a small network
    let nodes = vec!["A", "B", "C", "D", "E", "F"];
    for node in &nodes {
        graph.add_state(WorkflowNode::new(*node, *node, StateType::Normal))
            .expect("Failed to add node");
    }
    
    // Create edges to form an interesting topology
    let edges = vec![
        ("A", "B"), ("A", "C"), ("B", "C"), ("B", "D"),
        ("C", "D"), ("C", "E"), ("D", "E"), ("D", "F"),
        ("E", "F"),
    ];
    
    for (from, to) in &edges {
        graph.add_transition(from, to, "next").expect("Failed to add edge");
    }
    
    println!("Created graph with {} nodes and {} edges", 
        graph.graph().node_count(), 
        graph.graph().edge_count()
    );
    
    // Demonstrate pathfinding
    println!("\n=== Pathfinding ===");
    
    // Find shortest path
    if let Ok(Some(path)) = algorithms::shortest_path(graph.graph(), "A", "F") {
        println!("Shortest path from A to F: {:?}", path);
        println!("Path length: {}", path.len());
    }
    
    // Find all paths
    if let Ok(paths) = algorithms::all_paths(graph.graph(), "A", "F", 5) {
        println!("\nAll paths from A to F (max length 5):");
        for (i, path) in paths.iter().enumerate() {
            println!("  Path {}: {:?}", i + 1, path);
        }
    }
    
    // Demonstrate traversal algorithms
    println!("\n=== Traversal ===");
    
    // Depth-first search
    if let Ok(dfs_order) = algorithms::dfs(graph.graph(), "A") {
        println!("DFS from A: {:?}", dfs_order);
    }
    
    // Breadth-first search
    if let Ok(bfs_order) = algorithms::bfs(graph.graph(), "A") {
        println!("BFS from A: {:?}", bfs_order);
    }
    
    // Topological sort (our graph has cycles, so this should fail)
    match algorithms::topological_sort(graph.graph()) {
        Ok(order) => println!("Topological sort: {:?}", order),
        Err(e) => println!("Topological sort failed (expected): {}", e),
    }
    
    // Create a DAG for topological sort
    println!("\n=== Creating DAG for Topological Sort ===");
    let mut dag = WorkflowGraph::new();
    
    // Add nodes
    for node in &["Task1", "Task2", "Task3", "Task4", "Task5"] {
        dag.add_state(WorkflowNode::new(*node, *node, StateType::Normal)).unwrap();
    }
    
    // Add edges to create dependencies
    dag.add_transition("Task1", "Task2", "complete").unwrap();
    dag.add_transition("Task1", "Task3", "complete").unwrap();
    dag.add_transition("Task2", "Task4", "complete").unwrap();
    dag.add_transition("Task3", "Task4", "complete").unwrap();
    dag.add_transition("Task4", "Task5", "complete").unwrap();
    
    if let Ok(order) = algorithms::topological_sort(dag.graph()) {
        println!("Topological sort of tasks: {:?}", order);
        println!("(This gives a valid execution order for the tasks)");
    }
    
    // Demonstrate metrics
    println!("\n=== Graph Metrics ===");
    
    // Degree centrality
    if let Ok(centrality) = algorithms::centrality(graph.graph()) {
        println!("\nDegree centrality:");
        let mut sorted: Vec<_> = centrality.into_iter().collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        for (node, score) in sorted {
            println!("  {}: {:.3}", node, score);
        }
    }
    
    // Clustering coefficient
    if let Ok(clustering) = algorithms::clustering_coefficient(graph.graph()) {
        println!("\nAverage clustering coefficient: {:.3}", clustering);
    }
    
    // Create a more connected graph to demonstrate clustering
    println!("\n=== Clustering Example ===");
    let mut triangle_graph = WorkflowGraph::new();
    
    // Create two triangles connected by a single edge
    for node in &["A", "B", "C", "D", "E", "F"] {
        triangle_graph.add_state(WorkflowNode::new(*node, *node, StateType::Normal)).unwrap();
    }
    
    // First triangle
    triangle_graph.add_transition("A", "B", "next").unwrap();
    triangle_graph.add_transition("B", "C", "next").unwrap();
    triangle_graph.add_transition("C", "A", "next").unwrap();
    
    // Second triangle
    triangle_graph.add_transition("D", "E", "next").unwrap();
    triangle_graph.add_transition("E", "F", "next").unwrap();
    triangle_graph.add_transition("F", "D", "next").unwrap();
    
    // Connect the triangles
    triangle_graph.add_transition("C", "D", "next").unwrap();
    
    if let Ok(clustering) = algorithms::clustering_coefficient(triangle_graph.graph()) {
        println!("Clustering coefficient for two connected triangles: {:.3}", clustering);
    }
    
    println!("\n✅ Algorithms example complete!");
}