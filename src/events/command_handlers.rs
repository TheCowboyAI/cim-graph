//! Command handlers that validate commands and produce events
//!
//! Commands can be rejected based on current state.
//! Valid commands produce events that update projections.

use super::{GraphEvent, GraphCommand, EventPayload};
use super::{IpldCommand, IpldPayload, ContextCommand, ContextPayload};
use super::{WorkflowCommand, WorkflowPayload, ConceptCommand, ConceptPayload};
use super::{ComposedCommand, ComposedPayload};
use crate::core::GraphProjection;
use crate::error::{GraphError, Result};
use uuid::Uuid;
use std::collections::HashMap;

/// Command handler validates commands against current projection state
pub trait CommandHandler<P: GraphProjection> {
    /// Handle a command, returning events if valid
    fn handle(&self, command: GraphCommand, projection: &P) -> Result<Vec<GraphEvent>>;
}

/// Generic command handler for basic graph operations
#[derive(Debug)]
pub struct GenericCommandHandler;

impl<P: GraphProjection> CommandHandler<P> for GenericCommandHandler {
    fn handle(&self, command: GraphCommand, projection: &P) -> Result<Vec<GraphEvent>> {
        match command {
            GraphCommand::InitializeGraph { aggregate_id, graph_type, correlation_id } => {
                // Can only initialize if projection doesn't exist (version 0)
                if projection.version() > 0 {
                    return Err(GraphError::InvalidOperation(
                        "Graph already initialized".to_string()
                    ));
                }
                
                Ok(vec![GraphEvent {
                    event_id: Uuid::new_v4(),
                    aggregate_id,
                    correlation_id,
                    causation_id: None,
                    payload: EventPayload::Ipld(IpldPayload::CidAdded {
                        cid: format!("Qm{}", aggregate_id.to_string().chars().take(16).collect::<String>()),
                        codec: "dag-cbor".to_string(),
                        size: 0,
                        data: serde_json::json!({
                            "type": graph_type,
                            "initialized": true,
                        }),
                    }),
                }])
            }
            _ => Err(GraphError::InvalidOperation(
                "Handler does not support this command type".to_string()
            )),
        }
    }
}

/// IPLD-specific command handler
#[derive(Debug)]
pub struct IpldCommandHandler;

impl<P: GraphProjection> CommandHandler<P> for IpldCommandHandler {
    fn handle(&self, command: GraphCommand, projection: &P) -> Result<Vec<GraphEvent>> {
        match command {
            GraphCommand::Ipld { aggregate_id, correlation_id, command } => {
                let mut events = Vec::new();
                
                match command {
                    IpldCommand::AddCid { cid, codec, size, data } => {
                        // Check if CID already exists
                        if projection.get_node(&cid).is_some() {
                            return Err(GraphError::DuplicateNode(cid));
                        }
                        
                        events.push(GraphEvent {
                            event_id: Uuid::new_v4(),
                            aggregate_id,
                            correlation_id,
                            causation_id: None,
                            payload: EventPayload::Ipld(IpldPayload::CidAdded {
                                cid,
                                codec,
                                size,
                                data,
                            }),
                        });
                    }
                    IpldCommand::LinkCids { source_cid, target_cid, link_name } => {
                        // Validate both CIDs exist
                        if projection.get_node(&source_cid).is_none() {
                            return Err(GraphError::NodeNotFound(source_cid));
                        }
                        if projection.get_node(&target_cid).is_none() {
                            return Err(GraphError::NodeNotFound(target_cid));
                        }
                        
                        events.push(GraphEvent {
                            event_id: Uuid::new_v4(),
                            aggregate_id,
                            correlation_id,
                            causation_id: None,
                            payload: EventPayload::Ipld(IpldPayload::CidLinkAdded {
                                cid: source_cid,
                                link_name,
                                target_cid,
                            }),
                        });
                    }
                    IpldCommand::PinCid { cid, recursive } => {
                        // Validate CID exists
                        if projection.get_node(&cid).is_none() {
                            return Err(GraphError::NodeNotFound(cid));
                        }
                        
                        events.push(GraphEvent {
                            event_id: Uuid::new_v4(),
                            aggregate_id,
                            correlation_id,
                            causation_id: None,
                            payload: EventPayload::Ipld(IpldPayload::CidPinned {
                                cid,
                                recursive,
                            }),
                        });
                    }
                    IpldCommand::UnpinCid { cid } => {
                        // Validate CID exists
                        if projection.get_node(&cid).is_none() {
                            return Err(GraphError::NodeNotFound(cid));
                        }
                        
                        events.push(GraphEvent {
                            event_id: Uuid::new_v4(),
                            aggregate_id,
                            correlation_id,
                            causation_id: None,
                            payload: EventPayload::Ipld(IpldPayload::CidUnpinned { cid }),
                        });
                    }
                }
                
                Ok(events)
            }
            _ => Err(GraphError::InvalidOperation(
                "Not an IPLD command".to_string()
            )),
        }
    }
}

/// Context graph command handler
#[derive(Debug)]
pub struct ContextCommandHandler;

impl<P: GraphProjection> CommandHandler<P> for ContextCommandHandler {
    fn handle(&self, command: GraphCommand, projection: &P) -> Result<Vec<GraphEvent>> {
        match command {
            GraphCommand::Context { aggregate_id, correlation_id, command } => {
                let mut events = Vec::new();
                
                match command {
                    ContextCommand::CreateBoundedContext { context_id, name, description } => {
                        // Check if context already exists
                        if projection.get_node(&context_id).is_some() {
                            return Err(GraphError::DuplicateNode(context_id));
                        }
                        
                        events.push(GraphEvent {
                            event_id: Uuid::new_v4(),
                            aggregate_id,
                            correlation_id,
                            causation_id: None,
                            payload: EventPayload::Context(ContextPayload::BoundedContextCreated {
                                context_id,
                                name,
                                description,
                            }),
                        });
                    }
                    ContextCommand::AddAggregate { context_id, aggregate_id: agg_id, aggregate_type } => {
                        // Validate context exists
                        if projection.get_node(&context_id).is_none() {
                            return Err(GraphError::NodeNotFound(context_id));
                        }
                        
                        events.push(GraphEvent {
                            event_id: Uuid::new_v4(),
                            aggregate_id,
                            correlation_id,
                            causation_id: None,
                            payload: EventPayload::Context(ContextPayload::AggregateAdded {
                                context_id,
                                aggregate_id: agg_id,
                                aggregate_type,
                            }),
                        });
                    }
                    ContextCommand::AddEntity { aggregate_id: agg_id, entity_id, entity_type, properties } => {
                        // Validate aggregate exists
                        if projection.get_node(&agg_id.to_string()).is_none() {
                            return Err(GraphError::NodeNotFound(agg_id.to_string()));
                        }
                        
                        events.push(GraphEvent {
                            event_id: Uuid::new_v4(),
                            aggregate_id,
                            correlation_id,
                            causation_id: None,
                            payload: EventPayload::Context(ContextPayload::EntityAdded {
                                aggregate_id: agg_id,
                                entity_id,
                                entity_type,
                                properties,
                            }),
                        });
                    }
                }
                
                Ok(events)
            }
            _ => Err(GraphError::InvalidOperation(
                "Not a Context command".to_string()
            )),
        }
    }
}

/// Workflow command handler
#[derive(Debug)]
pub struct WorkflowCommandHandler {
    /// Track workflow instances and their current states
    instance_states: HashMap<Uuid, String>,
}

impl WorkflowCommandHandler {
    /// Create a new workflow command handler
    pub fn new() -> Self {
        Self {
            instance_states: HashMap::new(),
        }
    }
}

impl<P: GraphProjection> CommandHandler<P> for WorkflowCommandHandler {
    fn handle(&self, command: GraphCommand, projection: &P) -> Result<Vec<GraphEvent>> {
        match command {
            GraphCommand::Workflow { aggregate_id, correlation_id, command } => {
                let mut events = Vec::new();
                
                match command {
                    WorkflowCommand::DefineWorkflow { workflow_id, name, version } => {
                        events.push(GraphEvent {
                            event_id: Uuid::new_v4(),
                            aggregate_id,
                            correlation_id,
                            causation_id: None,
                            payload: EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
                                workflow_id,
                                name,
                                version,
                            }),
                        });
                    }
                    WorkflowCommand::AddState { workflow_id, state_id, state_type } => {
                        events.push(GraphEvent {
                            event_id: Uuid::new_v4(),
                            aggregate_id,
                            correlation_id,
                            causation_id: None,
                            payload: EventPayload::Workflow(WorkflowPayload::StateAdded {
                                workflow_id,
                                state_id,
                                state_type,
                            }),
                        });
                    }
                    WorkflowCommand::AddTransition { workflow_id, from_state, to_state, trigger } => {
                        // Validate states exist
                        if projection.get_node(&from_state).is_none() {
                            return Err(GraphError::NodeNotFound(from_state));
                        }
                        if projection.get_node(&to_state).is_none() {
                            return Err(GraphError::NodeNotFound(to_state));
                        }
                        
                        events.push(GraphEvent {
                            event_id: Uuid::new_v4(),
                            aggregate_id,
                            correlation_id,
                            causation_id: None,
                            payload: EventPayload::Workflow(WorkflowPayload::TransitionAdded {
                                workflow_id,
                                from_state,
                                to_state,
                                trigger,
                            }),
                        });
                    }
                    WorkflowCommand::CreateInstance { workflow_id, instance_id } => {
                        events.push(GraphEvent {
                            event_id: Uuid::new_v4(),
                            aggregate_id,
                            correlation_id,
                            causation_id: None,
                            payload: EventPayload::Workflow(WorkflowPayload::InstanceCreated {
                                workflow_id,
                                instance_id,
                                initial_state: "initial".to_string(),
                            }),
                        });
                    }
                    WorkflowCommand::TriggerTransition { instance_id, trigger: _ } => {
                        // In real implementation, would validate transition is valid
                        // For now, just create a transition event
                        let from_state = self.instance_states
                            .get(&instance_id)
                            .cloned()
                            .unwrap_or_else(|| "initial".to_string());
                        
                        events.push(GraphEvent {
                            event_id: Uuid::new_v4(),
                            aggregate_id,
                            correlation_id,
                            causation_id: None,
                            payload: EventPayload::Workflow(WorkflowPayload::StateTransitioned {
                                instance_id,
                                from_state,
                                to_state: "next".to_string(), // Would calculate based on trigger
                            }),
                        });
                    }
                }
                
                Ok(events)
            }
            _ => Err(GraphError::InvalidOperation(
                "Not a Workflow command".to_string()
            )),
        }
    }
}

/// Concept graph command handler
#[derive(Debug)]
pub struct ConceptCommandHandler;

impl<P: GraphProjection> CommandHandler<P> for ConceptCommandHandler {
    fn handle(&self, command: GraphCommand, projection: &P) -> Result<Vec<GraphEvent>> {
        match command {
            GraphCommand::Concept { aggregate_id, correlation_id, command } => {
                let mut events = Vec::new();
                
                match command {
                    ConceptCommand::DefineConcept { concept_id, name, definition } => {
                        if projection.get_node(&concept_id).is_some() {
                            return Err(GraphError::DuplicateNode(concept_id));
                        }
                        
                        events.push(GraphEvent {
                            event_id: Uuid::new_v4(),
                            aggregate_id,
                            correlation_id,
                            causation_id: None,
                            payload: EventPayload::Concept(ConceptPayload::ConceptDefined {
                                concept_id,
                                name,
                                definition,
                            }),
                        });
                    }
                    ConceptCommand::AddProperties { concept_id, properties } => {
                        if projection.get_node(&concept_id).is_none() {
                            return Err(GraphError::NodeNotFound(concept_id));
                        }
                        
                        events.push(GraphEvent {
                            event_id: Uuid::new_v4(),
                            aggregate_id,
                            correlation_id,
                            causation_id: None,
                            payload: EventPayload::Concept(ConceptPayload::PropertiesAdded {
                                concept_id,
                                properties,
                            }),
                        });
                    }
                    ConceptCommand::AddRelation { source_concept, target_concept, relation_type, strength } => {
                        if projection.get_node(&source_concept).is_none() {
                            return Err(GraphError::NodeNotFound(source_concept));
                        }
                        if projection.get_node(&target_concept).is_none() {
                            return Err(GraphError::NodeNotFound(target_concept));
                        }
                        
                        events.push(GraphEvent {
                            event_id: Uuid::new_v4(),
                            aggregate_id,
                            correlation_id,
                            causation_id: None,
                            payload: EventPayload::Concept(ConceptPayload::RelationAdded {
                                source_concept,
                                target_concept,
                                relation_type,
                                strength,
                            }),
                        });
                    }
                }
                
                Ok(events)
            }
            _ => Err(GraphError::InvalidOperation(
                "Not a Concept command".to_string()
            )),
        }
    }
}

/// Composed graph command handler
#[derive(Debug)]
pub struct ComposedCommandHandler;

impl<P: GraphProjection> CommandHandler<P> for ComposedCommandHandler {
    fn handle(&self, command: GraphCommand, _projection: &P) -> Result<Vec<GraphEvent>> {
        match command {
            GraphCommand::Composed { aggregate_id, correlation_id, command } => {
                let mut events = Vec::new();

                match command {
                    ComposedCommand::AddSubGraph { subgraph_id, graph_type, namespace } => {
                        events.push(GraphEvent {
                            event_id: Uuid::new_v4(),
                            aggregate_id,
                            correlation_id,
                            causation_id: None,
                            payload: EventPayload::Composed(ComposedPayload::SubGraphAdded {
                                subgraph_id,
                                graph_type,
                                namespace,
                            }),
                        });
                    }
                    ComposedCommand::LinkAcrossGraphs { source_graph, source_node, target_graph, target_node } => {
                        events.push(GraphEvent {
                            event_id: Uuid::new_v4(),
                            aggregate_id,
                            correlation_id,
                            causation_id: None,
                            payload: EventPayload::Composed(ComposedPayload::CrossGraphLinkCreated {
                                source_graph,
                                source_node,
                                target_graph,
                                target_node,
                            }),
                        });
                    }
                }

                Ok(events)
            }
            _ => Err(GraphError::InvalidOperation(
                "Not a Composed command".to_string()
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphs::workflow::{WorkflowNode, WorkflowEdge, WorkflowNodeType};
    use crate::core::projection_engine::GenericGraphProjection;
    use crate::core::GraphType;

    /// Test projection implementation for command handler tests
    type TestProjection = GenericGraphProjection<WorkflowNode, WorkflowEdge>;

    fn create_empty_projection() -> TestProjection {
        GenericGraphProjection::new(Uuid::new_v4(), GraphType::Generic)
    }

    fn create_projection_with_node(node_id: &str) -> TestProjection {
        let mut projection = create_empty_projection();
        // Manually insert a node for testing
        let node = WorkflowNode::new(node_id, WorkflowNodeType::Start);
        projection.nodes.insert(node_id.to_string(), node);
        projection.adjacency.insert(node_id.to_string(), Vec::new());
        projection.version = 1;
        projection
    }

    fn create_projection_with_nodes(node_ids: &[&str]) -> TestProjection {
        let mut projection = create_empty_projection();
        for node_id in node_ids {
            let node = WorkflowNode::new(*node_id, WorkflowNodeType::Start);
            projection.nodes.insert(node_id.to_string(), node);
            projection.adjacency.insert(node_id.to_string(), Vec::new());
        }
        projection.version = node_ids.len() as u64;
        projection
    }

    // ========== GenericCommandHandler Tests ==========

    #[test]
    fn test_generic_handler_initialize_graph_success() {
        let handler = GenericCommandHandler;
        let projection = create_empty_projection();
        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();

        let command = GraphCommand::InitializeGraph {
            aggregate_id,
            graph_type: "workflow".to_string(),
            correlation_id,
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].aggregate_id, aggregate_id);
        assert_eq!(events[0].correlation_id, correlation_id);

        // Verify it produces a CidAdded event with graph initialization data
        match &events[0].payload {
            EventPayload::Ipld(IpldPayload::CidAdded { data, .. }) => {
                assert_eq!(data["type"], "workflow");
                assert_eq!(data["initialized"], true);
            }
            _ => panic!("Expected IpldPayload::CidAdded"),
        }
    }

    #[test]
    fn test_generic_handler_initialize_already_initialized() {
        let handler = GenericCommandHandler;
        let mut projection = create_empty_projection();
        projection.version = 1; // Already initialized

        let command = GraphCommand::InitializeGraph {
            aggregate_id: Uuid::new_v4(),
            graph_type: "workflow".to_string(),
            correlation_id: Uuid::new_v4(),
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_err());

        match result.unwrap_err() {
            GraphError::InvalidOperation(msg) => {
                assert!(msg.contains("already initialized"));
            }
            _ => panic!("Expected InvalidOperation error"),
        }
    }

    #[test]
    fn test_generic_handler_unsupported_command() {
        let handler = GenericCommandHandler;
        let projection = create_empty_projection();

        let command = GraphCommand::Ipld {
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            command: IpldCommand::AddCid {
                cid: "Qm123".to_string(),
                codec: "dag-cbor".to_string(),
                size: 100,
                data: serde_json::json!({}),
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_err());

        match result.unwrap_err() {
            GraphError::InvalidOperation(msg) => {
                assert!(msg.contains("does not support"));
            }
            _ => panic!("Expected InvalidOperation error"),
        }
    }

    // ========== IpldCommandHandler Tests ==========

    #[test]
    fn test_ipld_handler_add_cid_success() {
        let handler = IpldCommandHandler;
        let projection = create_empty_projection();
        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();

        let command = GraphCommand::Ipld {
            aggregate_id,
            correlation_id,
            command: IpldCommand::AddCid {
                cid: "QmTest123".to_string(),
                codec: "dag-cbor".to_string(),
                size: 256,
                data: serde_json::json!({"test": "data"}),
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        match &events[0].payload {
            EventPayload::Ipld(IpldPayload::CidAdded { cid, codec, size, data }) => {
                assert_eq!(cid, "QmTest123");
                assert_eq!(codec, "dag-cbor");
                assert_eq!(*size, 256);
                assert_eq!(data["test"], "data");
            }
            _ => panic!("Expected IpldPayload::CidAdded"),
        }
    }

    #[test]
    fn test_ipld_handler_add_cid_duplicate() {
        let handler = IpldCommandHandler;
        let projection = create_projection_with_node("QmDuplicate");

        let command = GraphCommand::Ipld {
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            command: IpldCommand::AddCid {
                cid: "QmDuplicate".to_string(),
                codec: "dag-cbor".to_string(),
                size: 100,
                data: serde_json::json!({}),
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_err());

        match result.unwrap_err() {
            GraphError::DuplicateNode(cid) => {
                assert_eq!(cid, "QmDuplicate");
            }
            _ => panic!("Expected DuplicateNode error"),
        }
    }

    #[test]
    fn test_ipld_handler_link_cids_success() {
        let handler = IpldCommandHandler;
        let projection = create_projection_with_nodes(&["QmSource", "QmTarget"]);
        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();

        let command = GraphCommand::Ipld {
            aggregate_id,
            correlation_id,
            command: IpldCommand::LinkCids {
                source_cid: "QmSource".to_string(),
                target_cid: "QmTarget".to_string(),
                link_name: "next".to_string(),
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        match &events[0].payload {
            EventPayload::Ipld(IpldPayload::CidLinkAdded { cid, link_name, target_cid }) => {
                assert_eq!(cid, "QmSource");
                assert_eq!(link_name, "next");
                assert_eq!(target_cid, "QmTarget");
            }
            _ => panic!("Expected IpldPayload::CidLinkAdded"),
        }
    }

    #[test]
    fn test_ipld_handler_link_source_not_found() {
        let handler = IpldCommandHandler;
        let projection = create_projection_with_node("QmTarget");

        let command = GraphCommand::Ipld {
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            command: IpldCommand::LinkCids {
                source_cid: "QmMissing".to_string(),
                target_cid: "QmTarget".to_string(),
                link_name: "next".to_string(),
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_err());

        match result.unwrap_err() {
            GraphError::NodeNotFound(cid) => {
                assert_eq!(cid, "QmMissing");
            }
            _ => panic!("Expected NodeNotFound error"),
        }
    }

    #[test]
    fn test_ipld_handler_link_target_not_found() {
        let handler = IpldCommandHandler;
        let projection = create_projection_with_node("QmSource");

        let command = GraphCommand::Ipld {
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            command: IpldCommand::LinkCids {
                source_cid: "QmSource".to_string(),
                target_cid: "QmMissing".to_string(),
                link_name: "next".to_string(),
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_err());

        match result.unwrap_err() {
            GraphError::NodeNotFound(cid) => {
                assert_eq!(cid, "QmMissing");
            }
            _ => panic!("Expected NodeNotFound error"),
        }
    }

    #[test]
    fn test_ipld_handler_pin_cid_success() {
        let handler = IpldCommandHandler;
        let projection = create_projection_with_node("QmToPin");
        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();

        let command = GraphCommand::Ipld {
            aggregate_id,
            correlation_id,
            command: IpldCommand::PinCid {
                cid: "QmToPin".to_string(),
                recursive: true,
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        match &events[0].payload {
            EventPayload::Ipld(IpldPayload::CidPinned { cid, recursive }) => {
                assert_eq!(cid, "QmToPin");
                assert!(*recursive);
            }
            _ => panic!("Expected IpldPayload::CidPinned"),
        }
    }

    #[test]
    fn test_ipld_handler_pin_not_found() {
        let handler = IpldCommandHandler;
        let projection = create_empty_projection();

        let command = GraphCommand::Ipld {
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            command: IpldCommand::PinCid {
                cid: "QmMissing".to_string(),
                recursive: false,
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_err());

        match result.unwrap_err() {
            GraphError::NodeNotFound(cid) => {
                assert_eq!(cid, "QmMissing");
            }
            _ => panic!("Expected NodeNotFound error"),
        }
    }

    #[test]
    fn test_ipld_handler_unpin_cid_success() {
        let handler = IpldCommandHandler;
        let projection = create_projection_with_node("QmToUnpin");
        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();

        let command = GraphCommand::Ipld {
            aggregate_id,
            correlation_id,
            command: IpldCommand::UnpinCid {
                cid: "QmToUnpin".to_string(),
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        match &events[0].payload {
            EventPayload::Ipld(IpldPayload::CidUnpinned { cid }) => {
                assert_eq!(cid, "QmToUnpin");
            }
            _ => panic!("Expected IpldPayload::CidUnpinned"),
        }
    }

    #[test]
    fn test_ipld_handler_unpin_not_found() {
        let handler = IpldCommandHandler;
        let projection = create_empty_projection();

        let command = GraphCommand::Ipld {
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            command: IpldCommand::UnpinCid {
                cid: "QmMissing".to_string(),
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_err());
    }

    #[test]
    fn test_ipld_handler_wrong_command_type() {
        let handler = IpldCommandHandler;
        let projection = create_empty_projection();

        let command = GraphCommand::InitializeGraph {
            aggregate_id: Uuid::new_v4(),
            graph_type: "workflow".to_string(),
            correlation_id: Uuid::new_v4(),
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_err());

        match result.unwrap_err() {
            GraphError::InvalidOperation(msg) => {
                assert!(msg.contains("Not an IPLD command"));
            }
            _ => panic!("Expected InvalidOperation error"),
        }
    }

    // ========== ContextCommandHandler Tests ==========

    #[test]
    fn test_context_handler_create_bounded_context_success() {
        let handler = ContextCommandHandler;
        let projection = create_empty_projection();
        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();

        let command = GraphCommand::Context {
            aggregate_id,
            correlation_id,
            command: ContextCommand::CreateBoundedContext {
                context_id: "sales_context".to_string(),
                name: "Sales".to_string(),
                description: "Sales domain context".to_string(),
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        match &events[0].payload {
            EventPayload::Context(ContextPayload::BoundedContextCreated { context_id, name, description }) => {
                assert_eq!(context_id, "sales_context");
                assert_eq!(name, "Sales");
                assert_eq!(description, "Sales domain context");
            }
            _ => panic!("Expected ContextPayload::BoundedContextCreated"),
        }
    }

    #[test]
    fn test_context_handler_create_bounded_context_duplicate() {
        let handler = ContextCommandHandler;
        let projection = create_projection_with_node("existing_context");

        let command = GraphCommand::Context {
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            command: ContextCommand::CreateBoundedContext {
                context_id: "existing_context".to_string(),
                name: "Existing".to_string(),
                description: "Already exists".to_string(),
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_err());

        match result.unwrap_err() {
            GraphError::DuplicateNode(id) => {
                assert_eq!(id, "existing_context");
            }
            _ => panic!("Expected DuplicateNode error"),
        }
    }

    #[test]
    fn test_context_handler_add_aggregate_success() {
        let handler = ContextCommandHandler;
        let projection = create_projection_with_node("sales_context");
        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let new_aggregate_id = Uuid::new_v4();

        let command = GraphCommand::Context {
            aggregate_id,
            correlation_id,
            command: ContextCommand::AddAggregate {
                context_id: "sales_context".to_string(),
                aggregate_id: new_aggregate_id,
                aggregate_type: "Order".to_string(),
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        match &events[0].payload {
            EventPayload::Context(ContextPayload::AggregateAdded { context_id, aggregate_id: agg_id, aggregate_type }) => {
                assert_eq!(context_id, "sales_context");
                assert_eq!(*agg_id, new_aggregate_id);
                assert_eq!(aggregate_type, "Order");
            }
            _ => panic!("Expected ContextPayload::AggregateAdded"),
        }
    }

    #[test]
    fn test_context_handler_add_aggregate_context_not_found() {
        let handler = ContextCommandHandler;
        let projection = create_empty_projection();

        let command = GraphCommand::Context {
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            command: ContextCommand::AddAggregate {
                context_id: "missing_context".to_string(),
                aggregate_id: Uuid::new_v4(),
                aggregate_type: "Order".to_string(),
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_err());

        match result.unwrap_err() {
            GraphError::NodeNotFound(id) => {
                assert_eq!(id, "missing_context");
            }
            _ => panic!("Expected NodeNotFound error"),
        }
    }

    #[test]
    fn test_context_handler_add_entity_success() {
        let handler = ContextCommandHandler;
        let agg_id = Uuid::new_v4();
        let projection = create_projection_with_node(&agg_id.to_string());
        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let entity_id = Uuid::new_v4();

        let command = GraphCommand::Context {
            aggregate_id,
            correlation_id,
            command: ContextCommand::AddEntity {
                aggregate_id: agg_id,
                entity_id,
                entity_type: "OrderItem".to_string(),
                properties: serde_json::json!({"quantity": 5, "price": 10.0}),
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        match &events[0].payload {
            EventPayload::Context(ContextPayload::EntityAdded { aggregate_id: a_id, entity_id: e_id, entity_type, properties }) => {
                assert_eq!(*a_id, agg_id);
                assert_eq!(*e_id, entity_id);
                assert_eq!(entity_type, "OrderItem");
                assert_eq!(properties["quantity"], 5);
            }
            _ => panic!("Expected ContextPayload::EntityAdded"),
        }
    }

    #[test]
    fn test_context_handler_add_entity_aggregate_not_found() {
        let handler = ContextCommandHandler;
        let projection = create_empty_projection();

        let command = GraphCommand::Context {
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            command: ContextCommand::AddEntity {
                aggregate_id: Uuid::new_v4(),
                entity_id: Uuid::new_v4(),
                entity_type: "OrderItem".to_string(),
                properties: serde_json::json!({}),
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_err());

        match result.unwrap_err() {
            GraphError::NodeNotFound(_) => {}
            _ => panic!("Expected NodeNotFound error"),
        }
    }

    #[test]
    fn test_context_handler_wrong_command_type() {
        let handler = ContextCommandHandler;
        let projection = create_empty_projection();

        let command = GraphCommand::InitializeGraph {
            aggregate_id: Uuid::new_v4(),
            graph_type: "workflow".to_string(),
            correlation_id: Uuid::new_v4(),
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_err());

        match result.unwrap_err() {
            GraphError::InvalidOperation(msg) => {
                assert!(msg.contains("Not a Context command"));
            }
            _ => panic!("Expected InvalidOperation error"),
        }
    }

    // ========== WorkflowCommandHandler Tests ==========

    #[test]
    fn test_workflow_handler_define_workflow() {
        let handler = WorkflowCommandHandler::new();
        let projection = create_empty_projection();
        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let workflow_id = Uuid::new_v4();

        let command = GraphCommand::Workflow {
            aggregate_id,
            correlation_id,
            command: WorkflowCommand::DefineWorkflow {
                workflow_id,
                name: "Order Processing".to_string(),
                version: "1.0.0".to_string(),
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        match &events[0].payload {
            EventPayload::Workflow(WorkflowPayload::WorkflowDefined { workflow_id: wf_id, name, version }) => {
                assert_eq!(*wf_id, workflow_id);
                assert_eq!(name, "Order Processing");
                assert_eq!(version, "1.0.0");
            }
            _ => panic!("Expected WorkflowPayload::WorkflowDefined"),
        }
    }

    #[test]
    fn test_workflow_handler_add_state() {
        let handler = WorkflowCommandHandler::new();
        let projection = create_empty_projection();
        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let workflow_id = Uuid::new_v4();

        let command = GraphCommand::Workflow {
            aggregate_id,
            correlation_id,
            command: WorkflowCommand::AddState {
                workflow_id,
                state_id: "pending".to_string(),
                state_type: "intermediate".to_string(),
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        match &events[0].payload {
            EventPayload::Workflow(WorkflowPayload::StateAdded { workflow_id: wf_id, state_id, state_type }) => {
                assert_eq!(*wf_id, workflow_id);
                assert_eq!(state_id, "pending");
                assert_eq!(state_type, "intermediate");
            }
            _ => panic!("Expected WorkflowPayload::StateAdded"),
        }
    }

    #[test]
    fn test_workflow_handler_add_transition_success() {
        let handler = WorkflowCommandHandler::new();
        let projection = create_projection_with_nodes(&["pending", "approved"]);
        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let workflow_id = Uuid::new_v4();

        let command = GraphCommand::Workflow {
            aggregate_id,
            correlation_id,
            command: WorkflowCommand::AddTransition {
                workflow_id,
                from_state: "pending".to_string(),
                to_state: "approved".to_string(),
                trigger: "approve".to_string(),
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        match &events[0].payload {
            EventPayload::Workflow(WorkflowPayload::TransitionAdded { from_state, to_state, trigger, .. }) => {
                assert_eq!(from_state, "pending");
                assert_eq!(to_state, "approved");
                assert_eq!(trigger, "approve");
            }
            _ => panic!("Expected WorkflowPayload::TransitionAdded"),
        }
    }

    #[test]
    fn test_workflow_handler_add_transition_from_state_not_found() {
        let handler = WorkflowCommandHandler::new();
        let projection = create_projection_with_node("approved");

        let command = GraphCommand::Workflow {
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            command: WorkflowCommand::AddTransition {
                workflow_id: Uuid::new_v4(),
                from_state: "missing".to_string(),
                to_state: "approved".to_string(),
                trigger: "approve".to_string(),
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_err());

        match result.unwrap_err() {
            GraphError::NodeNotFound(id) => {
                assert_eq!(id, "missing");
            }
            _ => panic!("Expected NodeNotFound error"),
        }
    }

    #[test]
    fn test_workflow_handler_add_transition_to_state_not_found() {
        let handler = WorkflowCommandHandler::new();
        let projection = create_projection_with_node("pending");

        let command = GraphCommand::Workflow {
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            command: WorkflowCommand::AddTransition {
                workflow_id: Uuid::new_v4(),
                from_state: "pending".to_string(),
                to_state: "missing".to_string(),
                trigger: "approve".to_string(),
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_err());

        match result.unwrap_err() {
            GraphError::NodeNotFound(id) => {
                assert_eq!(id, "missing");
            }
            _ => panic!("Expected NodeNotFound error"),
        }
    }

    #[test]
    fn test_workflow_handler_create_instance() {
        let handler = WorkflowCommandHandler::new();
        let projection = create_empty_projection();
        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let workflow_id = Uuid::new_v4();
        let instance_id = Uuid::new_v4();

        let command = GraphCommand::Workflow {
            aggregate_id,
            correlation_id,
            command: WorkflowCommand::CreateInstance {
                workflow_id,
                instance_id,
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        match &events[0].payload {
            EventPayload::Workflow(WorkflowPayload::InstanceCreated { workflow_id: wf_id, instance_id: inst_id, initial_state }) => {
                assert_eq!(*wf_id, workflow_id);
                assert_eq!(*inst_id, instance_id);
                assert_eq!(initial_state, "initial");
            }
            _ => panic!("Expected WorkflowPayload::InstanceCreated"),
        }
    }

    #[test]
    fn test_workflow_handler_trigger_transition() {
        let handler = WorkflowCommandHandler::new();
        let projection = create_empty_projection();
        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let instance_id = Uuid::new_v4();

        let command = GraphCommand::Workflow {
            aggregate_id,
            correlation_id,
            command: WorkflowCommand::TriggerTransition {
                instance_id,
                trigger: "approve".to_string(),
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        match &events[0].payload {
            EventPayload::Workflow(WorkflowPayload::StateTransitioned { instance_id: inst_id, from_state, to_state }) => {
                assert_eq!(*inst_id, instance_id);
                assert_eq!(from_state, "initial"); // Default when not tracked
                assert_eq!(to_state, "next");
            }
            _ => panic!("Expected WorkflowPayload::StateTransitioned"),
        }
    }

    #[test]
    fn test_workflow_handler_wrong_command_type() {
        let handler = WorkflowCommandHandler::new();
        let projection = create_empty_projection();

        let command = GraphCommand::InitializeGraph {
            aggregate_id: Uuid::new_v4(),
            graph_type: "workflow".to_string(),
            correlation_id: Uuid::new_v4(),
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_err());

        match result.unwrap_err() {
            GraphError::InvalidOperation(msg) => {
                assert!(msg.contains("Not a Workflow command"));
            }
            _ => panic!("Expected InvalidOperation error"),
        }
    }

    // ========== ConceptCommandHandler Tests ==========

    #[test]
    fn test_concept_handler_define_concept_success() {
        let handler = ConceptCommandHandler;
        let projection = create_empty_projection();
        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();

        let command = GraphCommand::Concept {
            aggregate_id,
            correlation_id,
            command: ConceptCommand::DefineConcept {
                concept_id: "animal".to_string(),
                name: "Animal".to_string(),
                definition: "A living organism that feeds on organic matter".to_string(),
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        match &events[0].payload {
            EventPayload::Concept(ConceptPayload::ConceptDefined { concept_id, name, definition }) => {
                assert_eq!(concept_id, "animal");
                assert_eq!(name, "Animal");
                assert!(definition.contains("organic matter"));
            }
            _ => panic!("Expected ConceptPayload::ConceptDefined"),
        }
    }

    #[test]
    fn test_concept_handler_define_concept_duplicate() {
        let handler = ConceptCommandHandler;
        let projection = create_projection_with_node("animal");

        let command = GraphCommand::Concept {
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            command: ConceptCommand::DefineConcept {
                concept_id: "animal".to_string(),
                name: "Animal".to_string(),
                definition: "Already exists".to_string(),
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_err());

        match result.unwrap_err() {
            GraphError::DuplicateNode(id) => {
                assert_eq!(id, "animal");
            }
            _ => panic!("Expected DuplicateNode error"),
        }
    }

    #[test]
    fn test_concept_handler_add_properties_success() {
        let handler = ConceptCommandHandler;
        let projection = create_projection_with_node("animal");
        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();

        let command = GraphCommand::Concept {
            aggregate_id,
            correlation_id,
            command: ConceptCommand::AddProperties {
                concept_id: "animal".to_string(),
                properties: vec![
                    ("has_legs".to_string(), 0.95),
                    ("breathes".to_string(), 1.0),
                ],
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        match &events[0].payload {
            EventPayload::Concept(ConceptPayload::PropertiesAdded { concept_id, properties }) => {
                assert_eq!(concept_id, "animal");
                assert_eq!(properties.len(), 2);
                assert_eq!(properties[0].0, "has_legs");
                assert!((properties[0].1 - 0.95).abs() < 0.001);
            }
            _ => panic!("Expected ConceptPayload::PropertiesAdded"),
        }
    }

    #[test]
    fn test_concept_handler_add_properties_concept_not_found() {
        let handler = ConceptCommandHandler;
        let projection = create_empty_projection();

        let command = GraphCommand::Concept {
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            command: ConceptCommand::AddProperties {
                concept_id: "missing".to_string(),
                properties: vec![("test".to_string(), 1.0)],
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_err());

        match result.unwrap_err() {
            GraphError::NodeNotFound(id) => {
                assert_eq!(id, "missing");
            }
            _ => panic!("Expected NodeNotFound error"),
        }
    }

    #[test]
    fn test_concept_handler_add_relation_success() {
        let handler = ConceptCommandHandler;
        let projection = create_projection_with_nodes(&["animal", "dog"]);
        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();

        let command = GraphCommand::Concept {
            aggregate_id,
            correlation_id,
            command: ConceptCommand::AddRelation {
                source_concept: "dog".to_string(),
                target_concept: "animal".to_string(),
                relation_type: "is-a".to_string(),
                strength: 1.0,
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        match &events[0].payload {
            EventPayload::Concept(ConceptPayload::RelationAdded { source_concept, target_concept, relation_type, strength }) => {
                assert_eq!(source_concept, "dog");
                assert_eq!(target_concept, "animal");
                assert_eq!(relation_type, "is-a");
                assert!((strength - 1.0).abs() < 0.001);
            }
            _ => panic!("Expected ConceptPayload::RelationAdded"),
        }
    }

    #[test]
    fn test_concept_handler_add_relation_source_not_found() {
        let handler = ConceptCommandHandler;
        let projection = create_projection_with_node("animal");

        let command = GraphCommand::Concept {
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            command: ConceptCommand::AddRelation {
                source_concept: "missing".to_string(),
                target_concept: "animal".to_string(),
                relation_type: "is-a".to_string(),
                strength: 1.0,
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_err());

        match result.unwrap_err() {
            GraphError::NodeNotFound(id) => {
                assert_eq!(id, "missing");
            }
            _ => panic!("Expected NodeNotFound error"),
        }
    }

    #[test]
    fn test_concept_handler_add_relation_target_not_found() {
        let handler = ConceptCommandHandler;
        let projection = create_projection_with_node("dog");

        let command = GraphCommand::Concept {
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            command: ConceptCommand::AddRelation {
                source_concept: "dog".to_string(),
                target_concept: "missing".to_string(),
                relation_type: "is-a".to_string(),
                strength: 1.0,
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_err());

        match result.unwrap_err() {
            GraphError::NodeNotFound(id) => {
                assert_eq!(id, "missing");
            }
            _ => panic!("Expected NodeNotFound error"),
        }
    }

    #[test]
    fn test_concept_handler_wrong_command_type() {
        let handler = ConceptCommandHandler;
        let projection = create_empty_projection();

        let command = GraphCommand::InitializeGraph {
            aggregate_id: Uuid::new_v4(),
            graph_type: "concept".to_string(),
            correlation_id: Uuid::new_v4(),
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_err());

        match result.unwrap_err() {
            GraphError::InvalidOperation(msg) => {
                assert!(msg.contains("Not a Concept command"));
            }
            _ => panic!("Expected InvalidOperation error"),
        }
    }

    // ========== ComposedCommandHandler Tests ==========

    #[test]
    fn test_composed_handler_add_subgraph() {
        let handler = ComposedCommandHandler;
        let projection = create_empty_projection();
        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let subgraph_id = Uuid::new_v4();

        let command = GraphCommand::Composed {
            aggregate_id,
            correlation_id,
            command: ComposedCommand::AddSubGraph {
                subgraph_id,
                graph_type: "workflow".to_string(),
                namespace: "orders".to_string(),
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        match &events[0].payload {
            EventPayload::Composed(ComposedPayload::SubGraphAdded { subgraph_id: sg_id, graph_type, namespace }) => {
                assert_eq!(*sg_id, subgraph_id);
                assert_eq!(graph_type, "workflow");
                assert_eq!(namespace, "orders");
            }
            _ => panic!("Expected ComposedPayload::SubGraphAdded"),
        }
    }

    #[test]
    fn test_composed_handler_link_across_graphs() {
        let handler = ComposedCommandHandler;
        let projection = create_empty_projection();
        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let source_graph = Uuid::new_v4();
        let target_graph = Uuid::new_v4();

        let command = GraphCommand::Composed {
            aggregate_id,
            correlation_id,
            command: ComposedCommand::LinkAcrossGraphs {
                source_graph,
                source_node: "order_123".to_string(),
                target_graph,
                target_node: "customer_456".to_string(),
            },
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        match &events[0].payload {
            EventPayload::Composed(ComposedPayload::CrossGraphLinkCreated { source_graph: sg, source_node, target_graph: tg, target_node }) => {
                assert_eq!(*sg, source_graph);
                assert_eq!(source_node, "order_123");
                assert_eq!(*tg, target_graph);
                assert_eq!(target_node, "customer_456");
            }
            _ => panic!("Expected ComposedPayload::CrossGraphLinkCreated"),
        }
    }

    #[test]
    fn test_composed_handler_wrong_command_type() {
        let handler = ComposedCommandHandler;
        let projection = create_empty_projection();

        let command = GraphCommand::InitializeGraph {
            aggregate_id: Uuid::new_v4(),
            graph_type: "composed".to_string(),
            correlation_id: Uuid::new_v4(),
        };

        let result = handler.handle(command, &projection);
        assert!(result.is_err());

        match result.unwrap_err() {
            GraphError::InvalidOperation(msg) => {
                assert!(msg.contains("Not a Composed command"));
            }
            _ => panic!("Expected InvalidOperation error"),
        }
    }

    // ========== Event Correlation and Causation Tests ==========

    #[test]
    fn test_events_have_correct_correlation_id() {
        let handler = IpldCommandHandler;
        let projection = create_empty_projection();
        let correlation_id = Uuid::new_v4();

        let command = GraphCommand::Ipld {
            aggregate_id: Uuid::new_v4(),
            correlation_id,
            command: IpldCommand::AddCid {
                cid: "QmTest".to_string(),
                codec: "dag-cbor".to_string(),
                size: 100,
                data: serde_json::json!({}),
            },
        };

        let events = handler.handle(command, &projection).unwrap();
        assert_eq!(events[0].correlation_id, correlation_id);
    }

    #[test]
    fn test_events_have_unique_event_ids() {
        let handler = IpldCommandHandler;
        let projection = create_empty_projection();

        let command1 = GraphCommand::Ipld {
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            command: IpldCommand::AddCid {
                cid: "QmTest1".to_string(),
                codec: "dag-cbor".to_string(),
                size: 100,
                data: serde_json::json!({}),
            },
        };

        let command2 = GraphCommand::Ipld {
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            command: IpldCommand::AddCid {
                cid: "QmTest2".to_string(),
                codec: "dag-cbor".to_string(),
                size: 100,
                data: serde_json::json!({}),
            },
        };

        let events1 = handler.handle(command1, &projection).unwrap();
        let events2 = handler.handle(command2, &projection).unwrap();

        assert_ne!(events1[0].event_id, events2[0].event_id);
    }

    #[test]
    fn test_events_have_correct_aggregate_id() {
        let handler = WorkflowCommandHandler::new();
        let projection = create_empty_projection();
        let aggregate_id = Uuid::new_v4();

        let command = GraphCommand::Workflow {
            aggregate_id,
            correlation_id: Uuid::new_v4(),
            command: WorkflowCommand::DefineWorkflow {
                workflow_id: Uuid::new_v4(),
                name: "Test".to_string(),
                version: "1.0.0".to_string(),
            },
        };

        let events = handler.handle(command, &projection).unwrap();
        assert_eq!(events[0].aggregate_id, aggregate_id);
    }
}