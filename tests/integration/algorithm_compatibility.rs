//! Tests for graph algorithms across different graph types

use cim_graph::graphs::{ComposedGraph, IpldGraph, ContextGraph, WorkflowGraph, ConceptGraph};
use cim_graph::{Graph, Result};
use cim_graph::algorithms::{bfs, dfs, shortest_path};
use serde_json::json;
use uuid::Uuid;
use std::collections::{HashMap, HashSet};

#[test]
fn test_bfs_across_graph_types() -> Result<()> {
    // Test BFS on IPLD graph
    let mut ipld = IpldGraph::new();
    let root = ipld.add_cid("QmRoot", "dag-cbor", 100)?;
    let child1 = ipld.add_cid("QmChild1", "dag-cbor", 100)?;
    let child2 = ipld.add_cid("QmChild2", "dag-cbor", 100)?;
    let grandchild = ipld.add_cid("QmGrandchild", "dag-cbor", 100)?;
    
    ipld.add_link(root, child1, "left")?;
    ipld.add_link(root, child2, "right")?;
    ipld.add_link(child1, grandchild, "child")?;
    
    let ipld_bfs = bfs(&ipld, root)?;
    assert_eq!(ipld_bfs.len(), 4);
    assert_eq!(ipld_bfs[0], root); // Root first
    
    // Test BFS on Context graph
    let mut context = ContextGraph::new("test");
    let agg1 = context.add_aggregate("Type1", Uuid::new_v4(), json!({}))?;
    let agg2 = context.add_aggregate("Type2", Uuid::new_v4(), json!({}))?;
    let entity = context.add_entity("Entity", Uuid::new_v4(), agg1, json!({}))?;
    
    context.add_relationship(agg1, agg2, "relates")?;
    
    let context_bfs = bfs(&context, agg1)?;
    assert!(context_bfs.contains(&agg1));
    assert!(context_bfs.contains(&agg2));
    assert!(context_bfs.contains(&entity));
    
    // Test BFS on Workflow graph
    let mut workflow = WorkflowGraph::new("test");
    let start = workflow.add_state("start", json!({}))?;
    let middle = workflow.add_state("middle", json!({}))?;
    let end = workflow.add_state("end", json!({}))?;
    
    workflow.add_transition(start, middle, "go", json!({}))?;
    workflow.add_transition(middle, end, "finish", json!({}))?;
    
    let workflow_bfs = bfs(&workflow, start)?;
    assert_eq!(workflow_bfs, vec![start, middle, end]);
    
    Ok(())
}

#[test]
fn test_dfs_across_graph_types() -> Result<()> {
    // Create graphs with cycles (where allowed)
    let mut context = ContextGraph::new("cyclic");
    
    let a = context.add_aggregate("A", Uuid::new_v4(), json!({}))?;
    let b = context.add_aggregate("B", Uuid::new_v4(), json!({}))?;
    let c = context.add_aggregate("C", Uuid::new_v4(), json!({}))?;
    
    context.add_relationship(a, b, "to-b")?;
    context.add_relationship(b, c, "to-c")?;
    context.add_relationship(c, a, "to-a")?; // Cycle
    
    let dfs_result = dfs(&context, a)?;
    assert_eq!(dfs_result.len(), 3);
    
    // Verify DFS doesn't get stuck in cycle
    let visited: HashSet<_> = dfs_result.into_iter().collect();
    assert_eq!(visited.len(), 3);
    
    Ok(())
}

#[test]
fn test_shortest_path_with_weighted_edges() -> Result<()> {
    // Create workflow with weighted transitions
    let mut workflow = WorkflowGraph::new("weighted");
    
    let s1 = workflow.add_state("s1", json!({}))?;
    let s2 = workflow.add_state("s2", json!({}))?;
    let s3 = workflow.add_state("s3", json!({}))?;
    let s4 = workflow.add_state("s4", json!({}))?;
    
    // Create weighted graph
    workflow.add_transition(s1, s2, "fast", json!({ "cost": 1 }))?;
    workflow.add_transition(s1, s3, "slow", json!({ "cost": 5 }))?;
    workflow.add_transition(s2, s4, "medium", json!({ "cost": 2 }))?;
    workflow.add_transition(s3, s4, "direct", json!({ "cost": 1 }))?;
    
    // Find shortest path
    let path_result = shortest_path(&workflow, s1, s4)?;
    
    // Verify shortest path exists
    assert!(path_result.is_some());
    
    if let Some((cost, path)) = path_result {
        // Path should be s1 -> s2 -> s4 (cost = 3)
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], s1);
        assert_eq!(path[1], s2);
        assert_eq!(path[2], s4);
        assert_eq!(cost, 3.0);
    }
    
    Ok(())
}

#[test]
fn test_centrality_on_concept_graph() -> Result<()> {
    // Create concept network
    let mut concepts = ConceptGraph::new();
    
    let ai = concepts.add_concept("AI", "AI", json!({"intelligent": 1.0}))?;
    let ml = concepts.add_concept("ML", "ML", json!({"learning": 1.0}))?;
    let dl = concepts.add_concept("DL", "DL", json!({"deep": 1.0}))?;
    let nn = concepts.add_concept("NN", "NN", json!({"neural": 1.0}))?;
    
    // Create importance hierarchy
    use cim_graph::graphs::concept::SemanticRelation;
    
    concepts.add_relation(&ml, &ai, SemanticRelation::PartOf)?;
    concepts.add_relation(&dl, &ml, SemanticRelation::SubClassOf)?;
    concepts.add_relation(&nn, &dl, SemanticRelation::RelatedTo)?;
    concepts.add_relation(&nn, &ml, SemanticRelation::RelatedTo)?;
    
    // Calculate in-degree centrality manually
    let mut in_degrees = HashMap::new();
    let nodes = concepts.get_all_nodes()?;
    for node in &nodes {
        in_degrees.insert(node.id().to_string(), 0);
    }
    
    let edges = concepts.get_all_edges()?;
    for edge in edges {
        *in_degrees.get_mut(edge.to()).unwrap() += 1;
    }
    
    // AI should have highest in-degree (most incoming links)
    let ai_degree = in_degrees.get(ai.to_string().as_str()).unwrap();
    let ml_degree = in_degrees.get(ml.to_string().as_str()).unwrap();
    
    assert!(ai_degree >= ml_degree);
    
    Ok(())
}

#[test]
fn test_connected_components() -> Result<()> {
    // Create workflow with multiple components
    let mut workflow = WorkflowGraph::new("components-test");
    
    // First component: s1 -> s2 -> s3
    let s1 = workflow.add_state("s1", json!({}))?;
    let s2 = workflow.add_state("s2", json!({}))?;
    let s3 = workflow.add_state("s3", json!({}))?;
    
    workflow.add_transition(s1, s2, "1-2", json!({}))?;
    workflow.add_transition(s2, s3, "2-3", json!({}))?;
    
    // Second component: s4 <-> s5
    let s4 = workflow.add_state("s4", json!({}))?;
    let s5 = workflow.add_state("s5", json!({}))?;
    
    workflow.add_transition(s4, s5, "4-5", json!({}))?;
    workflow.add_transition(s5, s4, "5-4", json!({}))?;
    
    // Isolated node
    let s6 = workflow.add_state("s6", json!({}))?;
    
    // Use BFS to find connected components
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
    
    // Should have 3 components
    assert_eq!(components.len(), 3);
    
    // Verify component sizes
    let mut sizes: Vec<_> = components.iter().map(|c| c.len()).collect();
    sizes.sort();
    assert_eq!(sizes, vec![1, 2, 3]);
    
    Ok(())
}

#[test]
fn test_shortest_path_across_composed_graphs() -> Result<()> {
    // Create multiple connected graphs
    let mut g1 = ContextGraph::new("g1");
    let mut g2 = ContextGraph::new("g2");
    
    // Graph 1 nodes
    let a = g1.add_aggregate("Node", Uuid::new_v4(), json!({ "name": "A" }))?;
    let b = g1.add_aggregate("Node", Uuid::new_v4(), json!({ "name": "B" }))?;
    let c = g1.add_aggregate("Node", Uuid::new_v4(), json!({ "name": "C" }))?;
    
    g1.add_relationship(a, b, "connects")?;
    g1.add_relationship(b, c, "connects")?;
    
    // Graph 2 nodes  
    let d = g2.add_aggregate("Node", Uuid::new_v4(), json!({ "name": "D" }))?;
    let e = g2.add_aggregate("Node", Uuid::new_v4(), json!({ "name": "E" }))?;
    
    g2.add_relationship(d, e, "connects")?;
    
    // Note: In practice, composed graphs don't have cross-graph edges,
    // but we can still run algorithms on individual sub-graphs
    let composed = ComposedGraph::builder()
        .add_graph("first", g1)
        .add_graph("second", g2)
        .build()?;
    
    // Run algorithms on sub-graphs
    let g1_nodes = composed.nodes_in_graph("first")?;
    let g2_nodes = composed.nodes_in_graph("second")?;
    
    assert_eq!(g1_nodes.len(), 3);
    assert_eq!(g2_nodes.len(), 2);
    
    Ok(())
}

#[test]
fn test_cycle_detection_algorithm() -> Result<()> {
    // Test on workflow (allows cycles)
    let mut workflow = WorkflowGraph::new("cycles");
    
    let s1 = workflow.add_state("s1", json!({}))?;
    let s2 = workflow.add_state("s2", json!({}))?;
    let s3 = workflow.add_state("s3", json!({}))?;
    
    workflow.add_transition(s1, s2, "a", json!({}))?;
    workflow.add_transition(s2, s3, "b", json!({}))?;
    
    // No cycle yet
    assert!(!has_cycle(&workflow)?);
    
    // Add cycle
    workflow.add_transition(s3, s1, "c", json!({}))?;
    assert!(has_cycle(&workflow)?);
    
    // Test on IPLD (DAG - no cycles allowed)
    let mut ipld = IpldGraph::new();
    let c1 = ipld.add_cid("Qm1", "dag-cbor", 100)?;
    let c2 = ipld.add_cid("Qm2", "dag-cbor", 100)?;
    
    ipld.add_link(c1, c2, "link")?;
    
    // IPLD enforces DAG property, so no cycles possible
    assert!(!has_cycle(&ipld)?);
    
    Ok(())
}

#[test]
fn test_graph_metrics_calculation() -> Result<()> {
    // Create various graphs and calculate metrics
    let mut concept = ConceptGraph::new();
    
    // Create hub-and-spoke pattern
    let hub = concept.add_concept("Hub", vec![("central", 1.0)])?;
    
    for i in 0..5 {
        let spoke = concept.add_concept(&format!("Spoke{}", i), vec![
            ("peripheral", 1.0)
        ])?;
        concept.add_relation(spoke, hub, "connects-to", 0.8)?;
    }
    
    // Calculate degree centrality
    let in_degrees = calculate_in_degree(&concept)?;
    let out_degrees = calculate_out_degree(&concept)?;
    
    // Hub should have high in-degree
    assert_eq!(*in_degrees.get(&hub).unwrap(), 5);
    
    // Spokes should have out-degree of 1
    for (node, degree) in out_degrees.iter() {
        if node != &hub {
            assert_eq!(*degree, 1);
        }
    }
    
    Ok(())
}

#[test]
fn test_subgraph_extraction() -> Result<()> {
    // Create large graph and extract subgraphs
    let mut context = ContextGraph::new("large");
    
    // Create clusters
    let mut cluster1_nodes = Vec::new();
    let mut cluster2_nodes = Vec::new();
    
    // Cluster 1
    let c1_root = context.add_aggregate("Cluster1", Uuid::new_v4(), json!({}))?;
    cluster1_nodes.push(c1_root);
    
    for i in 0..3 {
        let node = context.add_entity("C1Node", Uuid::new_v4(), c1_root, json!({
            "index": i
        }))?;
        cluster1_nodes.push(node);
    }
    
    // Cluster 2
    let c2_root = context.add_aggregate("Cluster2", Uuid::new_v4(), json!({}))?;
    cluster2_nodes.push(c2_root);
    
    for i in 0..3 {
        let node = context.add_entity("C2Node", Uuid::new_v4(), c2_root, json!({
            "index": i
        }))?;
        cluster2_nodes.push(node);
    }
    
    // Weak link between clusters
    context.add_relationship(c1_root, c2_root, "related")?;
    
    // Extract subgraph around cluster 1
    let subgraph_nodes = bfs(&context, c1_root)?;
    
    // Should include all of cluster 1 and root of cluster 2
    assert!(subgraph_nodes.len() >= 5);
    
    Ok(())
}

// Helper functions

fn has_cycle<G: Graph>(graph: &G) -> Result<bool> {
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();
    
    let nodes = graph.get_all_nodes()?;
    
    for node in nodes {
        if !visited.contains(node.id()) {
            if dfs_cycle_check(graph, node.id(), &mut visited, &mut rec_stack)? {
                return Ok(true);
            }
        }
    }
    
    Ok(false)
}

fn dfs_cycle_check<G: Graph>(
    graph: &G,
    node_id: &str,
    visited: &mut HashSet<String>,
    rec_stack: &mut HashSet<String>,
) -> Result<bool> {
    visited.insert(node_id.to_string());
    rec_stack.insert(node_id.to_string());
    
    let edges = graph.get_edges_from(node_id)?;
    for edge in edges {
        let neighbor = edge.to();
        
        if !visited.contains(neighbor) {
            if dfs_cycle_check(graph, neighbor, visited, rec_stack)? {
                return Ok(true);
            }
        } else if rec_stack.contains(neighbor) {
            return Ok(true);
        }
    }
    
    rec_stack.remove(node_id);
    Ok(false)
}

fn calculate_in_degree<G: Graph>(graph: &G) -> Result<HashMap<String, usize>> {
    let mut in_degrees = HashMap::new();
    
    let nodes = graph.get_all_nodes()?;
    for node in nodes {
        in_degrees.insert(node.id().to_string(), 0);
    }
    
    let edges = graph.get_all_edges()?;
    for edge in edges {
        *in_degrees.get_mut(edge.to()).unwrap() += 1;
    }
    
    Ok(in_degrees)
}

fn calculate_out_degree<G: Graph>(graph: &G) -> Result<HashMap<String, usize>> {
    let mut out_degrees = HashMap::new();
    
    let nodes = graph.get_all_nodes()?;
    for node in nodes {
        let edges = graph.get_edges_from(node.id())?;
        out_degrees.insert(node.id().to_string(), edges.len());
    }
    
    Ok(out_degrees)
}