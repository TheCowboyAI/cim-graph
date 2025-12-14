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
    use crate::core::GraphType;
    use crate::core::projection_engine::GenericGraphProjection;
    use uuid::Uuid;

    // ========================================================================
    // WorkflowState tests
    // ========================================================================

    #[test]
    fn test_workflow_state_variants() {
        let draft = WorkflowState::Draft;
        let published = WorkflowState::Published;
        let running = WorkflowState::Running { current_state: "processing".to_string() };
        let completed = WorkflowState::Completed;
        let failed = WorkflowState::Failed { error: "timeout".to_string() };

        // Test debug output
        assert!(format!("{:?}", draft).contains("Draft"));
        assert!(format!("{:?}", published).contains("Published"));
        assert!(format!("{:?}", running).contains("processing"));
        assert!(format!("{:?}", completed).contains("Completed"));
        assert!(format!("{:?}", failed).contains("timeout"));
    }

    #[test]
    fn test_workflow_state_equality() {
        let s1 = WorkflowState::Draft;
        let s2 = WorkflowState::Draft;
        let s3 = WorkflowState::Published;

        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
    }

    #[test]
    fn test_workflow_state_running_equality() {
        let r1 = WorkflowState::Running { current_state: "a".to_string() };
        let r2 = WorkflowState::Running { current_state: "a".to_string() };
        let r3 = WorkflowState::Running { current_state: "b".to_string() };

        assert_eq!(r1, r2);
        assert_ne!(r1, r3);
    }

    // ========================================================================
    // WorkflowNodeType tests
    // ========================================================================

    #[test]
    fn test_workflow_node_type_variants() {
        let start = WorkflowNodeType::Start;
        let end = WorkflowNodeType::End;
        let state = WorkflowNodeType::State { name: "active".to_string() };
        let decision = WorkflowNodeType::Decision { condition: "x > 5".to_string() };
        let action = WorkflowNodeType::Action { operation: "send_email".to_string() };
        let wait = WorkflowNodeType::Wait { event_type: "user_response".to_string() };
        let error = WorkflowNodeType::Error { message: "invalid state".to_string() };

        // Test that all variants can be cloned
        let _ = start.clone();
        let _ = end.clone();
        let _ = state.clone();
        let _ = decision.clone();
        let _ = action.clone();
        let _ = wait.clone();
        let _ = error.clone();
    }

    #[test]
    fn test_workflow_node_type_equality() {
        let s1 = WorkflowNodeType::State { name: "active".to_string() };
        let s2 = WorkflowNodeType::State { name: "active".to_string() };
        let s3 = WorkflowNodeType::State { name: "inactive".to_string() };

        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
    }

    // ========================================================================
    // WorkflowNode tests
    // ========================================================================

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
    fn test_workflow_node_end() {
        let end = WorkflowNode::end("finish");
        assert!(matches!(end.node_type, WorkflowNodeType::End));
        assert_eq!(end.id, "finish");
    }

    #[test]
    fn test_workflow_node_action() {
        let action = WorkflowNode::action("send_notification", "email_send");
        assert!(matches!(action.node_type, WorkflowNodeType::Action { operation } if operation == "email_send"));
        assert_eq!(action.id, "send_notification");
    }

    #[test]
    fn test_workflow_node_wait() {
        let wait = WorkflowNode::wait("await_approval", "approval_received");
        assert!(matches!(wait.node_type, WorkflowNodeType::Wait { event_type } if event_type == "approval_received"));
    }

    #[test]
    fn test_workflow_node_error() {
        let error = WorkflowNode::error("error_state", "Processing failed");
        assert!(matches!(error.node_type, WorkflowNodeType::Error { message } if message == "Processing failed"));
    }

    #[test]
    fn test_workflow_node_default_state() {
        let node = WorkflowNode::start("s");
        assert_eq!(node.workflow_state, WorkflowState::Draft);
        assert!(node.metadata.is_empty());
    }

    #[test]
    fn test_workflow_node_implements_node_trait() {
        let node = WorkflowNode::state("test_node", "Testing");
        assert_eq!(node.id(), "test_node");
    }

    // ========================================================================
    // WorkflowEdgeType tests
    // ========================================================================

    #[test]
    fn test_workflow_edge_type_variants() {
        let trans = WorkflowEdgeType::Transition;
        let cond = WorkflowEdgeType::ConditionalTransition { condition: "approved".to_string() };
        let err = WorkflowEdgeType::ErrorTransition;
        let timeout = WorkflowEdgeType::TimeoutTransition { timeout_ms: 5000 };
        let event = WorkflowEdgeType::EventTransition { event_type: "payment".to_string() };

        // All should be cloneable
        let _ = trans.clone();
        let _ = cond.clone();
        let _ = err.clone();
        let _ = timeout.clone();
        let _ = event.clone();
    }

    // ========================================================================
    // WorkflowEdge tests
    // ========================================================================

    #[test]
    fn test_workflow_edge_creation() {
        let transition = WorkflowEdge::transition("t1", "s1", "s2");
        assert!(matches!(transition.edge_type, WorkflowEdgeType::Transition));

        let conditional = WorkflowEdge::conditional("c1", "d1", "s2", "approved");
        assert!(matches!(conditional.edge_type, WorkflowEdgeType::ConditionalTransition { condition } if condition == "approved"));

        let event = WorkflowEdge::event_triggered("e1", "wait", "process", "payment_received");
        assert!(matches!(event.edge_type, WorkflowEdgeType::EventTransition { event_type } if event_type == "payment_received"));
    }

    #[test]
    fn test_workflow_edge_with_trigger() {
        let edge = WorkflowEdge::transition("t1", "a", "b")
            .with_trigger("manual_trigger");

        assert_eq!(edge.trigger, Some("manual_trigger".to_string()));
    }

    #[test]
    fn test_workflow_edge_implements_edge_trait() {
        let edge = WorkflowEdge::transition("e1", "source", "target");
        assert_eq!(edge.id(), "e1");
        assert_eq!(edge.source(), "source");
        assert_eq!(edge.target(), "target");
    }

    #[test]
    fn test_workflow_edge_metadata() {
        let mut edge = WorkflowEdge::new("e1", "a", "b", WorkflowEdgeType::Transition);
        assert!(edge.metadata.is_empty());

        edge.metadata.insert("priority".to_string(), serde_json::json!(1));
        assert_eq!(edge.metadata.get("priority"), Some(&serde_json::json!(1)));
    }

    // ========================================================================
    // WorkflowProjection method tests
    // ========================================================================

    fn create_simple_workflow_projection() -> WorkflowProjection {
        let mut projection: WorkflowProjection = GenericGraphProjection::new(Uuid::new_v4(), GraphType::Generic);

        // Add nodes
        let start = WorkflowNode::start("start");
        let process = WorkflowNode::state("process", "Processing");
        let decision = WorkflowNode::decision("check", "amount > 100");
        let approve = WorkflowNode::state("approve", "Approved");
        let reject = WorkflowNode::state("reject", "Rejected");
        let end = WorkflowNode::end("end");

        projection.nodes.insert("start".to_string(), start);
        projection.nodes.insert("process".to_string(), process);
        projection.nodes.insert("check".to_string(), decision);
        projection.nodes.insert("approve".to_string(), approve);
        projection.nodes.insert("reject".to_string(), reject);
        projection.nodes.insert("end".to_string(), end);

        // Add edges
        let e1 = WorkflowEdge::transition("e1", "start", "process");
        let e2 = WorkflowEdge::transition("e2", "process", "check");
        let e3 = WorkflowEdge::conditional("e3", "check", "approve", "true");
        let e4 = WorkflowEdge::conditional("e4", "check", "reject", "false");
        let e5 = WorkflowEdge::transition("e5", "approve", "end");
        let e6 = WorkflowEdge::transition("e6", "reject", "end");

        projection.edges.insert("e1".to_string(), e1);
        projection.edges.insert("e2".to_string(), e2);
        projection.edges.insert("e3".to_string(), e3);
        projection.edges.insert("e4".to_string(), e4);
        projection.edges.insert("e5".to_string(), e5);
        projection.edges.insert("e6".to_string(), e6);

        // Setup adjacency
        projection.adjacency.insert("start".to_string(), vec!["process".to_string()]);
        projection.adjacency.insert("process".to_string(), vec!["check".to_string()]);
        projection.adjacency.insert("check".to_string(), vec!["approve".to_string(), "reject".to_string()]);
        projection.adjacency.insert("approve".to_string(), vec!["end".to_string()]);
        projection.adjacency.insert("reject".to_string(), vec!["end".to_string()]);
        projection.adjacency.insert("end".to_string(), vec![]);

        projection
    }

    #[test]
    fn test_get_states() {
        let projection = create_simple_workflow_projection();
        let states = projection.get_states();

        // Should have: process, approve, reject (not start, end, or decision)
        assert_eq!(states.len(), 3);

        let state_ids: Vec<&str> = states.iter().map(|s| s.id.as_str()).collect();
        assert!(state_ids.contains(&"process"));
        assert!(state_ids.contains(&"approve"));
        assert!(state_ids.contains(&"reject"));
    }

    #[test]
    fn test_get_start_node() {
        let projection = create_simple_workflow_projection();
        let start = projection.get_start_node();

        assert!(start.is_some());
        assert_eq!(start.unwrap().id, "start");
    }

    #[test]
    fn test_get_end_nodes() {
        let projection = create_simple_workflow_projection();
        let ends = projection.get_end_nodes();

        assert_eq!(ends.len(), 1);
        assert_eq!(ends[0].id, "end");
    }

    #[test]
    fn test_get_decision_nodes() {
        let projection = create_simple_workflow_projection();
        let decisions = projection.get_decision_nodes();

        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].id, "check");
    }

    #[test]
    fn test_get_transitions_from() {
        let projection = create_simple_workflow_projection();

        let from_check = projection.get_transitions_from("check");
        assert_eq!(from_check.len(), 2);

        let from_start = projection.get_transitions_from("start");
        assert_eq!(from_start.len(), 1);

        let from_end = projection.get_transitions_from("end");
        assert_eq!(from_end.len(), 0);
    }

    #[test]
    fn test_get_transitions_to() {
        let projection = create_simple_workflow_projection();

        let to_end = projection.get_transitions_to("end");
        assert_eq!(to_end.len(), 2);

        let to_start = projection.get_transitions_to("start");
        assert_eq!(to_start.len(), 0);
    }

    #[test]
    fn test_find_path() {
        let projection = create_simple_workflow_projection();

        // Find path from start to end
        let path = projection.find_path("start", "end");
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.first(), Some(&"start".to_string()));
        assert_eq!(path.last(), Some(&"end".to_string()));

        // Path should go through process and check
        assert!(path.contains(&"process".to_string()));
        assert!(path.contains(&"check".to_string()));
    }

    #[test]
    fn test_find_path_no_path() {
        let projection = create_simple_workflow_projection();

        // No path from end back to start (directed graph)
        let path = projection.find_path("end", "start");
        assert!(path.is_none());
    }

    #[test]
    fn test_find_path_same_node() {
        let projection = create_simple_workflow_projection();

        let path = projection.find_path("start", "start");
        assert!(path.is_some());
        assert_eq!(path.unwrap(), vec!["start".to_string()]);
    }

    #[test]
    fn test_validate_success() {
        let projection = create_simple_workflow_projection();
        let result = projection.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_missing_start() {
        let mut projection: WorkflowProjection = GenericGraphProjection::new(Uuid::new_v4(), GraphType::Generic);
        projection.nodes.insert("end".to_string(), WorkflowNode::end("end"));
        projection.adjacency.insert("end".to_string(), vec![]);

        let result = projection.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("start node"));
    }

    #[test]
    fn test_validate_missing_end() {
        let mut projection: WorkflowProjection = GenericGraphProjection::new(Uuid::new_v4(), GraphType::Generic);
        projection.nodes.insert("start".to_string(), WorkflowNode::start("start"));
        projection.adjacency.insert("start".to_string(), vec![]);

        let result = projection.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("end node"));
    }

    #[test]
    fn test_validate_unreachable_node() {
        let mut projection: WorkflowProjection = GenericGraphProjection::new(Uuid::new_v4(), GraphType::Generic);

        projection.nodes.insert("start".to_string(), WorkflowNode::start("start"));
        projection.nodes.insert("end".to_string(), WorkflowNode::end("end"));
        projection.nodes.insert("orphan".to_string(), WorkflowNode::state("orphan", "Orphan"));

        projection.adjacency.insert("start".to_string(), vec!["end".to_string()]);
        projection.adjacency.insert("end".to_string(), vec![]);
        projection.adjacency.insert("orphan".to_string(), vec![]);

        let result = projection.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unreachable"));
    }

    #[test]
    fn test_is_running() {
        let mut projection = create_simple_workflow_projection();

        // Initially not running
        assert!(!projection.is_running());

        // Change a node state to running
        if let Some(node) = projection.nodes.get_mut("process") {
            node.workflow_state = WorkflowState::Running { current_state: "process".to_string() };
        }

        assert!(projection.is_running());
    }

    #[test]
    fn test_get_current_state() {
        let mut projection = create_simple_workflow_projection();

        // Initially no running state
        assert!(projection.get_current_state().is_none());

        // Set a node to running
        if let Some(node) = projection.nodes.get_mut("approve") {
            node.workflow_state = WorkflowState::Running { current_state: "approve".to_string() };
        }

        let current = projection.get_current_state();
        assert!(current.is_some());
        assert_eq!(current.unwrap().id, "approve");
    }
}