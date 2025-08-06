//! Event system for graph operations

use crate::core::{GraphId, GraphType};
use serde::{Deserialize, Serialize};

/// Events emitted by graph operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphEvent {
    /// Graph was created
    GraphCreated {
        /// Unique identifier for the graph
        graph_id: GraphId,
        /// Type of graph being created (Generic, Hierarchical, etc.)
        graph_type: GraphType,
    },
    
    /// Node was added
    NodeAdded {
        /// Graph that the node is being added to
        graph_id: GraphId,
        /// Unique identifier for the node within the graph
        node_id: String,
    },
    
    /// Node was removed
    NodeRemoved {
        /// Graph that the node is being removed from
        graph_id: GraphId,
        /// Identifier of the node to remove
        node_id: String,
    },
    
    /// Edge was added
    EdgeAdded {
        /// Graph that the edge is being added to
        graph_id: GraphId,
        /// Unique identifier for the edge
        edge_id: String,
        /// Source node ID for the edge
        source: String,
        /// Target node ID for the edge
        target: String,
    },
    
    /// Edge was removed
    EdgeRemoved {
        /// Graph that the edge is being removed from
        graph_id: GraphId,
        /// Identifier of the edge to remove
        edge_id: String,
    },
    
    /// Graph was cleared
    GraphCleared {
        /// Graph that is being cleared of all nodes and edges
        graph_id: GraphId,
    },
    
    /// Metadata was updated
    MetadataUpdated {
        /// Graph whose metadata is being updated
        graph_id: GraphId,
        /// Name of the metadata field being updated
        field: String,
        /// Previous value of the field (if any)
        old_value: Option<serde_json::Value>,
        /// New value of the field (if any)
        new_value: Option<serde_json::Value>,
    }
}

/// Trait for types that can handle graph events
pub trait EventHandler: Send + Sync {
    /// Handle a graph event
    fn handle_event(&self, event: &GraphEvent);
}

/// Simple event handler that stores events in memory
#[derive(Debug, Default)]
pub struct MemoryEventHandler {
    /// In-memory storage of all handled events
    events: std::sync::Mutex<Vec<GraphEvent>>,
}

impl MemoryEventHandler {
    /// Create a new memory event handler
    pub fn new() -> Self {
        Self {
            events: std::sync::Mutex::new(Vec::new()),
        }
    }
    
    /// Get all stored events
    pub fn events(&self) -> Vec<GraphEvent> {
        self.events.lock().unwrap().clone()
    }
    
    /// Clear all stored events
    pub fn clear(&self) {
        self.events.lock().unwrap().clear();
    }
}

impl EventHandler for MemoryEventHandler {
    fn handle_event(&self, event: &GraphEvent) {
        self.events.lock().unwrap().push(event.clone());
    }
}