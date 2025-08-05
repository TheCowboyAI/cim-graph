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
pub struct WorkflowCommandHandler {
    /// Track workflow instances and their current states
    instance_states: HashMap<Uuid, String>,
}

impl WorkflowCommandHandler {
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
                    WorkflowCommand::TriggerTransition { instance_id, trigger } => {
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
pub struct ComposedCommandHandler;

impl<P: GraphProjection> CommandHandler<P> for ComposedCommandHandler {
    fn handle(&self, command: GraphCommand, projection: &P) -> Result<Vec<GraphEvent>> {
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