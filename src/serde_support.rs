//! Serialization and deserialization support for graphs
//!
//! Provides JSON serialization/deserialization for all graph types

use crate::core::{GraphType, Node, Edge};
use crate::error::{GraphError, Result};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::HashMap;

/// Serializable graph representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedGraph {
    /// Graph type
    pub graph_type: GraphType,
    /// Graph metadata
    pub metadata: GraphMetadata,
    /// Nodes in the graph
    pub nodes: Vec<SerializedNode>,
    /// Edges in the graph
    pub edges: Vec<SerializedEdge>,
    /// Additional graph-specific data
    pub extra_data: Option<Value>,
}

/// Serializable node representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedNode {
    /// Node ID
    pub id: String,
    /// Node type (for composed graphs)
    pub node_type: Option<String>,
    /// Node data
    pub data: Value,
}

/// Serializable edge representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedEdge {
    /// Edge ID
    pub id: String,
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Edge type (for composed graphs)
    pub edge_type: Option<String>,
    /// Edge data
    pub data: Value,
}

/// Graph metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    /// Graph name
    pub name: Option<String>,
    /// Graph description
    pub description: Option<String>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Custom metadata
    pub custom: HashMap<String, Value>,
}

impl Default for GraphMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            name: None,
            description: None,
            created_at: now,
            updated_at: now,
            custom: HashMap::new(),
        }
    }
}

/// Trait for types that can be serialized to/from graphs
pub trait GraphSerialize: Sized {
    /// Serialize the graph to a SerializedGraph
    fn to_serialized(&self) -> Result<SerializedGraph>;
    
    /// Deserialize from a SerializedGraph
    fn from_serialized(serialized: SerializedGraph) -> Result<Self>;
    
    /// Serialize to JSON string
    fn to_json(&self) -> Result<String> {
        let serialized = self.to_serialized()?;
        serde_json::to_string_pretty(&serialized)
            .map_err(|e| GraphError::SerializationError(e.to_string()))
    }
    
    /// Deserialize from JSON string
    fn from_json(json: &str) -> Result<Self> {
        let serialized: SerializedGraph = serde_json::from_str(json)
            .map_err(|e| GraphError::SerializationError(e.to_string()))?;
        Self::from_serialized(serialized)
    }
    
    /// Save to file
    fn save_to_file(&self, path: &str) -> Result<()> {
        let json = self.to_json()?;
        std::fs::write(path, json)
            .map_err(|e| GraphError::SerializationError(format!("Failed to write file: {}", e)))
    }
    
    /// Load from file
    fn load_from_file(path: &str) -> Result<Self> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| GraphError::SerializationError(format!("Failed to read file: {}", e)))?;
        Self::from_json(&json)
    }
}

/// Helper function to serialize a node
pub fn serialize_node<N: Node + Serialize>(node: &N) -> Result<SerializedNode> {
    let data = serde_json::to_value(node)
        .map_err(|e| GraphError::SerializationError(e.to_string()))?;
    
    Ok(SerializedNode {
        id: node.id(),
        node_type: None,
        data,
    })
}

/// Helper function to serialize an edge
pub fn serialize_edge<E: Edge + Serialize>(edge: &E) -> Result<SerializedEdge> {
    let data = serde_json::to_value(edge)
        .map_err(|e| GraphError::SerializationError(e.to_string()))?;
    
    Ok(SerializedEdge {
        id: edge.id(),
        source: edge.source(),
        target: edge.target(),
        edge_type: None,
        data,
    })
}
#[cfg(test)]
mod tests;
