//! Tests for cross-graph operations and conversions

use cim_graph::graphs::{ComposedGraph, IpldGraph, ContextGraph, WorkflowGraph, ConceptGraph};
use cim_graph::{Graph, Result};
use serde_json::json;
use uuid::Uuid;
use std::collections::HashMap;

use super::utils::*;

#[test]
fn test_converting_between_graph_types() -> Result<()> {
    // Create source graphs
    let ipld = create_test_ipld_graph()?;
    let context = create_test_context_graph()?;
    
    // Convert IPLD nodes to context graph entities
    let mut new_context = ContextGraph::new("converted");
    
    // Get all IPLD nodes
    let ipld_nodes = ipld.get_all_nodes()?;
    
    // Map CIDs to UUIDs
    let mut cid_to_uuid = HashMap::new();
    
    // Convert nodes
    for node in ipld_nodes {
        let uuid = Uuid::new_v4();
        cid_to_uuid.insert(node.id().clone(), uuid);
        
        // Create entity in context graph
        new_context.add_aggregate("IpldNode", uuid, json!({
            "cid": node.id(),
            "codec": node.data().get("codec"),
            "size": node.data().get("size")
        }))?;
    }
    
    // Convert edges
    let ipld_edges = ipld.get_all_edges()?;
    for edge in ipld_edges {
        if let (Some(&from_uuid), Some(&to_uuid)) = (
            cid_to_uuid.get(edge.from()),
            cid_to_uuid.get(edge.to())
        ) {
            new_context.add_relationship(from_uuid, to_uuid, edge.label())?;
        }
    }
    
    // Verify conversion
    assert_eq!(new_context.node_count(), ipld.node_count());
    assert_eq!(new_context.edge_count(), ipld.edge_count());
    
    Ok(())
}

#[test]
fn test_composing_multiple_graphs() -> Result<()> {
    // Create individual graphs
    let ipld = create_test_ipld_graph()?;
    let context = create_test_context_graph()?;
    let workflow = create_test_workflow_graph()?;
    let concept = create_test_concept_graph()?;
    
    // Compose them
    let composed = ComposedGraph::builder()
        .add_graph("data", ipld)
        .add_graph("domain", context)
        .add_graph("workflow", workflow)
        .add_graph("concepts", concept)
        .build()?;
    
    // Verify composition
    assert_eq!(composed.graph_count(), 4);
    
    // Test cross-graph queries
    let data_nodes = composed.nodes_in_graph("data")?;
    assert_eq!(data_nodes.len(), 3); // Root + 2 children
    
    let domain_nodes = composed.nodes_in_graph("domain")?;
    assert!(domain_nodes.len() >= 3); // Customer, Order, OrderItem
    
    Ok(())
}

#[test]
fn test_graph_transformations() -> Result<()> {
    // Create a workflow graph
    let workflow = create_test_workflow_graph()?;
    
    // Transform to concept graph (states as concepts)
    let mut concept = ConceptGraph::new();
    
    // Map workflow states to concepts
    let states = workflow.get_states()?;
    let mut state_to_concept = HashMap::new();
    
    for state in states {
        let features = vec![
            ("workflow_state", 1.0),
            ("active", if state.name() == "draft" { 1.0 } else { 0.0 }),
            ("terminal", if state.name() == "approved" || state.name() == "rejected" { 1.0 } else { 0.0 }),
        ];
        
        let concept_id = concept.add_concept(state.name(), features)?;
        state_to_concept.insert(state.id(), concept_id);
    }
    
    // Transform transitions to concept relations
    let transitions = workflow.get_transitions()?;
    for transition in transitions {
        if let (Some(&from_concept), Some(&to_concept)) = (
            state_to_concept.get(&transition.from_state),
            state_to_concept.get(&transition.to_state)
        ) {
            concept.add_relation(from_concept, to_concept, &transition.action, 0.8)?;
        }
    }
    
    // Verify transformation
    assert_eq!(concept.node_count(), workflow.node_count());
    
    Ok(())
}

#[test]
fn test_bidirectional_graph_mapping() -> Result<()> {
    // Create source graphs
    let mut ipld = create_test_ipld_graph()?;
    let mut context = create_test_context_graph()?;
    
    // Create bidirectional mapping
    let mut ipld_to_context = HashMap::new();
    let mut context_to_ipld = HashMap::new();
    
    // Map IPLD CIDs to context entities
    let ipld_nodes = ipld.get_all_nodes()?;
    for node in ipld_nodes {
        let uuid = Uuid::new_v4();
        let entity_id = context.add_entity(
            "IpldReference",
            uuid,
            context.get_all_nodes()?.first().unwrap().id(), // Parent aggregate
            json!({
                "cid": node.id(),
                "metadata": node.data()
            })
        )?;
        
        ipld_to_context.insert(node.id().clone(), entity_id);
        context_to_ipld.insert(entity_id, node.id().clone());
    }
    
    // Test bidirectional lookup
    let root_cid = "QmRoot";
    if let Some(&context_id) = ipld_to_context.get(root_cid) {
        assert_eq!(context_to_ipld.get(&context_id), Some(&root_cid.to_string()));
    }
    
    Ok(())
}

#[test]
fn test_graph_merging() -> Result<()> {
    // Create two context graphs from same domain
    let mut graph1 = ContextGraph::new("sales");
    let mut graph2 = ContextGraph::new("sales");
    
    // Add data to graph1
    let customer1 = graph1.add_aggregate("Customer", Uuid::new_v4(), json!({
        "name": "Alice",
        "tier": "gold"
    }))?;
    
    let order1 = graph1.add_aggregate("Order", Uuid::new_v4(), json!({
        "total": 150.00
    }))?;
    
    graph1.add_relationship(customer1, order1, "placed")?;
    
    // Add data to graph2
    let customer2 = graph2.add_aggregate("Customer", Uuid::new_v4(), json!({
        "name": "Bob",
        "tier": "silver"
    }))?;
    
    let order2 = graph2.add_aggregate("Order", Uuid::new_v4(), json!({
        "total": 200.00
    }))?;
    
    graph2.add_relationship(customer2, order2, "placed")?;
    
    // Merge graphs using composed graph
    let merged = ComposedGraph::builder()
        .add_graph("batch1", graph1)
        .add_graph("batch2", graph2)
        .build()?;
    
    // Verify merge
    assert_eq!(merged.total_nodes(), 4); // 2 customers + 2 orders
    assert_eq!(merged.total_edges(), 2); // 2 relationships
    
    Ok(())
}

#[test]
fn test_graph_projection() -> Result<()> {
    // Create a rich context graph
    let mut context = ContextGraph::new("ecommerce");
    
    // Add complex domain model
    let customer = context.add_aggregate("Customer", Uuid::new_v4(), json!({
        "name": "Alice",
        "segment": "premium"
    }))?;
    
    let account = context.add_aggregate("Account", Uuid::new_v4(), json!({
        "balance": 1000.00,
        "currency": "USD"
    }))?;
    
    let order = context.add_aggregate("Order", Uuid::new_v4(), json!({
        "total": 250.00,
        "status": "processing"
    }))?;
    
    let product = context.add_aggregate("Product", Uuid::new_v4(), json!({
        "name": "Widget Pro",
        "price": 125.00
    }))?;
    
    // Add relationships
    context.add_relationship(customer, account, "owns")?;
    context.add_relationship(customer, order, "placed")?;
    context.add_relationship(order, product, "contains")?;
    
    // Project to workflow graph (order lifecycle)
    let mut workflow = WorkflowGraph::new("order-lifecycle");
    
    // Extract order states from context
    let orders = context.get_aggregates_by_type("Order")?;
    for order in orders {
        let status = order.data().get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown");
        
        workflow.add_state(status, json!({
            "order_id": order.id(),
            "total": order.data().get("total")
        }))?;
    }
    
    // Add possible transitions
    if workflow.has_state("processing")? {
        workflow.add_state("shipped", json!({}))?;
        workflow.add_transition(
            workflow.get_state_by_name("processing")?.id(),
            workflow.get_state_by_name("shipped")?.id(),
            "ship",
            json!({})
        )?;
    }
    
    assert!(workflow.node_count() > 0);
    
    Ok(())
}

#[test]
fn test_cross_graph_references() -> Result<()> {
    // Create graphs with cross-references
    let mut ipld = IpldGraph::new();
    let mut context = ContextGraph::new("storage");
    
    // Add IPLD data
    let data_cid = ipld.add_cid("QmData123", "dag-cbor", 2048)?;
    
    // Add context entity that references IPLD
    let storage_entity = context.add_aggregate("StorageObject", Uuid::new_v4(), json!({
        "name": "user-data.cbor",
        "ipld_cid": "QmData123",
        "size": 2048
    }))?;
    
    // Compose graphs
    let composed = ComposedGraph::builder()
        .add_graph("content", ipld)
        .add_graph("metadata", context)
        .build()?;
    
    // Verify cross-references can be resolved
    let storage_nodes = composed.nodes_in_graph("metadata")?;
    let storage_node = storage_nodes.first().unwrap();
    
    let referenced_cid = storage_node.data()
        .get("ipld_cid")
        .and_then(|v| v.as_str())
        .unwrap();
    
    assert_eq!(referenced_cid, "QmData123");
    
    // Find corresponding IPLD node
    let ipld_nodes = composed.nodes_in_graph("content")?;
    let ipld_node = ipld_nodes.iter()
        .find(|n| n.id() == "QmData123")
        .unwrap();
    
    assert_eq!(ipld_node.node_type(), "cid");
    
    Ok(())
}