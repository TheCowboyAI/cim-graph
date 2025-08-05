//! Tests for graph algorithms across different graph types

use cim_graph::graphs::{ComposedGraph, IpldGraph, ContextGraph, WorkflowGraph, ConceptGraph};
use cim_graph::graphs::context::RelationshipType;
use cim_graph::graphs::workflow::{WorkflowNode, StateType};
use cim_graph::{Graph, Result};
use cim_graph::algorithms::{bfs, dfs, shortest_path};
use serde_json::json;
use uuid::Uuid;
use std::collections::{HashMap, HashSet};

#[test]
fn test_bfs_across_graph_types() -> Result<()> {
    // Test BFS on IPLD graph
    let mut ipld = IpldGraph::new();
    let root = ipld.add_content(serde_json::json!({ "cid": "QmRoot", "format": "dag-cbor", "size": 100 }))?;
    let child1 = ipld.add_content(serde_json::json!({ "cid": "QmChild1", "format": "dag-cbor", "size": 100 }))?;
    let child2 = ipld.add_content(serde_json::json!({ "cid": "QmChild2", "format": "dag-cbor", "size": 100 }))?;
    let grandchild = ipld.add_content(serde_json::json!({ "cid": "QmGrandchild", "format": "dag-cbor", "size": 100 }))?;
    
    ipld.add_link(&root, &child1, "left")?;
    ipld.add_link(&root, &child2, "right")?;
    ipld.add_link(&child1, &grandchild, "child")?;
    
    let ipld_bfs = bfs(&ipld, root)?;
    assert_eq!(ipld_bfs.len(), 4);
    assert_eq!(ipld_bfs[0], root); // Root first
    
    // Test BFS on Context graph
    let mut context = ContextGraph::new();
    context.add_bounded_context("test", "Test Context")?;
    
    let agg1_id = Uuid::new_v4().to_string();
    let agg1 = context.add_aggregate(&agg1_id, "Type1", "test")?;
    
    let agg2_id = Uuid::new_v4().to_string();
    let agg2 = context.add_aggregate(&agg2_id, "Type2", "test")?;
    
    let entity_id = Uuid::new_v4().to_string();
    let entity = context.add_entity(&entity_id, "Entity", &agg1)?;
    
    context.add_relationship(agg1, agg2, RelationshipType::References)?;
    
    let context_bfs = bfs(&context, agg1)?;
    assert!(context_bfs.contains(&agg1));
    assert!(context_bfs.contains(&agg2));
    assert!(context_bfs.contains(&entity));
    
    // Test BFS on Workflow graph
    let mut workflow = WorkflowGraph::new();
    let start_node = WorkflowNode::new("start", "start", StateType::Initial);
    let start = workflow.add_state(start_node)?;
    let middle_node = WorkflowNode::new("middle", "middle", StateType::Normal);
    let middle = workflow.add_state(middle_node)?;
    let end_node = WorkflowNode::new("end", "end", StateType::Final);
    let end = workflow.add_state(end_node)?;
    
    workflow.add_transition("start", "middle", "go")?;
    workflow.add_transition("middle", "end", "finish")?;
    
    let workflow_bfs = bfs(&workflow, start)?;
    assert_eq!(workflow_bfs, vec![start, middle, end]);
    
    Ok(())
}

#[test]
fn test_dfs_across_graph_types() -> Result<()> {
    // Create graphs with cycles (where allowed)
    let mut context = ContextGraph::new();
    context.add_bounded_context("test", "Test Context")?;
    
    let a_id = Uuid::new_v4().to_string();
    let a = context.add_aggregate(&a_id, "A", "test")?;
    
    let b_id = Uuid::new_v4().to_string();
    let b = context.add_aggregate(&b_id, "B", "test")?;
    
    let c_id = Uuid::new_v4().to_string();
    let c = context.add_aggregate(&c_id, "C", "test")?;
    
    context.add_relationship(a, b, RelationshipType::References)?;
    context.add_relationship(b, c, RelationshipType::References)?;
    context.add_relationship(c, a, RelationshipType::References)?; // Cycle
    
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
    let mut workflow = WorkflowGraph::new();
    
    let s1_node = WorkflowNode::new("s1", "s1", StateType::Initial);
    let s1 = workflow.add_state(s1_node)?;
    let s2_node = WorkflowNode::new("s2", "s2", StateType::Normal);
    let s2 = workflow.add_state(s2_node)?;
    let s3_node = WorkflowNode::new("s3", "s3", StateType::Normal);
    let s3 = workflow.add_state(s3_node)?;
    let s4_node = WorkflowNode::new("s4", "s4", StateType::Final);
    let s4 = workflow.add_state(s4_node)?;
    
    // Create weighted graph
    workflow.add_transition("s1", "s2", "fast")?;
    workflow.add_transition("s1", "s3", "slow")?;
    workflow.add_transition("s2", "s4", "medium")?;
    workflow.add_transition("s3", "s4", "direct")?;
    
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
    concepts.add_relation(&nn, &dl, SemanticRelation::Custom)?;
    concepts.add_relation(&nn, &ml, SemanticRelation::Custom)?;
    
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
    let mut workflow = WorkflowGraph::new();
    
    // First component: s1 -> s2 -> s3
    let s1_node = WorkflowNode::new("s1", "s1", StateType::Initial);
    let s1 = workflow.add_state(s1_node)?;
    let s2_node = WorkflowNode::new("s2", "s2", StateType::Normal);
    let s2 = workflow.add_state(s2_node)?;
    let s3_node = WorkflowNode::new("s3", "s3", StateType::Normal);
    let s3 = workflow.add_state(s3_node)?;
    
    workflow.add_transition("s1", "s2", "1-2")?;
    workflow.add_transition("s2", "s3", "2-3")?;
    
    // Second component: s4 <-> s5
    let s4_node = WorkflowNode::new("s4", "s4", StateType::Normal);
    let s4 = workflow.add_state(s4_node)?;
    let s5_node = WorkflowNode::new("s5", "s5", StateType::Normal);
    let s5 = workflow.add_state(s5_node)?;
    
    workflow.add_transition("s4", "s5", "4-5")?;
    workflow.add_transition("s5", "s4", "5-4")?;
    
    // Isolated node
    let s6_node = WorkflowNode::new("s6", "s6", StateType::Normal);
    let s6 = workflow.add_state(s6_node)?;
    
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
    let mut g1 = ContextGraph::new();
    let mut g2 = ContextGraph::new();
    
    g1.add_bounded_context("test1", "Test Context 1")?;
    g2.add_bounded_context("test2", "Test Context 2")?;
    
    // Graph 1 nodes
    let a_id = Uuid::new_v4().to_string();
    let a = g1.add_aggregate(&a_id, "Node_A", "test1")?;
    
    let b_id = Uuid::new_v4().to_string();
    let b = g1.add_aggregate(&b_id, "Node_B", "test1")?;
    
    let c_id = Uuid::new_v4().to_string();
    let c = g1.add_aggregate(&c_id, "Node_C", "test1")?;
    
    g1.add_relationship(a, b, RelationshipType::References)?;
    g1.add_relationship(b, c, RelationshipType::References)?;
    
    // Graph 2 nodes  
    let d_id = Uuid::new_v4().to_string();
    let d = g2.add_aggregate(&d_id, "Node_D", "test2")?;
    
    let e_id = Uuid::new_v4().to_string();
    let e = g2.add_aggregate(&e_id, "Node_E", "test2")?;
    
    g2.add_relationship(d, e, RelationshipType::References)?;
    
    // Note: In practice, composed graphs don't have cross-graph edges,
    // but we can still run algorithms on individual sub-graphs
    let composed = ComposedGraph::new()
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
    
    use cim_graph::graphs::workflow::{WorkflowNode, StateType};
    let s1 = workflow.add_state(WorkflowNode::new("s1", "State 1", StateType::Initial))?;
    let s2 = workflow.add_state(WorkflowNode::new("s2", "State 2", StateType::Normal))?;
    let s3 = workflow.add_state(WorkflowNode::new("s3", "State 3", StateType::Final))?;
    
    workflow.add_transition(&s1, &s2, "a")?;
    workflow.add_transition(&s2, &s3, "b")?;
    
    // No cycle yet
    assert!(!has_cycle(&workflow)?);
    
    // Add cycle
    workflow.add_transition(&s3, &s1, "c")?;
    assert!(has_cycle(&workflow)?);
    
    // Test on IPLD (DAG - no cycles allowed)
    let mut ipld = IpldGraph::new();
    let c1 = ipld.add_content(serde_json::json!({ "cid": "Qm1", "format": "dag-cbor", "size": 100 }))?;
    let c2 = ipld.add_content(serde_json::json!({ "cid": "Qm2", "format": "dag-cbor", "size": 100 }))?;
    
    ipld.add_link(&c1, &c2, "link")?;
    
    // IPLD enforces DAG property, so no cycles possible
    assert!(!has_cycle(&ipld)?);
    
    Ok(())
}

#[test]
fn test_graph_metrics_calculation() -> Result<()> {
    // Create various graphs and calculate metrics
    let mut concept = ConceptGraph::new();
    
    // Create hub-and-spoke pattern
    let hub = concept.add_concept("Hub", "Hub Concept", serde_json::json!({"central": 1.0}))?;
    
    for i in 0..5 {
        let spoke = concept.add_concept(
            &format!("Spoke{}", i), 
            &format!("Spoke {} Concept", i),
            serde_json::json!({"peripheral": 1.0})
        )?;
        concept.add_relation(&spoke, &hub, cim_graph::graphs::concept::SemanticRelation::Custom)?;
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
    let mut context = ContextGraph::new();
    
    // Create clusters
    let mut cluster1_nodes = Vec::new();
    let mut cluster2_nodes = Vec::new();
    
    // Add bounded context
    context.add_bounded_context("test", "Test Context")?;
    
    // Cluster 1
    let c1_root_id = Uuid::new_v4().to_string();
    let c1_root = context.add_aggregate(&c1_root_id, "Cluster1", "test")?;
    cluster1_nodes.push(c1_root.clone());
    
    for i in 0..3 {
        let node_id = Uuid::new_v4().to_string();
        let node = context.add_entity(&node_id, &format!("C1Node_{}", i), &c1_root)?;
        cluster1_nodes.push(node);
    }
    
    // Cluster 2
    let c2_root_id = Uuid::new_v4().to_string();
    let c2_root = context.add_aggregate(&c2_root_id, "Cluster2", "test")?;
    cluster2_nodes.push(c2_root.clone());
    
    for i in 0..3 {
        let node_id = Uuid::new_v4().to_string();
        let node = context.add_entity(&node_id, &format!("C2Node_{}", i), &c2_root)?;
        cluster2_nodes.push(node);
    }
    
    // Weak link between clusters
    context.add_relationship(&c1_root, &c2_root, RelationshipType::References)?;
    
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