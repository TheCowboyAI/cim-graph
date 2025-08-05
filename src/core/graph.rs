//! Core graph types for event-driven projections
//!
//! In CIM, graphs are read-only projections computed from event streams.
//! This module contains only the type definitions needed for projections.
//! The actual projection logic is in cim_graph.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Supported graph types with their semantic properties
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphType {
    /// Generic graph with no specific semantics
    Generic,
    /// Content-addressed graph (IPLD)
    IpldGraph,
    /// Domain object relationships (DDD)
    ContextGraph,
    /// State machine graph
    WorkflowGraph,
    /// Semantic reasoning graph
    ConceptGraph,
    /// Multi-type composition
    ComposedGraph,
}

impl GraphType {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            GraphType::Generic => "generic",
            GraphType::IpldGraph => "ipld",
            GraphType::ContextGraph => "context",
            GraphType::WorkflowGraph => "workflow",
            GraphType::ConceptGraph => "concept",
            GraphType::ComposedGraph => "composed",
        }
    }
}

/// Unique identifier for graphs (aggregates in event sourcing)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GraphId(pub Uuid);

impl GraphId {
    /// Create a new unique graph ID
    pub fn new() -> Self {
        GraphId(Uuid::new_v4())
    }
}

impl Default for GraphId {
    fn default() -> Self {
        Self::new()
    }
}

/// Metadata associated with a graph projection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    /// Human-readable name
    pub name: Option<String>,
    /// Description of the graph's purpose
    pub description: Option<String>,
    /// Creation timestamp (from first event)
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last modification timestamp (from last event)
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Version information
    pub version: String,
    /// Additional properties from events
    pub properties: HashMap<String, serde_json::Value>,
}

impl Default for GraphMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            name: None,
            description: None,
            created_at: now,
            updated_at: now,
            version: "1.0.0".to_string(),
            properties: HashMap::new(),
        }
    }
}

// Note: The Graph trait has been removed. Graphs are now represented by
// the GraphProjection trait in cim_graph.rs, which provides read-only access
// to graph state computed from event streams.
//
// For mutable operations, use GraphCommand to request changes, which will
// emit GraphEvent instances if valid. The events update the projections.