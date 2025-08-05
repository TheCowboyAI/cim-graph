//! Integration tests for CIM Graph library
//!
//! These tests validate complex interactions between graph types,
//! real-world workflows, and edge cases that unit tests might miss.

mod cross_graph_operations;
mod real_world_workflows;
mod error_scenarios;
mod serialization_roundtrips;
mod algorithm_compatibility;
mod performance_scenarios;
mod event_propagation;
mod concurrent_operations;

use cim_graph::Result;

/// Common test utilities and helpers
pub mod utils {
    use cim_graph::graphs::{IpldGraph, ContextGraph, WorkflowGraph, ConceptGraph};
    use cim_graph::graphs::context::RelationshipType;
    use cim_graph::{Result};
    use uuid::Uuid;
    use serde_json::json;
    
    /// Create a sample IPLD graph with test data
    pub fn create_test_ipld_graph() -> Result<IpldGraph> {
        let mut graph = IpldGraph::new();
        
        // Add some CIDs
        let root = graph.add_content(serde_json::json!({ "cid": "QmRoot", "format": "dag-cbor", "size": 1024 }))?;
        let child1 = graph.add_content(serde_json::json!({ "cid": "QmChild1", "format": "dag-json", "size": 512 }))?;
        let child2 = graph.add_content(serde_json::json!({ "cid": "QmChild2", "format": "dag-cbor", "size": 768 }))?;
        
        // Create links
        graph.add_link(&root, &child1, "data")?;
        graph.add_link(&root, &child2, "metadata")?;
        
        Ok(graph)
    }
    
    /// Create a sample context graph with domain objects
    pub fn create_test_context_graph() -> Result<ContextGraph> {
        let mut graph = ContextGraph::new();
        
        // Add bounded context first
        let context = graph.add_bounded_context("test-domain", "Test Domain")?;
        
        // Add aggregates
        let customer_id = Uuid::new_v4().to_string();
        let customer = graph.add_aggregate(&customer_id, "Customer", "test-domain")?;
        
        let order_id = Uuid::new_v4().to_string();
        let order = graph.add_aggregate(&order_id, "Order", "test-domain")?;
        
        // Add entities
        let item_id = Uuid::new_v4().to_string();
        let item1 = graph.add_entity(&item_id, "OrderItem", &order)?;
        
        // Add relationships
        graph.add_relationship(&customer, &order, RelationshipType::References)?;
        
        Ok(graph)
    }
    
    /// Create a sample workflow graph
    pub fn create_test_workflow_graph() -> Result<WorkflowGraph> {
        use cim_graph::graphs::workflow::{WorkflowNode, StateType};
        
        let mut graph = WorkflowGraph::new("order-processing");
        
        // Add states
        let draft = graph.add_state(WorkflowNode::new("draft", "Draft", StateType::Initial))?;
        
        let submitted = graph.add_state(WorkflowNode::new("submitted", "Submitted", StateType::Normal))?;
        
        let approved = graph.add_state(WorkflowNode::new("approved", "Approved", StateType::Final))?;
        
        let rejected = graph.add_state(WorkflowNode::new("rejected", "Rejected", StateType::Final))?;
        
        // Add transitions
        graph.add_transition(&draft, &submitted, "submit")?;
        
        graph.add_transition(&submitted, &approved, "approve")?;
        
        graph.add_transition(&submitted, &rejected, "reject")?;
        
        Ok(graph)
    }
    
    /// Create a sample concept graph
    pub fn create_test_concept_graph() -> Result<ConceptGraph> {
        use cim_graph::graphs::concept::SemanticRelation;
        
        let mut graph = ConceptGraph::new();
        
        // Add concepts
        let vehicle = graph.add_concept("Vehicle", "Vehicle", json!({
            "wheels": 0.8,
            "engine": 0.7,
            "transport": 1.0
        }))?;
        
        let car = graph.add_concept("Car", "Car", json!({
            "wheels": 1.0,
            "engine": 1.0,
            "transport": 1.0,
            "doors": 0.9
        }))?;
        
        let bicycle = graph.add_concept("Bicycle", "Bicycle", json!({
            "wheels": 1.0,
            "engine": 0.0,
            "transport": 1.0,
            "pedals": 1.0
        }))?;
        
        // Add relationships
        graph.add_relation(&car, &vehicle, SemanticRelation::SubClassOf)?;
        graph.add_relation(&bicycle, &vehicle, SemanticRelation::SubClassOf)?;
        
        Ok(graph)
    }
}