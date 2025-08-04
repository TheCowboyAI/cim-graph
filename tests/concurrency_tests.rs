//! Concurrency and thread safety tests
//!
//! These tests verify that CIM Graph operations are thread-safe
//! and can handle concurrent access correctly.

#![cfg(test)]

use cim_graph::graphs::workflow::{WorkflowGraph, WorkflowNode, StateType};
use cim_graph::graphs::ipld::IpldGraph;
use std::sync::{Arc, Mutex, RwLock, Barrier};
use std::thread;
use std::time::Duration;
use std::collections::HashSet;

/// Wrapper to make graphs thread-safe
struct SharedGraph<G> {
    graph: Arc<Mutex<G>>,
}

impl<G> SharedGraph<G> {
    fn new(graph: G) -> Self {
        Self {
            graph: Arc::new(Mutex::new(graph)),
        }
    }
    
    fn clone_ref(&self) -> Arc<Mutex<G>> {
        Arc::clone(&self.graph)
    }
}

#[test]
fn test_concurrent_node_addition() {
    let graph = SharedGraph::new(WorkflowGraph::new());
    let num_threads = 10;
    let nodes_per_thread = 100;
    
    let mut handles = vec![];
    
    for thread_id in 0..num_threads {
        let graph_ref = graph.clone_ref();
        
        let handle = thread::spawn(move || {
            for i in 0..nodes_per_thread {
                let node_id = format!("thread_{}_node_{}", thread_id, i);
                let node = WorkflowNode::new(&node_id, &node_id, StateType::Normal);
                
                let mut g = graph_ref.lock().unwrap();
                g.add_state(node).unwrap();
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify all nodes were added
    let final_graph = graph.graph.lock().unwrap();
    assert_eq!(
        final_graph.graph().node_count(),
        num_threads * nodes_per_thread
    );
}

#[test]
fn test_concurrent_edge_addition() {
    let graph = SharedGraph::new(WorkflowGraph::new());
    
    // Pre-populate with nodes
    {
        let mut g = graph.graph.lock().unwrap();
        for i in 0..100 {
            let node = WorkflowNode::new(
                &format!("node_{}", i),
                &format!("Node {}", i),
                StateType::Normal,
            );
            g.add_state(node).unwrap();
        }
    }
    
    let num_threads = 5;
    let mut handles = vec![];
    
    for thread_id in 0..num_threads {
        let graph_ref = graph.clone_ref();
        
        let handle = thread::spawn(move || {
            for i in 0..20 {
                let from = format!("node_{}", i * 5 + thread_id);
                let to = format!("node_{}", (i * 5 + thread_id + 1) % 100);
                
                let mut g = graph_ref.lock().unwrap();
                let _ = g.add_transition(&from, &to, &format!("edge_{}_{}", thread_id, i));
            }
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let final_graph = graph.graph.lock().unwrap();
    assert_eq!(final_graph.graph().edge_count(), num_threads * 20);
}

#[test]
fn test_concurrent_reads() {
    // Use RwLock for better read concurrency
    let graph = Arc::new(RwLock::new(WorkflowGraph::new()));
    
    // Populate graph
    {
        let mut g = graph.write().unwrap();
        for i in 0..1000 {
            let node = WorkflowNode::new(
                &format!("node_{}", i),
                &format!("Node {}", i),
                StateType::Normal,
            );
            g.add_state(node).unwrap();
        }
    }
    
    let num_readers = 10;
    let mut handles = vec![];
    
    for reader_id in 0..num_readers {
        let graph_ref = Arc::clone(&graph);
        
        let handle = thread::spawn(move || {
            let mut found_count = 0;
            
            for i in 0..1000 {
                let g = graph_ref.read().unwrap();
                if g.get_state(&format!("node_{}", i)).is_some() {
                    found_count += 1;
                }
            }
            
            assert_eq!(found_count, 1000);
            reader_id
        });
        
        handles.push(handle);
    }
    
    // All readers should complete successfully
    for handle in handles {
        let reader_id = handle.join().unwrap();
        println!("Reader {} completed", reader_id);
    }
}

#[test]
fn test_concurrent_modifications_with_barriers() {
    let graph = SharedGraph::new(WorkflowGraph::new());
    let barrier = Arc::new(Barrier::new(3));
    let mut handles = vec![];
    
    // Thread 1: Add nodes
    {
        let graph_ref = graph.clone_ref();
        let barrier_ref = Arc::clone(&barrier);
        
        let handle = thread::spawn(move || {
            // Phase 1: Add nodes
            {
                let mut g = graph_ref.lock().unwrap();
                for i in 0..50 {
                    let node = WorkflowNode::new(
                        &format!("node_{}", i),
                        &format!("Node {}", i),
                        StateType::Normal,
                    );
                    g.add_state(node).unwrap();
                }
            }
            
            barrier_ref.wait();
            
            // Phase 2: Add more nodes
            {
                let mut g = graph_ref.lock().unwrap();
                for i in 50..100 {
                    let node = WorkflowNode::new(
                        &format!("node_{}", i),
                        &format!("Node {}", i),
                        StateType::Normal,
                    );
                    g.add_state(node).unwrap();
                }
            }
        });
        
        handles.push(handle);
    }
    
    // Thread 2: Add edges
    {
        let graph_ref = graph.clone_ref();
        let barrier_ref = Arc::clone(&barrier);
        
        let handle = thread::spawn(move || {
            barrier_ref.wait();
            
            // Phase 2: Add edges after nodes exist
            {
                let mut g = graph_ref.lock().unwrap();
                for i in 0..49 {
                    let _ = g.add_transition(
                        &format!("node_{}", i),
                        &format!("node_{}", i + 1),
                        &format!("edge_{}", i),
                    );
                }
            }
        });
        
        handles.push(handle);
    }
    
    // Thread 3: Query operations
    {
        let graph_ref = graph.clone_ref();
        let barrier_ref = Arc::clone(&barrier);
        
        let handle = thread::spawn(move || {
            barrier_ref.wait();
            
            // Phase 2: Query after initial nodes added
            {
                let g = graph_ref.lock().unwrap();
                let count = g.graph().node_count();
                assert!(count >= 50); // At least the first batch
            }
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Final verification
    let final_graph = graph.graph.lock().unwrap();
    assert_eq!(final_graph.graph().node_count(), 100);
    assert!(final_graph.graph().edge_count() >= 49);
}

#[test]
fn test_ipld_concurrent_block_operations() {
    let graph = Arc::new(Mutex::new(IpldGraph::new()));
    let num_threads = 8;
    let blocks_per_thread = 50;
    
    let mut handles = vec![];
    
    for thread_id in 0..num_threads {
        let graph_ref = Arc::clone(&graph);
        
        let handle = thread::spawn(move || {
            for i in 0..blocks_per_thread {
                let block_id = format!("thread_{}_block_{}", thread_id, i);
                let content = format!("Content from thread {} block {}", thread_id, i).into_bytes();
                
                // Add block
                {
                    let mut g = graph_ref.lock().unwrap();
                    g.add_block(block_id.clone(), content.clone()).unwrap();
                }
                
                // Verify block can be retrieved
                {
                    let g = graph_ref.lock().unwrap();
                    let retrieved = g.get_block(&block_id).unwrap();
                    assert_eq!(retrieved.content(), &content);
                }
            }
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let final_graph = graph.lock().unwrap();
    assert_eq!(
        final_graph.graph().node_count(),
        num_threads * blocks_per_thread
    );
}

#[test]
fn test_race_condition_detection() {
    let counter = Arc::new(Mutex::new(0));
    let graph = SharedGraph::new(WorkflowGraph::new());
    
    // Add initial nodes
    {
        let mut g = graph.graph.lock().unwrap();
        for i in 0..10 {
            let node = WorkflowNode::new(
                &format!("node_{}", i),
                &format!("Node {}", i),
                StateType::Normal,
            );
            g.add_state(node).unwrap();
        }
    }
    
    let num_threads = 5;
    let mut handles = vec![];
    
    for _ in 0..num_threads {
        let graph_ref = graph.clone_ref();
        let counter_ref = Arc::clone(&counter);
        
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                // Simulate race condition scenario
                let g = graph_ref.lock().unwrap();
                let node_count = g.graph().node_count();
                drop(g); // Release lock
                
                // Increment counter based on node count
                let mut count = counter_ref.lock().unwrap();
                *count += node_count;
            }
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify counter has expected value
    let final_count = *counter.lock().unwrap();
    assert_eq!(final_count, num_threads * 100 * 10); // threads * iterations * nodes
}

#[test]
fn test_deadlock_prevention() {
    // Test that operations don't deadlock with multiple locks
    let graph1 = Arc::new(Mutex::new(WorkflowGraph::new()));
    let graph2 = Arc::new(Mutex::new(WorkflowGraph::new()));
    
    // Add nodes to both graphs
    for i in 0..10 {
        {
            let mut g1 = graph1.lock().unwrap();
            let node = WorkflowNode::new(&format!("g1_n{}", i), "Node", StateType::Normal);
            g1.add_state(node).unwrap();
        }
        {
            let mut g2 = graph2.lock().unwrap();
            let node = WorkflowNode::new(&format!("g2_n{}", i), "Node", StateType::Normal);
            g2.add_state(node).unwrap();
        }
    }
    
    let mut handles = vec![];
    
    // Thread 1: Lock order graph1 -> graph2
    {
        let g1_ref = Arc::clone(&graph1);
        let g2_ref = Arc::clone(&graph2);
        
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let _g1 = g1_ref.lock().unwrap();
                thread::sleep(Duration::from_micros(1));
                let _g2 = g2_ref.lock().unwrap();
            }
        });
        
        handles.push(handle);
    }
    
    // Thread 2: Same lock order to prevent deadlock
    {
        let g1_ref = Arc::clone(&graph1);
        let g2_ref = Arc::clone(&graph2);
        
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let _g1 = g1_ref.lock().unwrap();
                thread::sleep(Duration::from_micros(1));
                let _g2 = g2_ref.lock().unwrap();
            }
        });
        
        handles.push(handle);
    }
    
    // All threads should complete without deadlock
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_concurrent_graph_algorithms() {
    use cim_graph::algorithms;
    
    let graph = Arc::new(RwLock::new(WorkflowGraph::new()));
    
    // Build a graph
    {
        let mut g = graph.write().unwrap();
        for i in 0..100 {
            let node = WorkflowNode::new(
                &format!("node_{}", i),
                &format!("Node {}", i),
                StateType::Normal,
            );
            g.add_state(node).unwrap();
        }
        
        for i in 0..99 {
            g.add_transition(
                &format!("node_{}", i),
                &format!("node_{}", i + 1),
                "next",
            ).unwrap();
        }
    }
    
    let num_threads = 4;
    let mut handles = vec![];
    
    for thread_id in 0..num_threads {
        let graph_ref = Arc::clone(&graph);
        
        let handle = thread::spawn(move || {
            let mut results = Vec::new();
            
            for i in 0..25 {
                let start = format!("node_{}", thread_id * 25 + i);
                let end = format!("node_{}", (thread_id * 25 + i + 10) % 100);
                
                let g = graph_ref.read().unwrap();
                
                // Run algorithm
                if let Ok(Some(path)) = algorithms::shortest_path(g.graph(), &start, &end) {
                    results.push(path.len());
                }
            }
            
            results
        });
        
        handles.push(handle);
    }
    
    // Collect all results
    let mut all_results = Vec::new();
    for handle in handles {
        let results = handle.join().unwrap();
        all_results.extend(results);
    }
    
    // Verify we got results from all threads
    assert!(!all_results.is_empty());
}

#[test]
#[should_panic(expected = "already exists")]
fn test_concurrent_duplicate_detection() {
    // This test verifies that duplicate detection works under concurrent access
    let graph = SharedGraph::new(WorkflowGraph::new());
    let barrier = Arc::new(Barrier::new(2));
    
    let mut handles = vec![];
    
    for thread_id in 0..2 {
        let graph_ref = graph.clone_ref();
        let barrier_ref = Arc::clone(&barrier);
        
        let handle = thread::spawn(move || {
            // Synchronize threads
            barrier_ref.wait();
            
            // Both threads try to add the same node
            let node = WorkflowNode::new("duplicate", "Duplicate Node", StateType::Normal);
            let mut g = graph_ref.lock().unwrap();
            g.add_state(node).unwrap(); // One should succeed, one should fail
            
            thread_id
        });
        
        handles.push(handle);
    }
    
    // Wait for threads - one should panic
    for handle in handles {
        let _ = handle.join();
    }
}