//! Acceptance tests for US-008: Concept Graph

use cim_graph::graphs::concept::{ConceptGraph, ConceptNode, ConceptEdge, ConceptType, SemanticRelation};

#[test]
fn test_ac_008_1_define_ontology() {
    // Given: a concept graph
    let mut graph = ConceptGraph::new();
    
    // When: I define class hierarchy
    let thing = ConceptNode::new("Thing", "Thing", ConceptType::Class);
    let animal = ConceptNode::new("Animal", "Animal", ConceptType::Class);
    let mammal = ConceptNode::new("Mammal", "Mammal", ConceptType::Class);
    let dog = ConceptNode::new("Dog", "Dog", ConceptType::Class);
    
    graph.add_concept(thing).unwrap();
    graph.add_concept(animal).unwrap();
    graph.add_concept(mammal).unwrap();
    graph.add_concept(dog).unwrap();
    
    // And: define relationships
    graph.add_relation(ConceptEdge::new("Animal", "Thing", SemanticRelation::SubClassOf)).unwrap();
    graph.add_relation(ConceptEdge::new("Mammal", "Animal", SemanticRelation::SubClassOf)).unwrap();
    graph.add_relation(ConceptEdge::new("Dog", "Mammal", SemanticRelation::SubClassOf)).unwrap();
    
    // Then: ontology is created
    assert_eq!(graph.graph().node_count(), 4);
    assert_eq!(graph.graph().edge_count(), 3);
}

#[test]
fn test_ac_008_2_semantic_inference() {
    // Given: an ontology with instances
    let mut graph = ConceptGraph::new();
    
    // Classes
    graph.add_concept(ConceptNode::new("LivingThing", "Living Thing", ConceptType::Class)).unwrap();
    graph.add_concept(ConceptNode::new("Animal", "Animal", ConceptType::Class)).unwrap();
    graph.add_concept(ConceptNode::new("Dog", "Dog", ConceptType::Class)).unwrap();
    
    // Hierarchy
    graph.add_relation(ConceptEdge::new("Animal", "LivingThing", SemanticRelation::SubClassOf)).unwrap();
    graph.add_relation(ConceptEdge::new("Dog", "Animal", SemanticRelation::SubClassOf)).unwrap();
    
    // Instance
    graph.add_concept(ConceptNode::new("fido", "Fido", ConceptType::Instance)).unwrap();
    graph.add_relation(ConceptEdge::new("fido", "Dog", SemanticRelation::InstanceOf)).unwrap();
    
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
    let mut has_age = ConceptNode::new("hasAge", "has age", ConceptType::Property);
    has_age.add_annotation("domain", "Person");
    has_age.add_annotation("range", "Integer");
    
    graph.add_concept(has_age).unwrap();
    
    // Define classes
    graph.add_concept(ConceptNode::new("Person", "Person", ConceptType::Class)).unwrap();
    graph.add_concept(ConceptNode::new("Adult", "Adult", ConceptType::Class)).unwrap();
    
    // Adult is a Person with age >= 18 (simplified)
    graph.add_relation(ConceptEdge::new("Adult", "Person", SemanticRelation::SubClassOf)).unwrap();
    
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
    graph.add_concept(ConceptNode::new("Plant", "Plant", ConceptType::Class)).unwrap();
    graph.add_concept(ConceptNode::new("Animal", "Animal", ConceptType::Class)).unwrap();
    
    graph.add_relation(ConceptEdge::new("Plant", "Animal", SemanticRelation::DisjointWith)).unwrap();
    
    // Create an inconsistent instance
    graph.add_concept(ConceptNode::new("confused", "Confused Thing", ConceptType::Instance)).unwrap();
    graph.add_relation(ConceptEdge::new("confused", "Plant", SemanticRelation::InstanceOf)).unwrap();
    graph.add_relation(ConceptEdge::new("confused", "Animal", SemanticRelation::InstanceOf)).unwrap();
    
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
    graph.add_concept(ConceptNode::new("Vehicle", "Vehicle", ConceptType::Class)).unwrap();
    graph.add_concept(ConceptNode::new("Car", "Car", ConceptType::Class)).unwrap();
    graph.add_concept(ConceptNode::new("Truck", "Truck", ConceptType::Class)).unwrap();
    
    graph.add_relation(ConceptEdge::new("Car", "Vehicle", SemanticRelation::SubClassOf)).unwrap();
    graph.add_relation(ConceptEdge::new("Truck", "Vehicle", SemanticRelation::SubClassOf)).unwrap();
    
    // Add instances
    graph.add_concept(ConceptNode::new("sedan1", "Sedan 1", ConceptType::Instance)).unwrap();
    graph.add_concept(ConceptNode::new("truck1", "Truck 1", ConceptType::Instance)).unwrap();
    
    graph.add_relation(ConceptEdge::new("sedan1", "Car", SemanticRelation::InstanceOf)).unwrap();
    graph.add_relation(ConceptEdge::new("truck1", "Truck", SemanticRelation::InstanceOf)).unwrap();
    
    // Run inference
    graph.run_inference();
    
    // When: I query for all vehicles
    let vehicles = graph.get_instances("Vehicle");
    
    // Then: all vehicle instances are found
    assert_eq!(vehicles.len(), 2);
    assert!(vehicles.contains("sedan1"));
    assert!(vehicles.contains("truck1"));
}