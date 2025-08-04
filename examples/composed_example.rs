//! Example: Using ComposedGraph for multi-layer graph composition
//! 
//! This example demonstrates how to compose multiple graph types into
//! a unified multi-layer graph structure.

use cim_graph::graphs::ComposedGraph;
use cim_graph::graphs::composed::LayerType;
use cim_graph::graphs::context::ContextGraph;
use cim_graph::graphs::workflow::WorkflowGraph;
use cim_graph::graphs::concept::ConceptGraph;
use cim_graph::core::GraphType;
use serde_json::json;

fn main() {
    println!("=== Composed Graph Example: Healthcare System ===\n");
    println!("This example models a healthcare system with multiple layers:");
    println!("- Domain layer: Bounded contexts and aggregates");
    println!("- Process layer: Patient treatment workflows");
    println!("- Knowledge layer: Medical concepts and relationships\n");
    
    // Create a composed graph
    let mut composed = ComposedGraph::new();
    
    // Layer 1: Domain Model (Context Graph)
    println!("=== Building Domain Layer ===");
    let mut domain_layer = ContextGraph::new();
    
    // Define bounded contexts
    domain_layer.add_bounded_context("patient_management", "Patient Management").unwrap();
    domain_layer.add_bounded_context("clinical", "Clinical").unwrap();
    domain_layer.add_bounded_context("billing", "Billing").unwrap();
    
    // Add aggregates
    domain_layer.add_aggregate("patient", "Patient", "patient_management").unwrap();
    domain_layer.add_aggregate("appointment", "Appointment", "patient_management").unwrap();
    domain_layer.add_aggregate("treatment", "Treatment", "clinical").unwrap();
    domain_layer.add_aggregate("invoice", "Invoice", "billing").unwrap();
    
    // Add to composed graph
    composed.add_layer("domain", LayerType::Domain).unwrap();
    println!("✓ Domain layer added");
    
    // Layer 2: Process/Workflow (Workflow Graph)
    println!("\n=== Building Process Layer ===");
    let mut process_layer = WorkflowGraph::new();
    
    // Define patient journey workflow
    use cim_graph::graphs::workflow::{WorkflowNode, StateType};
    
    process_layer.add_state(WorkflowNode::new("registration", "Patient Registration", StateType::Initial)).unwrap();
    process_layer.add_state(WorkflowNode::new("triage", "Triage Assessment", StateType::Intermediate)).unwrap();
    process_layer.add_state(WorkflowNode::new("consultation", "Doctor Consultation", StateType::Intermediate)).unwrap();
    process_layer.add_state(WorkflowNode::new("treatment", "Treatment", StateType::Intermediate)).unwrap();
    process_layer.add_state(WorkflowNode::new("billing", "Billing", StateType::Intermediate)).unwrap();
    process_layer.add_state(WorkflowNode::new("discharge", "Discharge", StateType::Final)).unwrap();
    
    // Add transitions
    process_layer.add_transition("registration", "triage", "complete_registration").unwrap();
    process_layer.add_transition("triage", "consultation", "assign_doctor").unwrap();
    process_layer.add_transition("consultation", "treatment", "prescribe_treatment").unwrap();
    process_layer.add_transition("treatment", "billing", "complete_treatment").unwrap();
    process_layer.add_transition("billing", "discharge", "payment_processed").unwrap();
    
    composed.add_layer("process", LayerType::Process).unwrap();
    println!("✓ Process layer added");
    
    // Layer 3: Knowledge/Concept (Concept Graph)
    println!("\n=== Building Knowledge Layer ===");
    let mut knowledge_layer = ConceptGraph::new();
    
    // Medical concepts
    knowledge_layer.add_concept("symptom", "Symptom", json!({
        "type": "medical_observation"
    })).unwrap();
    
    knowledge_layer.add_concept("fever", "Fever", json!({
        "measurement": "temperature > 38°C",
        "severity": ["mild", "moderate", "severe"]
    })).unwrap();
    
    knowledge_layer.add_concept("cough", "Cough", json!({
        "types": ["dry", "productive"],
        "duration": "variable"
    })).unwrap();
    
    knowledge_layer.add_concept("diagnosis", "Diagnosis", json!({
        "type": "medical_conclusion"
    })).unwrap();
    
    knowledge_layer.add_concept("flu", "Influenza", json!({
        "category": "viral_infection",
        "contagious": true
    })).unwrap();
    
    knowledge_layer.add_concept("treatment_plan", "Treatment Plan", json!({
        "components": ["medication", "rest", "monitoring"]
    })).unwrap();
    
    // Add relationships
    use cim_graph::graphs::concept::SemanticRelation;
    
    knowledge_layer.add_relation("fever", "symptom", SemanticRelation::IsA).unwrap();
    knowledge_layer.add_relation("cough", "symptom", SemanticRelation::IsA).unwrap();
    knowledge_layer.add_relation("flu", "diagnosis", SemanticRelation::IsA).unwrap();
    knowledge_layer.add_relation("flu", "fever", SemanticRelation::Causes).unwrap();
    knowledge_layer.add_relation("flu", "cough", SemanticRelation::Causes).unwrap();
    knowledge_layer.add_relation("treatment_plan", "flu", SemanticRelation::Treats).unwrap();
    
    composed.add_layer("knowledge", LayerType::Knowledge).unwrap();
    println!("✓ Knowledge layer added");
    
    // Create cross-layer connections
    println!("\n=== Establishing Cross-Layer Connections ===");
    
    // Connect domain patient to process registration
    composed.connect_layers(
        "domain", "patient",
        "process", "registration",
        "initiates"
    ).unwrap();
    println!("✓ Connected Patient aggregate to Registration process");
    
    // Connect process consultation to knowledge diagnosis
    composed.connect_layers(
        "process", "consultation",
        "knowledge", "diagnosis",
        "produces"
    ).unwrap();
    println!("✓ Connected Consultation process to Diagnosis concept");
    
    // Connect domain treatment to knowledge treatment_plan
    composed.connect_layers(
        "domain", "treatment",
        "knowledge", "treatment_plan",
        "implements"
    ).unwrap();
    println!("✓ Connected Treatment aggregate to Treatment Plan concept");
    
    // Query the composed graph
    println!("\n\n=== Querying the Composed Graph ===");
    
    // Get layer information
    println!("\nLayers in the system:");
    for (name, layer_type) in composed.get_layers() {
        let stats = composed.get_layer_stats(&name).unwrap();
        println!("  - {} ({:?}): {} nodes, {} edges", 
            name, layer_type, stats.0, stats.1);
    }
    
    // Find cross-layer connections
    println!("\nCross-layer connections:");
    let connections = composed.get_cross_layer_connections();
    for conn in &connections {
        println!("  - {} → {}", conn.source_node, conn.target_node);
        println!("    {} layer to {} layer via '{}'", 
            conn.source_layer, conn.target_layer, conn.connection_type);
    }
    
    // Demonstrate layer interaction
    println!("\n\n=== Example: Patient Journey ===");
    println!("1. Patient registered (Domain: Patient aggregate)");
    println!("2. Registration complete (Process: Registration → Triage)");
    println!("3. Triage identifies symptoms: fever, cough (Knowledge: Symptom concepts)");
    println!("4. Doctor consultation (Process: Consultation state)");
    println!("5. Diagnosis: Influenza (Knowledge: Flu concept)");
    println!("6. Treatment prescribed (Domain: Treatment aggregate)");
    println!("7. Treatment plan created (Knowledge: Treatment Plan)");
    println!("8. Billing processed (Domain: Invoice aggregate)");
    println!("9. Patient discharged (Process: Discharge state)");
    
    // Show how different layers work together
    println!("\n\n=== Layer Interactions ===");
    println!("Domain Layer: Manages business entities and rules");
    println!("  - Ensures patient data integrity");
    println!("  - Handles appointment scheduling");
    println!("  - Manages billing and invoicing");
    
    println!("\nProcess Layer: Controls workflow and state transitions");
    println!("  - Enforces proper patient flow");
    println!("  - Tracks current state of each patient");
    println!("  - Ensures all steps are completed");
    
    println!("\nKnowledge Layer: Provides medical intelligence");
    println!("  - Links symptoms to diagnoses");
    println!("  - Suggests treatment plans");
    println!("  - Maintains medical knowledge base");
    
    // Statistics
    println!("\n\n=== Composed Graph Statistics ===");
    let total_nodes: usize = composed.get_layers()
        .iter()
        .map(|(name, _)| composed.get_layer_stats(name).unwrap().0)
        .sum();
    
    let total_edges: usize = composed.get_layers()
        .iter()
        .map(|(name, _)| composed.get_layer_stats(name).unwrap().1)
        .sum();
    
    println!("Total layers: {}", composed.get_layers().len());
    println!("Total nodes across all layers: {}", total_nodes);
    println!("Total edges across all layers: {}", total_edges);
    println!("Cross-layer connections: {}", connections.len());
    
    // Benefits of composition
    println!("\n\n=== Benefits of Graph Composition ===");
    println!("1. Separation of Concerns: Each layer handles its specific domain");
    println!("2. Flexibility: Layers can evolve independently");
    println!("3. Reusability: Layers can be reused in different compositions");
    println!("4. Comprehensive Modeling: Captures multiple aspects of the system");
    println!("5. Cross-cutting Insights: Connections reveal system-wide patterns");
}