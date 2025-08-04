//! Demo: Knowledge Base using CIM Graph
//! 
//! This example shows how to build a simple knowledge base
//! using ConceptGraph for semantic relationships.

use cim_graph::graphs::concept::{ConceptGraph, ConceptNode, SemanticRelation};
use cim_graph::error::Result;

/// A simple knowledge base system
struct KnowledgeBase {
    graph: ConceptGraph,
}

impl KnowledgeBase {
    fn new() -> Self {
        Self {
            graph: ConceptGraph::new(),
        }
    }
    
    fn add_concept(&mut self, id: &str, name: &str, description: &str) -> Result<()> {
        let concept = ConceptNode::new(id, name, description);
        self.graph.add_concept(concept)?;
        println!("✓ Added concept: {} - {}", id, name);
        Ok(())
    }
    
    fn add_relationship(&mut self, from: &str, to: &str, relation: SemanticRelation) -> Result<()> {
        self.graph.add_relation(from, to, relation)?;
        println!("✓ Added relationship: {} --{:?}--> {}", from, relation, to);
        Ok(())
    }
    
    fn query_related(&self, concept_id: &str, relation: SemanticRelation) {
        println!("\n🔍 Concepts related to '{}' by {:?}:", concept_id, relation);
        
        if let Ok(edges) = self.graph.graph().edges_from(concept_id) {
            for edge_id in edges {
                if let Some(edge) = self.graph.graph().get_edge(&edge_id) {
                    if edge.relation == relation {
                        if let Some(target) = self.graph.get_concept(&edge.target) {
                            println!("  - {} ({})", target.label(), target.description());
                        }
                    }
                }
            }
        }
    }
    
    fn find_hierarchy(&self, root: &str) {
        println!("\n🌳 Hierarchy starting from '{}':", root);
        self.print_hierarchy(root, 0);
    }
    
    fn print_hierarchy(&self, concept_id: &str, depth: usize) {
        let indent = "  ".repeat(depth);
        
        if let Some(concept) = self.graph.get_concept(concept_id) {
            println!("{}- {} ({})", indent, concept.label(), concept.id());
            
            // Find all children (IsA relationships pointing to this concept)
            if let Ok(edges) = self.graph.graph().edges_from(concept_id) {
                for edge_id in edges {
                    if let Some(edge) = self.graph.graph().get_edge(&edge_id) {
                        if edge.relation == SemanticRelation::IsA {
                            self.print_hierarchy(&edge.target, depth + 1);
                        }
                    }
                }
            }
        }
    }
}

fn main() -> Result<()> {
    println!("🧠 Knowledge Base Demo\n");
    
    let mut kb = KnowledgeBase::new();
    
    // Build a simple taxonomy of programming concepts
    println!("📚 Building programming knowledge base...\n");
    
    // Top-level concepts
    kb.add_concept("programming", "Programming", "The art and science of creating software")?;
    kb.add_concept("paradigm", "Programming Paradigm", "A style or way of programming")?;
    kb.add_concept("language", "Programming Language", "A formal language for writing programs")?;
    
    // Paradigms
    kb.add_concept("imperative", "Imperative Programming", "Programming with explicit commands")?;
    kb.add_concept("declarative", "Declarative Programming", "Programming by declaring what should be done")?;
    kb.add_concept("oop", "Object-Oriented Programming", "Programming with objects and classes")?;
    kb.add_concept("functional", "Functional Programming", "Programming with mathematical functions")?;
    
    // Languages
    kb.add_concept("rust", "Rust", "A systems programming language focused on safety")?;
    kb.add_concept("python", "Python", "A high-level, interpreted programming language")?;
    kb.add_concept("javascript", "JavaScript", "A dynamic language for web development")?;
    kb.add_concept("haskell", "Haskell", "A purely functional programming language")?;
    
    // Language features
    kb.add_concept("memory_safety", "Memory Safety", "Protection against memory-related bugs")?;
    kb.add_concept("type_safety", "Type Safety", "Prevention of type errors at compile time")?;
    kb.add_concept("gc", "Garbage Collection", "Automatic memory management")?;
    
    // Add relationships
    println!("\n🔗 Adding relationships...\n");
    
    // Paradigm relationships
    kb.add_relationship("paradigm", "programming", SemanticRelation::PartOf)?;
    kb.add_relationship("language", "programming", SemanticRelation::PartOf)?;
    
    kb.add_relationship("imperative", "paradigm", SemanticRelation::IsA)?;
    kb.add_relationship("declarative", "paradigm", SemanticRelation::IsA)?;
    kb.add_relationship("oop", "imperative", SemanticRelation::IsA)?;
    kb.add_relationship("functional", "declarative", SemanticRelation::IsA)?;
    
    // Language classifications
    kb.add_relationship("rust", "language", SemanticRelation::IsA)?;
    kb.add_relationship("python", "language", SemanticRelation::IsA)?;
    kb.add_relationship("javascript", "language", SemanticRelation::IsA)?;
    kb.add_relationship("haskell", "language", SemanticRelation::IsA)?;
    
    // Language features
    kb.add_relationship("memory_safety", "rust", SemanticRelation::PartOf)?;
    kb.add_relationship("type_safety", "rust", SemanticRelation::PartOf)?;
    kb.add_relationship("gc", "python", SemanticRelation::PartOf)?;
    kb.add_relationship("gc", "javascript", SemanticRelation::PartOf)?;
    kb.add_relationship("type_safety", "haskell", SemanticRelation::PartOf)?;
    
    // Query the knowledge base
    println!("\n📊 Querying knowledge base...");
    
    kb.query_related("paradigm", SemanticRelation::IsA);
    kb.query_related("rust", SemanticRelation::PartOf);
    
    // Show hierarchy
    kb.find_hierarchy("programming");
    
    // Demonstrate inference
    println!("\n💡 Inference example:");
    println!("If Rust IsA Language, and Language PartOf Programming,");
    println!("then we can infer: Rust is related to Programming");
    
    // Show statistics
    println!("\n📈 Knowledge Base Statistics:");
    println!("  - Total concepts: {}", kb.graph.graph().node_count());
    println!("  - Total relationships: {}", kb.graph.graph().edge_count());
    
    Ok(())
}