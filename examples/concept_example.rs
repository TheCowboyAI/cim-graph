//! Example: Using ConceptGraph for semantic reasoning
//! 
//! This example demonstrates how to build a knowledge graph with
//! concepts, relationships, and inference rules.

use cim_graph::graphs::ConceptGraph;
use cim_graph::graphs::concept::{ConceptNode, SemanticRelation, InferenceRule};
use cim_graph::core::Node;
use serde_json::json;

fn main() {
    println!("=== Concept Graph Example: Animal Taxonomy ===\n");
    
    // Create a new concept graph
    let mut graph = ConceptGraph::new();
    
    // Define concepts in the taxonomy
    println!("Building taxonomy...");
    
    // Top-level concepts
    graph.add_concept("animal", "Animal", json!({
        "kingdom": "Animalia",
        "characteristics": ["multicellular", "heterotrophic", "motile"]
    })).expect("Failed to add animal");
    
    // Vertebrates
    graph.add_concept("vertebrate", "Vertebrate", json!({
        "backbone": true,
        "characteristics": ["spinal_column", "internal_skeleton"]
    })).expect("Failed to add vertebrate");
    
    graph.add_concept("mammal", "Mammal", json!({
        "characteristics": ["warm_blooded", "hair_or_fur", "milk_production"],
        "reproduction": "live_birth"
    })).expect("Failed to add mammal");
    
    graph.add_concept("bird", "Bird", json!({
        "characteristics": ["warm_blooded", "feathers", "beaks", "lay_eggs"],
        "abilities": ["flight"] // most birds
    })).expect("Failed to add bird");
    
    graph.add_concept("reptile", "Reptile", json!({
        "characteristics": ["cold_blooded", "scales", "lay_eggs"],
        "habitat": ["land", "water"]
    })).expect("Failed to add reptile");
    
    // Specific animals
    graph.add_concept("dog", "Dog", json!({
        "species": "Canis familiaris",
        "characteristics": ["domestic", "loyal", "pack_animal"],
        "sounds": ["bark", "howl"]
    })).expect("Failed to add dog");
    
    graph.add_concept("cat", "Cat", json!({
        "species": "Felis catus",
        "characteristics": ["domestic", "independent", "nocturnal"],
        "sounds": ["meow", "purr"]
    })).expect("Failed to add cat");
    
    graph.add_concept("eagle", "Eagle", json!({
        "characteristics": ["predator", "sharp_vision", "powerful_talons"],
        "habitat": "mountains"
    })).expect("Failed to add eagle");
    
    graph.add_concept("penguin", "Penguin", json!({
        "characteristics": ["flightless", "aquatic", "social"],
        "habitat": "antarctica"
    })).expect("Failed to add penguin");
    
    graph.add_concept("snake", "Snake", json!({
        "characteristics": ["no_legs", "venomous_some", "carnivorous"],
        "movement": "slithering"
    })).expect("Failed to add snake");
    
    // Build relationships
    println!("\nEstablishing relationships...");
    
    // Taxonomic relationships
    graph.add_relation("vertebrate", "animal", SemanticRelation::IsA).unwrap();
    graph.add_relation("mammal", "vertebrate", SemanticRelation::IsA).unwrap();
    graph.add_relation("bird", "vertebrate", SemanticRelation::IsA).unwrap();
    graph.add_relation("reptile", "vertebrate", SemanticRelation::IsA).unwrap();
    
    graph.add_relation("dog", "mammal", SemanticRelation::IsA).unwrap();
    graph.add_relation("cat", "mammal", SemanticRelation::IsA).unwrap();
    graph.add_relation("eagle", "bird", SemanticRelation::IsA).unwrap();
    graph.add_relation("penguin", "bird", SemanticRelation::IsA).unwrap();
    graph.add_relation("snake", "reptile", SemanticRelation::IsA).unwrap();
    
    // Other relationships
    graph.add_relation("mammal", "milk_production", SemanticRelation::HasProperty).unwrap();
    graph.add_relation("bird", "flight", SemanticRelation::HasProperty).unwrap();
    graph.add_relation("penguin", "flight", SemanticRelation::Contradicts).unwrap();
    
    // Add properties as concepts
    graph.add_concept("milk_production", "Milk Production", json!({
        "type": "biological_function"
    })).unwrap();
    
    graph.add_concept("flight", "Flight", json!({
        "type": "ability",
        "requirements": ["wings", "lightweight_bones"]
    })).unwrap();
    
    // Add inference rules
    println!("\nAdding inference rules...");
    
    // Apply inference
    println!("\nApplying inference rules...");
    let inferences = graph.apply_inference();
    println!("Generated {} inferences", inferences);
    
    // Query the graph
    println!("\n\n=== Querying the Knowledge Graph ===");
    
    // What is a dog?
    println!("\nWhat is a dog?");
    let dog_relations = graph.get_relations("dog");
    for (target, relation) in dog_relations {
        if relation == SemanticRelation::IsA {
            println!("  Dog {} {}", relation, target);
        }
    }
    
    // What are all the inferred relationships for dog?
    println!("\nInferred relationships for dog:");
    let dog_inferred = graph.get_inferred("dog");
    for (target, relation) in dog_inferred {
        println!("  Dog {} {} (inferred)", relation, target);
    }
    
    // Find all mammals
    println!("\n\nAll mammals in the graph:");
    let all_concepts = graph.get_all_concepts();
    for concept in all_concepts {
        let relations = graph.get_relations(&concept.id());
        let inferred = graph.get_inferred(&concept.id());
        
        let is_mammal = relations.iter().any(|(target, rel)| 
            target == "mammal" && *rel == SemanticRelation::IsA
        ) || inferred.iter().any(|(target, rel)| 
            target == "mammal" && *rel == SemanticRelation::IsA
        );
        
        if is_mammal {
            println!("  - {}", concept.name());
        }
    }
    
    // Check contradictions
    println!("\n\nChecking for contradictions:");
    for concept in graph.get_all_concepts() {
        let relations = graph.get_relations(&concept.id());
        for (target, relation) in relations {
            if relation == SemanticRelation::Contradicts {
                println!("  {} contradicts {}", concept.name(), target);
            }
        }
    }
    
    // Semantic similarity example
    println!("\n\n=== Semantic Similarity ===");
    println!("Dog and Cat are both mammals - they share common properties");
    println!("Eagle and Penguin are both birds - but have different flight capabilities");
    
    // Graph statistics
    println!("\n\n=== Graph Statistics ===");
    println!("Total concepts: {}", graph.graph().node_count());
    println!("Total relationships: {}", graph.graph().edge_count());
    println!("Inference rules: {}", graph.rules().len());
    println!("Inferred relationships: {}", 
        graph.get_all_concepts().iter()
            .map(|c| graph.get_inferred(&c.id()).len())
            .sum::<usize>()
    );
    
    // Demonstrate reasoning path
    println!("\n\n=== Reasoning Path Example ===");
    println!("Why does a dog have the property of milk_production?");
    println!("1. Dog IsA Mammal (direct relationship)");
    println!("2. Mammal HasProperty milk_production (direct relationship)");
    println!("3. Therefore: Dog HasProperty milk_production (inferred by PropertyInheritance rule)");
}