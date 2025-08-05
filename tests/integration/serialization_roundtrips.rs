//! Tests for serialization and deserialization round-trips

use cim_graph::graphs::{ComposedGraph, IpldGraph, ContextGraph, WorkflowGraph, ConceptGraph};
use cim_graph::graphs::context::RelationshipType;
use cim_graph::graphs::workflow::{WorkflowNode, StateType};
use cim_graph::graphs::concept::SemanticRelation;
use cim_graph::{Graph, Result};
use serde_json::{json, Value};
use uuid::Uuid;
use std::str::FromStr;

#[test]
fn test_ipld_graph_roundtrip() -> Result<()> {
    // Create complex IPLD graph
    let mut original = IpldGraph::new();
    
    // Add various CID types
    let root = original.add_content(serde_json::json!({ "cid": "QmRoot", "format": "dag-cbor", "size": 4096 }))?;
    let json_node = original.add_content(serde_json::json!({ "cid": "QmJson", "format": "dag-json", "size": 2048 }))?;
    let raw_node = original.add_content(serde_json::json!({ "cid": "QmRaw", "format": "raw", "size": 1024 }))?;
    let pb_node = original.add_content(serde_json::json!({ "cid": "QmPb", "format": "dag-pb", "size": 512 }))?;
    
    // Create complex link structure
    original.add_link(&root, &json_node, "data")?;
    original.add_link(&root, &raw_node, "binary")?;
    original.add_link(&json_node, &pb_node, "metadata")?;
    original.add_link(&raw_node, &pb_node, "related")?;
    
    // Serialize to JSON
    let serialized = serde_json::to_string_pretty(&original)?;
    
    // Deserialize back
    let deserialized: IpldGraph = serde_json::from_str(&serialized)?;
    
    // Verify structure preserved
    assert_eq!(deserialized.node_count(), original.node_count());
    assert_eq!(deserialized.edge_count(), original.edge_count());
    
    // Verify nodes
    for node_id in ["QmRoot", "QmJson", "QmRaw", "QmPb"] {
        let orig_node = original.get_node_by_id(node_id)?;
        let deser_node = deserialized.get_node_by_id(node_id)?;
        
        assert_eq!(orig_node.id(), deser_node.id());
        assert_eq!(orig_node.node_type(), deser_node.node_type());
        assert_eq!(orig_node.data(), deser_node.data());
    }
    
    // Verify edges preserved
    let orig_edges = original.get_all_edges()?;
    let deser_edges = deserialized.get_all_edges()?;
    
    assert_eq!(orig_edges.len(), deser_edges.len());
    
    Ok(())
}

#[test]
fn test_context_graph_roundtrip() -> Result<()> {
    // Create rich domain model
    let mut original = ContextGraph::new();
    
    // Add bounded context
    original.add_bounded_context("crm", "Customer Relationship Management")?;
    
    // Add aggregates with complex data
    let customer_id = Uuid::new_v4().to_string();
    let customer = original.add_aggregate(&customer_id, "Customer_JohnDoe", "crm")?;
    
    let address_id = Uuid::new_v4().to_string();
    let address = original.add_entity(&address_id, "Address_Shipping", &customer)?;
    
    let order_id = Uuid::new_v4().to_string();
    let order = original.add_aggregate(&order_id, "Order_2024_0001", "crm")?;
    
    // Add relationships
    original.add_relationship(customer, order, RelationshipType::References)?;
    original.add_relationship(order, address, RelationshipType::References)?;
    
    // Serialize
    let serialized = serde_json::to_string_pretty(&original)?;
    
    // Deserialize
    let deserialized: ContextGraph = serde_json::from_str(&serialized)?;
    
    // Verify domain context preserved
    assert_eq!(deserialized.context_name(), original.context_name());
    assert_eq!(deserialized.node_count(), original.node_count());
    assert_eq!(deserialized.edge_count(), original.edge_count());
    
    // Verify aggregate data integrity
    let orig_aggregates = original.get_aggregates_by_type("Customer")?;
    let deser_aggregates = deserialized.get_aggregates_by_type("Customer")?;
    
    assert_eq!(orig_aggregates.len(), deser_aggregates.len());
    
    // Deep verify JSON data
    let orig_data = orig_aggregates[0].data();
    let deser_data = deser_aggregates[0].data();
    
    assert_eq!(
        orig_data.get("preferences").unwrap().get("newsletter"),
        deser_data.get("preferences").unwrap().get("newsletter")
    );
    
    Ok(())
}

#[test]
fn test_workflow_graph_roundtrip() -> Result<()> {
    // Create complex state machine
    let mut original = WorkflowGraph::new();
    
    // Add states with metadata
    let created_node = WorkflowNode::new("created", "created", StateType::Initial);
    let created = original.add_state(created_node)?;
    
    let validated_node = WorkflowNode::new("validated", "validated", StateType::Normal);
    let validated = original.add_state(validated_node)?;
    
    let picked_node = WorkflowNode::new("picked", "picked", StateType::Normal);
    let picked = original.add_state(picked_node)?;
    
    let packed_node = WorkflowNode::new("packed", "packed", StateType::Normal);
    let packed = original.add_state(packed_node)?;
    
    let shipped_node = WorkflowNode::new("shipped", "shipped", StateType::Final);
    let shipped = original.add_state(shipped_node)?;
    
    // Add complex transitions
    original.add_transition("created", "validated", "validate")?;
    
    original.add_transition("validated", "picked", "pick")?;
    
    original.add_transition("picked", "packed", "pack")?;
    
    original.add_transition("packed", "shipped", "ship")?;
    
    // Add error transitions
    let error_node = WorkflowNode::new("error", "error", StateType::Normal);
    let error = original.add_state(error_node)?;
    
    for state_name in ["validated", "picked", "packed"] {
        original.add_transition(state_name, "error", "error")?;
    }
    
    // Serialize
    let serialized = serde_json::to_string_pretty(&original)?;
    
    // Deserialize
    let deserialized: WorkflowGraph = serde_json::from_str(&serialized)?;
    
    // Verify workflow preserved
    assert_eq!(deserialized.workflow_name(), original.workflow_name());
    assert_eq!(deserialized.node_count(), original.node_count());
    assert_eq!(deserialized.edge_count(), original.edge_count());
    
    // Verify state metadata
    let orig_state = original.get_state_by_name("validated")?;
    let deser_state = deserialized.get_state_by_name("validated")?;
    
    assert_eq!(orig_state.data(), deser_state.data());
    
    // Verify transition metadata
    let orig_transitions = original.get_transitions()?;
    let deser_transitions = deserialized.get_transitions()?;
    
    assert_eq!(orig_transitions.len(), deser_transitions.len());
    
    Ok(())
}

#[test]
fn test_concept_graph_roundtrip() -> Result<()> {
    // Create semantic network
    let mut original = ConceptGraph::new();
    
    // Add concepts with features
    let animal = original.add_concept("Animal", "Animal", serde_json::json!({
        "living": 1.0,
        "mobile": 0.9,
        "organic": 1.0,
        "sentient": 0.8
    }))?;
    
    let mammal = original.add_concept("Mammal", "Mammal", serde_json::json!({
        "living": 1.0,
        "mobile": 0.95,
        "organic": 1.0,
        "sentient": 0.9,
        "warm-blooded": 1.0,
        "hair": 0.9
    }))?;
    
    let dog = original.add_concept("Dog", "Dog", serde_json::json!({
        "living": 1.0,
        "mobile": 1.0,
        "organic": 1.0,
        "sentient": 0.95,
        "warm-blooded": 1.0,
        "hair": 1.0,
        "barks": 0.9,
        "loyal": 0.8
    }))?;
    
    let cat = original.add_concept("Cat", "Cat", serde_json::json!({
        "living": 1.0,
        "mobile": 1.0,
        "organic": 1.0,
        "sentient": 0.95,
        "warm-blooded": 1.0,
        "hair": 1.0,
        "meows": 0.9,
        "independent": 0.9
    }))?;
    
    // Add semantic relations
    original.add_relation("Mammal", "Animal", SemanticRelation::SubClassOf)?;
    original.add_relation("Dog", "Mammal", SemanticRelation::SubClassOf)?;
    original.add_relation("Cat", "Mammal", SemanticRelation::SubClassOf)?;
    original.add_relation("Dog", "Cat", SemanticRelation::Custom)?;
    
    // Serialize
    let serialized = serde_json::to_string_pretty(&original)?;
    
    // Deserialize
    let deserialized: ConceptGraph = serde_json::from_str(&serialized)?;
    
    // Verify concepts preserved
    assert_eq!(deserialized.node_count(), original.node_count());
    assert_eq!(deserialized.edge_count(), original.edge_count());
    
    // Verify concept features
    let orig_dog = original.get_concept_by_name("Dog")?;
    let deser_dog = deserialized.get_concept_by_name("Dog")?;
    
    assert_eq!(orig_dog.features().len(), deser_dog.features().len());
    
    // Verify feature values preserved with precision
    for (feature, value) in orig_dog.features() {
        let deser_value = deser_dog.features().iter()
            .find(|(f, _)| f == feature)
            .map(|(_, v)| v)
            .unwrap();
        
        assert!((value - deser_value).abs() < f64::EPSILON);
    }
    
    // Verify similarity calculations preserved
    let orig_sim = original.calculate_similarity(dog, cat)?;
    let deser_sim = deserialized.calculate_similarity(dog, cat)?;
    
    assert!((orig_sim - deser_sim).abs() < 0.001);
    
    Ok(())
}

#[test]
fn test_composed_graph_roundtrip() -> Result<()> {
    // Create complex composed graph
    let mut ipld = IpldGraph::new();
    let cid1 = ipld.add_content(serde_json::json!({ "cid": "QmData1", "format": "dag-cbor", "size": 1024 }))?;
    let cid2 = ipld.add_content(serde_json::json!({ "cid": "QmData2", "format": "dag-cbor", "size": 2048 }))?;
    ipld.add_link(&cid1, &cid2, "next")?;
    
    let mut context = ContextGraph::new();
    context.add_bounded_context("sales", "Sales Context")?;
    
    let customer_id = Uuid::new_v4().to_string();
    let customer = context.add_aggregate(&customer_id, "Customer_Acme", "sales")?;
    
    let mut workflow = WorkflowGraph::new();
    let pending_node = WorkflowNode::new("pending", "pending", StateType::Initial);
    let pending = workflow.add_state(pending_node)?;
    let approved_node = WorkflowNode::new("approved", "approved", StateType::Final);
    let approved = workflow.add_state(approved_node)?;
    workflow.add_transition("pending", "approved", "approve")?;
    
    let mut concepts = ConceptGraph::new();
    let business = concepts.add_concept("Business", "Business", serde_json::json!({
        "commercial": 1.0,
        "profitable": 0.8
    }))?;
    
    // Create composed graph
    let original = ComposedGraph::builder()
        .add_graph("data", ipld)
        .add_graph("domain", context)
        .add_graph("process", workflow)
        .add_graph("knowledge", concepts)
        .build()?;
    
    // Serialize
    let serialized = serde_json::to_string_pretty(&original)?;
    
    // Deserialize
    let deserialized: ComposedGraph = serde_json::from_str(&serialized)?;
    
    // Verify composition preserved
    assert_eq!(deserialized.graph_count(), original.graph_count());
    assert_eq!(deserialized.total_nodes(), original.total_nodes());
    assert_eq!(deserialized.total_edges(), original.total_edges());
    
    // Verify each sub-graph
    for name in ["data", "domain", "process", "knowledge"] {
        assert_eq!(
            deserialized.nodes_in_graph(name)?.len(),
            original.nodes_in_graph(name)?.len()
        );
    }
    
    Ok(())
}

#[test]
fn test_special_value_serialization() -> Result<()> {
    let mut context = ContextGraph::new();
    
    // Test various JSON value types
    let special_data = json!({
        "null_value": null,
        "bool_true": true,
        "bool_false": false,
        "integer": 42,
        "negative": -123,
        "float": 3.14159,
        "exponential": 1.23e-4,
        "empty_string": "",
        "unicode": "Hello 世界 🌍",
        "escape_chars": "Line1\nLine2\tTabbed\"Quoted\"",
        "empty_array": [],
        "empty_object": {},
        "nested_nulls": [null, {"key": null}],
        "mixed_array": [1, "two", true, null, {"five": 5}]
    });
    
    context.add_bounded_context("test", "Test Context")?;
    
    let entity_id = Uuid::new_v4().to_string();
    let entity = context.add_aggregate(&entity_id, "SpecialEntity", "test")?;
    
    // Serialize and deserialize
    let serialized = serde_json::to_string(&context)?;
    let deserialized: ContextGraph = serde_json::from_str(&serialized)?;
    
    // Verify special values preserved
    let deser_entity = deserialized.get_aggregates_by_type("SpecialEntity")?[0];
    let deser_data = deser_entity.data();
    
    // Check each special value
    assert_eq!(deser_data.get("null_value"), special_data.get("null_value"));
    assert_eq!(deser_data.get("bool_true"), special_data.get("bool_true"));
    assert_eq!(deser_data.get("unicode"), special_data.get("unicode"));
    assert_eq!(deser_data.get("mixed_array"), special_data.get("mixed_array"));
    
    Ok(())
}

#[test]
fn test_large_graph_serialization() -> Result<()> {
    // Create large graph to test performance
    let mut workflow = WorkflowGraph::new();
    
    // Create 100 states
    let mut state_names = Vec::new();
    for i in 0..100 {
        let state_name = format!("state-{}", i);
        let state_type = if i == 0 { StateType::Initial } else if i == 99 { StateType::Final } else { StateType::Normal };
        let state_node = WorkflowNode::new(&state_name, &state_name, state_type);
        workflow.add_state(state_node)?;
        state_names.push(state_name);
    }
    
    // Create transitions (each state to next 3)
    for i in 0..97 {
        for j in 1..=3 {
            if i + j < 100 {
                workflow.add_transition(
                    &state_names[i],
                    &state_names[i + j],
                    &format!("transition-{}-{}", i, i + j)
                )?;
            }
        }
    }
    
    // Serialize
    let serialized = serde_json::to_string(&workflow)?;
    
    // Verify size is reasonable
    assert!(serialized.len() > 10000); // Should be substantial
    assert!(serialized.len() < 1_000_000); // But not too large
    
    // Deserialize
    let deserialized: WorkflowGraph = serde_json::from_str(&serialized)?;
    
    // Verify structure
    assert_eq!(deserialized.node_count(), 100);
    assert_eq!(deserialized.edge_count(), 97 * 3);
    
    Ok(())
}

#[test]
fn test_uuid_serialization_formats() -> Result<()> {
    let mut context = ContextGraph::new();
    
    // Test different UUID formats
    let uuid_hyphenated = Uuid::from_str("550e8400-e29b-41d4-a716-446655440000")?;
    let uuid_simple = Uuid::from_str("550e8400e29b41d4a716446655440000")?;
    
    context.add_bounded_context("test", "Test Context")?;
    
    context.add_aggregate(&uuid_hyphenated.to_string(), "Entity1", "test")?;
    context.add_aggregate(&uuid_simple.to_string(), "Entity2", "test")?;
    
    // Serialize
    let serialized = serde_json::to_string(&context)?;
    
    // Verify UUIDs are in canonical format
    assert!(serialized.contains("550e8400-e29b-41d4-a716-446655440000"));
    
    // Deserialize
    let deserialized: ContextGraph = serde_json::from_str(&serialized)?;
    
    assert_eq!(deserialized.node_count(), 2);
    
    Ok(())
}