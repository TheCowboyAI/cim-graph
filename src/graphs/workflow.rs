//! Workflow graph - state machines (event-driven projection)

// Projections are ephemeral - no serialization
use std::collections::{HashMap, HashSet};

pub use crate::core::projection_engine::GenericGraphProjection;
pub use crate::core::{Node, Edge};

/// Workflow graph projection
pub type WorkflowGraph = GenericGraphProjection<WorkflowNode, WorkflowEdge>;

/// Workflow projection with additional workflow-specific methods
pub type WorkflowProjection = WorkflowGraph;

/// Workflow state enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WorkflowState {
    /// Workflow is being designed
    Draft,
    /// Workflow is published and ready to execute
    Published,
    /// Workflow is currently executing
    Running { 
        /// Current state in the workflow execution
        current_state: String 
    },
    /// Workflow has completed successfully
    Completed,
    /// Workflow execution failed
    Failed { 
        /// Error message describing the failure
        error: String 
    },
}

/// Type of workflow node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WorkflowNodeType {
    /// Start state
    Start,
    /// End state
    End,
    /// State in the workflow
    State { 
        /// Name of the state
        name: String 
    },
    /// Decision point
    Decision { 
        /// Condition expression to evaluate
        condition: String 
    },
    /// Action to perform
    Action { 
        /// Operation to execute
        operation: String 
    },
    /// Wait for external event
    Wait { 
        /// Type of event to wait for
        event_type: String 
    },
    /// Error state
    Error { 
        /// Error message
        message: String 
    },
}

/// Workflow node represents a state or action in a state machine
#[derive(Debug, Clone)]
pub struct WorkflowNode {
    /// Unique identifier for the node
    pub id: String,
    /// Type of workflow node
    pub node_type: WorkflowNodeType,
    /// Additional metadata for the node
    pub metadata: HashMap<String, serde_json::Value>,
    /// Current state of the workflow
    pub workflow_state: WorkflowState,
}

impl WorkflowNode {
    /// Create a new workflow node
    pub fn new(id: impl Into<String>, node_type: WorkflowNodeType) -> Self {
        Self {
            id: id.into(),
            node_type,
            metadata: HashMap::new(),
            workflow_state: WorkflowState::Draft,
        }
    }

    /// Create a start node
    pub fn start(id: impl Into<String>) -> Self {
        Self::new(id, WorkflowNodeType::Start)
    }

    /// Create an end node
    pub fn end(id: impl Into<String>) -> Self {
        Self::new(id, WorkflowNodeType::End)
    }

    /// Create a state node
    pub fn state(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self::new(id, WorkflowNodeType::State { name: name.into() })
    }

    /// Create a decision node
    pub fn decision(id: impl Into<String>, condition: impl Into<String>) -> Self {
        Self::new(id, WorkflowNodeType::Decision { condition: condition.into() })
    }

    /// Create an action node
    pub fn action(id: impl Into<String>, operation: impl Into<String>) -> Self {
        Self::new(id, WorkflowNodeType::Action { operation: operation.into() })
    }

    /// Create a wait node
    pub fn wait(id: impl Into<String>, event_type: impl Into<String>) -> Self {
        Self::new(id, WorkflowNodeType::Wait { event_type: event_type.into() })
    }

    /// Create an error node
    pub fn error(id: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(id, WorkflowNodeType::Error { message: message.into() })
    }
}

impl Node for WorkflowNode {
    fn id(&self) -> String {
        self.id.clone()
    }
}

/// Type of workflow edge (transition)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WorkflowEdgeType {
    /// Normal transition
    Transition,
    /// Conditional transition
    ConditionalTransition { 
        /// Condition that must be true for transition
        condition: String 
    },
    /// Error transition
    ErrorTransition,
    /// Timeout transition
    TimeoutTransition { 
        /// Timeout duration in milliseconds
        timeout_ms: u64 
    },
    /// Event-triggered transition
    EventTransition { 
        /// Type of event that triggers this transition
        event_type: String 
    },
}

/// Workflow edge represents a transition between states
#[derive(Debug, Clone)]
pub struct WorkflowEdge {
    /// Unique identifier for the edge
    pub id: String,
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Type of transition
    pub edge_type: WorkflowEdgeType,
    /// Additional metadata for the edge
    pub metadata: HashMap<String, serde_json::Value>,
    /// Optional trigger name for the transition
    pub trigger: Option<String>,
}

impl WorkflowEdge {
    /// Create a new workflow edge
    pub fn new(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
        edge_type: WorkflowEdgeType,
    ) -> Self {
        Self {
            id: id.into(),
            source: source.into(),
            target: target.into(),
            edge_type,
            metadata: HashMap::new(),
            trigger: None,
        }
    }

    /// Create a simple transition
    pub fn transition(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        Self::new(id, source, target, WorkflowEdgeType::Transition)
    }

    /// Create a conditional transition
    pub fn conditional(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
        condition: impl Into<String>,
    ) -> Self {
        Self::new(
            id,
            source,
            target,
            WorkflowEdgeType::ConditionalTransition {
                condition: condition.into(),
            },
        )
    }

    /// Create an event-triggered transition
    pub fn event_triggered(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
        event_type: impl Into<String>,
    ) -> Self {
        Self::new(
            id,
            source,
            target,
            WorkflowEdgeType::EventTransition {
                event_type: event_type.into(),
            },
        )
    }

    /// Set the trigger for this transition
    pub fn with_trigger(mut self, trigger: impl Into<String>) -> Self {
        self.trigger = Some(trigger.into());
        self
    }
}

impl Edge for WorkflowEdge {
    fn id(&self) -> String {
        self.id.clone()
    }
    fn source(&self) -> String {
        self.source.clone()
    }
    fn target(&self) -> String {
        self.target.clone()
    }
}

/// Extension methods for WorkflowProjection
impl WorkflowProjection {
    /// Get all states in the workflow
    pub fn get_states(&self) -> Vec<&WorkflowNode> {
        self.nodes()
            .filter(|n| matches!(n.node_type, WorkflowNodeType::State { .. }))
            .collect()
    }

    /// Get the start node
    pub fn get_start_node(&self) -> Option<&WorkflowNode> {
        self.nodes()
            .find(|n| matches!(n.node_type, WorkflowNodeType::Start))
    }

    /// Get the end nodes
    pub fn get_end_nodes(&self) -> Vec<&WorkflowNode> {
        self.nodes()
            .filter(|n| matches!(n.node_type, WorkflowNodeType::End))
            .collect()
    }

    /// Get all decision nodes
    pub fn get_decision_nodes(&self) -> Vec<&WorkflowNode> {
        self.nodes()
            .filter(|n| matches!(n.node_type, WorkflowNodeType::Decision { .. }))
            .collect()
    }

    /// Get all transitions from a state
    pub fn get_transitions_from(&self, state_id: &str) -> Vec<&WorkflowEdge> {
        self.edges()
            .filter(|e| e.source() == state_id)
            .collect()
    }

    /// Get all transitions to a state
    pub fn get_transitions_to(&self, state_id: &str) -> Vec<&WorkflowEdge> {
        self.edges()
            .filter(|e| e.target() == state_id)
            .collect()
    }

    /// Find a path from start to end
    pub fn find_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        // Simple BFS path finding
        use std::collections::{VecDeque, HashSet};
        
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent: HashMap<String, String> = HashMap::new();
        
        queue.push_back(from.to_string());
        visited.insert(from.to_string());
        
        while let Some(current) = queue.pop_front() {
            if current == to {
                // Reconstruct path
                let mut path = vec![current.clone()];
                let mut node = current;
                while let Some(p) = parent.get(&node) {
                    path.push(p.clone());
                    node = p.clone();
                }
                path.reverse();
                return Some(path);
            }
            
            for edge in self.get_transitions_from(&current) {
                let target = edge.target();
                if !visited.contains(&target) {
                    visited.insert(target.clone());
                    parent.insert(target.clone(), current.clone());
                    queue.push_back(target);
                }
            }
        }
        
        None
    }

    /// Validate the workflow structure
    pub fn validate(&self) -> Result<(), String> {
        // Check for start node
        if self.get_start_node().is_none() {
            return Err("Workflow must have a start node".to_string());
        }

        // Check for at least one end node
        if self.get_end_nodes().is_empty() {
            return Err("Workflow must have at least one end node".to_string());
        }

        // Check for unreachable states
        if let Some(start) = self.get_start_node() {
            let mut reachable = HashSet::new();
            let mut to_visit = vec![start.id.clone()];
            
            while let Some(node_id) = to_visit.pop() {
                if reachable.insert(node_id.clone()) {
                    for edge in self.get_transitions_from(&node_id) {
                        to_visit.push(edge.target());
                    }
                }
            }
            
            for node in self.nodes() {
                if !reachable.contains(&node.id) {
                    return Err(format!("Node {} is unreachable from start", node.id));
                }
            }
        }

        Ok(())
    }

    /// Check if the workflow is in a running state
    pub fn is_running(&self) -> bool {
        self.nodes()
            .any(|n| matches!(n.workflow_state, WorkflowState::Running { .. }))
    }

    /// Get the current running state
    pub fn get_current_state(&self) -> Option<&WorkflowNode> {
        self.nodes()
            .find(|n| matches!(n.workflow_state, WorkflowState::Running { .. }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_node_creation() {
        let start = WorkflowNode::start("start");
        assert!(matches!(start.node_type, WorkflowNodeType::Start));
        
        let state = WorkflowNode::state("s1", "Processing");
        assert!(matches!(state.node_type, WorkflowNodeType::State { name } if name == "Processing"));
        
        let decision = WorkflowNode::decision("d1", "amount > 100");
        assert!(matches!(decision.node_type, WorkflowNodeType::Decision { condition } if condition == "amount > 100"));
    }

    #[test]
    fn test_workflow_edge_creation() {
        let transition = WorkflowEdge::transition("t1", "s1", "s2");
        assert!(matches!(transition.edge_type, WorkflowEdgeType::Transition));
        
        let conditional = WorkflowEdge::conditional("c1", "d1", "s2", "approved");
        assert!(matches!(conditional.edge_type, WorkflowEdgeType::ConditionalTransition { condition } if condition == "approved"));
        
        let event = WorkflowEdge::event_triggered("e1", "wait", "process", "payment_received");
        assert!(matches!(event.edge_type, WorkflowEdgeType::EventTransition { event_type } if event_type == "payment_received"));
    }
}