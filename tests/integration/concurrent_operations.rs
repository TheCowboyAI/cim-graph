//! Tests for concurrent operations and thread safety

use cim_graph::graphs::{ComposedGraph, IpldGraph, ContextGraph, WorkflowGraph, ConceptGraph};
use cim_graph::{Graph, Result, GraphError};
use serde_json::json;
use uuid::Uuid;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

#[test]
fn test_concurrent_node_additions() -> Result<()> {
    let graph = Arc::new(std::sync::Mutex::new(ContextGraph::new("concurrent")));
    let num_threads = 4;
    let nodes_per_thread = 25;
    
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = vec![];
    
    for thread_id in 0..num_threads {
        let graph_clone = Arc::clone(&graph);
        let barrier_clone = Arc::clone(&barrier);
        
        let handle = thread::spawn(move || {
            barrier_clone.wait();
            
            for i in 0..nodes_per_thread {
                let mut g = graph_clone.lock().unwrap();
                g.add_aggregate(
                    &format!("Entity-{}-{}", thread_id, i),
                    Uuid::new_v4(),
                    json!({
                        "thread": thread_id,
                        "index": i
                    })
                ).unwrap();
            }
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let final_graph = graph.lock().unwrap();
    assert_eq!(final_graph.node_count(), num_threads * nodes_per_thread);
    
    Ok(())
}

#[test]
fn test_concurrent_reads() -> Result<()> {
    // Create graph with data
    let mut base_graph = IpldGraph::new();
    let node_count = 100;
    
    for i in 0..node_count {
        base_graph.add_cid(&format!("Qm{}", i), "dag-cbor", i * 100)?;
    }
    
    let graph = Arc::new(base_graph);
    let num_readers = 8;
    let barrier = Arc::new(Barrier::new(num_readers));
    let mut handles = vec![];
    
    for reader_id in 0..num_readers {
        let graph_clone = Arc::clone(&graph);
        let barrier_clone = Arc::clone(&barrier);
        
        let handle = thread::spawn(move || {
            barrier_clone.wait();
            
            // Each reader performs multiple reads
            for _ in 0..50 {
                for i in 0..node_count {
                    let node = graph_clone.get_node_by_id(&format!("Qm{}", i)).unwrap();
                    assert_eq!(node.node_type(), "cid");
                }
            }
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    Ok(())
}

#[test]
fn test_concurrent_graph_composition() -> Result<()> {
    let num_threads = 4;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = vec![];
    
    let composed_builder = Arc::new(std::sync::Mutex::new(ComposedGraph::builder()));
    
    for thread_id in 0..num_threads {
        let builder_clone = Arc::clone(&composed_builder);
        let barrier_clone = Arc::clone(&barrier);
        
        let handle = thread::spawn(move || {
            barrier_clone.wait();
            
            // Each thread creates its own graph
            let mut graph = ContextGraph::new(&format!("context-{}", thread_id));
            
            for i in 0..10 {
                graph.add_aggregate(
                    "Entity",
                    Uuid::new_v4(),
                    json!({ "thread": thread_id, "index": i })
                ).unwrap();
            }
            
            // Add to composed builder
            let mut builder = builder_clone.lock().unwrap();
            *builder = builder.clone().add_graph(&format!("graph-{}", thread_id), graph);
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let builder = composed_builder.lock().unwrap();
    let composed = builder.clone().build()?;
    
    assert_eq!(composed.graph_count(), num_threads);
    assert_eq!(composed.total_nodes(), num_threads * 10);
    
    Ok(())
}

#[test]
fn test_race_condition_prevention() -> Result<()> {
    let graph = Arc::new(std::sync::Mutex::new(WorkflowGraph::new("race-test")));
    
    // Add initial states
    let (s1, s2) = {
        let mut g = graph.lock().unwrap();
        let state1 = g.add_state("state1", json!({}))?;
        let state2 = g.add_state("state2", json!({}))?;
        (state1, state2)
    };
    
    let barrier = Arc::new(Barrier::new(2));
    let graph1 = Arc::clone(&graph);
    let graph2 = Arc::clone(&graph);
    
    // Thread 1 tries to add transition
    let handle1 = thread::spawn(move || {
        barrier.wait();
        let mut g = graph1.lock().unwrap();
        g.add_transition(s1, s2, "transition-1", json!({}))
    });
    
    // Thread 2 tries to add same transition
    let handle2 = thread::spawn(move || {
        barrier.wait();
        thread::sleep(Duration::from_millis(10)); // Small delay
        let mut g = graph2.lock().unwrap();
        g.add_transition(s1, s2, "transition-1", json!({}))
    });
    
    let result1 = handle1.join().unwrap();
    let result2 = handle2.join().unwrap();
    
    // One should succeed, one should fail
    assert!(result1.is_ok() ^ result2.is_ok());
    
    Ok(())
}

#[test]
fn test_concurrent_traversals() -> Result<()> {
    // Create a complex graph
    let mut base_graph = WorkflowGraph::new("traversal-test");
    
    // Create branching workflow
    let start = base_graph.add_state("start", json!({}))?;
    let mut branches = vec![];
    
    for i in 0..4 {
        let branch_start = base_graph.add_state(&format!("branch-{}-start", i), json!({}))?;
        let branch_mid = base_graph.add_state(&format!("branch-{}-mid", i), json!({}))?;
        let branch_end = base_graph.add_state(&format!("branch-{}-end", i), json!({}))?;
        
        base_graph.add_transition(start, branch_start, &format!("to-branch-{}", i), json!({}))?;
        base_graph.add_transition(branch_start, branch_mid, "continue", json!({}))?;
        base_graph.add_transition(branch_mid, branch_end, "finish", json!({}))?;
        
        branches.push((branch_start, branch_mid, branch_end));
    }
    
    let graph = Arc::new(base_graph);
    let barrier = Arc::new(Barrier::new(branches.len()));
    let mut handles = vec![];
    
    // Each thread traverses from a different branch
    for (i, (branch_start, _, branch_end)) in branches.into_iter().enumerate() {
        let graph_clone = Arc::clone(&graph);
        let barrier_clone = Arc::clone(&barrier);
        
        let handle = thread::spawn(move || {
            barrier_clone.wait();
            
            // Traverse multiple times
            for _ in 0..100 {
                let path = graph_clone.find_path(branch_start, branch_end);
                assert!(path.is_some());
                assert_eq!(path.unwrap().len(), 3);
            }
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    Ok(())
}

#[test]
fn test_concurrent_serialization() -> Result<()> {
    let mut graph = ConceptGraph::new();
    
    // Add concepts
    for i in 0..50 {
        let features = vec![
            ("feature1", (i as f64) / 50.0),
            ("feature2", 1.0 - (i as f64) / 50.0),
        ];
        graph.add_concept(&format!("concept-{}", i), features)?;
    }
    
    let graph = Arc::new(graph);
    let num_threads = 4;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = vec![];
    
    for thread_id in 0..num_threads {
        let graph_clone = Arc::clone(&graph);
        let barrier_clone = Arc::clone(&barrier);
        
        let handle = thread::spawn(move || {
            barrier_clone.wait();
            
            // Each thread serializes multiple times
            for i in 0..25 {
                let serialized = serde_json::to_string(&*graph_clone).unwrap();
                let deserialized: ConceptGraph = serde_json::from_str(&serialized).unwrap();
                
                assert_eq!(deserialized.node_count(), graph_clone.node_count());
            }
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    Ok(())
}

#[test]
fn test_concurrent_algorithm_execution() -> Result<()> {
    use cim_graph::algorithms::{bfs, dfs, shortest_path};
    
    // Create shared graph
    let mut base_graph = WorkflowGraph::new("algo-test");
    
    // Create interconnected states
    let states: Vec<_> = (0..20)
        .map(|i| base_graph.add_state(&format!("state-{}", i), json!({})).unwrap())
        .collect();
    
    // Create mesh of transitions
    for i in 0..states.len() {
        for j in 1..=3 {
            let target = (i + j) % states.len();
            base_graph.add_transition(
                states[i],
                states[target],
                &format!("t-{}-{}", i, target),
                json!({ "weight": j as f64 })
            ).ok(); // Ignore duplicate edge errors
        }
    }
    
    let graph = Arc::new(base_graph);
    let barrier = Arc::new(Barrier::new(3));
    
    // Thread 1: BFS
    let g1 = Arc::clone(&graph);
    let b1 = Arc::clone(&barrier);
    let handle1 = thread::spawn(move || {
        b1.wait();
        for _ in 0..50 {
            let result = bfs(&*g1, states[0]).unwrap();
            assert_eq!(result.len(), states.len());
        }
    });
    
    // Thread 2: DFS
    let g2 = Arc::clone(&graph);
    let b2 = Arc::clone(&barrier);
    let handle2 = thread::spawn(move || {
        b2.wait();
        for _ in 0..50 {
            let result = dfs(&*g2, states[0]).unwrap();
            assert_eq!(result.len(), states.len());
        }
    });
    
    // Thread 3: Dijkstra
    let g3 = Arc::clone(&graph);
    let b3 = Arc::clone(&barrier);
    let handle3 = thread::spawn(move || {
        b3.wait();
        for _ in 0..50 {
            let path_result = shortest_path(&*g3, states[0], states[states.len()-1]).unwrap();
            assert!(path_result.is_some());
        }
    });
    
    handle1.join().unwrap();
    handle2.join().unwrap();
    handle3.join().unwrap();
    
    Ok(())
}

#[test]
fn test_memory_consistency() -> Result<()> {
    let graph = Arc::new(std::sync::Mutex::new(ContextGraph::new("memory-test")));
    let num_operations = 1000;
    let num_threads = 4;
    
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = vec![];
    
    for thread_id in 0..num_threads {
        let graph_clone = Arc::clone(&graph);
        let barrier_clone = Arc::clone(&barrier);
        
        let handle = thread::spawn(move || {
            barrier_clone.wait();
            
            for i in 0..num_operations / num_threads {
                let mut g = graph_clone.lock().unwrap();
                
                // Alternate between adds and reads
                if i % 2 == 0 {
                    g.add_aggregate(
                        "Entity",
                        Uuid::new_v4(),
                        json!({ "thread": thread_id, "op": i })
                    ).unwrap();
                } else {
                    // Read current node count
                    let count = g.node_count();
                    assert!(count > 0);
                }
            }
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let final_graph = graph.lock().unwrap();
    let expected_nodes = (num_operations / num_threads / 2) * num_threads;
    assert_eq!(final_graph.node_count(), expected_nodes);
    
    Ok(())
}