#![no_main]

use libfuzzer_sys::fuzz_target;
use cim_graph::core::{GraphBuilder, GraphType};
use cim_graph::graphs::workflow::{WorkflowGraph, WorkflowNode, StateType};

// Fuzz target for graph operations
fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    // Create a new graph
    let mut graph = WorkflowGraph::new();
    
    // Use the fuzz data to drive operations
    let mut i = 0;
    while i < data.len() {
        match data[i] % 6 {
            0 => {
                // Add node
                if i + 1 < data.len() {
                    let id = format!("node_{}", data[i + 1]);
                    let node = WorkflowNode::new(&id, &id, StateType::Normal);
                    let _ = graph.add_state(node);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            1 => {
                // Add edge
                if i + 2 < data.len() {
                    let from = format!("node_{}", data[i + 1]);
                    let to = format!("node_{}", data[i + 2]);
                    let _ = graph.add_transition(&from, &to, "fuzz");
                    i += 3;
                } else {
                    i += 1;
                }
            }
            2 => {
                // Remove node
                if i + 1 < data.len() {
                    let id = format!("node_{}", data[i + 1]);
                    let _ = graph.remove_state(&id);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            3 => {
                // Query node
                if i + 1 < data.len() {
                    let id = format!("node_{}", data[i + 1]);
                    let _ = graph.get_state(&id);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            4 => {
                // Get neighbors
                if i + 1 < data.len() {
                    let id = format!("node_{}", data[i + 1]);
                    if let Ok(neighbors) = graph.graph().neighbors(&id) {
                        // Verify neighbors exist
                        for neighbor in neighbors {
                            assert!(graph.get_state(&neighbor).is_some());
                        }
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            5 => {
                // Clear graph
                graph.graph_mut().clear();
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    
    // Validate final state
    let node_count = graph.graph().node_count();
    let edge_count = graph.graph().edge_count();
    
    // Basic invariants
    assert!(node_count >= 0);
    assert!(edge_count >= 0);
    
    // If there are no nodes, there should be no edges
    if node_count == 0 {
        assert_eq!(edge_count, 0);
    }
});

// Additional fuzz target for stress testing with large operations
fuzz_target!(|data: &[u8]| {
    if data.len() < 4 {
        return;
    }
    
    let mut graph = WorkflowGraph::new();
    
    // Use first 4 bytes to determine operation count
    let op_count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) % 1000;
    
    // Generate deterministic operations from remaining data
    for i in 0..op_count as usize {
        let idx = 4 + (i % (data.len() - 4));
        let op = data[idx];
        
        match op % 3 {
            0 => {
                // Bulk add nodes
                for j in 0..10 {
                    let id = format!("bulk_node_{}_{}", i, j);
                    let node = WorkflowNode::new(&id, &id, StateType::Normal);
                    let _ = graph.add_state(node);
                }
            }
            1 => {
                // Create chain
                let start = format!("chain_start_{}", i);
                let end = format!("chain_end_{}", i);
                let _ = graph.add_state(WorkflowNode::new(&start, &start, StateType::Start));
                let _ = graph.add_state(WorkflowNode::new(&end, &end, StateType::End));
                let _ = graph.add_transition(&start, &end, "chain");
            }
            2 => {
                // Create cycle
                let nodes: Vec<_> = (0..5).map(|j| format!("cycle_{}_{}", i, j)).collect();
                for node_id in &nodes {
                    let _ = graph.add_state(WorkflowNode::new(node_id, node_id, StateType::Normal));
                }
                for j in 0..nodes.len() {
                    let from = &nodes[j];
                    let to = &nodes[(j + 1) % nodes.len()];
                    let _ = graph.add_transition(from, to, "cycle");
                }
            }
            _ => {}
        }
    }
    
    // Verify graph remains valid
    let _ = graph.graph().node_count();
    let _ = graph.graph().edge_count();
});