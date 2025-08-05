//! Demo: Task Management System using CIM Graph
//! 
//! This example demonstrates how to build a simple task management
//! system using WorkflowGraph for task states and dependencies.

use cim_graph::graphs::workflow::{WorkflowGraph, WorkflowNode, StateType};
use cim_graph::error::Result;
use std::collections::HashMap;

/// A simple task management system
struct TaskManager {
    workflow: WorkflowGraph,
    task_data: HashMap<String, TaskData>,
}

#[derive(Debug, Clone)]
struct TaskData {
    title: String,
    description: String,
    assignee: Option<String>,
    priority: Priority,
}

#[derive(Debug, Clone)]
enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl TaskManager {
    fn new() -> Result<Self> {
        let mut workflow = WorkflowGraph::new();
        
        // Define task states
        workflow.add_state(WorkflowNode::new("backlog", "Backlog", StateType::Initial))?;
        workflow.add_state(WorkflowNode::new("todo", "To Do", StateType::Normal))?;
        workflow.add_state(WorkflowNode::new("in_progress", "In Progress", StateType::Normal))?;
        workflow.add_state(WorkflowNode::new("review", "In Review", StateType::Normal))?;
        workflow.add_state(WorkflowNode::new("done", "Done", StateType::Final))?;
        workflow.add_state(WorkflowNode::new("blocked", "Blocked", StateType::Normal))?;
        workflow.add_state(WorkflowNode::new("cancelled", "Cancelled", StateType::Final))?;
        
        // Define transitions
        workflow.add_transition("backlog", "todo", "prioritize")?;
        workflow.add_transition("todo", "in_progress", "start")?;
        workflow.add_transition("in_progress", "review", "submit")?;
        workflow.add_transition("review", "done", "approve")?;
        workflow.add_transition("review", "in_progress", "request_changes")?;
        workflow.add_transition("in_progress", "blocked", "block")?;
        workflow.add_transition("blocked", "in_progress", "unblock")?;
        
        // Allow cancellation from most states
        workflow.add_transition("backlog", "cancelled", "cancel")?;
        workflow.add_transition("todo", "cancelled", "cancel")?;
        workflow.add_transition("in_progress", "cancelled", "cancel")?;
        workflow.add_transition("blocked", "cancelled", "cancel")?;
        
        Ok(Self {
            workflow,
            task_data: HashMap::new(),
        })
    }
    
    fn create_task(&mut self, id: &str, title: String, description: String) -> Result<()> {
        // Start the workflow for this task
        self.workflow.start(id)?;
        
        // Store task data
        self.task_data.insert(id.to_string(), TaskData {
            title,
            description,
            assignee: None,
            priority: Priority::Medium,
        });
        
        println!("✓ Created task: {} - {}", id, self.task_data[id].title);
        Ok(())
    }
    
    fn assign_task(&mut self, id: &str, assignee: &str) -> Result<()> {
        if let Some(task) = self.task_data.get_mut(id) {
            task.assignee = Some(assignee.to_string());
            println!("✓ Assigned {} to {}", id, assignee);
            Ok(())
        } else {
            Err(cim_graph::error::GraphError::NodeNotFound(id.to_string()).into())
        }
    }
    
    fn move_task(&mut self, id: &str, action: &str) -> Result<()> {
        let current_state = self.workflow.current_state_for(id)
            .ok_or_else(|| cim_graph::error::GraphError::NodeNotFound(id.to_string()))?;
        
        self.workflow.process_event_for(id, action)?;
        
        let new_state = self.workflow.current_state_for(id)
            .ok_or_else(|| cim_graph::error::GraphError::NodeNotFound(id.to_string()))?;
        
        println!("✓ Moved {} from {} to {} via '{}'", id, current_state, new_state, action);
        Ok(())
    }
    
    fn list_tasks_by_state(&self) {
        let mut tasks_by_state: HashMap<String, Vec<String>> = HashMap::new();
        
        for (id, _) in &self.task_data {
            if let Some(state) = self.workflow.current_state_for(id) {
                tasks_by_state.entry(state.to_string())
                    .or_insert_with(Vec::new)
                    .push(id.clone());
            }
        }
        
        println!("\n📋 Tasks by State:");
        for state in ["backlog", "todo", "in_progress", "review", "done", "blocked", "cancelled"] {
            if let Some(tasks) = tasks_by_state.get(state) {
                println!("\n  {}:", state.to_uppercase());
                for task_id in tasks {
                    if let Some(task) = self.task_data.get(task_id) {
                        let assignee = task.assignee.as_ref()
                            .map(|a| format!(" (@{})", a))
                            .unwrap_or_default();
                        println!("    - {} - {}{}", task_id, task.title, assignee);
                    }
                }
            }
        }
    }
}

fn main() -> Result<()> {
    println!("🚀 Task Management System Demo\n");
    
    let mut task_manager = TaskManager::new()?;
    
    // Create some tasks
    task_manager.create_task("TASK-001", "Set up CI/CD pipeline".to_string(), 
        "Configure GitHub Actions for automated testing".to_string())?;
    
    task_manager.create_task("TASK-002", "Write documentation".to_string(),
        "Create user guide and API documentation".to_string())?;
    
    task_manager.create_task("TASK-003", "Fix login bug".to_string(),
        "Users unable to login with special characters in password".to_string())?;
    
    task_manager.create_task("TASK-004", "Add dark mode".to_string(),
        "Implement dark mode theme toggle".to_string())?;
    
    // Move tasks through workflow
    println!("\n📝 Processing tasks...\n");
    
    // Prioritize and start work on bug fix
    task_manager.move_task("TASK-003", "prioritize")?;
    task_manager.assign_task("TASK-003", "alice")?;
    task_manager.move_task("TASK-003", "start")?;
    
    // Start documentation
    task_manager.move_task("TASK-002", "prioritize")?;
    task_manager.assign_task("TASK-002", "bob")?;
    task_manager.move_task("TASK-002", "start")?;
    
    // Documentation gets blocked
    task_manager.move_task("TASK-002", "block")?;
    
    // Bug fix moves to review
    task_manager.move_task("TASK-003", "submit")?;
    
    // Start CI/CD task
    task_manager.move_task("TASK-001", "prioritize")?;
    task_manager.assign_task("TASK-001", "charlie")?;
    task_manager.move_task("TASK-001", "start")?;
    
    // Bug fix approved
    task_manager.move_task("TASK-003", "approve")?;
    
    // Cancel dark mode (deprioritized)
    task_manager.move_task("TASK-004", "cancel")?;
    
    // Show current state
    task_manager.list_tasks_by_state();
    
    // Demonstrate graph capabilities
    println!("\n\n📊 Workflow Statistics:");
    let graph = task_manager.workflow.graph();
    println!("  - Total states: {}", graph.node_count());
    println!("  - Total transitions: {}", graph.edge_count());
    println!("  - Active tasks: {}", task_manager.task_data.len());
    
    Ok(())
}