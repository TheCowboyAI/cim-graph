//! Tests for real-world workflow scenarios

use cim_graph::graphs::{ComposedGraph, IpldGraph, ContextGraph, WorkflowGraph, ConceptGraph};
use cim_graph::{Graph, Result, GraphError};
use serde_json::json;
use uuid::Uuid;
use std::collections::{HashMap, HashSet};

#[test]
fn test_building_knowledge_graph_from_ipld() -> Result<()> {
    // Scenario: Building a knowledge graph from IPLD-stored documents
    let mut ipld = IpldGraph::new();
    let mut knowledge = ConceptGraph::new();
    
    // Add document CIDs
    let doc1 = ipld.add_cid("QmDoc1", "dag-cbor", 4096)?;
    let doc2 = ipld.add_cid("QmDoc2", "dag-cbor", 3072)?;
    let doc3 = ipld.add_cid("QmDoc3", "dag-cbor", 2048)?;
    
    // Add metadata CIDs
    let meta1 = ipld.add_cid("QmMeta1", "dag-json", 512)?;
    let meta2 = ipld.add_cid("QmMeta2", "dag-json", 512)?;
    let meta3 = ipld.add_cid("QmMeta3", "dag-json", 512)?;
    
    // Link documents to metadata
    ipld.add_link(doc1, meta1, "metadata")?;
    ipld.add_link(doc2, meta2, "metadata")?;
    ipld.add_link(doc3, meta3, "metadata")?;
    
    // Cross-reference documents
    ipld.add_link(doc1, doc2, "references")?;
    ipld.add_link(doc2, doc3, "references")?;
    
    // Simulate extracting concepts from documents
    let doc_concepts = vec![
        ("QmDoc1", vec!["machine-learning", "neural-networks", "AI"]),
        ("QmDoc2", vec!["deep-learning", "neural-networks", "computer-vision"]),
        ("QmDoc3", vec!["computer-vision", "image-processing", "AI"]),
    ];
    
    // Build knowledge graph
    let mut concept_ids = HashMap::new();
    
    for (doc_cid, concepts) in doc_concepts {
        for concept_name in concepts {
            // Add concept if not exists
            if !concept_ids.contains_key(concept_name) {
                let features = match concept_name {
                    "machine-learning" => vec![("statistical", 0.8), ("computational", 0.9), ("predictive", 1.0)],
                    "neural-networks" => vec![("computational", 1.0), ("layered", 0.9), ("trainable", 1.0)],
                    "deep-learning" => vec![("neural-networks", 1.0), ("multi-layer", 1.0), ("computational", 0.9)],
                    "computer-vision" => vec![("visual", 1.0), ("computational", 0.8), ("pattern-recognition", 0.9)],
                    "image-processing" => vec![("visual", 1.0), ("transformative", 0.8), ("algorithmic", 0.9)],
                    "AI" => vec![("intelligent", 1.0), ("computational", 0.8), ("adaptive", 0.9)],
                    _ => vec![("general", 0.5)],
                };
                
                let concept_id = knowledge.add_concept(concept_name, features)?;
                concept_ids.insert(concept_name, concept_id);
            }
        }
    }
    
    // Add concept relationships based on co-occurrence
    knowledge.add_relation(
        concept_ids["machine-learning"],
        concept_ids["AI"],
        "subset-of",
        0.9
    )?;
    
    knowledge.add_relation(
        concept_ids["deep-learning"],
        concept_ids["machine-learning"],
        "subset-of",
        0.95
    )?;
    
    knowledge.add_relation(
        concept_ids["neural-networks"],
        concept_ids["deep-learning"],
        "used-by",
        1.0
    )?;
    
    knowledge.add_relation(
        concept_ids["computer-vision"],
        concept_ids["AI"],
        "application-of",
        0.85
    )?;
    
    // Compose graphs for unified view
    let unified = ComposedGraph::builder()
        .add_graph("documents", ipld)
        .add_graph("knowledge", knowledge)
        .build()?;
    
    // Verify knowledge extraction
    assert_eq!(unified.nodes_in_graph("knowledge")?.len(), 6); // 6 concepts
    assert!(unified.nodes_in_graph("documents")?.len() >= 6); // docs + metadata
    
    Ok(())
}

#[test]
fn test_workflow_pipeline_creation() -> Result<()> {
    // Scenario: Multi-stage data processing pipeline
    let mut workflow = WorkflowGraph::new("data-pipeline");
    let mut context = ContextGraph::new("pipeline-context");
    
    // Define pipeline stages
    let stages = vec![
        ("ingestion", "Data ingestion from sources"),
        ("validation", "Data validation and cleansing"),
        ("transformation", "Data transformation and enrichment"),
        ("analysis", "Data analysis and insights"),
        ("storage", "Data storage and indexing"),
        ("notification", "Notification and reporting"),
    ];
    
    // Create workflow states
    let mut state_ids = Vec::new();
    for (name, description) in &stages {
        let state_id = workflow.add_state(name, json!({
            "description": description,
            "retry_count": 3,
            "timeout_seconds": 300
        }))?;
        state_ids.push(state_id);
    }
    
    // Add error states
    let error_state = workflow.add_state("error", json!({
        "description": "Error handling state",
        "alert": true
    }))?;
    
    let retry_state = workflow.add_state("retry", json!({
        "description": "Retry failed operations",
        "max_retries": 3
    }))?;
    
    // Connect pipeline stages
    for i in 0..state_ids.len()-1 {
        workflow.add_transition(
            state_ids[i],
            state_ids[i+1],
            "success",
            json!({ "auto": true })
        )?;
        
        // Add error transitions
        workflow.add_transition(
            state_ids[i],
            error_state,
            "error",
            json!({ "capture_context": true })
        )?;
    }
    
    // Add retry logic
    workflow.add_transition(
        error_state,
        retry_state,
        "retry",
        json!({ "backoff": "exponential" })
    )?;
    
    // Retry can go back to any stage
    for (i, state_id) in state_ids.iter().enumerate() {
        workflow.add_transition(
            retry_state,
            *state_id,
            &format!("retry-stage-{}", i),
            json!({ "condition": format!("failed_stage == {}", i) })
        )?;
    }
    
    // Add pipeline execution context
    let pipeline_run = context.add_aggregate("PipelineRun", Uuid::new_v4(), json!({
        "run_id": Uuid::new_v4().to_string(),
        "started_at": "2024-01-15T10:00:00Z",
        "status": "running"
    }))?;
    
    // Add stage execution entities
    for (name, _) in &stages {
        context.add_entity(
            "StageExecution",
            Uuid::new_v4(),
            pipeline_run,
            json!({
                "stage": name,
                "status": "pending",
                "attempts": 0
            })
        )?;
    }
    
    // Compose for complete pipeline view
    let pipeline = ComposedGraph::builder()
        .add_graph("workflow", workflow)
        .add_graph("executions", context)
        .build()?;
    
    // Verify pipeline structure
    assert!(pipeline.nodes_in_graph("workflow")?.len() >= 8); // stages + error states
    assert!(pipeline.nodes_in_graph("executions")?.len() >= 7); // run + stage executions
    
    Ok(())
}

#[test]
fn test_domain_modeling_with_context_graphs() -> Result<()> {
    // Scenario: E-commerce domain model with multiple bounded contexts
    let mut catalog_ctx = ContextGraph::new("catalog");
    let mut order_ctx = ContextGraph::new("order-management");
    let mut inventory_ctx = ContextGraph::new("inventory");
    let mut customer_ctx = ContextGraph::new("customer");
    
    // Catalog context
    let category = catalog_ctx.add_aggregate("Category", Uuid::new_v4(), json!({
        "name": "Electronics",
        "slug": "electronics"
    }))?;
    
    let product = catalog_ctx.add_aggregate("Product", Uuid::new_v4(), json!({
        "name": "Laptop Pro X",
        "sku": "LPX-001",
        "price": 1299.99
    }))?;
    
    catalog_ctx.add_relationship(product, category, "belongs-to")?;
    
    // Customer context
    let customer = customer_ctx.add_aggregate("Customer", Uuid::new_v4(), json!({
        "email": "john@example.com",
        "tier": "premium"
    }))?;
    
    let address = customer_ctx.add_entity("Address", Uuid::new_v4(), customer, json!({
        "type": "shipping",
        "street": "123 Main St",
        "city": "Seattle"
    }))?;
    
    // Order context
    let order = order_ctx.add_aggregate("Order", Uuid::new_v4(), json!({
        "order_number": "ORD-2024-001",
        "customer_id": customer.to_string(),
        "total": 1299.99,
        "status": "pending"
    }))?;
    
    let order_item = order_ctx.add_entity("OrderItem", Uuid::new_v4(), order, json!({
        "product_sku": "LPX-001",
        "quantity": 1,
        "price": 1299.99
    }))?;
    
    // Inventory context
    let stock = inventory_ctx.add_aggregate("StockItem", Uuid::new_v4(), json!({
        "sku": "LPX-001",
        "quantity_on_hand": 50,
        "reserved": 1,
        "available": 49
    }))?;
    
    let reservation = inventory_ctx.add_entity("Reservation", Uuid::new_v4(), stock, json!({
        "order_id": order.to_string(),
        "quantity": 1,
        "expires_at": "2024-01-16T10:00:00Z"
    }))?;
    
    // Create domain model
    let domain_model = ComposedGraph::builder()
        .add_graph("catalog", catalog_ctx)
        .add_graph("orders", order_ctx)
        .add_graph("inventory", inventory_ctx)
        .add_graph("customers", customer_ctx)
        .build()?;
    
    // Verify bounded contexts
    assert_eq!(domain_model.graph_count(), 4);
    
    // Simulate cross-context query: Find all orders for products in a category
    let catalog_products = domain_model.nodes_in_graph("catalog")?
        .into_iter()
        .filter(|n| n.node_type() == "Product")
        .collect::<Vec<_>>();
    
    let order_items = domain_model.nodes_in_graph("orders")?
        .into_iter()
        .filter(|n| n.node_type() == "OrderItem")
        .collect::<Vec<_>>();
    
    // Cross-reference by SKU
    let mut matching_items = 0;
    for item in order_items {
        if let Some(sku) = item.data().get("product_sku").and_then(|v| v.as_str()) {
            if catalog_products.iter().any(|p| {
                p.data().get("sku").and_then(|v| v.as_str()) == Some(sku)
            }) {
                matching_items += 1;
            }
        }
    }
    
    assert_eq!(matching_items, 1);
    
    Ok(())
}

#[test]
fn test_event_sourcing_workflow() -> Result<()> {
    // Scenario: Event-sourced order processing
    let mut events = IpldGraph::new();
    let mut workflow = WorkflowGraph::new("order-workflow");
    let mut context = ContextGraph::new("order-aggregate");
    
    // Create event stream
    let event_cids = vec![
        ("QmOrderCreated", "OrderCreated", json!({
            "order_id": "123",
            "customer": "alice@example.com",
            "items": [{"sku": "ABC", "qty": 2}],
            "timestamp": "2024-01-15T10:00:00Z"
        })),
        ("QmPaymentReceived", "PaymentReceived", json!({
            "order_id": "123",
            "amount": 99.99,
            "timestamp": "2024-01-15T10:05:00Z"
        })),
        ("QmOrderShipped", "OrderShipped", json!({
            "order_id": "123",
            "tracking": "1Z999AA1234567890",
            "timestamp": "2024-01-15T14:00:00Z"
        })),
    ];
    
    // Add events to IPLD
    let mut prev_cid = None;
    for (cid, event_type, data) in event_cids {
        let event_id = events.add_cid(cid, "dag-cbor", 256)?;
        
        // Chain events
        if let Some(prev) = prev_cid {
            events.add_link(event_id, prev, "previous")?;
        }
        prev_cid = Some(event_id);
    }
    
    // Build workflow from event types
    let created = workflow.add_state("created", json!({ "initial": true }))?;
    let paid = workflow.add_state("paid", json!({}))?;
    let shipped = workflow.add_state("shipped", json!({}))?;
    let delivered = workflow.add_state("delivered", json!({ "final": true }))?;
    
    workflow.add_transition(created, paid, "payment-received", json!({}))?;
    workflow.add_transition(paid, shipped, "order-shipped", json!({}))?;
    workflow.add_transition(shipped, delivered, "order-delivered", json!({}))?;
    
    // Build current aggregate state
    let order_aggregate = context.add_aggregate("Order", Uuid::new_v4(), json!({
        "order_id": "123",
        "status": "shipped",
        "customer": "alice@example.com",
        "total": 99.99,
        "tracking": "1Z999AA1234567890"
    }))?;
    
    // Compose for complete view
    let event_sourced = ComposedGraph::builder()
        .add_graph("events", events)
        .add_graph("workflow", workflow)
        .add_graph("aggregate", context)
        .build()?;
    
    // Verify event sourcing structure
    assert_eq!(event_sourced.nodes_in_graph("events")?.len(), 3);
    assert_eq!(event_sourced.nodes_in_graph("workflow")?.len(), 4);
    assert_eq!(event_sourced.nodes_in_graph("aggregate")?.len(), 1);
    
    Ok(())
}

#[test]
fn test_recommendation_system() -> Result<()> {
    // Scenario: Content recommendation using concept graphs
    let mut content = ContextGraph::new("content");
    let mut user_prefs = ConceptGraph::new();
    let mut content_features = ConceptGraph::new();
    
    // Add content items
    let article1 = content.add_aggregate("Article", Uuid::new_v4(), json!({
        "title": "Introduction to Machine Learning",
        "category": "tech",
        "tags": ["ML", "AI", "beginner"]
    }))?;
    
    let article2 = content.add_aggregate("Article", Uuid::new_v4(), json!({
        "title": "Deep Learning with PyTorch",
        "category": "tech",
        "tags": ["ML", "deep-learning", "pytorch", "advanced"]
    }))?;
    
    let article3 = content.add_aggregate("Article", Uuid::new_v4(), json!({
        "title": "Web Development Best Practices",
        "category": "tech",
        "tags": ["web", "javascript", "frontend"]
    }))?;
    
    // Add user preferences as concepts
    let user_ml = user_prefs.add_concept("user-likes-ml", vec![
        ("machine-learning", 0.9),
        ("artificial-intelligence", 0.8),
        ("data-science", 0.7)
    ])?;
    
    let user_beginner = user_prefs.add_concept("user-level-beginner", vec![
        ("beginner-friendly", 1.0),
        ("tutorials", 0.8),
        ("step-by-step", 0.9)
    ])?;
    
    // Add content features as concepts
    let ml_content = content_features.add_concept("ml-content", vec![
        ("machine-learning", 1.0),
        ("technical", 0.8),
        ("mathematical", 0.7)
    ])?;
    
    let beginner_content = content_features.add_concept("beginner-content", vec![
        ("beginner-friendly", 1.0),
        ("introductory", 0.9),
        ("foundational", 0.8)
    ])?;
    
    let advanced_content = content_features.add_concept("advanced-content", vec![
        ("advanced", 1.0),
        ("complex", 0.9),
        ("specialized", 0.8)
    ])?;
    
    // Add relationships
    content_features.add_relation(ml_content, beginner_content, "can-be", 0.7)?;
    content_features.add_relation(ml_content, advanced_content, "can-be", 0.8)?;
    
    // Compose for recommendation engine
    let recommender = ComposedGraph::builder()
        .add_graph("content", content)
        .add_graph("user_preferences", user_prefs)
        .add_graph("content_features", content_features)
        .build()?;
    
    // Simulate recommendation scoring
    let content_items = recommender.nodes_in_graph("content")?;
    let mut recommendations = Vec::new();
    
    for item in content_items {
        if let Some(tags) = item.data().get("tags").and_then(|v| v.as_array()) {
            let mut score = 0.0;
            
            // Score based on tag matching
            for tag in tags {
                if let Some(tag_str) = tag.as_str() {
                    match tag_str {
                        "ML" | "AI" => score += 0.9,
                        "beginner" => score += 0.8,
                        "deep-learning" => score += 0.3, // Lower for beginner
                        "advanced" => score -= 0.5, // Penalty for beginner
                        _ => {}
                    }
                }
            }
            
            recommendations.push((item.id(), score));
        }
    }
    
    // Sort by score
    recommendations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    // Best recommendation should be the intro ML article
    assert!(recommendations.first().unwrap().1 > 1.0);
    
    Ok(())
}