//! Example: Using WorkflowGraph for state machine workflows
//! 
//! This example demonstrates how to model a order processing workflow
//! with states and transitions.

use cim_graph::graphs::WorkflowGraph;
use cim_graph::graphs::workflow::{WorkflowNode, StateType};
use serde_json::json;

fn main() {
    println!("=== Workflow Graph Example: Order Processing ===\n");
    
    // Create a new workflow graph
    let mut workflow = WorkflowGraph::new();
    
    // Define states
    println!("Defining workflow states...");
    let states = vec![
        ("new_order", "New Order", StateType::Initial),
        ("payment_pending", "Payment Pending", StateType::Intermediate),
        ("payment_received", "Payment Received", StateType::Intermediate),
        ("processing", "Processing", StateType::Intermediate),
        ("shipped", "Shipped", StateType::Intermediate),
        ("delivered", "Delivered", StateType::Final),
        ("cancelled", "Cancelled", StateType::Final),
        ("refunded", "Refunded", StateType::Final),
    ];
    
    for (id, name, state_type) in states {
        let state = WorkflowNode::new(id, name, state_type);
        workflow.add_state(state).expect("Failed to add state");
    }
    
    // Define transitions
    println!("\nDefining transitions...");
    let transitions = vec![
        ("new_order", "payment_pending", "submit_order"),
        ("payment_pending", "payment_received", "payment_confirmed"),
        ("payment_pending", "cancelled", "payment_failed"),
        ("payment_received", "processing", "start_processing"),
        ("processing", "shipped", "ship_order"),
        ("shipped", "delivered", "confirm_delivery"),
        ("payment_received", "cancelled", "cancel_order"),
        ("cancelled", "refunded", "process_refund"),
    ];
    
    for (from, to, event) in transitions {
        workflow.add_transition(from, to, event).expect("Failed to add transition");
    }
    
    // Start the workflow
    println!("\nStarting workflow...");
    workflow.start("new_order").expect("Failed to start workflow");
    println!("Current state: {:?}", workflow.current_states());
    
    // Process some events
    println!("\n=== Processing Events ===");
    
    // Submit order
    println!("\nEvent: submit_order");
    let result = workflow.process_event("submit_order");
    match result {
        Ok(new_states) => println!("✓ Transitioned to: {:?}", new_states),
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Confirm payment
    println!("\nEvent: payment_confirmed");
    let result = workflow.process_event("payment_confirmed");
    match result {
        Ok(new_states) => println!("✓ Transitioned to: {:?}", new_states),
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Start processing
    println!("\nEvent: start_processing");
    let result = workflow.process_event("start_processing");
    match result {
        Ok(new_states) => println!("✓ Transitioned to: {:?}", new_states),
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Try an invalid event
    println!("\nEvent: invalid_event");
    let result = workflow.process_event("invalid_event");
    match result {
        Ok(new_states) => println!("✓ Transitioned to: {:?}", new_states),
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Ship order
    println!("\nEvent: ship_order");
    let result = workflow.process_event("ship_order");
    match result {
        Ok(new_states) => println!("✓ Transitioned to: {:?}", new_states),
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Deliver order
    println!("\nEvent: confirm_delivery");
    let result = workflow.process_event("confirm_delivery");
    match result {
        Ok(new_states) => println!("✓ Transitioned to: {:?}", new_states),
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Show transition history
    println!("\n\n=== Transition History ===");
    for (i, (from, to, event)) in workflow.transition_history().iter().enumerate() {
        println!("{}. {} --[{}]--> {}", i + 1, from, event, to);
    }
    
    // Validate the workflow
    println!("\n\n=== Workflow Validation ===");
    let issues = workflow.validate();
    if issues.is_empty() {
        println!("✅ Workflow is valid!");
    } else {
        println!("❌ Validation issues found:");
        for issue in issues {
            println!("  - {}", issue);
        }
    }
    
    // Create a parallel workflow example
    println!("\n\n=== Parallel Workflow Example ===");
    let mut parallel_workflow = WorkflowGraph::new();
    
    // States for parallel processing
    parallel_workflow.add_state(WorkflowNode::new("start", "Start", StateType::Initial)).unwrap();
    parallel_workflow.add_state(WorkflowNode::new("check_inventory", "Check Inventory", StateType::Intermediate)).unwrap();
    parallel_workflow.add_state(WorkflowNode::new("check_credit", "Check Credit", StateType::Intermediate)).unwrap();
    parallel_workflow.add_state(WorkflowNode::new("approved", "Approved", StateType::Intermediate)).unwrap();
    parallel_workflow.add_state(WorkflowNode::new("rejected", "Rejected", StateType::Final)).unwrap();
    parallel_workflow.add_state(WorkflowNode::new("complete", "Complete", StateType::Final)).unwrap();
    
    // Parallel transitions
    parallel_workflow.add_transition("start", "check_inventory", "begin_checks").unwrap();
    parallel_workflow.add_transition("start", "check_credit", "begin_checks").unwrap();
    parallel_workflow.add_transition("check_inventory", "approved", "inventory_ok").unwrap();
    parallel_workflow.add_transition("check_credit", "approved", "credit_ok").unwrap();
    parallel_workflow.add_transition("check_inventory", "rejected", "inventory_fail").unwrap();
    parallel_workflow.add_transition("check_credit", "rejected", "credit_fail").unwrap();
    parallel_workflow.add_transition("approved", "complete", "finalize").unwrap();
    
    // Start parallel workflow
    parallel_workflow.start("start").unwrap();
    println!("Initial state: {:?}", parallel_workflow.current_states());
    
    // Process parallel event
    parallel_workflow.process_event("begin_checks").unwrap();
    println!("After begin_checks: {:?}", parallel_workflow.current_states());
    println!("(Both inventory and credit checks are running in parallel)");
    
    // Statistics
    println!("\n\n=== Workflow Statistics ===");
    println!("Total states: {}", workflow.graph().node_count());
    println!("Total transitions: {}", workflow.graph().edge_count());
    println!("Transitions processed: {}", workflow.transition_history().len());
    println!("Current states: {:?}", workflow.current_states());
    
    // Check if workflow is in final state
    let in_final_state = workflow.current_states().iter()
        .all(|state_id| {
            workflow.graph().get_node(state_id)
                .map(|node| node.state_type() == StateType::Final)
                .unwrap_or(false)
        });
    println!("Workflow completed: {}", in_final_state);
}