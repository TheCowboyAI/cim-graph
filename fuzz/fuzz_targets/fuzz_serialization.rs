#![no_main]

use libfuzzer_sys::fuzz_target;
use cim_graph::graphs::ipld::IpldGraph;
use cim_graph::graphs::context::ContextGraph;
use cim_graph::graphs::workflow::WorkflowGraph;
use cim_graph::serde_support::GraphSerialize;

// Fuzz JSON serialization/deserialization
fuzz_target!(|data: &[u8]| {
    // Try to deserialize arbitrary data as different graph types
    if let Ok(json_str) = std::str::from_utf8(data) {
        // Fuzz IpldGraph deserialization
        if let Ok(graph) = IpldGraph::from_json(json_str) {
            // If deserialization succeeds, try to serialize it back
            if let Ok(serialized) = graph.to_json() {
                // Try to deserialize again
                let _ = IpldGraph::from_json(&serialized);
            }
        }
        
        // Fuzz ContextGraph deserialization
        if let Ok(graph) = ContextGraph::from_json(json_str) {
            if let Ok(serialized) = graph.to_json() {
                let _ = ContextGraph::from_json(&serialized);
            }
        }
        
        // Fuzz WorkflowGraph deserialization
        if let Ok(graph) = WorkflowGraph::from_json(json_str) {
            if let Ok(serialized) = graph.to_json() {
                let _ = WorkflowGraph::from_json(&serialized);
            }
        }
    }
});

// Fuzz round-trip serialization with graph construction
fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    
    // Build a graph based on fuzz input
    let mut graph = IpldGraph::new();
    
    // Add some nodes based on input
    for (i, &byte) in data.iter().enumerate().take(20) {
        let block_id = format!("block_{:02x}", byte);
        let content = vec![byte; (i % 10) + 1];
        let _ = graph.add_block(block_id, content);
    }
    
    // Add some edges
    for i in 0..data.len().saturating_sub(2) {
        if data[i] % 3 == 0 {
            let from = format!("block_{:02x}", data[i]);
            let to = format!("block_{:02x}", data[i + 1]);
            let _ = graph.add_link(&from, &to, "fuzz_link");
        }
    }
    
    // Test serialization round-trip
    if let Ok(serialized) = graph.to_serialized() {
        if let Ok(deserialized) = IpldGraph::from_serialized(serialized) {
            // Verify basic properties are preserved
            assert_eq!(graph.graph().node_count(), deserialized.graph().node_count());
            assert_eq!(graph.graph().edge_count(), deserialized.graph().edge_count());
        }
    }
    
    // Test JSON round-trip
    if let Ok(json) = graph.to_json() {
        if let Ok(deserialized) = IpldGraph::from_json(&json) {
            assert_eq!(graph.graph().node_count(), deserialized.graph().node_count());
        }
    }
});

// Fuzz malformed JSON handling
fuzz_target!(|data: &[u8]| {
    // Create potentially malformed JSON by mixing valid JSON structures with random data
    let mut json_attempt = String::new();
    
    // Start with valid JSON structure
    json_attempt.push_str(r#"{"nodes":["#);
    
    // Add fuzzed content
    for &byte in data.iter().take(100) {
        match byte % 10 {
            0 => json_attempt.push_str(r#"{"id":""#),
            1 => json_attempt.push_str(r#"","data":""#),
            2 => json_attempt.push_str(r#""},"#),
            3 => json_attempt.push('"'),
            4 => json_attempt.push(','),
            5 => json_attempt.push('{'),
            6 => json_attempt.push('}'),
            7 => json_attempt.push('['),
            8 => json_attempt.push(']'),
            _ => json_attempt.push((byte % 128) as char),
        }
    }
    
    json_attempt.push_str(r#"],"edges":[]}"#);
    
    // Try to parse - should handle gracefully without panic
    let _ = IpldGraph::from_json(&json_attempt);
    let _ = ContextGraph::from_json(&json_attempt);
    let _ = WorkflowGraph::from_json(&json_attempt);
});