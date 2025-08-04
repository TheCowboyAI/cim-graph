//! Example: Graph serialization and deserialization
//! 
//! This example demonstrates how to save and load graphs to/from JSON.

use cim_graph::graphs::IpldGraph;
use cim_graph::serde_support::GraphSerialize;
use serde_json::json;
use std::fs;

fn main() {
    println!("=== Graph Serialization Example ===\n");
    
    // Create a sample IPLD graph
    println!("Creating an IPLD graph...");
    let mut graph = IpldGraph::new();
    
    // Add some content
    let project_data = json!({
        "type": "project",
        "name": "My Project",
        "version": "1.0.0",
        "created": "2024-01-01"
    });
    
    let readme_data = json!({
        "type": "file",
        "name": "README.md",
        "content": "# My Project\n\nA sample project for demonstrating serialization."
    });
    
    let src_data = json!({
        "type": "directory",
        "name": "src",
        "files": []
    });
    
    let main_data = json!({
        "type": "file",
        "name": "main.rs",
        "content": "fn main() {\n    println!(\"Hello, world!\");\n}"
    });
    
    // Build the graph structure
    let project_cid = graph.add_content(project_data).expect("Failed to add project");
    let readme_cid = graph.add_content(readme_data).expect("Failed to add readme");
    let src_cid = graph.add_content(src_data).expect("Failed to add src");
    let main_cid = graph.add_content(main_data).expect("Failed to add main");
    
    // Create links
    graph.add_link(&project_cid, &readme_cid, "contains").unwrap();
    graph.add_link(&project_cid, &src_cid, "contains").unwrap();
    graph.add_link(&src_cid, &main_cid, "contains").unwrap();
    
    println!("Graph created with {} nodes and {} edges", 
        graph.graph().node_count(), 
        graph.graph().edge_count()
    );
    
    // Serialize to JSON
    println!("\n=== Serializing to JSON ===");
    let json = graph.to_json().expect("Failed to serialize");
    
    // Save to file
    let filename = "example_graph.json";
    fs::write(filename, &json).expect("Failed to write file");
    println!("Graph saved to {}", filename);
    println!("File size: {} bytes", json.len());
    
    // Display a sample of the JSON
    let preview = if json.len() > 200 {
        format!("{}...", &json[..200])
    } else {
        json.clone()
    };
    println!("\nJSON preview:\n{}", preview);
    
    // Load from file
    println!("\n=== Loading from JSON ===");
    let loaded_graph = IpldGraph::load_from_file(filename)
        .expect("Failed to load graph");
    
    println!("Graph loaded successfully!");
    println!("Loaded graph has {} nodes and {} edges", 
        loaded_graph.graph().node_count(), 
        loaded_graph.graph().edge_count()
    );
    
    // Verify the content
    println!("\n=== Verifying Content ===");
    
    // Check project node
    if let Some(project_content) = loaded_graph.get_content(&project_cid) {
        println!("✓ Project: {}", project_content["name"]);
    }
    
    // Check file nodes
    if let Some(readme_content) = loaded_graph.get_content(&readme_cid) {
        println!("✓ README: {}", readme_content["name"]);
    }
    
    if let Some(main_content) = loaded_graph.get_content(&main_cid) {
        println!("✓ Main file: {}", main_content["name"]);
    }
    
    // Traverse the loaded graph
    println!("\n=== Traversing Loaded Graph ===");
    let visited = loaded_graph.traverse(&project_cid, 10);
    println!("Traversal found {} nodes", visited.len());
    
    // Clean up
    fs::remove_file(filename).ok();
    println!("\nExample file cleaned up.");
    
    // Demonstrate error handling
    println!("\n=== Error Handling ===");
    match IpldGraph::load_from_file("non_existent_file.json") {
        Ok(_) => println!("Unexpected success!"),
        Err(e) => println!("Expected error when loading non-existent file: {}", e),
    }
    
    println!("\n✅ Serialization example complete!");
}