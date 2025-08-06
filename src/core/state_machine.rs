//! State machine for graph aggregate transitions
//!
//! All state transitions in aggregates happen ONLY through state machines

use crate::events::{GraphEvent, GraphCommand, EventPayload, IpldPayload, ContextPayload, WorkflowPayload, ConceptPayload, ComposedPayload};
use crate::core::aggregate_projection::GraphAggregateProjection;
use crate::error::{GraphError, Result};
use uuid::Uuid;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Graph lifecycle states from event storming
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GraphState {
    /// Graph does not exist yet
    Uninitialized,
    /// Graph has been initialized with a type
    Initialized { 
        /// Type of graph that was initialized
        graph_type: String 
    },
    /// Graph is active with nodes and edges
    Active { 
        /// Number of nodes in the graph
        nodes: usize, 
        /// Number of edges in the graph
        edges: usize 
    },
    /// Graph has been archived
    Archived,
}

/// State machine for graph aggregates
#[derive(Debug)]
pub struct GraphStateMachine {
    /// Valid states for each graph type
    pub valid_states: HashMap<String, Vec<String>>,
    
    /// Valid transitions: (graph_type, from_state, to_state) -> allowed
    pub valid_transitions: HashMap<(String, String, String), bool>,
    
    /// Current aggregate states (aggregate_id -> GraphState)
    aggregate_states: HashMap<Uuid, GraphState>,
}

impl GraphStateMachine {
    /// Create a new state machine with predefined rules
    pub fn new() -> Self {
        let mut valid_states = HashMap::new();
        let mut valid_transitions = HashMap::new();
        
        // Define valid states for workflow graphs
        valid_states.insert("workflow".to_string(), vec![
            "draft".to_string(),
            "published".to_string(),
            "executing".to_string(),
            "completed".to_string(),
            "failed".to_string(),
        ]);
        
        // Define valid transitions for workflows
        valid_transitions.insert(("workflow".to_string(), "draft".to_string(), "published".to_string()), true);
        valid_transitions.insert(("workflow".to_string(), "published".to_string(), "executing".to_string()), true);
        valid_transitions.insert(("workflow".to_string(), "executing".to_string(), "completed".to_string()), true);
        valid_transitions.insert(("workflow".to_string(), "executing".to_string(), "failed".to_string()), true);
        valid_transitions.insert(("workflow".to_string(), "failed".to_string(), "executing".to_string()), true); // Retry
        
        // IPLD graphs are immutable - no state transitions
        valid_states.insert("ipld".to_string(), vec!["immutable".to_string()]);
        
        // Context graphs have bounded context states
        valid_states.insert("context".to_string(), vec![
            "defining".to_string(),
            "bounded".to_string(),
            "integrated".to_string(),
        ]);
        
        // Concept graphs have knowledge states
        valid_states.insert("concept".to_string(), vec![
            "learning".to_string(),
            "reasoning".to_string(),
            "inferring".to_string(),
        ]);
        
        // Composed graphs have composition states
        valid_states.insert("composed".to_string(), vec![
            "composing".to_string(),
            "linked".to_string(),
            "synchronized".to_string(),
        ]);
        
        Self {
            valid_states,
            valid_transitions,
            aggregate_states: HashMap::new(),
        }
    }
    
    /// Get the current state of an aggregate
    pub fn get_state(&self, aggregate_id: &Uuid) -> GraphState {
        self.aggregate_states
            .get(aggregate_id)
            .cloned()
            .unwrap_or(GraphState::Uninitialized)
    }
    
    /// Validate a command can be executed given current projection state
    pub fn validate_command(
        &self,
        command: &GraphCommand,
        _projection: &GraphAggregateProjection,
    ) -> Result<()> {
        let aggregate_id = command.aggregate_id();
        let current_state = self.get_state(&aggregate_id);
        
        match (&current_state, command) {
            // Uninitialized state transitions
            (GraphState::Uninitialized, GraphCommand::InitializeGraph { .. }) => Ok(()),
            (GraphState::Uninitialized, _) => {
                Err(GraphError::InvalidOperation("Graph must be initialized first".to_string()))
            }
            
            // Initialized state transitions
            (GraphState::Initialized { .. }, command) => {
                match command {
                    GraphCommand::Generic { command, .. } => {
                        // Check if this is an AddNode command
                        if command == "AddNode" {
                            Ok(())
                        } else {
                            Err(GraphError::InvalidOperation(
                                "Graph must have nodes before other operations".to_string()
                            ))
                        }
                    }
                    GraphCommand::Ipld { command, .. } => {
                        match command {
                            crate::events::IpldCommand::AddCid { .. } => Ok(()),
                            _ => Err(GraphError::InvalidOperation(
                                "Must add CIDs before creating links".to_string()
                            )),
                        }
                    }
                    GraphCommand::Context { command, .. } => {
                        match command {
                            crate::events::ContextCommand::CreateBoundedContext { .. } => Ok(()),
                            _ => Err(GraphError::InvalidOperation(
                                "Must create bounded context first".to_string()
                            )),
                        }
                    }
                    GraphCommand::Workflow { command, .. } => {
                        match command {
                            crate::events::WorkflowCommand::DefineWorkflow { .. } => Ok(()),
                            _ => Err(GraphError::InvalidOperation(
                                "Must define workflow first".to_string()
                            )),
                        }
                    }
                    GraphCommand::Concept { command, .. } => {
                        match command {
                            crate::events::ConceptCommand::DefineConcept { .. } => Ok(()),
                            _ => Err(GraphError::InvalidOperation(
                                "Must define concept first".to_string()
                            )),
                        }
                    }
                    GraphCommand::Composed { command, .. } => {
                        match command {
                            crate::events::ComposedCommand::AddSubGraph { .. } => Ok(()),
                            _ => Err(GraphError::InvalidOperation(
                                "Must add subgraph first".to_string()
                            )),
                        }
                    }
                    _ => Ok(()),
                }
            }
            
            // Active state transitions
            (GraphState::Active { .. }, GraphCommand::InitializeGraph { .. }) => {
                Err(GraphError::InvalidOperation("Graph already initialized".to_string()))
            }
            (GraphState::Active { .. }, GraphCommand::ArchiveGraph { .. }) => Ok(()),
            (GraphState::Active { .. }, _) => Ok(()), // Most operations allowed
            
            // Archived state transitions
            (GraphState::Archived, _) => {
                Err(GraphError::InvalidOperation("Cannot modify archived graph".to_string()))
            }
        }
    }
    
    /// Apply an event to update state
    pub fn apply_event(&mut self, event: &GraphEvent) {
        let aggregate_id = event.aggregate_id;
        let current_state = self.get_state(&aggregate_id);
        
        let new_state = match (&current_state, &event.payload) {
            // Any state + initialization
            (_, EventPayload::Generic(generic)) if generic.event_type == "GraphInitialized" => {
                let graph_type = generic.data.get("graph_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("generic")
                    .to_string();
                GraphState::Initialized { graph_type }
            }
            
            // Initialized + first node
            (GraphState::Initialized { graph_type: _ }, EventPayload::Generic(generic)) 
                if generic.event_type == "NodeAdded" => {
                GraphState::Active { nodes: 1, edges: 0 }
            }
            
            // Active + node operations
            (GraphState::Active { nodes, edges }, EventPayload::Generic(generic)) => {
                match generic.event_type.as_str() {
                    "NodeAdded" => GraphState::Active { nodes: nodes + 1, edges: *edges },
                    "NodeRemoved" if *nodes > 1 => GraphState::Active { nodes: nodes - 1, edges: *edges },
                    "EdgeAdded" => GraphState::Active { nodes: *nodes, edges: edges + 1 },
                    "EdgeRemoved" if *edges > 0 => GraphState::Active { nodes: *nodes, edges: edges - 1 },
                    _ => current_state.clone(),
                }
            }
            
            // Archive command
            (_, EventPayload::Generic(generic)) if generic.event_type == "GraphArchived" => {
                GraphState::Archived
            }
            
            // Domain-specific initializations
            (GraphState::Uninitialized, EventPayload::Ipld(IpldPayload::CidAdded { .. })) => {
                GraphState::Initialized { graph_type: "ipld".to_string() }
            }
            (GraphState::Uninitialized, EventPayload::Context(ContextPayload::BoundedContextCreated { .. })) => {
                GraphState::Initialized { graph_type: "context".to_string() }
            }
            (GraphState::Uninitialized, EventPayload::Workflow(WorkflowPayload::WorkflowDefined { .. })) => {
                GraphState::Initialized { graph_type: "workflow".to_string() }
            }
            (GraphState::Uninitialized, EventPayload::Concept(ConceptPayload::ConceptDefined { .. })) => {
                GraphState::Initialized { graph_type: "concept".to_string() }
            }
            (GraphState::Uninitialized, EventPayload::Composed(ComposedPayload::SubGraphAdded { .. })) => {
                GraphState::Initialized { graph_type: "composed".to_string() }
            }
            
            // Default: maintain current state
            _ => current_state.clone(),
        };
        
        self.aggregate_states.insert(aggregate_id, new_state);
    }
    
    /// Generate events from a valid command
    pub fn handle_command(
        &mut self,
        command: GraphCommand,
        projection: &GraphAggregateProjection,
    ) -> Result<Vec<GraphEvent>> {
        // First validate
        self.validate_command(&command, projection)?;
        
        // Then generate events
        let mut events = Vec::new();
        
        match command {
            GraphCommand::InitializeGraph { aggregate_id, graph_type, correlation_id } => {
                // Generate initialization event based on graph type
                let payload = match graph_type.as_str() {
                    "ipld" => EventPayload::Ipld(IpldPayload::CidAdded {
                        cid: format!("Qm{}", aggregate_id.to_string().chars().take(16).collect::<String>()),
                        codec: "dag-cbor".to_string(),
                        size: 0,
                        data: serde_json::json!({
                            "type": "ipld_graph",
                            "initialized": true,
                        }),
                    }),
                    "workflow" => EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
                        workflow_id: aggregate_id,
                        name: "New Workflow".to_string(),
                        version: "1.0.0".to_string(),
                    }),
                    "context" => EventPayload::Context(ContextPayload::BoundedContextCreated {
                        context_id: aggregate_id.to_string(),
                        name: "NewContext".to_string(),
                        description: "A new bounded context".to_string(),
                    }),
                    "concept" => EventPayload::Concept(ConceptPayload::ConceptDefined {
                        concept_id: "concept_1".to_string(),
                        name: "NewConcept".to_string(),
                        definition: "A new concept".to_string(),
                    }),
                    "composed" => EventPayload::Composed(ComposedPayload::SubGraphAdded {
                        subgraph_id: Uuid::new_v4(),
                        graph_type: "generic".to_string(),
                        namespace: "default".to_string(),
                    }),
                    _ => EventPayload::Generic(crate::events::GenericPayload {
                        event_type: "GraphInitialized".to_string(),
                        data: serde_json::json!({
                            "graph_type": graph_type,
                            "aggregate_id": aggregate_id,
                        }),
                    }),
                };
                
                let event = GraphEvent {
                    event_id: Uuid::new_v4(),
                    aggregate_id,
                    correlation_id,
                    causation_id: None,
                    payload,
                };
                
                // Apply event to update state
                self.apply_event(&event);
                events.push(event);
            }
            
            GraphCommand::ArchiveGraph { aggregate_id, correlation_id } => {
                let event = GraphEvent {
                    event_id: Uuid::new_v4(),
                    aggregate_id,
                    correlation_id,
                    causation_id: None,
                    payload: EventPayload::Generic(crate::events::GenericPayload {
                        event_type: "GraphArchived".to_string(),
                        data: serde_json::json!({
                            "aggregate_id": aggregate_id,
                            "archived_at": chrono::Utc::now().to_rfc3339(),
                        }),
                    }),
                };
                
                self.apply_event(&event);
                events.push(event);
            }
            
            _ => {
                // Delegate to specific command handlers
                // This is simplified - real implementation would have
                // proper command handlers for each aggregate type
            }
        }
        
        Ok(events)
    }
    
    /// Rebuild state from event history
    pub fn replay_events(&mut self, events: &[GraphEvent]) {
        // Clear current state
        self.aggregate_states.clear();
        
        // Replay all events
        for event in events {
            self.apply_event(event);
        }
    }
}

/// Workflow-specific state machine
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkflowState {
    /// Workflow is being designed
    Draft,
    /// Workflow has been published and is ready to run
    Published,
    /// Workflow is currently executing
    Running { 
        /// Current state in the workflow execution
        current_state: String 
    },
    /// Workflow has completed successfully
    Completed,
    /// Workflow has failed
    Failed { 
        /// Error message describing the failure
        error: String 
    },
}

impl WorkflowState {
    /// Check if transition is valid
    pub fn can_transition_to(&self, new_state: &WorkflowState) -> bool {
        match (self, new_state) {
            (WorkflowState::Draft, WorkflowState::Published) => true,
            (WorkflowState::Published, WorkflowState::Running { .. }) => true,
            (WorkflowState::Running { .. }, WorkflowState::Completed) => true,
            (WorkflowState::Running { .. }, WorkflowState::Failed { .. }) => true,
            (WorkflowState::Failed { .. }, WorkflowState::Running { .. }) => true, // Retry
            _ => false,
        }
    }
}

/// System: Process commands through state machine
pub fn process_command(
    state_machine: &mut GraphStateMachine,
    command: GraphCommand,
    projection: &GraphAggregateProjection,
) -> Result<Vec<GraphEvent>> {
    state_machine.handle_command(command, projection)
}

/// System: Query current state of an aggregate
pub fn get_aggregate_state(
    state_machine: &GraphStateMachine,
    aggregate_id: &Uuid,
) -> GraphState {
    state_machine.get_state(aggregate_id)
}

/// Extension method for GraphCommand to get aggregate_id
impl GraphCommand {
    fn aggregate_id(&self) -> Uuid {
        match self {
            GraphCommand::InitializeGraph { aggregate_id, .. } |
            GraphCommand::ArchiveGraph { aggregate_id, .. } |
            GraphCommand::Generic { aggregate_id, .. } |
            GraphCommand::Ipld { aggregate_id, .. } |
            GraphCommand::Context { aggregate_id, .. } |
            GraphCommand::Workflow { aggregate_id, .. } |
            GraphCommand::Concept { aggregate_id, .. } |
            GraphCommand::Composed { aggregate_id, .. } => *aggregate_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_state_transitions() {
        let mut sm = GraphStateMachine::new();
        let aggregate_id = Uuid::new_v4();
        
        // Start uninitialized
        assert_eq!(sm.get_state(&aggregate_id), GraphState::Uninitialized);
        
        // Initialize event
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Generic(crate::events::GenericPayload {
                event_type: "GraphInitialized".to_string(),
                data: serde_json::json!({ "graph_type": "ipld" }),
            }),
        };
        sm.apply_event(&event);
        
        assert!(matches!(
            sm.get_state(&aggregate_id),
            GraphState::Initialized { graph_type } if graph_type == "ipld"
        ));
        
        // Add node event
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Generic(crate::events::GenericPayload {
                event_type: "NodeAdded".to_string(),
                data: serde_json::json!({ "node_id": "n1" }),
            }),
        };
        sm.apply_event(&event);
        
        assert!(matches!(
            sm.get_state(&aggregate_id),
            GraphState::Active { nodes: 1, edges: 0 }
        ));
    }
    
    #[test]
    fn test_workflow_state_transitions() {
        let draft = WorkflowState::Draft;
        let published = WorkflowState::Published;
        let running = WorkflowState::Running { current_state: "step1".to_string() };
        let completed = WorkflowState::Completed;
        let failed = WorkflowState::Failed { error: "error".to_string() };
        
        // Valid transitions
        assert!(draft.can_transition_to(&published));
        assert!(published.can_transition_to(&running));
        assert!(running.can_transition_to(&completed));
        assert!(running.can_transition_to(&failed));
        assert!(failed.can_transition_to(&running)); // Retry
        
        // Invalid transitions
        assert!(!completed.can_transition_to(&draft));
        assert!(!published.can_transition_to(&completed));
    }
}