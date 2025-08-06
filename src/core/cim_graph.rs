//! The CORRECT CIM graph implementation - graphs as event projections
//! 
//! In CIM, graphs are NOT mutable data structures. They are read-only
//! projections computed from event streams. This has ALWAYS been the design.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use crate::Result;

/// Graph projection - a read-only view computed from events
/// This is what a graph actually is in CIM
pub trait GraphProjection {
    /// Node type for this projection
    type Node;
    /// Edge type for this projection
    type Edge;
    
    /// Get the aggregate ID this projection represents
    fn aggregate_id(&self) -> Uuid;
    
    /// Get the current version (last event sequence number)
    fn version(&self) -> u64;
    
    /// Get a node by ID (read-only)
    fn get_node(&self, node_id: &str) -> Option<&Self::Node>;
    
    /// Get an edge by ID (read-only)
    fn get_edge(&self, edge_id: &str) -> Option<&Self::Edge>;
    
    /// Get all nodes (read-only)
    fn nodes(&self) -> Vec<&Self::Node>;
    
    /// Get all edges (read-only)
    fn edges(&self) -> Vec<&Self::Edge>;
    
    /// Get node count
    fn node_count(&self) -> usize;
    
    /// Get edge count
    fn edge_count(&self) -> usize;
    
    /// Find edges between nodes
    fn edges_between(&self, from: &str, to: &str) -> Vec<&Self::Edge>;
    
    /// Get neighbors of a node
    fn neighbors(&self, node_id: &str) -> Vec<&str>;
}

/// Events are the ONLY way to change graph state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEvent {
    /// Unique event ID
    pub event_id: Uuid,
    
    /// Aggregate this event belongs to
    pub aggregate_id: Uuid,
    
    /// Event sequence number (from NATS JetStream)
    pub sequence: u64,
    
    /// NATS subject (from cim-subject algebra)
    pub subject: String,
    
    /// When this happened
    pub timestamp: DateTime<Utc>,
    
    /// Correlation ID for related events
    pub correlation_id: Uuid,
    
    /// Event that caused this one
    pub causation_id: Option<Uuid>,
    
    /// The actual event data
    pub data: EventData,
}

/// Event data variants - these define ALL possible state changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventData {
    /// Graph initialized
    GraphInitialized {
        /// Type of graph (e.g., "workflow", "ipld", "composed")
        graph_type: String,
        /// Initial metadata for the graph
        metadata: HashMap<String, serde_json::Value>,
    },
    
    /// Node added to graph
    NodeAdded {
        /// Unique identifier for the node
        node_id: String,
        /// Type/category of the node
        node_type: String,
        /// Additional node data
        data: serde_json::Value,
    },
    
    /// Edge added between nodes
    EdgeAdded {
        /// Unique identifier for the edge
        edge_id: String,
        /// ID of the source node
        source_id: String,
        /// ID of the target node
        target_id: String,
        /// Type/category of the edge
        edge_type: String,
        /// Additional edge data
        data: serde_json::Value,
    },
    
    /// Node removed
    NodeRemoved {
        /// ID of the node to remove
        node_id: String,
    },
    
    /// Edge removed
    EdgeRemoved {
        /// ID of the edge to remove
        edge_id: String,
    },
    
    /// Node data updated
    NodeUpdated {
        /// ID of the node to update
        node_id: String,
        /// New data for the node
        data: serde_json::Value,
    },
    
    /// Edge data updated
    EdgeUpdated {
        /// ID of the edge to update
        edge_id: String,
        /// New data for the edge
        data: serde_json::Value,
    },
}

/// Commands request state changes - they can be rejected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphCommand {
    /// Initialize a new graph
    InitializeGraph {
        /// Aggregate ID for the graph
        aggregate_id: Uuid,
        /// Type of graph to create
        graph_type: String,
        /// Initial metadata for the graph
        metadata: HashMap<String, serde_json::Value>,
    },
    
    /// Add a node
    AddNode {
        /// Graph aggregate ID
        aggregate_id: Uuid,
        /// Unique identifier for the node
        node_id: String,
        /// Type/category of the node
        node_type: String,
        /// Additional node data
        data: serde_json::Value,
    },
    
    /// Add an edge
    AddEdge {
        /// Graph aggregate ID
        aggregate_id: Uuid,
        /// Unique identifier for the edge
        edge_id: String,
        /// ID of the source node
        source_id: String,
        /// ID of the target node
        target_id: String,
        /// Type/category of the edge
        edge_type: String,
        /// Additional edge data
        data: serde_json::Value,
    },
    
    /// Remove a node
    RemoveNode {
        /// Graph aggregate ID
        aggregate_id: Uuid,
        /// ID of the node to remove
        node_id: String,
    },
    
    /// Remove an edge
    RemoveEdge {
        /// Graph aggregate ID
        aggregate_id: Uuid,
        /// ID of the edge to remove
        edge_id: String,
    },
}

/// Command handler validates commands and produces events
pub trait CommandHandler<P: GraphProjection> {
    /// Handle a command, returning events if valid
    fn handle(&self, command: GraphCommand, projection: &P) -> Result<Vec<GraphEvent>>;
}

/// Event handler processes events (projections, side effects, etc)
pub trait EventHandler {
    /// Process an event after it's committed to the stream
    fn handle(&mut self, event: &GraphEvent);
}

/// Projector builds projections from event streams
pub trait Projector<P: GraphProjection> {
    /// Build projection from events
    fn project(&self, events: Vec<GraphEvent>) -> P;
    
    /// Update existing projection with new event
    fn apply(&self, projection: &mut P, event: &GraphEvent);
}