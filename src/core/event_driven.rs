//! Pure event-driven graph system for CIM
//! 
//! This is THE ONLY WAY graphs work in CIM. There are no direct state mutations.
//! All changes flow through: Command → Event → State Change
//! 
//! Events are streamed through NATS JetStream with subjects defined by cim-subject.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::core::GraphType;

/// Commands are requests to change state - they may be rejected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphCommand {
    CreateGraph {
        graph_id: Uuid,
        graph_type: GraphType,
        name: Option<String>,
    },
    AddNode {
        graph_id: Uuid,
        node_id: String,
        node_type: String,
        data: serde_json::Value,
    },
    AddEdge {
        graph_id: Uuid,
        edge_id: String,
        source_id: String,
        target_id: String,
        edge_type: String,
        data: serde_json::Value,
    },
    RemoveNode {
        graph_id: Uuid,
        node_id: String,
    },
    RemoveEdge {
        graph_id: Uuid,
        edge_id: String,
    },
}

/// Events are facts - they have happened and cannot be undone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEvent {
    /// Event ID - unique identifier
    pub event_id: Uuid,
    
    /// Stream sequence number from NATS JetStream
    pub sequence: u64,
    
    /// Subject following cim-subject algebra (e.g., "graph.workflow.created")
    pub subject: String,
    
    /// When this event occurred
    pub timestamp: DateTime<Utc>,
    
    /// The aggregate this event belongs to
    pub aggregate_id: Uuid,
    
    /// Correlation ID for related events
    pub correlation_id: Uuid,
    
    /// Event that caused this one (if any)
    pub causation_id: Option<Uuid>,
    
    /// The actual event data
    pub data: EventData,
}

/// The event data variants - these are the ONLY ways state can change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventData {
    GraphCreated {
        graph_type: GraphType,
        name: Option<String>,
    },
    NodeAdded {
        node_id: String,
        node_type: String,
        data: serde_json::Value,
    },
    EdgeAdded {
        edge_id: String,
        source_id: String,
        target_id: String,
        edge_type: String,
        data: serde_json::Value,
    },
    NodeRemoved {
        node_id: String,
    },
    EdgeRemoved {
        edge_id: String,
    },
}

/// Graph projection - computed from event stream
/// This is READ-ONLY - it cannot be modified directly
#[derive(Debug)]
pub struct GraphProjection {
    pub aggregate_id: Uuid,
    pub graph_type: GraphType,
    pub version: u64,
    pub nodes: std::collections::HashMap<String, NodeProjection>,
    pub edges: std::collections::HashMap<String, EdgeProjection>,
}

#[derive(Debug, Clone)]
pub struct NodeProjection {
    pub node_id: String,
    pub node_type: String,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub created_by_event: Uuid,
}

#[derive(Debug, Clone)]
pub struct EdgeProjection {
    pub edge_id: String,
    pub source_id: String,
    pub target_id: String,
    pub edge_type: String,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub created_by_event: Uuid,
}

impl GraphProjection {
    /// Create empty projection
    pub fn new(aggregate_id: Uuid) -> Self {
        Self {
            aggregate_id,
            graph_type: GraphType::Generic,
            version: 0,
            nodes: std::collections::HashMap::new(),
            edges: std::collections::HashMap::new(),
        }
    }
    
    /// Apply event to update projection
    /// This is the ONLY way the projection changes
    pub fn apply(&mut self, event: &GraphEvent) {
        self.version = event.sequence;
        
        match &event.data {
            EventData::GraphCreated { graph_type, .. } => {
                self.graph_type = *graph_type;
            }
            EventData::NodeAdded { node_id, node_type, data } => {
                self.nodes.insert(
                    node_id.clone(),
                    NodeProjection {
                        node_id: node_id.clone(),
                        node_type: node_type.clone(),
                        data: data.clone(),
                        created_at: event.timestamp,
                        created_by_event: event.event_id,
                    },
                );
            }
            EventData::EdgeAdded { edge_id, source_id, target_id, edge_type, data } => {
                self.edges.insert(
                    edge_id.clone(),
                    EdgeProjection {
                        edge_id: edge_id.clone(),
                        source_id: source_id.clone(),
                        target_id: target_id.clone(),
                        edge_type: edge_type.clone(),
                        data: data.clone(),
                        created_at: event.timestamp,
                        created_by_event: event.event_id,
                    },
                );
            }
            EventData::NodeRemoved { node_id } => {
                self.nodes.remove(node_id);
                // Also remove edges connected to this node
                self.edges.retain(|_, edge| {
                    edge.source_id != *node_id && edge.target_id != *node_id
                });
            }
            EventData::EdgeRemoved { edge_id } => {
                self.edges.remove(edge_id);
            }
        }
    }
    
    /// Rebuild projection from event stream
    pub fn from_events(aggregate_id: Uuid, events: impl Iterator<Item = GraphEvent>) -> Self {
        let mut projection = Self::new(aggregate_id);
        for event in events {
            projection.apply(&event);
        }
        projection
    }
}

/// Command handler - validates commands and emits events
pub trait CommandHandler {
    /// Handle a command, returning events to be appended to the stream
    /// Commands can be rejected by returning an error
    fn handle(&self, command: GraphCommand, projection: &GraphProjection) -> Result<Vec<GraphEvent>, String>;
}

/// Event handler - reacts to events (side effects, projections, etc.)
pub trait EventHandler {
    /// Handle an event after it has been committed to the stream
    fn handle(&mut self, event: &GraphEvent);
}

/// Subject builder following cim-subject algebra
pub fn build_subject(graph_type: GraphType, event_type: &str) -> String {
    format!("graph.{}.{}", graph_type.as_str(), event_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_driven_graph() {
        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        
        // Create events - this is the ONLY way to change state
        let events = vec![
            GraphEvent {
                event_id: Uuid::new_v4(),
                sequence: 1,
                subject: build_subject(GraphType::WorkflowGraph, "created"),
                timestamp: Utc::now(),
                aggregate_id,
                correlation_id,
                causation_id: None,
                data: EventData::GraphCreated {
                    graph_type: GraphType::WorkflowGraph,
                    name: Some("Order Workflow".to_string()),
                },
            },
            GraphEvent {
                event_id: Uuid::new_v4(),
                sequence: 2,
                subject: build_subject(GraphType::WorkflowGraph, "node.added"),
                timestamp: Utc::now(),
                aggregate_id,
                correlation_id,
                causation_id: None,
                data: EventData::NodeAdded {
                    node_id: "start".to_string(),
                    node_type: "StartState".to_string(),
                    data: serde_json::json!({"label": "Start"}),
                },
            },
        ];
        
        // Build projection from events
        let projection = GraphProjection::from_events(aggregate_id, events.into_iter());
        
        assert_eq!(projection.version, 2);
        assert_eq!(projection.graph_type, GraphType::WorkflowGraph);
        assert_eq!(projection.nodes.len(), 1);
        assert!(projection.nodes.contains_key("start"));
    }
}