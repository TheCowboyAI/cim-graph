//! Acceptance tests for US-006: Context Graph

use cim_graph::graphs::context::{ContextGraph, DomainObjectType, RelationshipType};

#[test]
fn test_ac_006_1_model_bounded_contexts() {
    // Given: a context graph
    let mut graph = ContextGraph::new();
    
    // When: I model bounded contexts
    let sales_id = graph.add_bounded_context("sales_ctx", "Sales Context").unwrap();
    let inventory_id = graph.add_bounded_context("inventory_ctx", "Inventory Context").unwrap();
    
    // Then: contexts are created
    assert_eq!(graph.graph().node_count(), 2);
    
    // And: I can establish context relationships
    let rel_id = graph.add_relationship(
        &sales_id,
        &inventory_id,
        RelationshipType::CustomerSupplier
    ).unwrap();
    
    assert!(!rel_id.is_empty());
}

#[test]
fn test_ac_006_2_model_aggregates() {
    // Given: a context graph with a bounded context
    let mut graph = ContextGraph::new();
    graph.add_bounded_context("sales", "Sales Context").unwrap();
    
    // When: I add aggregates to the context
    let order_id = graph.add_aggregate("order_agg", "Order", "sales").unwrap();
    let customer_id = graph.add_aggregate("customer_agg", "Customer", "sales").unwrap();
    
    // Then: aggregates are in the context
    let sales_objects = graph.get_context_objects("sales");
    assert_eq!(sales_objects.len(), 2);
    
    let aggregates = graph.get_aggregates("sales");
    assert_eq!(aggregates.len(), 2);
}

#[test]
fn test_ac_006_3_aggregate_composition() {
    // Given: a context graph with an aggregate
    let mut graph = ContextGraph::new();
    graph.add_bounded_context("sales", "Sales").unwrap();
    graph.add_aggregate("order", "Order", "sales").unwrap();
    
    // When: I add entities to the aggregate
    let order_item = graph.add_entity("order_item", "OrderItem", "order").unwrap();
    let shipping = graph.add_entity("shipping_info", "ShippingInfo", "order").unwrap();
    
    // Then: entities are contained in the aggregate
    assert_eq!(graph.graph().node_count(), 4); // context + aggregate + 2 entities
    assert_eq!(graph.graph().edge_count(), 2); // 2 contains relationships
    
    // And: entities inherit the bounded context
    let item_node = graph.graph().get_node(&order_item).unwrap();
    assert_eq!(item_node.bounded_context(), Some("sales"));
}

#[test]
fn test_ac_006_4_cross_context_validation() {
    // Given: two bounded contexts with aggregates
    let mut graph = ContextGraph::new();
    
    graph.add_bounded_context("sales", "Sales").unwrap();
    graph.add_bounded_context("shipping", "Shipping").unwrap();
    
    let order = graph.add_aggregate("order", "Order", "sales").unwrap();
    let shipment = graph.add_aggregate("shipment", "Shipment", "shipping").unwrap();
    
    // When: I create a direct reference across contexts
    graph.add_relationship(&order, &shipment, RelationshipType::References).unwrap();
    
    // Then: boundary validation detects the violation
    let violations = graph.validate_boundaries();
    assert_eq!(violations.len(), 1);
    assert!(violations[0].contains("Cross-context reference"));
    
    // But: approved cross-context relationships are allowed
    graph.add_relationship(
        "sales",
        "shipping",
        RelationshipType::CustomerSupplier
    ).unwrap();
    
    // No new violations for the approved relationship type
    let violations = graph.validate_boundaries();
    assert_eq!(violations.len(), 1); // Still just the direct reference
}

#[test]
fn test_domain_event_relationships() {
    // Given: a context with aggregates
    let mut graph = ContextGraph::new();
    
    graph.add_bounded_context("orders", "Orders").unwrap();
    let order = graph.add_aggregate("order", "Order", "orders").unwrap();
    
    // When: I model domain events
    let event_node = cim_graph::graphs::context::ContextNode::new(
        "order_placed",
        "OrderPlaced",
        DomainObjectType::DomainEvent
    ).with_context("orders");
    
    let event_id = "order_placed";
    graph.graph_mut().add_node(event_node).unwrap();
    
    // Then: I can show which aggregate emits the event
    graph.add_relationship(&order, event_id, RelationshipType::EmittedBy).unwrap();
    
    assert_eq!(graph.graph().edge_count(), 1);
}