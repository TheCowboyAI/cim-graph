//! Tests for error scenarios and edge cases

use cim_graph::graphs::{ComposedGraph, IpldGraph, ContextGraph, WorkflowGraph, ConceptGraph};
use cim_graph::{Graph, GraphError, Result};
use cim_graph::algorithms::bfs;
use serde_json::json;
use uuid::Uuid;
use std::collections::HashSet;

#[test]
fn test_cyclic_dependency_detection() -> Result<()> {
    let mut workflow = WorkflowGraph::new("cyclic-test");
    
    // Create states
    let state_a = workflow.add_state("A", json!({}))?;
    let state_b = workflow.add_state("B", json!({}))?;
    let state_c = workflow.add_state("C", json!({}))?;
    
    // Create valid transitions
    workflow.add_transition(state_a, state_b, "a-to-b", json!({}))?;
    workflow.add_transition(state_b, state_c, "b-to-c", json!({}))?;
    
    // Attempt to create cycle - this should be allowed in workflow graphs
    // as they can have loops for retry logic
    let result = workflow.add_transition(state_c, state_a, "c-to-a", json!({}));
    assert!(result.is_ok());
    
    // However, IPLD graphs should prevent cycles
    let mut ipld = IpldGraph::new();
    let cid1 = ipld.add_cid("Qm1", "dag-cbor", 100)?;
    let cid2 = ipld.add_cid("Qm2", "dag-cbor", 100)?;
    let cid3 = ipld.add_cid("Qm3", "dag-cbor", 100)?;
    
    ipld.add_link(cid1, cid2, "link1")?;
    ipld.add_link(cid2, cid3, "link2")?;
    
    // This should fail for IPLD (DAG constraint)
    let cycle_result = ipld.add_link(cid3, cid1, "cycle");
    assert!(cycle_result.is_err());
    
    Ok(())
}

#[test]
fn test_invalid_node_references() -> Result<()> {
    let mut context = ContextGraph::new("test");
    
    // Create valid aggregate
    let aggregate = context.add_aggregate("User", Uuid::new_v4(), json!({
        "name": "Test"
    }))?;
    
    // Try to add entity with non-existent parent
    let fake_parent = Uuid::new_v4();
    let result = context.add_entity("Profile", Uuid::new_v4(), fake_parent, json!({}));
    
    // Should fail with invalid parent
    assert!(result.is_err());
    
    // Try to add relationship with non-existent nodes
    let fake_id = Uuid::new_v4();
    let result = context.add_relationship(aggregate, fake_id, "relates-to");
    assert!(result.is_err());
    
    let result = context.add_relationship(fake_id, aggregate, "relates-to");
    assert!(result.is_err());
    
    Ok(())
}

#[test]
fn test_graph_capacity_limits() -> Result<()> {
    // Test graph behavior at capacity
    let mut small_graph = IpldGraph::with_capacity(5, 5);
    
    // Add nodes up to capacity
    for i in 0..5 {
        small_graph.add_cid(&format!("Qm{}", i), "dag-cbor", 100)?;
    }
    
    // Adding beyond capacity should still work (dynamic growth)
    let result = small_graph.add_cid("QmExtra", "dag-cbor", 100);
    assert!(result.is_ok());
    
    // Verify graph grew
    assert_eq!(small_graph.node_count(), 6);
    
    Ok(())
}

#[test]
fn test_duplicate_node_handling() -> Result<()> {
    let mut ipld = IpldGraph::new();
    
    // Add a CID
    let cid1 = ipld.add_cid("QmDuplicate", "dag-cbor", 256)?;
    
    // Try to add same CID again
    let result = ipld.add_cid("QmDuplicate", "dag-cbor", 256);
    
    // Should fail with duplicate
    assert!(result.is_err());
    match result {
        Err(GraphError::DuplicateNode(id)) => assert_eq!(id, "QmDuplicate"),
        _ => panic!("Expected DuplicateNode error"),
    }
    
    Ok(())
}

#[test]
fn test_invalid_state_transitions() -> Result<()> {
    let mut workflow = WorkflowGraph::new("invalid-transitions");
    
    // Create disconnected states
    let start = workflow.add_state("start", json!({}))?;
    let end = workflow.add_state("end", json!({}))?;
    let orphan = workflow.add_state("orphan", json!({}))?;
    
    workflow.add_transition(start, end, "complete", json!({}))?;
    
    // Try to check if states are connected
    let neighbors = workflow.graph().neighbors(&start)?;
    assert!(!neighbors.contains(&orphan.to_string()));
    
    // Try to add transition with same action name (should fail)
    let duplicate_result = workflow.add_transition(start, orphan, "complete", json!({}));
    assert!(duplicate_result.is_err());
    
    Ok(())
}

#[test]
fn test_malformed_data_handling() -> Result<()> {
    let mut context = ContextGraph::new("malformed-test");
    
    // Add aggregate with deeply nested data
    let deeply_nested = json!({
        "level1": {
            "level2": {
                "level3": {
                    "level4": {
                        "level5": {
                            "data": "deep"
                        }
                    }
                }
            }
        }
    });
    
    let result = context.add_aggregate("DeepData", Uuid::new_v4(), deeply_nested);
    assert!(result.is_ok());
    
    // Add aggregate with very large data
    let mut large_array = Vec::new();
    for i in 0..10000 {
        large_array.push(json!({ "index": i, "data": "x".repeat(100) }));
    }
    
    let large_data = json!({
        "items": large_array
    });
    
    let result = context.add_aggregate("LargeData", Uuid::new_v4(), large_data);
    assert!(result.is_ok());
    
    Ok(())
}

#[test]
fn test_concurrent_modification_simulation() -> Result<()> {
    // Simulate concurrent modifications by interleaving operations
    let mut graph = ContextGraph::new("concurrent-test");
    
    // Add initial data
    let user1 = graph.add_aggregate("User", Uuid::new_v4(), json!({
        "name": "User1",
        "version": 1
    }))?;
    
    let user2 = graph.add_aggregate("User", Uuid::new_v4(), json!({
        "name": "User2",
        "version": 1
    }))?;
    
    // Simulate concurrent relationship additions
    let rel1 = graph.add_relationship(user1, user2, "follows");
    let rel2 = graph.add_relationship(user2, user1, "follows");
    
    // Both should succeed
    assert!(rel1.is_ok());
    assert!(rel2.is_ok());
    
    // Try to add duplicate relationship (should fail)
    let duplicate = graph.add_relationship(user1, user2, "follows");
    assert!(duplicate.is_err());
    
    Ok(())
}

#[test]
fn test_graph_composition_conflicts() -> Result<()> {
    // Create graphs with conflicting node IDs
    let mut graph1 = ContextGraph::new("domain1");
    let mut graph2 = ContextGraph::new("domain2");
    
    let shared_id = Uuid::new_v4();
    
    // Add same ID to both graphs (valid in separate graphs)
    graph1.add_aggregate("Entity", shared_id, json!({ "source": "graph1" }))?;
    graph2.add_aggregate("Entity", shared_id, json!({ "source": "graph2" }))?;
    
    // Compose graphs - should handle ID conflicts
    let composed = ComposedGraph::builder()
        .add_graph("g1", graph1)
        .add_graph("g2", graph2)
        .build()?;
    
    // Both nodes should exist in their respective graphs
    assert_eq!(composed.nodes_in_graph("g1")?.len(), 1);
    assert_eq!(composed.nodes_in_graph("g2")?.len(), 1);
    
    Ok(())
}

#[test]
fn test_invalid_concept_features() -> Result<()> {
    let mut concepts = ConceptGraph::new();
    
    // Test empty features
    let result = concepts.add_concept("Empty", vec![]);
    assert!(result.is_err());
    
    // Test invalid similarity values
    let invalid_features = vec![
        ("feature1", -0.5), // Negative
        ("feature2", 1.5),  // Greater than 1
        ("feature3", f64::NAN), // NaN
    ];
    
    for (name, value) in invalid_features {
        let result = concepts.add_concept("Invalid", vec![(name, value)]);
        assert!(result.is_err());
    }
    
    // Test duplicate feature names
    let duplicate_features = vec![
        ("color", 0.8),
        ("size", 0.6),
        ("color", 0.9), // Duplicate
    ];
    
    let result = concepts.add_concept("Duplicate", duplicate_features);
    assert!(result.is_err());
    
    Ok(())
}

#[test]
fn test_orphaned_nodes_cleanup() -> Result<()> {
    let mut workflow = WorkflowGraph::new("cleanup-test");
    
    // Create connected component
    let s1 = workflow.add_state("start", json!({}))?;
    let s2 = workflow.add_state("middle", json!({}))?;
    let s3 = workflow.add_state("end", json!({}))?;
    
    workflow.add_transition(s1, s2, "next", json!({}))?;
    workflow.add_transition(s2, s3, "complete", json!({}))?;
    
    // Create orphaned nodes
    let orphan1 = workflow.add_state("orphan1", json!({}))?;
    let orphan2 = workflow.add_state("orphan2", json!({}))?;
    
    // Connect orphans to each other but not main graph
    workflow.add_transition(orphan1, orphan2, "orphan-link", json!({}))?;
    
    // Find connected components using BFS
    let mut visited = HashSet::new();
    let mut components = Vec::new();
    let all_nodes = workflow.get_all_nodes()?;
    
    for node in all_nodes {
        if !visited.contains(node.id()) {
            let component = bfs(&workflow, node.id())?;
            for node_id in &component {
                visited.insert(node_id.clone());
            }
            components.push(component);
        }
    }
    
    // Should have 2 separate components
    assert_eq!(components.len(), 2);
    
    // Verify component sizes
    let sizes: Vec<usize> = components.iter().map(|c| c.len()).collect();
    assert!(sizes.contains(&3)); // Main component
    assert!(sizes.contains(&2)); // Orphan component
    
    Ok(())
}

#[test]
fn test_edge_case_node_types() -> Result<()> {
    let mut context = ContextGraph::new("edge-cases");
    
    // Test empty strings
    let result = context.add_aggregate("", Uuid::new_v4(), json!({}));
    assert!(result.is_err());
    
    // Test very long type names
    let long_type = "A".repeat(1000);
    let result = context.add_aggregate(&long_type, Uuid::new_v4(), json!({}));
    assert!(result.is_ok()); // Should handle gracefully
    
    // Test special characters
    let special_chars = vec![
        "Type-With-Dashes",
        "Type.With.Dots",
        "Type_With_Underscores",
        "Type With Spaces",
        "Type/With/Slashes",
        "Type\\With\\Backslashes",
    ];
    
    for type_name in special_chars {
        let result = context.add_aggregate(type_name, Uuid::new_v4(), json!({}));
        assert!(result.is_ok(), "Failed on type: {}", type_name);
    }
    
    Ok(())
}

#[test]
fn test_recursive_graph_operations() -> Result<()> {
    let mut ipld = IpldGraph::new();
    
    // Create a deep chain
    let mut current = ipld.add_cid("QmRoot", "dag-cbor", 100)?;
    
    for i in 1..100 {
        let next = ipld.add_cid(&format!("Qm{}", i), "dag-cbor", 100)?;
        ipld.add_link(current, next, "child")?;
        current = next;
    }
    
    // Test deep traversal
    let root = ipld.get_node_by_id("QmRoot")?;
    let mut depth = 0;
    let mut current_id = root.id();
    
    while let Ok(edges) = ipld.get_edges_from(current_id) {
        if edges.is_empty() {
            break;
        }
        depth += 1;
        current_id = edges[0].to();
    }
    
    assert_eq!(depth, 99);
    
    Ok(())
}

#[test]
fn test_memory_stress_scenarios() -> Result<()> {
    // Test with many small graphs
    let mut composed = ComposedGraph::builder();
    
    for i in 0..100 {
        let mut small_graph = ContextGraph::new(&format!("ctx-{}", i));
        for j in 0..10 {
            small_graph.add_aggregate(
                "Entity",
                Uuid::new_v4(),
                json!({ "index": j })
            )?;
        }
        composed = composed.add_graph(&format!("graph-{}", i), small_graph);
    }
    
    let large_composed = composed.build()?;
    
    // Should handle 100 graphs with 10 nodes each
    assert_eq!(large_composed.graph_count(), 100);
    assert_eq!(large_composed.total_nodes(), 1000);
    
    Ok(())
}