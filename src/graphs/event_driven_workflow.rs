//! Event-driven workflow graph - a projection of workflow events
//! 
//! This demonstrates how workflow graphs work in the pure event-driven model.
//! The workflow is ONLY changed through events - there are no mutation methods.

use crate::core::event_driven::{GraphEvent, EventData, GraphProjection};
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Workflow-specific event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowEventData {
    /// Transition was triggered
    TransitionTriggered {
        from_state: String,
        to_state: String,
        trigger: String,
    },
    /// State was entered
    StateEntered {
        state_id: String,
    },
    /// State was exited
    StateExited {
        state_id: String,
    },
    /// Workflow completed
    WorkflowCompleted {
        final_state: String,
    },
}

/// Workflow projection - computed from event stream
#[derive(Debug)]
pub struct WorkflowProjection {
    /// Base graph projection
    pub graph: GraphProjection,
    
    /// Currently active states (supports parallel states)
    pub active_states: HashSet<String>,
    
    /// Transition history for debugging
    pub transition_history: Vec<TransitionRecord>,
    
    /// Is workflow completed?
    pub is_completed: bool,
}

#[derive(Debug, Clone)]
pub struct TransitionRecord {
    pub from_state: String,
    pub to_state: String,
    pub trigger: String,
    pub event_id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl WorkflowProjection {
    /// Create new workflow projection
    pub fn new(aggregate_id: Uuid) -> Self {
        Self {
            graph: GraphProjection::new(aggregate_id),
            active_states: HashSet::new(),
            transition_history: Vec::new(),
            is_completed: false,
        }
    }
    
    /// Apply event to update projection
    pub fn apply(&mut self, event: &GraphEvent) {
        // First apply base graph event
        self.graph.apply(event);
        
        // Then apply workflow-specific events
        if let Ok(workflow_data) = serde_json::from_value::<WorkflowEventData>(
            serde_json::to_value(&event.data).unwrap_or_default()
        ) {
            match workflow_data {
                WorkflowEventData::StateEntered { state_id } => {
                    self.active_states.insert(state_id);
                }
                WorkflowEventData::StateExited { state_id } => {
                    self.active_states.remove(&state_id);
                }
                WorkflowEventData::TransitionTriggered { from_state, to_state, trigger } => {
                    self.transition_history.push(TransitionRecord {
                        from_state,
                        to_state,
                        trigger,
                        event_id: event.event_id,
                        timestamp: event.timestamp,
                    });
                }
                WorkflowEventData::WorkflowCompleted { .. } => {
                    self.is_completed = true;
                }
            }
        }
    }
    
    /// Build projection from event stream
    pub fn from_events(aggregate_id: Uuid, events: impl Iterator<Item = GraphEvent>) -> Self {
        let mut projection = Self::new(aggregate_id);
        for event in events {
            projection.apply(&event);
        }
        projection
    }
    
    /// Query methods - these are READ-ONLY views into the projection
    
    /// Get all states of a specific type
    pub fn get_states_by_type(&self, state_type: &str) -> Vec<&str> {
        self.graph.nodes.values()
            .filter(|node| node.node_type == state_type)
            .map(|node| node.node_id.as_str())
            .collect()
    }
    
    /// Get possible transitions from current states
    pub fn get_available_transitions(&self) -> Vec<(&str, &str, &str)> {
        let mut transitions = Vec::new();
        
        for state in &self.active_states {
            for edge in self.graph.edges.values() {
                if edge.source_id == *state {
                    if let Some(trigger) = edge.data.get("trigger").and_then(|v| v.as_str()) {
                        transitions.push((
                            edge.source_id.as_str(),
                            edge.target_id.as_str(),
                            trigger,
                        ));
                    }
                }
            }
        }
        
        transitions
    }
    
    /// Check if a specific transition is available
    pub fn can_transition(&self, trigger: &str) -> bool {
        self.get_available_transitions()
            .iter()
            .any(|(_, _, t)| *t == trigger)
    }
}

/// Commands specific to workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowCommand {
    /// Trigger a transition
    TriggerTransition {
        workflow_id: Uuid,
        trigger: String,
    },
    /// Reset workflow to initial state
    ResetWorkflow {
        workflow_id: Uuid,
    },
}

/// Workflow command handler - validates workflow rules
pub struct WorkflowCommandHandler;

impl WorkflowCommandHandler {
    /// Handle workflow command, returning events if valid
    pub fn handle(
        &self,
        command: WorkflowCommand,
        projection: &WorkflowProjection,
    ) -> Result<Vec<GraphEvent>, String> {
        match command {
            WorkflowCommand::TriggerTransition { workflow_id, trigger } => {
                // Find valid transition
                let transitions = projection.get_available_transitions();
                
                if let Some((from, to, _)) = transitions.iter().find(|(_, _, t)| *t == &trigger) {
                    let event = GraphEvent {
                        event_id: Uuid::new_v4(),
                        sequence: projection.graph.version + 1,
                        subject: format!("workflow.{}.transition.triggered", workflow_id),
                        timestamp: chrono::Utc::now(),
                        aggregate_id: workflow_id,
                        correlation_id: Uuid::new_v4(),
                        causation_id: None,
                        data: EventData::NodeAdded {
                            node_id: String::new(),
                            node_type: String::new(),
                            data: serde_json::to_value(WorkflowEventData::TransitionTriggered {
                                from_state: from.to_string(),
                                to_state: to.to_string(),
                                trigger: trigger.clone(),
                            }).unwrap(),
                        },
                    };
                    
                    Ok(vec![event])
                } else {
                    Err(format!("No valid transition for trigger '{}'", trigger))
                }
            }
            WorkflowCommand::ResetWorkflow { .. } => {
                // Would emit events to reset workflow
                Ok(vec![])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::GraphType;
    
    #[test]
    fn test_workflow_projection() {
        let workflow_id = Uuid::new_v4();
        
        // Create workflow through events
        let events = vec![
            // Create workflow
            GraphEvent {
                event_id: Uuid::new_v4(),
                sequence: 1,
                subject: "workflow.created".to_string(),
                timestamp: chrono::Utc::now(),
                aggregate_id: workflow_id,
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                data: EventData::GraphCreated {
                    graph_type: GraphType::WorkflowGraph,
                    name: Some("Order Processing".to_string()),
                },
            },
            // Add start state
            GraphEvent {
                event_id: Uuid::new_v4(),
                sequence: 2,
                subject: "workflow.node.added".to_string(),
                timestamp: chrono::Utc::now(),
                aggregate_id: workflow_id,
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                data: EventData::NodeAdded {
                    node_id: "start".to_string(),
                    node_type: "InitialState".to_string(),
                    data: serde_json::json!({"label": "Start"}),
                },
            },
            // Enter start state
            GraphEvent {
                event_id: Uuid::new_v4(),
                sequence: 3,
                subject: "workflow.state.entered".to_string(),
                timestamp: chrono::Utc::now(),
                aggregate_id: workflow_id,
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                data: EventData::NodeAdded {
                    node_id: String::new(),
                    node_type: String::new(),
                    data: serde_json::to_value(WorkflowEventData::StateEntered {
                        state_id: "start".to_string(),
                    }).unwrap(),
                },
            },
        ];
        
        // Build projection
        let projection = WorkflowProjection::from_events(workflow_id, events.into_iter());
        
        // Verify state
        assert!(projection.active_states.contains("start"));
        assert!(!projection.is_completed);
        assert_eq!(projection.graph.nodes.len(), 1);
    }
}