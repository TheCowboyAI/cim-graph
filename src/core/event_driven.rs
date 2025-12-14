//! Pure event-driven graph system for CIM
//! 
//! This is THE ONLY WAY graphs work in CIM. There are no direct state mutations.
//! All changes flow through: Command → Event → State Change
//! 
//! Events are streamed through NATS JetStream with subjects defined by cim-domain's subject module.

use serde::{Deserialize, Serialize};
use cim_domain::{Subject, SubjectSegment};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::core::GraphType;

/// Commands are requests to change state - they may be rejected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphCommand {
    /// Create a new graph
    CreateGraph {
        /// Unique identifier for the graph
        graph_id: Uuid,
        /// Type of graph to create
        graph_type: GraphType,
        /// Optional human-readable name
        name: Option<String>,
    },
    /// Add a node to a graph
    AddNode {
        /// Graph to add the node to
        graph_id: Uuid,
        /// Unique identifier for the node
        node_id: String,
        /// Type/category of the node
        node_type: String,
        /// Additional node data
        data: serde_json::Value,
    },
    /// Add an edge between nodes
    AddEdge {
        /// Graph to add the edge to
        graph_id: Uuid,
        /// Unique identifier for the edge
        edge_id: String,
        /// Source node ID
        source_id: String,
        /// Target node ID
        target_id: String,
        /// Type/category of the edge
        edge_type: String,
        /// Additional edge data
        data: serde_json::Value,
    },
    /// Remove a node from a graph
    RemoveNode {
        /// Graph to remove the node from
        graph_id: Uuid,
        /// ID of the node to remove
        node_id: String,
    },
    /// Remove an edge from a graph
    RemoveEdge {
        /// Graph to remove the edge from
        graph_id: Uuid,
        /// ID of the edge to remove
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
    
    /// Subject following cim-domain's subject module (e.g., "graph.workflow.created")
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
    /// Graph was created
    GraphCreated {
        /// Type of graph being created
        graph_type: GraphType,
        /// Optional name for the graph
        name: Option<String>,
    },
    /// Node was added to the graph
    NodeAdded {
        /// Unique identifier for the node
        node_id: String,
        /// Type/category of the node
        node_type: String,
        /// Additional node data
        data: serde_json::Value,
    },
    /// Edge was added to the graph
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
    /// Node was removed from the graph
    NodeRemoved {
        /// ID of the node that was removed
        node_id: String,
    },
    /// Edge was removed from the graph
    EdgeRemoved {
        /// ID of the edge that was removed
        edge_id: String,
    },
}

/// Graph projection - computed from event stream
/// This is READ-ONLY - it cannot be modified directly
#[derive(Debug)]
pub struct GraphProjection {
    /// Aggregate ID this projection belongs to
    pub aggregate_id: Uuid,
    /// Type of graph
    pub graph_type: GraphType,
    /// Current version (sequence number of last applied event)
    pub version: u64,
    /// All nodes in the graph indexed by node ID
    pub nodes: std::collections::HashMap<String, NodeProjection>,
    /// All edges in the graph indexed by edge ID
    pub edges: std::collections::HashMap<String, EdgeProjection>,
}

/// Node projection - represents a node in the graph
#[derive(Debug, Clone)]
pub struct NodeProjection {
    /// Unique identifier for the node
    pub node_id: String,
    /// Type/category of the node
    pub node_type: String,
    /// Additional node data
    pub data: serde_json::Value,
    /// When the node was created
    pub created_at: DateTime<Utc>,
    /// Event ID that created this node
    pub created_by_event: Uuid,
}

/// Edge projection - represents an edge in the graph
#[derive(Debug, Clone)]
pub struct EdgeProjection {
    /// Unique identifier for the edge
    pub edge_id: String,
    /// ID of the source node
    pub source_id: String,
    /// ID of the target node
    pub target_id: String,
    /// Type/category of the edge
    pub edge_type: String,
    /// Additional edge data
    pub data: serde_json::Value,
    /// When the edge was created
    pub created_at: DateTime<Utc>,
    /// Event ID that created this edge
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

/// Subject builder using cim-domain's subject module conventions
pub fn build_subject(graph_type: GraphType, event_type: &str) -> String {
    // Compose: cim.graph.{type}.{event[.subevent...]}
    let mut segments = Vec::new();
    segments.push(SubjectSegment::new("cim").unwrap());
    segments.push(SubjectSegment::new("graph").unwrap());
    segments.push(SubjectSegment::new(graph_type.as_str()).unwrap());
    for part in event_type.split('.') {
        segments.push(SubjectSegment::new(part).unwrap());
    }
    Subject::from_segments(segments)
        .expect("valid subject segments")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // GraphCommand tests
    // ========================================================================

    #[test]
    fn test_graph_command_create_graph() {
        let graph_id = Uuid::new_v4();
        let cmd = GraphCommand::CreateGraph {
            graph_id,
            graph_type: GraphType::WorkflowGraph,
            name: Some("Test Graph".to_string()),
        };

        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("CreateGraph"));
        assert!(json.contains("Test Graph"));
    }

    #[test]
    fn test_graph_command_add_node() {
        let graph_id = Uuid::new_v4();
        let cmd = GraphCommand::AddNode {
            graph_id,
            node_id: "node1".to_string(),
            node_type: "State".to_string(),
            data: serde_json::json!({"label": "Start"}),
        };

        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("AddNode"));
        assert!(json.contains("node1"));
    }

    #[test]
    fn test_graph_command_add_edge() {
        let graph_id = Uuid::new_v4();
        let cmd = GraphCommand::AddEdge {
            graph_id,
            edge_id: "edge1".to_string(),
            source_id: "node1".to_string(),
            target_id: "node2".to_string(),
            edge_type: "transition".to_string(),
            data: serde_json::json!({}),
        };

        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("AddEdge"));
        assert!(json.contains("edge1"));
    }

    #[test]
    fn test_graph_command_remove_node() {
        let cmd = GraphCommand::RemoveNode {
            graph_id: Uuid::new_v4(),
            node_id: "node_to_remove".to_string(),
        };

        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("RemoveNode"));
        assert!(json.contains("node_to_remove"));
    }

    #[test]
    fn test_graph_command_remove_edge() {
        let cmd = GraphCommand::RemoveEdge {
            graph_id: Uuid::new_v4(),
            edge_id: "edge_to_remove".to_string(),
        };

        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("RemoveEdge"));
    }

    // ========================================================================
    // EventData tests
    // ========================================================================

    #[test]
    fn test_event_data_graph_created() {
        let data = EventData::GraphCreated {
            graph_type: GraphType::IpldGraph,
            name: Some("IPLD Test".to_string()),
        };

        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("GraphCreated"));
        assert!(json.contains("IPLD Test"));
    }

    #[test]
    fn test_event_data_node_added() {
        let data = EventData::NodeAdded {
            node_id: "new_node".to_string(),
            node_type: "State".to_string(),
            data: serde_json::json!({"key": "value"}),
        };

        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("NodeAdded"));
        assert!(json.contains("new_node"));
    }

    #[test]
    fn test_event_data_edge_added() {
        let data = EventData::EdgeAdded {
            edge_id: "edge1".to_string(),
            source_id: "src".to_string(),
            target_id: "tgt".to_string(),
            edge_type: "link".to_string(),
            data: serde_json::json!({}),
        };

        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("EdgeAdded"));
    }

    #[test]
    fn test_event_data_node_removed() {
        let data = EventData::NodeRemoved {
            node_id: "removed_node".to_string(),
        };

        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("NodeRemoved"));
        assert!(json.contains("removed_node"));
    }

    #[test]
    fn test_event_data_edge_removed() {
        let data = EventData::EdgeRemoved {
            edge_id: "removed_edge".to_string(),
        };

        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("EdgeRemoved"));
    }

    // ========================================================================
    // GraphEvent tests
    // ========================================================================

    #[test]
    fn test_graph_event_creation() {
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            sequence: 1,
            subject: "test.subject".to_string(),
            timestamp: Utc::now(),
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            data: EventData::GraphCreated {
                graph_type: GraphType::Generic,
                name: None,
            },
        };

        assert_eq!(event.sequence, 1);
        assert!(event.causation_id.is_none());
    }

    #[test]
    fn test_graph_event_with_causation() {
        let cause_id = Uuid::new_v4();
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            sequence: 2,
            subject: "test.subject".to_string(),
            timestamp: Utc::now(),
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            causation_id: Some(cause_id),
            data: EventData::NodeAdded {
                node_id: "node".to_string(),
                node_type: "type".to_string(),
                data: serde_json::json!({}),
            },
        };

        assert_eq!(event.causation_id, Some(cause_id));
    }

    // ========================================================================
    // GraphProjection tests
    // ========================================================================

    #[test]
    fn test_graph_projection_new() {
        let aggregate_id = Uuid::new_v4();
        let projection = GraphProjection::new(aggregate_id);

        assert_eq!(projection.aggregate_id, aggregate_id);
        assert_eq!(projection.version, 0);
        assert_eq!(projection.graph_type, GraphType::Generic);
        assert!(projection.nodes.is_empty());
        assert!(projection.edges.is_empty());
    }

    #[test]
    fn test_graph_projection_apply_graph_created() {
        let aggregate_id = Uuid::new_v4();
        let mut projection = GraphProjection::new(aggregate_id);

        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            sequence: 1,
            subject: "test".to_string(),
            timestamp: Utc::now(),
            aggregate_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            data: EventData::GraphCreated {
                graph_type: GraphType::ContextGraph,
                name: Some("Context".to_string()),
            },
        };

        projection.apply(&event);

        assert_eq!(projection.version, 1);
        assert_eq!(projection.graph_type, GraphType::ContextGraph);
    }

    #[test]
    fn test_graph_projection_apply_node_added() {
        let aggregate_id = Uuid::new_v4();
        let mut projection = GraphProjection::new(aggregate_id);

        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            sequence: 1,
            subject: "test".to_string(),
            timestamp: Utc::now(),
            aggregate_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            data: EventData::NodeAdded {
                node_id: "test_node".to_string(),
                node_type: "State".to_string(),
                data: serde_json::json!({"label": "Test"}),
            },
        };

        projection.apply(&event);

        assert_eq!(projection.nodes.len(), 1);
        let node = projection.nodes.get("test_node").unwrap();
        assert_eq!(node.node_id, "test_node");
        assert_eq!(node.node_type, "State");
    }

    #[test]
    fn test_graph_projection_apply_edge_added() {
        let aggregate_id = Uuid::new_v4();
        let mut projection = GraphProjection::new(aggregate_id);

        let edge_event = GraphEvent {
            event_id: Uuid::new_v4(),
            sequence: 1,
            subject: "test".to_string(),
            timestamp: Utc::now(),
            aggregate_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            data: EventData::EdgeAdded {
                edge_id: "edge1".to_string(),
                source_id: "node1".to_string(),
                target_id: "node2".to_string(),
                edge_type: "transition".to_string(),
                data: serde_json::json!({"trigger": "submit"}),
            },
        };

        projection.apply(&edge_event);

        assert_eq!(projection.edges.len(), 1);
        let edge = projection.edges.get("edge1").unwrap();
        assert_eq!(edge.source_id, "node1");
        assert_eq!(edge.target_id, "node2");
    }

    #[test]
    fn test_graph_projection_apply_node_removed() {
        let aggregate_id = Uuid::new_v4();
        let mut projection = GraphProjection::new(aggregate_id);

        // Add node first
        let add_event = GraphEvent {
            event_id: Uuid::new_v4(),
            sequence: 1,
            subject: "test".to_string(),
            timestamp: Utc::now(),
            aggregate_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            data: EventData::NodeAdded {
                node_id: "to_remove".to_string(),
                node_type: "State".to_string(),
                data: serde_json::json!({}),
            },
        };
        projection.apply(&add_event);
        assert_eq!(projection.nodes.len(), 1);

        // Remove node
        let remove_event = GraphEvent {
            event_id: Uuid::new_v4(),
            sequence: 2,
            subject: "test".to_string(),
            timestamp: Utc::now(),
            aggregate_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            data: EventData::NodeRemoved {
                node_id: "to_remove".to_string(),
            },
        };
        projection.apply(&remove_event);

        assert!(projection.nodes.is_empty());
    }

    #[test]
    fn test_graph_projection_node_removal_cascades_to_edges() {
        let aggregate_id = Uuid::new_v4();
        let mut projection = GraphProjection::new(aggregate_id);

        // Add edge connected to node
        let edge_event = GraphEvent {
            event_id: Uuid::new_v4(),
            sequence: 1,
            subject: "test".to_string(),
            timestamp: Utc::now(),
            aggregate_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            data: EventData::EdgeAdded {
                edge_id: "edge1".to_string(),
                source_id: "node1".to_string(),
                target_id: "node2".to_string(),
                edge_type: "link".to_string(),
                data: serde_json::json!({}),
            },
        };
        projection.apply(&edge_event);
        assert_eq!(projection.edges.len(), 1);

        // Remove source node - edge should be removed too
        let remove_event = GraphEvent {
            event_id: Uuid::new_v4(),
            sequence: 2,
            subject: "test".to_string(),
            timestamp: Utc::now(),
            aggregate_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            data: EventData::NodeRemoved {
                node_id: "node1".to_string(),
            },
        };
        projection.apply(&remove_event);

        assert!(projection.edges.is_empty());
    }

    #[test]
    fn test_graph_projection_apply_edge_removed() {
        let aggregate_id = Uuid::new_v4();
        let mut projection = GraphProjection::new(aggregate_id);

        // Add edge
        let add_event = GraphEvent {
            event_id: Uuid::new_v4(),
            sequence: 1,
            subject: "test".to_string(),
            timestamp: Utc::now(),
            aggregate_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            data: EventData::EdgeAdded {
                edge_id: "edge_to_remove".to_string(),
                source_id: "a".to_string(),
                target_id: "b".to_string(),
                edge_type: "link".to_string(),
                data: serde_json::json!({}),
            },
        };
        projection.apply(&add_event);

        // Remove edge
        let remove_event = GraphEvent {
            event_id: Uuid::new_v4(),
            sequence: 2,
            subject: "test".to_string(),
            timestamp: Utc::now(),
            aggregate_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            data: EventData::EdgeRemoved {
                edge_id: "edge_to_remove".to_string(),
            },
        };
        projection.apply(&remove_event);

        assert!(projection.edges.is_empty());
    }

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

    #[test]
    fn test_graph_projection_from_empty_events() {
        let aggregate_id = Uuid::new_v4();
        let events: Vec<GraphEvent> = vec![];
        let projection = GraphProjection::from_events(aggregate_id, events.into_iter());

        assert_eq!(projection.version, 0);
        assert!(projection.nodes.is_empty());
        assert!(projection.edges.is_empty());
    }

    // ========================================================================
    // build_subject tests
    // ========================================================================

    #[test]
    fn test_build_subject() {
        let subject = build_subject(GraphType::WorkflowGraph, "created");
        assert!(subject.contains("workflow"));
        assert!(subject.contains("created"));
    }

    #[test]
    fn test_build_subject_with_dots() {
        let subject = build_subject(GraphType::IpldGraph, "node.added");
        assert!(subject.contains("ipld"));
        assert!(subject.contains("node"));
        assert!(subject.contains("added"));
    }

    // ========================================================================
    // NodeProjection and EdgeProjection tests
    // ========================================================================

    #[test]
    fn test_node_projection_clone() {
        let node = NodeProjection {
            node_id: "test".to_string(),
            node_type: "State".to_string(),
            data: serde_json::json!({"key": "value"}),
            created_at: Utc::now(),
            created_by_event: Uuid::new_v4(),
        };

        let cloned = node.clone();
        assert_eq!(node.node_id, cloned.node_id);
        assert_eq!(node.node_type, cloned.node_type);
        assert_eq!(node.data, cloned.data);
    }

    #[test]
    fn test_edge_projection_clone() {
        let edge = EdgeProjection {
            edge_id: "e1".to_string(),
            source_id: "src".to_string(),
            target_id: "tgt".to_string(),
            edge_type: "link".to_string(),
            data: serde_json::json!({}),
            created_at: Utc::now(),
            created_by_event: Uuid::new_v4(),
        };

        let cloned = edge.clone();
        assert_eq!(edge.edge_id, cloned.edge_id);
        assert_eq!(edge.source_id, cloned.source_id);
        assert_eq!(edge.target_id, cloned.target_id);
    }
}
