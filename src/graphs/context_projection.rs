//! Context graph projection - Domain-Driven Design relationships
//!
//! This is a read-only projection computed from events.

use crate::core::{GraphProjection, Node, Edge};
use crate::core::projection_engine::GenericGraphProjection;
// Projections are ephemeral - no serialization
use uuid::Uuid;

/// Context node types
#[derive(Debug, Clone)]
pub enum ContextNodeType {
    BoundedContext,
    Aggregate,
    Entity,
    ValueObject,
}

/// Context node
#[derive(Debug, Clone, Default)]
pub struct ContextNode {
    pub id: String,
    pub node_type: ContextNodeType,
    pub name: String,
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
    pub id: String,
    pub source: String,
    pub target: String,
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

// Re-export for compatibility
pub type ContextGraph = ContextProjection;