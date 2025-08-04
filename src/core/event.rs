//! Event system for graph operations

use crate::core::{GraphId, GraphType};
use serde::{Deserialize, Serialize};

/// Events emitted by graph operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphEvent {
    /// Graph was created
    GraphCreated {
        graph_id: GraphId,
        graph_type: GraphType,
    },
    
    /// Node was added
    NodeAdded {
        graph_id: GraphId,
        node_id: String,
    },
    
    /// Node was removed
    NodeRemoved {
        graph_id: GraphId,
        node_id: String,
    },
    
    /// Edge was added
    EdgeAdded {
        graph_id: GraphId,
        edge_id: String,
        source: String,
        target: String,
    },
    
    /// Edge was removed
    EdgeRemoved {
        graph_id: GraphId,
        edge_id: String,
    },
    
    /// Graph was cleared
    GraphCleared {
        graph_id: GraphId,
    },
    
    /// Metadata was updated
    MetadataUpdated {
        graph_id: GraphId,
        field: String,
        old_value: Option<serde_json::Value>,
        new_value: Option<serde_json::Value>,
    },
}

/// Trait for types that can handle graph events
pub trait EventHandler: Send + Sync {
    /// Handle a graph event
    fn handle_event(&self, event: &GraphEvent);
}

/// Simple event handler that stores events in memory
#[derive(Debug, Default)]
pub struct MemoryEventHandler {
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