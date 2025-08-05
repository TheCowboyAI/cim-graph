//! Acceptance tests for US-008: Concept Graph

use cim_graph::graphs::concept::{ConceptGraph, ConceptNode, ConceptEdge, ConceptType, SemanticRelation};

#[test]
fn test_ac_008_1_define_ontology() {
    // Given: a concept graph
    let mut graph = ConceptGraph::new();
    
    // When: I define class hierarchy
    graph.add_concept("Thing", "Thing", serde_json::json!({})).unwrap();
    graph.add_concept("Animal", "Animal", serde_json::json!({})).unwrap();
    graph.add_concept("Mammal", "Mammal", serde_json::json!({})).unwrap();
    graph.add_concept("Dog", "Dog", serde_json::json!({})).unwrap();
    
    // And: define relationships
    graph.add_relation("Animal", "Thing", SemanticRelation::SubClassOf).unwrap();
    graph.add_relation("Mammal", "Animal", SemanticRelation::SubClassOf).unwrap();
    graph.add_relation("Dog", "Mammal", SemanticRelation::SubClassOf).unwrap();
    
    // Then: ontology is created
    assert_eq!(graph.graph().node_count(), 4);
    assert_eq!(graph.graph().edge_count(), 3);
}

#[test]
fn test_ac_008_2_semantic_inference() {
    // Given: an ontology with instances
    let mut graph = ConceptGraph::new();
    
    // Classes
    graph.add_concept("LivingThing", "Living Thing", serde_json::json!({})).unwrap();
    graph.add_concept("Animal", "Animal", serde_json::json!({})).unwrap();
    graph.add_concept("Dog", "Dog", serde_json::json!({})).unwrap();
    
    // Hierarchy
    graph.add_relation("Animal", "LivingThing", SemanticRelation::SubClassOf).unwrap();
    graph.add_relation("Dog", "Animal", SemanticRelation::SubClassOf).unwrap();
    
    // Instance
    graph.add_concept("fido", "Fido", serde_json::json!({"type": "Instance"})).unwrap();
    graph.add_relation("fido", "Dog", SemanticRelation::InstanceOf).unwrap();
    
    // When: I run inference
    graph.run_inference();
    
    // Then: transitive relationships are inferred
    assert!(graph.has_relation("Dog", "LivingThing", SemanticRelation::SubClassOf));
    assert!(graph.has_relation("fido", "Animal", SemanticRelation::InstanceOf));
    
    // The get_instances method should find fido as an instance of LivingThing
    // through the class hierarchy
    let living_instances = graph.get_instances("LivingThing");
    assert!(living_instances.contains("fido"));
}

#[test]
fn test_ac_008_3_properties_and_constraints() {
    // Given: a concept graph with properties
    let mut graph = ConceptGraph::new();
    
    // Define property
    graph.add_concept("hasAge", "has age", serde_json::json!({
        "type": "Property",
        "domain": "Person",
        "range": "Integer"
    })).unwrap();
    
    // Define classes
    graph.add_concept("Person", "Person", serde_json::json!({})).unwrap();
    graph.add_concept("Adult", "Adult", serde_json::json!({})).unwrap();
    
    // Adult is a Person with age >= 18 (simplified)
    graph.add_relation("Adult", "Person", SemanticRelation::SubClassOf).unwrap();
    
    // When: I query the property
    let property = graph.graph().get_node("hasAge").unwrap();
    
    // Then: property metadata is available
    assert_eq!(property.concept_type(), ConceptType::Property);
}

#[test]
fn test_ac_008_4_consistency_checking() {
    // Given: a concept graph with disjoint classes
    let mut graph = ConceptGraph::new();
    
    // Define disjoint classes
    graph.add_concept("Plant", "Plant", serde_json::json!({})).unwrap();
    graph.add_concept("Animal", "Animal", serde_json::json!({})).unwrap();
    
    graph.add_relation("Plant", "Animal", SemanticRelation::DisjointWith).unwrap();
    
    // Create an inconsistent instance
    graph.add_concept("confused", "Confused Thing", serde_json::json!({"type": "Instance"})).unwrap();
    graph.add_relation("confused", "Plant", SemanticRelation::InstanceOf).unwrap();
    graph.add_relation("confused", "Animal", SemanticRelation::InstanceOf).unwrap();
    
    // When: I check consistency
    let violations = graph.check_consistency();
    
    // Then: inconsistency is detected
    assert!(!violations.is_empty());
    assert!(violations[0].contains("disjoint classes"));
}

#[test]
fn test_semantic_queries() {
    // Given: a populated concept graph
    let mut graph = ConceptGraph::new();
    
    // Build a small taxonomy
    graph.add_concept("Vehicle", "Vehicle", serde_json::json!({})).unwrap();
    graph.add_concept("Car", "Car", serde_json::json!({})).unwrap();
    graph.add_concept("Truck", "Truck", serde_json::json!({})).unwrap();
    
    graph.add_relation("Car", "Vehicle", SemanticRelation::SubClassOf).unwrap();
    graph.add_relation("Truck", "Vehicle", SemanticRelation::SubClassOf).unwrap();
    
    // Add instances
    graph.add_concept("sedan1", "Sedan 1", serde_json::json!({"type": "Instance"})).unwrap();
    graph.add_concept("truck1", "Truck 1", serde_json::json!({"type": "Instance"})).unwrap();
    
    graph.add_relation("sedan1", "Car", SemanticRelation::InstanceOf).unwrap();
    graph.add_relation("truck1", "Truck", SemanticRelation::InstanceOf).unwrap();
    
    // Run inference
    graph.run_inference();
    
    // When: I query for all vehicles
    let vehicles = graph.get_instances("Vehicle");
    
    // Then: all vehicle instances are found
    assert_eq!(vehicles.len(), 2);
    assert!(vehicles.contains("sedan1"));
    assert!(vehicles.contains("truck1"));
}