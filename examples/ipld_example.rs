//! Example: Using IpldGraph for content-addressed data
//! 
//! This example demonstrates how to use IpldGraph to create a content-addressed
//! file system structure similar to IPFS.

use cim_graph::graphs::ipld::IpldGraph;
use serde_json::json;

fn main() {
    println!("=== IPLD Graph Example ===\n");
    
    // Create a new IPLD graph
    let mut graph = IpldGraph::new();
    
    // Add content nodes - imagine these are files and directories
    println!("Adding content to the graph...");
    
    // Root directory
    let root_data = json!({
        "type": "directory",
        "name": "project",
        "created": "2024-01-01T00:00:00Z",
        "permissions": "rwxr-xr-x"
    });
    let root_cid = graph.add_content(root_data).expect("Failed to add root");
    println!("Root CID: {}", root_cid.as_str());
    
    // Source directory
    let src_data = json!({
        "type": "directory", 
        "name": "src",
        "created": "2024-01-01T00:00:00Z"
    });
    let src_cid = graph.add_content(src_data).expect("Failed to add src");
    
    // Main.rs file
    let main_data = json!({
        "type": "file",
        "name": "main.rs",
        "size": 1024,
        "hash": "sha256:abcdef123456",
        "content": "fn main() { println!(\"Hello, IPLD!\"); }"
    });
    let main_cid = graph.add_content(main_data).expect("Failed to add main.rs");
    
    // Lib.rs file
    let lib_data = json!({
        "type": "file",
        "name": "lib.rs", 
        "size": 2048,
        "hash": "sha256:fedcba654321"
    });
    let lib_cid = graph.add_content(lib_data).expect("Failed to add lib.rs");
    
    // README file
    let readme_data = json!({
        "type": "file",
        "name": "README.md",
        "size": 512,
        "content": "# My Project\n\nA demonstration of IPLD graphs."
    });
    let readme_cid = graph.add_content(readme_data).expect("Failed to add README");
    
    // Create links between content (directory structure)
    println!("\nCreating directory structure...");
    graph.add_link(&root_cid, &src_cid, "src").expect("Failed to link src");
    graph.add_link(&root_cid, &readme_cid, "README.md").expect("Failed to link README");
    graph.add_link(&src_cid, &main_cid, "main.rs").expect("Failed to link main.rs");
    graph.add_link(&src_cid, &lib_cid, "lib.rs").expect("Failed to link lib.rs");
    
    // Traverse the graph from root
    println!("\nTraversing from root:");
    let visited = graph.traverse(&root_cid, 10);
    println!("Found {} nodes in the graph", visited.len());
    
    // Display the directory structure
    println!("\nDirectory structure:");
    display_tree(&graph, &root_cid, 0);
    
    // Demonstrate content retrieval
    println!("\n\nRetrieving content by CID:");
    if let Some(content) = graph.get_content(&main_cid) {
        println!("main.rs content: {}", serde_json::to_string_pretty(content).unwrap());
    }
    
    // Show how content-addressing enables deduplication
    println!("\n\nDemonstrating content addressing:");
    let duplicate_readme = json!({
        "type": "file",
        "name": "README.md",
        "size": 512,
        "content": "# My Project\n\nA demonstration of IPLD graphs."
    });
    let dup_cid = graph.add_content(duplicate_readme).expect("Failed to add duplicate");
    println!("Original README CID: {}", readme_cid.as_str());
    println!("Duplicate README CID: {}", dup_cid.as_str());
    println!("(In a real content-addressed system, these would be identical)");
    
    // Statistics
    println!("\n\nGraph statistics:");
    println!("Total nodes: {}", graph.graph().node_count());
    println!("Total edges: {}", graph.graph().edge_count());
}

fn display_tree(graph: &IpldGraph, cid: &cim_graph::graphs::ipld::Cid, depth: usize) {
    let indent = "  ".repeat(depth);
    
    if let Some(node) = graph.get_node(cid) {
        if let Some(content) = graph.get_content(cid) {
            let name = content.get("name").and_then(|v| v.as_str()).unwrap_or("?");
            let typ = content.get("type").and_then(|v| v.as_str()).unwrap_or("?");
            
            match typ {
                "directory" => println!("{}📁 {}/", indent, name),
                "file" => {
                    let size = content.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
                    println!("{}📄 {} ({}B)", indent, name, size);
                }
                _ => println!("{}{}", indent, name),
            }
        }
        
        // Display children
        for (link_name, child_cid) in node.links() {
            if depth == 0 {
                // For root level, use link name to show the connection
                print!("{}", indent);
            }
            display_tree(graph, child_cid, depth + 1);
        }
    }
}