//! Context graph projection - Domain-Driven Design relationships
//!
//! This is a read-only projection computed from events.

use crate::core::{Node, Edge};
use crate::core::projection_engine::GenericGraphProjection;
// Projections are ephemeral - no serialization

/// Context node types
#[derive(Debug, Clone)]
pub enum ContextNodeType {
    /// Bounded context - defines a boundary
    BoundedContext,
    /// Aggregate root - consistency boundary
    Aggregate,
    /// Entity - has identity
    Entity,
    /// Value object - immutable value
    ValueObject,
}

/// Context node
#[derive(Debug, Clone, Default)]
pub struct ContextNode {
    /// Unique identifier for the node
    pub id: String,
    /// Type of DDD element
    pub node_type: ContextNodeType,
    /// Human-readable name
    pub name: String,
    /// Additional data for the node
    pub data: serde_json::Value,
}

impl Node for ContextNode {
    fn id(&self) -> String {
        self.id.clone()
    }
}

impl Default for ContextNodeType {
    fn default() -> Self {
        ContextNodeType::Entity
    }
}

/// Context edge - relationships in DDD
#[derive(Debug, Clone, Default)]
pub struct ContextEdge {
    /// Unique identifier for the edge
    pub id: String,
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Type of relationship (contains, references, etc.)
    pub relationship: String,
}

impl Edge for ContextEdge {
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

/// Context graph projection
pub type ContextProjection = GenericGraphProjection<ContextNode, ContextEdge>;

/// Context graph type alias for backward compatibility
pub type ContextGraph = ContextProjection;