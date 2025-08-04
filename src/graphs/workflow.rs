//! Workflow graph for state machines and process flows
//! 
//! Represents states, transitions, and guards in workflows

use crate::core::{EventGraph, EventHandler, GraphBuilder, GraphType, Node};
use crate::error::{GraphError, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Types of workflow nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateType {
    /// Initial state (entry point)
    Initial,
    /// Regular state
    Normal,
    /// Intermediate state (alias for Normal)
    Intermediate,
    /// Final state (exit point)
    Final,
    /// Choice/decision point
    Choice,
    /// Fork (parallel split)
    Fork,
    /// Join (parallel merge)
    Join,
    /// Composite state (contains sub-states)
    Composite,
}

/// Node representing a state in the workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    /// Unique identifier
    id: String,
    /// State name
    name: String,
    /// Type of state
    state_type: StateType,
    /// Entry actions
    entry_actions: Vec<String>,
    /// Exit actions
    exit_actions: Vec<String>,
    /// Internal activities
    activities: Vec<String>,
    /// Parent state (for nested states)
    parent: Option<String>,
    /// Metadata
    metadata: serde_json::Value,
}

impl WorkflowNode {
    /// Create a new workflow node
    pub fn new(id: impl Into<String>, name: impl Into<String>, state_type: StateType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            state_type,
            entry_actions: Vec::new(),
            exit_actions: Vec::new(),
            activities: Vec::new(),
            parent: None,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
    
    /// Add an entry action
    pub fn add_entry_action(&mut self, action: impl Into<String>) {
        self.entry_actions.push(action.into());
    }
    
    /// Add an exit action
    pub fn add_exit_action(&mut self, action: impl Into<String>) {
        self.exit_actions.push(action.into());
    }
    
    /// Add an activity
    pub fn add_activity(&mut self, activity: impl Into<String>) {
        self.activities.push(activity.into());
    }
    
    /// Set parent state
    pub fn with_parent(mut self, parent: impl Into<String>) -> Self {
        self.parent = Some(parent.into());
        self
    }
    
    /// Get state type
    pub fn state_type(&self) -> StateType {
        self.state_type
    }
    
    /// Get name
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl crate::core::Node for WorkflowNode {
    fn id(&self) -> String {
        self.id.clone()
    }
}

/// Workflow transition between states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEdge {
    /// Unique identifier
    id: String,
    /// Source state
    source: String,
    /// Target state
    target: String,
    /// Trigger event
    trigger: Option<String>,
    /// Guard condition
    guard: Option<String>,
    /// Actions to execute during transition
    actions: Vec<String>,
    /// Priority (for choice states)
    priority: u32,
}

impl WorkflowEdge {
    /// Create a new workflow transition
    pub fn new(source: impl Into<String>, target: impl Into<String>) -> Self {
        let source = source.into();
        let target = target.into();
        let id = format!("{}->{}:{}", source, target, uuid::Uuid::new_v4());
        
        Self {
            id,
            source,
            target,
            trigger: None,
            guard: None,
            actions: Vec::new(),
            priority: 0,
        }
    }
    
    /// Set the trigger event
    pub fn with_trigger(mut self, trigger: impl Into<String>) -> Self {
        self.trigger = Some(trigger.into());
        self
    }
    
    /// Set the guard condition
    pub fn with_guard(mut self, guard: impl Into<String>) -> Self {
        self.guard = Some(guard.into());
        self
    }
    
    /// Add an action
    pub fn add_action(&mut self, action: impl Into<String>) {
        self.actions.push(action.into());
    }
    
    /// Set priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
    
    /// Get the trigger
    pub fn trigger(&self) -> Option<&str> {
        self.trigger.as_deref()
    }
    
    /// Get the guard
    pub fn guard(&self) -> Option<&str> {
        self.guard.as_deref()
    }
}

impl crate::core::Edge for WorkflowEdge {
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

/// State machine workflow graph
pub struct WorkflowGraph {
    /// Underlying event-driven graph
    graph: EventGraph<WorkflowNode, WorkflowEdge>,
    /// Current active states (supports parallel states)
    active_states: Vec<String>,
    /// History of state transitions
    transition_history: Vec<(String, String, String)>, // (from, to, trigger)
}

impl WorkflowGraph {
    /// Create a new workflow graph
    pub fn new() -> Self {
        let graph = GraphBuilder::new()
            .graph_type(GraphType::WorkflowGraph)
            .build_event()
            .expect("Failed to create workflow graph");
            
        Self {
            graph,
            active_states: Vec::new(),
            transition_history: Vec::new(),
        }
    }
    
    /// Create a new workflow graph with an event handler
    pub fn with_handler(handler: Arc<dyn EventHandler>) -> Self {
        let graph = GraphBuilder::new()
            .graph_type(GraphType::WorkflowGraph)
            .add_handler(handler)
            .build_event()
            .expect("Failed to create workflow graph");
            
        Self {
            graph,
            active_states: Vec::new(),
            transition_history: Vec::new(),
        }
    }
    
    /// Add a state to the workflow
    pub fn add_state(&mut self, state: WorkflowNode) -> Result<String> {
        let id = state.id();
        
        // If it's an initial state and we have active states, reject
        if state.state_type() == StateType::Initial && !self.active_states.is_empty() {
            return Err(GraphError::InvalidOperation(
                "Cannot add initial state to active workflow".to_string()
            ));
        }
        
        self.graph.add_node(state)?;
        
        // If it's the initial state, make it active
        if self.graph.get_node(&id).unwrap().state_type() == StateType::Initial {
            self.active_states.push(id.clone());
        }
        
        Ok(id)
    }
    
    /// Add a transition between states
    pub fn add_transition(&mut self, from: &str, to: &str, event: &str) -> Result<String> {
        let mut transition = WorkflowEdge::new(from, to);
        transition = transition.with_trigger(event);
        self.graph.add_edge(transition)
    }
    
    /// Add a transition edge directly
    pub fn add_transition_edge(&mut self, transition: WorkflowEdge) -> Result<String> {
        self.graph.add_edge(transition)
    }
    
    /// Start the workflow from an initial state
    pub fn start(&mut self, initial_state: &str) -> Result<()> {
        // Verify the state exists and is initial
        let state = self.graph.get_node(initial_state)
            .ok_or_else(|| GraphError::NodeNotFound(initial_state.to_string()))?;
            
        if state.state_type() != StateType::Initial {
            return Err(GraphError::InvalidOperation(
                format!("State '{}' is not an initial state", initial_state)
            ));
        }
        
        self.active_states.clear();
        self.active_states.push(initial_state.to_string());
        self.transition_history.clear();
        
        Ok(())
    }
    
    /// Process an event and transition states
    pub fn process_event(&mut self, event: &str) -> Result<Vec<String>> {
        let mut transitions_taken = Vec::new();
        let mut new_active_states = Vec::new();
        
        for active_state in &self.active_states {
            let mut transitioned = false;
            
            // Find applicable transitions
            let transitions = self.graph.neighbors(active_state)?;
            
            for target in transitions {
                let edges = self.graph.edges_between(active_state, &target);
                
                // Find matching transition
                for edge in edges {
                    if edge.trigger() == Some(event) {
                        // Check guard condition (simplified - in real implementation would evaluate)
                        if edge.guard().is_none() || self.evaluate_guard(edge.guard().unwrap()) {
                            // Execute exit actions
                            if let Some(state) = self.graph.get_node(active_state) {
                                for action in &state.exit_actions {
                                    self.execute_action(action);
                                }
                            }
                            
                            // Execute transition actions
                            for action in &edge.actions {
                                self.execute_action(action);
                            }
                            
                            // Execute entry actions
                            if let Some(state) = self.graph.get_node(&target) {
                                for action in &state.entry_actions {
                                    self.execute_action(action);
                                }
                            }
                            
                            // Record transition
                            self.transition_history.push((
                                active_state.clone(),
                                target.clone(),
                                event.to_string(),
                            ));
                            
                            transitions_taken.push(format!("{} -> {}", active_state, target));
                            new_active_states.push(target.clone());
                            transitioned = true;
                            break;
                        }
                    }
                }
                
                if transitioned {
                    break;
                }
            }
            
            // If no transition was taken, state remains active
            if !transitioned {
                new_active_states.push(active_state.clone());
            }
        }
        
        self.active_states = new_active_states;
        Ok(transitions_taken)
    }
    
    /// Check if workflow is in a final state
    pub fn is_final_state(&self) -> bool {
        self.active_states.iter().all(|state| {
            self.graph.get_node(state)
                .map(|n| n.state_type() == StateType::Final)
                .unwrap_or(false)
        })
    }
    
    /// Get current active states
    pub fn active_states(&self) -> &[String] {
        &self.active_states
    }
    
    /// Get current active states (alternate name for compatibility)
    pub fn current_states(&self) -> &[String] {
        &self.active_states
    }
    
    /// Get transition history
    pub fn transition_history(&self) -> &[(String, String, String)] {
        &self.transition_history
    }
    
    /// Validate the workflow structure
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        
        // Check for initial state
        let initial_states: Vec<_> = self.graph.node_ids()
            .into_iter()
            .filter(|id| {
                self.graph.get_node(id)
                    .map(|n| n.state_type() == StateType::Initial)
                    .unwrap_or(false)
            })
            .collect();
            
        if initial_states.is_empty() {
            errors.push("No initial state found".to_string());
        } else if initial_states.len() > 1 {
            errors.push(format!("Multiple initial states found: {:?}", initial_states));
        }
        
        // Check for unreachable states
        if let Some(initial) = initial_states.first() {
            let reachable = self.find_reachable_states(initial);
            let all_states: std::collections::HashSet<_> = self.graph.node_ids().into_iter().collect();
            let unreachable: Vec<_> = all_states.difference(&reachable).collect();
            
            if !unreachable.is_empty() {
                errors.push(format!("Unreachable states: {:?}", unreachable));
            }
        }
        
        // Check for deadlock states (non-final states with no outgoing transitions)
        for state_id in self.graph.node_ids() {
            if let Some(state) = self.graph.get_node(&state_id) {
                if state.state_type() != StateType::Final {
                    if let Ok(neighbors) = self.graph.neighbors(&state_id) {
                        if neighbors.is_empty() {
                            errors.push(format!("Deadlock state (no outgoing transitions): {}", state_id));
                        }
                    }
                }
            }
        }
        
        errors
    }
    
    /// Find all states reachable from a given state
    fn find_reachable_states(&self, start: &str) -> std::collections::HashSet<String> {
        let mut reachable = std::collections::HashSet::new();
        let mut queue = vec![start.to_string()];
        
        while let Some(state) = queue.pop() {
            if reachable.insert(state.clone()) {
                if let Ok(neighbors) = self.graph.neighbors(&state) {
                    queue.extend(neighbors);
                }
            }
        }
        
        reachable
    }
    
    /// Evaluate a guard condition (simplified)
    fn evaluate_guard(&self, _guard: &str) -> bool {
        // In a real implementation, this would evaluate the guard expression
        true
    }
    
    /// Execute an action (simplified)
    fn execute_action(&self, _action: &str) {
        // In a real implementation, this would execute the action
    }
    
    /// Get the underlying graph
    pub fn graph(&self) -> &EventGraph<WorkflowNode, WorkflowEdge> {
        &self.graph
    }
    
    /// Get mutable access to the underlying graph
    pub fn graph_mut(&mut self) -> &mut EventGraph<WorkflowNode, WorkflowEdge> {
        &mut self.graph
    }
}

impl Default for WorkflowGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_workflow_creation() {
        let graph = WorkflowGraph::new();
        assert_eq!(graph.graph().node_count(), 0);
        assert_eq!(graph.graph().graph_type(), GraphType::WorkflowGraph);
    }
    
    #[test]
    fn test_simple_workflow() {
        let mut workflow = WorkflowGraph::new();
        
        // Add states
        let initial = WorkflowNode::new("start", "Start", StateType::Initial);
        let processing = WorkflowNode::new("processing", "Processing", StateType::Normal);
        let done = WorkflowNode::new("done", "Done", StateType::Final);
        
        workflow.add_state(initial).unwrap();
        workflow.add_state(processing).unwrap();
        workflow.add_state(done).unwrap();
        
        // Add transitions
        let t1 = WorkflowEdge::new("start", "processing")
            .with_trigger("begin");
        let t2 = WorkflowEdge::new("processing", "done")
            .with_trigger("complete");
            
        workflow.add_transition_edge(t1).unwrap();
        workflow.add_transition_edge(t2).unwrap();
        
        // Verify initial state
        assert_eq!(workflow.active_states(), vec!["start"]);
        
        // Process events
        workflow.process_event("begin").unwrap();
        assert_eq!(workflow.active_states(), vec!["processing"]);
        
        workflow.process_event("complete").unwrap();
        assert_eq!(workflow.active_states(), vec!["done"]);
        assert!(workflow.is_final_state());
    }
    
    #[test]
    fn test_workflow_validation() {
        let mut workflow = WorkflowGraph::new();
        
        // Add disconnected states
        let s1 = WorkflowNode::new("s1", "State1", StateType::Initial);
        let s2 = WorkflowNode::new("s2", "State2", StateType::Normal);
        let s3 = WorkflowNode::new("s3", "State3", StateType::Normal); // Unreachable
        
        workflow.add_state(s1).unwrap();
        workflow.add_state(s2).unwrap();
        workflow.add_state(s3).unwrap();
        
        // Only connect s1 to s2
        let t1 = WorkflowEdge::new("s1", "s2");
        workflow.add_transition_edge(t1).unwrap();
        
        // Validate
        let errors = workflow.validate();
        assert!(errors.iter().any(|e| e.contains("Unreachable")));
        assert!(errors.iter().any(|e| e.contains("Deadlock")));
    }
}