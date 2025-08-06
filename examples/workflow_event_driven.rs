//! Workflow event-driven example
//! 
//! This example shows how to create a workflow using events and build projections.

use cim_graph::{
    core::{
        PolicyEngine, PolicyContext,
        CidGenerationPolicy, StateValidationPolicy, GraphStateMachine,
        PolicyMetrics,
    },
    events::{GraphEvent, EventPayload, WorkflowPayload},
    contexts::{BoundedContext},
};
use uuid::Uuid;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Workflow Event-Driven Example ===\n");
    
    // 1. Set up policy engine
    let mut policy_engine = PolicyEngine::new();
    policy_engine.add_policy(Box::new(CidGenerationPolicy));
    policy_engine.add_policy(Box::new(StateValidationPolicy));
    
    // 2. Create workflow events
    let workflow_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    
    println!("Creating workflow: Order Processing");
    
    // Define workflow
    let define_event = GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id: workflow_id,
        correlation_id,
        causation_id: None,
        payload: EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
            workflow_id,
            name: "Order Processing".to_string(),
            version: "1.0.0".to_string(),
        }),
    };
    
    // Add states
    let state_events = vec![
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            correlation_id,
            causation_id: Some(define_event.event_id),
            payload: EventPayload::Workflow(WorkflowPayload::StateAdded {
                workflow_id,
                state_id: "submitted".to_string(),
                state_type: "initial".to_string(),
            }),
        },
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            correlation_id,
            causation_id: Some(define_event.event_id),
            payload: EventPayload::Workflow(WorkflowPayload::StateAdded {
                workflow_id,
                state_id: "processing".to_string(),
                state_type: "normal".to_string(),
            }),
        },
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            correlation_id,
            causation_id: Some(define_event.event_id),
            payload: EventPayload::Workflow(WorkflowPayload::StateAdded {
                workflow_id,
                state_id: "completed".to_string(),
                state_type: "final".to_string(),
            }),
        },
    ];
    
    // Add transitions
    let transition_events = vec![
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            correlation_id,
            causation_id: Some(state_events[0].event_id),
            payload: EventPayload::Workflow(WorkflowPayload::TransitionAdded {
                workflow_id,
                from_state: "submitted".to_string(),
                to_state: "processing".to_string(),
                trigger: "approve".to_string(),
            }),
        },
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            correlation_id,
            causation_id: Some(state_events[1].event_id),
            payload: EventPayload::Workflow(WorkflowPayload::TransitionAdded {
                workflow_id,
                from_state: "processing".to_string(),
                to_state: "completed".to_string(),
                trigger: "finish".to_string(),
            }),
        },
    ];
    
    // Collect all events
    let mut all_events = vec![define_event];
    all_events.extend(state_events);
    all_events.extend(transition_events);
    
    println!("Created {} workflow events", all_events.len());
    
    // 3. Apply policies
    let mut state_machine = GraphStateMachine::new();
    let mut ipld_chains = HashMap::new();
    let mut policy_context = PolicyContext {
        state_machine: &mut state_machine,
        ipld_chains: &mut ipld_chains,
        metrics: PolicyMetrics::default(),
    };
    
    println!("\nApplying policies to events...");
    for event in &all_events {
        match policy_engine.execute_policies(event, &mut policy_context) {
            Ok(actions) => {
                println!("  Event {} -> {} policy actions", 
                         event.event_id, actions.len());
            }
            Err(e) => {
                println!("  Policy error for event {}: {}", event.event_id, e);
            }
        }
    }
    
    // 4. Show event causation chain
    println!("\nEvent Causation Chain:");
    for (i, event) in all_events.iter().enumerate() {
        let event_type = match &event.payload {
            EventPayload::Workflow(wp) => match wp {
                WorkflowPayload::WorkflowDefined { .. } => "WorkflowDefined",
                WorkflowPayload::StateAdded { .. } => "StateAdded",
                WorkflowPayload::TransitionAdded { .. } => "TransitionAdded",
                _ => "Other",
            },
            _ => "Unknown",
        };
        
        println!("  {}. {} (caused by: {:?})", 
                 i + 1, event_type, event.causation_id);
    }
    
    // 5. Show bounded context
    println!("\nBounded Context:");
    let workflow_context = BoundedContext::Workflow;
    println!("  Context: {}", workflow_context.name());
    println!("  Aggregate type: {}", workflow_context.aggregate_type());
    println!("  Upstream contexts: {:?}", 
             workflow_context.upstream_contexts()
                .iter()
                .map(|c| c.name())
                .collect::<Vec<_>>());
    
    // 6. Save events
    use cim_graph::serde_support::{serialize_events, EventJournal};
    
    let json = serialize_events(&all_events)?;
    println!("\nSerialized events: {} bytes", json.len());
    
    let journal = EventJournal::new(all_events);
    println!("Event journal created with {} events", journal.metadata.event_count);
    
    // Save to file
    let filename = "workflow_events.json";
    journal.save_to_file(filename)?;
    println!("Saved events to {}", filename);
    
    println!("\n=== Example Complete ===");
    Ok(())
}