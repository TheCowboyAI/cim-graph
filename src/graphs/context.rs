//! Context graph for Domain-Driven Design (DDD) modeling
//! 
//! Represents bounded contexts, aggregates, entities, and their relationships

use crate::core::{Edge, EventGraph, EventHandler, GraphBuilder, GraphType};
use crate::error::{GraphError, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Types of domain objects in a context graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainObjectType {
    /// Bounded context - a boundary within which a domain model is defined
    BoundedContext,
    /// Aggregate root - cluster of domain objects treated as a unit
    AggregateRoot,
    /// Entity - object with unique identity
    Entity,
    /// Value object - object without unique identity
    ValueObject,
    /// Domain service - stateless operation
    DomainService,
    /// Domain event - something that happened in the domain
    DomainEvent,
}

/// Node representing a domain object in the context graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextNode {
    /// Unique identifier
    id: String,
    /// Name of the domain object
    name: String,
    /// Type of domain object
    object_type: DomainObjectType,
    /// Bounded context this object belongs to
    bounded_context: Option<String>,
    /// Properties of the domain object
    properties: serde_json::Value,
    /// Invariants/business rules
    invariants: Vec<String>,
}

impl ContextNode {
    /// Create a new context node
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        object_type: DomainObjectType,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            object_type,
            bounded_context: None,
            properties: serde_json::Value::Object(serde_json::Map::new()),
            invariants: Vec::new(),
        }
    }
    
    /// Set the bounded context
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.bounded_context = Some(context.into());
        self
    }
    
    /// Add properties
    pub fn with_properties(mut self, properties: serde_json::Value) -> Self {
        self.properties = properties;
        self
    }
    
    /// Add an invariant
    pub fn add_invariant(&mut self, invariant: impl Into<String>) {
        self.invariants.push(invariant.into());
    }
    
    /// Get the object type
    pub fn object_type(&self) -> DomainObjectType {
        self.object_type
    }
    
    /// Get the name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get the bounded context
    pub fn bounded_context(&self) -> Option<&str> {
        self.bounded_context.as_deref()
    }
}

impl crate::core::Node for ContextNode {
    fn id(&self) -> String {
        self.id.clone()
    }
}

/// Types of relationships in a context graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipType {
    /// Aggregate contains entity/value object
    Contains,
    /// Entity references another entity
    References,
    /// Domain event is emitted by aggregate
    EmittedBy,
    /// Command is handled by aggregate
    HandledBy,
    /// Context depends on another context
    DependsOn,
    /// Anti-corruption layer between contexts
    AntiCorruptionLayer,
    /// Shared kernel between contexts
    SharedKernel,
    /// Customer-supplier relationship
    CustomerSupplier,
    /// Conformist relationship
    Conformist,
}

/// Edge representing a relationship between domain objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEdge {
    /// Unique identifier
    id: String,
    /// Source object
    source: String,
    /// Target object
    target: String,
    /// Type of relationship
    relationship_type: RelationshipType,
    /// Additional metadata
    metadata: serde_json::Value,
}

impl ContextEdge {
    /// Create a new context edge
    pub fn new(
        source: impl Into<String>,
        target: impl Into<String>,
        relationship_type: RelationshipType,
    ) -> Self {
        let source = source.into();
        let target = target.into();
        let id = format!("{}:{}:{}", source, relationship_type as u8, target);
        
        Self {
            id,
            source,
            target,
            relationship_type,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
    
    /// Get the relationship type
    pub fn relationship_type(&self) -> RelationshipType {
        self.relationship_type
    }
}

impl crate::core::Edge for ContextEdge {
    fn id(&self) -> String {
        self.id.clone()
    }
    
    fn source(&self) -> String {
        self.source.clone()
    }
    
    fn target(&self) -> String {
        self.target.clone()
    }
}

/// Domain-driven design context graph
pub struct ContextGraph {
    /// Underlying event-driven graph
    graph: EventGraph<ContextNode, ContextEdge>,
}

impl ContextGraph {
    /// Create a new context graph
    pub fn new() -> Self {
        let graph = GraphBuilder::new()
            .graph_type(GraphType::ContextGraph)
            .build_event()
            .expect("Failed to create context graph");
            
        Self { graph }
    }
    
    /// Create a new context graph with an event handler
    pub fn with_handler(handler: Arc<dyn EventHandler>) -> Self {
        let graph = GraphBuilder::new()
            .graph_type(GraphType::ContextGraph)
            .add_handler(handler)
            .build_event()
            .expect("Failed to create context graph");
            
        Self { graph }
    }
    
    /// Add a bounded context
    pub fn add_bounded_context(&mut self, id: impl Into<String>, name: impl Into<String>) -> Result<String> {
        let node = ContextNode::new(id, name, DomainObjectType::BoundedContext);
        self.graph.add_node(node)
    }
    
    /// Add an aggregate root
    pub fn add_aggregate(
        &mut self,
        id: impl Into<String>,
        name: impl Into<String>,
        context: impl Into<String>,
    ) -> Result<String> {
        let node = ContextNode::new(id, name, DomainObjectType::AggregateRoot)
            .with_context(context);
        self.graph.add_node(node)
    }
    
    /// Add an entity
    pub fn add_entity(
        &mut self,
        id: impl Into<String>,
        name: impl Into<String>,
        aggregate: impl Into<String>,
    ) -> Result<String> {
        let id_str = id.into();
        let aggregate_str = aggregate.into();
        
        // Get the aggregate's context
        let context = self.graph.get_node(&aggregate_str)
            .and_then(|n| n.bounded_context())
            .ok_or_else(|| GraphError::NodeNotFound(aggregate_str.clone()))?
            .to_string();
        
        let node = ContextNode::new(id_str.clone(), name, DomainObjectType::Entity)
            .with_context(context);
        
        self.graph.add_node(node)?;
        
        // Add contains relationship
        let edge = ContextEdge::new(aggregate_str, id_str.clone(), RelationshipType::Contains);
        self.graph.add_edge(edge)?;
        
        Ok(id_str)
    }
    
    /// Add a relationship between domain objects
    pub fn add_relationship(
        &mut self,
        from: impl Into<String>,
        to: impl Into<String>,
        relationship: RelationshipType,
    ) -> Result<String> {
        let edge = ContextEdge::new(from, to, relationship);
        self.graph.add_edge(edge)
    }
    
    /// Get all objects in a bounded context
    pub fn get_context_objects(&self, context: &str) -> Vec<&ContextNode> {
        self.graph.node_ids()
            .into_iter()
            .filter_map(|id| self.graph.get_node(&id))
            .filter(|node| node.bounded_context() == Some(context))
            .collect()
    }
    
    /// Get all aggregates in a context
    pub fn get_aggregates(&self, context: &str) -> Vec<&ContextNode> {
        self.get_context_objects(context)
            .into_iter()
            .filter(|node| node.object_type() == DomainObjectType::AggregateRoot)
            .collect()
    }
    
    /// Validate context boundaries (no direct references across contexts)
    pub fn validate_boundaries(&self) -> Vec<String> {
        let mut violations = Vec::new();
        
        for edge_id in self.graph.edge_ids() {
            if let Some(edge) = self.graph.get_edge(&edge_id) {
                // Skip certain allowed cross-context relationships
                match edge.relationship_type() {
                    RelationshipType::DependsOn |
                    RelationshipType::AntiCorruptionLayer |
                    RelationshipType::SharedKernel |
                    RelationshipType::CustomerSupplier |
                    RelationshipType::Conformist => continue,
                    _ => {}
                }
                
                // Check if source and target are in different contexts
                let source_context = self.graph.get_node(&edge.source())
                    .and_then(|n| n.bounded_context());
                let target_context = self.graph.get_node(&edge.target())
                    .and_then(|n| n.bounded_context());
                
                if source_context.is_some() && target_context.is_some() 
                    && source_context != target_context {
                    violations.push(format!(
                        "Cross-context reference: {} -> {} (contexts: {:?} -> {:?})",
                        edge.source(), edge.target(), source_context, target_context
                    ));
                }
            }
        }
        
        violations
    }
    
    /// Get the underlying graph
    pub fn graph(&self) -> &EventGraph<ContextNode, ContextEdge> {
        &self.graph
    }
    
    /// Get mutable access to the underlying graph
    pub fn graph_mut(&mut self) -> &mut EventGraph<ContextNode, ContextEdge> {
        &mut self.graph
    }
}

impl Default for ContextGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_context_graph_creation() {
        let graph = ContextGraph::new();
        assert_eq!(graph.graph().node_count(), 0);
        assert_eq!(graph.graph().graph_type(), GraphType::ContextGraph);
    }
    
    #[test]
    fn test_bounded_contexts() {
        let mut graph = ContextGraph::new();
        
        // Add bounded contexts
        graph.add_bounded_context("sales", "Sales Context").unwrap();
        graph.add_bounded_context("inventory", "Inventory Context").unwrap();
        
        assert_eq!(graph.graph().node_count(), 2);
    }
    
    #[test]
    fn test_aggregate_hierarchy() {
        let mut graph = ContextGraph::new();
        
        // Add context and aggregate
        graph.add_bounded_context("sales", "Sales Context").unwrap();
        graph.add_aggregate("order", "Order", "sales").unwrap();
        
        // Add entity to aggregate
        let entity_id = graph.add_entity("order_item", "OrderItem", "order").unwrap();
        
        // Verify entity was added with contains relationship
        assert_eq!(graph.graph().node_count(), 3);
        assert_eq!(graph.graph().edge_count(), 1);
        
        // Check the entity has the right context
        let entity = graph.graph().get_node(&entity_id).unwrap();
        assert_eq!(entity.bounded_context(), Some("sales"));
    }
    
    #[test]
    fn test_context_validation() {
        let mut graph = ContextGraph::new();
        
        // Set up two contexts
        graph.add_bounded_context("sales", "Sales").unwrap();
        graph.add_bounded_context("inventory", "Inventory").unwrap();
        
        // Add aggregates
        graph.add_aggregate("order", "Order", "sales").unwrap();
        graph.add_aggregate("product", "Product", "inventory").unwrap();
        
        // Add invalid cross-context reference
        graph.add_relationship("order", "product", RelationshipType::References).unwrap();
        
        // Validate boundaries
        let violations = graph.validate_boundaries();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].contains("Cross-context reference"));
    }
}