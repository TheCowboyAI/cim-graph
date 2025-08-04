//! Example: Using ContextGraph for Domain-Driven Design
//! 
//! This example demonstrates how to model a e-commerce system using
//! bounded contexts, aggregates, and domain events.

use cim_graph::graphs::ContextGraph;
use cim_graph::graphs::context::{ContextNode, DomainObjectType, RelationshipType};
use serde_json::json;

fn main() {
    println!("=== Context Graph Example: E-Commerce System ===\n");
    
    // Create a new context graph
    let mut graph = ContextGraph::new();
    
    // Define bounded contexts
    println!("Defining bounded contexts...");
    graph.add_bounded_context("sales", "Sales").expect("Failed to add sales context");
    graph.add_bounded_context("inventory", "Inventory").expect("Failed to add inventory context");
    graph.add_bounded_context("shipping", "Shipping").expect("Failed to add shipping context");
    graph.add_bounded_context("customer", "Customer Management").expect("Failed to add customer context");
    
    // Define context relationships
    graph.add_relationship("sales", "inventory", RelationshipType::CustomerSupplier)
        .expect("Failed to add relationship");
    graph.add_relationship("sales", "shipping", RelationshipType::CustomerSupplier)
        .expect("Failed to add relationship");
    graph.add_relationship("customer", "sales", RelationshipType::SharedKernel)
        .expect("Failed to add relationship");
    
    // Add aggregates to Sales context
    println!("\nDefining Sales aggregates...");
    graph.add_aggregate("order", "Order", "sales").expect("Failed to add order aggregate");
    graph.add_aggregate("cart", "ShoppingCart", "sales").expect("Failed to add cart aggregate");
    
    // Add entities to Order aggregate
    println!("\nAdding entities to Order aggregate...");
    graph.add_entity("order_item", "OrderItem", "order").expect("Failed to add order item");
    graph.add_entity("payment", "Payment", "order").expect("Failed to add payment");
    
    // Add value objects
    let mut address = ContextNode::new("shipping_address", "ShippingAddress", DomainObjectType::ValueObject);
    address = address.with_context("sales");
    address = address.with_properties(json!({
        "street": "String",
        "city": "String",
        "country": "String",
        "postal_code": "String"
    }));
    graph.graph_mut().add_node(address).expect("Failed to add address");
    
    // Add domain events
    println!("\nDefining domain events...");
    let events = vec![
        ("order_placed", "OrderPlaced", "order"),
        ("order_cancelled", "OrderCancelled", "order"),
        ("payment_processed", "PaymentProcessed", "order"),
        ("item_shipped", "ItemShipped", "order"),
    ];
    
    for (id, name, aggregate) in events {
        let event = ContextNode::new(id, name, DomainObjectType::DomainEvent)
            .with_context("sales");
        graph.graph_mut().add_node(event).expect("Failed to add event");
        graph.add_relationship(aggregate, id, RelationshipType::EmittedBy)
            .expect("Failed to add event relationship");
    }
    
    // Add aggregates to other contexts
    println!("\nDefining other context aggregates...");
    graph.add_aggregate("product", "Product", "inventory").expect("Failed to add product");
    graph.add_aggregate("stock", "Stock", "inventory").expect("Failed to add stock");
    graph.add_aggregate("shipment", "Shipment", "shipping").expect("Failed to add shipment");
    graph.add_aggregate("customer_profile", "CustomerProfile", "customer").expect("Failed to add customer");
    
    // Add some cross-context references (through ACL)
    graph.add_relationship("sales", "inventory", RelationshipType::AntiCorruptionLayer)
        .expect("Failed to add ACL");
    
    // Display the contexts
    println!("\n\n=== Bounded Contexts ===");
    for context_id in ["sales", "inventory", "shipping", "customer"] {
        println!("\n📦 {} Context:", context_id.to_uppercase());
        
        let aggregates = graph.get_aggregates(context_id);
        for agg in aggregates {
            println!("  🔷 Aggregate: {}", agg.name());
        }
        
        let all_objects = graph.get_context_objects(context_id);
        let events: Vec<_> = all_objects.iter()
            .filter(|obj| obj.object_type() == DomainObjectType::DomainEvent)
            .collect();
        
        if !events.is_empty() {
            println!("  📨 Domain Events:");
            for event in events {
                println!("    - {}", event.name());
            }
        }
    }
    
    // Validate boundaries
    println!("\n\n=== Boundary Validation ===");
    
    // Try to add an invalid cross-context reference
    println!("Adding direct reference from Order to Product (cross-context)...");
    graph.add_relationship("order", "product", RelationshipType::References)
        .expect("Added invalid reference");
    
    let violations = graph.validate_boundaries();
    if violations.is_empty() {
        println!("✅ No boundary violations detected");
    } else {
        println!("❌ Boundary violations found:");
        for violation in violations {
            println!("  - {}", violation);
        }
    }
    
    // Statistics
    println!("\n\n=== Graph Statistics ===");
    println!("Total nodes: {}", graph.graph().node_count());
    println!("Total relationships: {}", graph.graph().edge_count());
    println!("Bounded contexts: 4");
    
    // Example of how to add business rules
    println!("\n\n=== Business Rules Example ===");
    if let Some(order) = graph.graph_mut().get_node_mut("order") {
        order.add_invariant("Order total must be greater than 0");
        order.add_invariant("Order must have at least one item");
        order.add_invariant("Order cannot be modified after shipment");
        
        println!("Order aggregate invariants:");
        println!("  - Order total must be greater than 0");
        println!("  - Order must have at least one item");
        println!("  - Order cannot be modified after shipment");
    }
}