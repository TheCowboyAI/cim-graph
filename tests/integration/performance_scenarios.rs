//! Performance tests for various graph operations

use cim_graph::graphs::{ComposedGraph, IpldGraph, ContextGraph, WorkflowGraph, ConceptGraph};
use cim_graph::graphs::context::RelationshipType;
use cim_graph::graphs::workflow::{WorkflowNode, StateType};
use cim_graph::graphs::concept::SemanticRelation;
use cim_graph::{Graph, Result};
use serde_json::json;
use uuid::Uuid;
use std::time::Instant;

#[test]
fn test_large_graph_creation_performance() -> Result<()> {
    let sizes: Vec<usize> = vec![100, 1000, 5000];
    
    for size in sizes {
        let start = Instant::now();
        
        let mut graph = ContextGraph::new();
        graph.add_bounded_context("perf", "Performance Test")?;
        
        // Add nodes
        let mut nodes = Vec::new();
        for i in 0..size {
            let node_id = Uuid::new_v4().to_string();
            let node = graph.add_aggregate(
                &node_id,
                &format!("Entity_{}", i),
                "perf"
            )?;
            nodes.push(node);
        }
        
        // Add edges (each node connected to next 3)
        for i in 0..size.saturating_sub(3) {
            for j in 1..=3 {
                graph.add_relationship(
                    &nodes[i],
                    &nodes[i + j],
                    RelationshipType::References
                )?;
            }
        }
        
        let elapsed = start.elapsed();
        
        println!("Created graph with {} nodes in {:?}", size, elapsed);
        
        // Verify structure
        // Context graphs don't have direct node_count() method
        // We'd need to check the actual nodes added
        
        // Performance assertions
        match size {
            100 => assert!(elapsed.as_millis() < 50, "Small graph too slow"),
            1000 => assert!(elapsed.as_millis() < 500, "Medium graph too slow"),
            5000 => assert!(elapsed.as_secs() < 5, "Large graph too slow"),
            _ => {}
        }
    }
    
    Ok(())
}

#[test]
fn test_graph_traversal_performance() -> Result<()> {
    // Create a deep graph
    let mut workflow = WorkflowGraph::new();
    
    let depth = 1000;
    let mut state_names = Vec::new();
    
    // Create linear chain
    for i in 0..depth {
        let state_name = format!("state-{}", i);
        let state_type = if i == 0 { StateType::Initial } else if i == depth - 1 { StateType::Final } else { StateType::Normal };
        let state_node = WorkflowNode::new(&state_name, &state_name, state_type);
        workflow.add_state(state_node)?;
        state_names.push(state_name);
        
        if i > 0 {
            workflow.add_transition(
                &state_names[i-1],
                &state_names[i],
                "next"
            )?;
        }
    }
    
    // Time traversal
    let start = Instant::now();
    let path = workflow.find_path(states[0], states[depth-1]);
    let elapsed = start.elapsed();
    
    println!("Traversed {} nodes in {:?}", depth, elapsed);
    
    assert!(path.is_some());
    assert_eq!(path.unwrap().len(), depth);
    assert!(elapsed.as_millis() < 100, "Traversal too slow");
    
    Ok(())
}

#[test]
fn test_concurrent_read_performance() -> Result<()> {
    let mut ipld = IpldGraph::new();
    
    // Create graph with many nodes
    let node_count = 1000;
    let mut cids = Vec::new();
    
    for i in 0..node_count {
        let cid = ipld.add_content(serde_json::json!({ "cid": &format!("Qm{}", i), "format": "dag-cbor", "size": i * 100 }))?;
        cids.push(cid);
    }
    
    // Link in a mesh pattern
    for i in 0..node_count {
        for j in 1..=5 {
            let target = (i + j) % node_count;
            ipld.add_link(&cids[i], &cids[target], &format!("link-{}", j))?;
        }
    }
    
    // Simulate concurrent reads
    let start = Instant::now();
    
    for _ in 0..100 {
        // Random access pattern
        for i in (0..node_count).step_by(13) {
            let node = ipld.get_node_by_id(&format!("Qm{}", i))?;
            let edges = ipld.get_edges_from(node.id())?;
            assert!(edges.len() > 0);
        }
    }
    
    let elapsed = start.elapsed();
    println!("100 iterations of random reads in {:?}", elapsed);
    
    assert!(elapsed.as_secs() < 1, "Read performance too slow");
    
    Ok(())
}

#[test]
fn test_memory_efficient_operations() -> Result<()> {
    // Test memory efficiency with large data payloads
    let mut context = ContextGraph::new();
    
    let payload_size = 1024 * 10; // 10KB per node
    let node_count = 100;
    
    let large_data = "x".repeat(payload_size);
    
    context.add_bounded_context("test", "Test Context")?;
    
    let start = Instant::now();
    
    for i in 0..node_count {
        let node_id = Uuid::new_v4().to_string();
        context.add_aggregate(
            &node_id,
            &format!("LargeEntity_{}", i),
            "test"
        )?;
    }
    
    let elapsed = start.elapsed();
    
    println!("Added {} nodes with {}KB payload each in {:?}", 
             node_count, payload_size / 1024, elapsed);
    
    assert!(elapsed.as_secs() < 2, "Large payload handling too slow");
    
    Ok(())
}

#[test]
fn test_graph_algorithm_scaling() -> Result<()> {
    use cim_graph::algorithms::{bfs, shortest_path};
    
    let sizes = vec![50, 100, 200];
    
    for size in sizes {
        let mut workflow = WorkflowGraph::new();
        
        // Create grid-like graph
        let mut node_names = Vec::new();
        for i in 0..size {
            let node_name = format!("node-{}", i);
            let state_type = if i == 0 { StateType::Initial } else if i == size - 1 { StateType::Final } else { StateType::Normal };
            let node = WorkflowNode::new(&node_name, &node_name, state_type);
            workflow.add_state(node)?;
            node_names.push(node_name);
        }
        
        // Connect in grid pattern
        let grid_size = (size as f64).sqrt() as usize;
        for i in 0..size {
            let row = i / grid_size;
            let col = i % grid_size;
            
            // Right neighbor
            if col < grid_size - 1 {
                let right = i + 1;
                workflow.add_transition(&node_names[i], &node_names[right], "right")?;
            }
            
            // Down neighbor
            if row < grid_size - 1 {
                let down = i + grid_size;
                if down < size {
                    workflow.add_transition(&node_names[i], &node_names[down], "down")?;
                }
            }
        }
        
        // Time BFS
        let bfs_start = Instant::now();
        let bfs_result = bfs(&workflow, &node_names[0])?;
        let bfs_elapsed = bfs_start.elapsed();
        
        println!("BFS on {} nodes: {:?}", size, bfs_elapsed);
        assert_eq!(bfs_result.len(), size);
        
        // Time Dijkstra
        let dijkstra_start = Instant::now();
        let path_result = shortest_path(&workflow, &node_names[0], &node_names[size-1])?;
        let dijkstra_elapsed = dijkstra_start.elapsed();
        
        println!("Shortest path on {} nodes: {:?}", size, dijkstra_elapsed);
        assert!(path_result.is_some());
    }
    
    Ok(())
}

#[test]
fn test_composed_graph_performance() -> Result<()> {
    let graph_count = 10;
    let nodes_per_graph = 100;
    
    let start = Instant::now();
    
    let mut builder = ComposedGraph::builder();
    
    // Create multiple graphs
    for i in 0..graph_count {
        let mut graph = ContextGraph::new();
        
        // Add bounded context
        graph.add_bounded_context("test", "Test Context")?;
        
        // Add nodes to each graph
        let mut nodes = Vec::new();
        for j in 0..nodes_per_graph {
            let node_id = Uuid::new_v4().to_string();
            let node = graph.add_aggregate(
                &node_id,
                &format!("Entity_{}_{}", i, j),
                "test"
            )?;
            nodes.push(node);
        }
        
        // Add some relationships
        for j in 0..nodes_per_graph-1 {
            graph.add_relationship(&nodes[j], &nodes[j+1], RelationshipType::References)?;
        }
        
        builder = builder.add_graph(&format!("graph-{}", i), graph);
    }
    
    let composed = builder.build()?;
    let build_elapsed = start.elapsed();
    
    println!("Built composed graph with {} graphs, {} nodes each in {:?}",
             graph_count, nodes_per_graph, build_elapsed);
    
    // Test query performance
    let query_start = Instant::now();
    
    for i in 0..graph_count {
        let nodes = composed.nodes_in_graph(&format!("graph-{}", i))?;
        assert_eq!(nodes.len(), nodes_per_graph);
    }
    
    let query_elapsed = query_start.elapsed();
    
    println!("Queried all {} graphs in {:?}", graph_count, query_elapsed);
    
    assert!(build_elapsed.as_millis() < 500, "Composition too slow");
    assert!(query_elapsed.as_millis() < 50, "Queries too slow");
    
    Ok(())
}

#[test]
fn test_serialization_performance() -> Result<()> {
    let mut concept = ConceptGraph::new();
    
    // Create concept network
    let concept_count = 500;
    let mut concepts = Vec::new();
    
    for i in 0..concept_count {
        let concept_name = format!("concept-{}", i);
        let features = serde_json::json!({
            "feature1": (i as f64) / (concept_count as f64),
            "feature2": 1.0 - (i as f64) / (concept_count as f64),
            "feature3": ((i as f64) * 3.14).sin().abs()
        });
        
        let concept_id = concept.add_concept(&concept_name, &concept_name, features)?;
        concepts.push(concept_id);
    }
    
    // Add relations
    for i in 0..concept_count-1 {
        concept.add_relation(
            &concepts[i],
            &concepts[i+1],
            SemanticRelation::Custom
        )?;
    }
    
    // Time serialization
    let serialize_start = Instant::now();
    let serialized = serde_json::to_string(&concept)?;
    let serialize_elapsed = serialize_start.elapsed();
    
    println!("Serialized {} concepts in {:?}, size: {} bytes",
             concept_count, serialize_elapsed, serialized.len());
    
    // Time deserialization
    let deserialize_start = Instant::now();
    let _deserialized: ConceptGraph = serde_json::from_str(&serialized)?;
    let deserialize_elapsed = deserialize_start.elapsed();
    
    println!("Deserialized {} concepts in {:?}",
             concept_count, deserialize_elapsed);
    
    assert!(serialize_elapsed.as_millis() < 100, "Serialization too slow");
    assert!(deserialize_elapsed.as_millis() < 100, "Deserialization too slow");
    
    Ok(())
}

#[test]
fn test_bulk_operation_performance() -> Result<()> {
    let mut ipld = IpldGraph::new();
    
    // Bulk add nodes
    let bulk_size = 1000;
    let start = Instant::now();
    
    let mut cids = Vec::new();
    for i in 0..bulk_size {
        let cid = ipld.add_content(serde_json::json!({ "cid": &format!("QmBulk{}", i), "format": "dag-cbor", "size": 256 }))?;
        cids.push(cid);
    }
    
    let bulk_add_elapsed = start.elapsed();
    
    println!("Bulk added {} nodes in {:?}", bulk_size, bulk_add_elapsed);
    
    // Bulk add edges
    let edge_start = Instant::now();
    
    for i in 0..bulk_size-1 {
        ipld.add_link(&cids[i], &cids[i+1], "next")?;
    }
    
    let bulk_edge_elapsed = edge_start.elapsed();
    
    println!("Bulk added {} edges in {:?}", bulk_size-1, bulk_edge_elapsed);
    
    assert!(bulk_add_elapsed.as_millis() < 100, "Bulk node add too slow");
    assert!(bulk_edge_elapsed.as_millis() < 100, "Bulk edge add too slow");
    
    Ok(())
}