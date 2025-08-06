//! Basic example of event-driven architecture
//! 
//! This example shows the fundamental concepts without complex features.

use cim_graph::{
    core::build_projection,
    events::{GraphEvent, EventPayload, WorkflowPayload},
    serde_support::{serialize_events, EventJournal},
};
use uuid::Uuid;

fn main() {
    println!("=== Basic Event-Driven Example ===\n");
    
    // 1. Create a workflow using events
    let workflow_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    
    // Event 1: Define the workflow
    let define_event = GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id: workflow_id,
        correlation_id,
        causation_id: None,  // This is the first event
        payload: EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
            workflow_id,
            name: "Order Processing".to_string(),
            version: "1.0.0".to_string(),
        }),
    };
    
    // Event 2: Add a state (caused by the workflow definition)
    let add_state_event = GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id: workflow_id,
        correlation_id,  // Same correlation - these events are related
        causation_id: Some(define_event.event_id),  // Caused by define_event
        payload: EventPayload::Workflow(WorkflowPayload::StateAdded {
            workflow_id,
            state_id: "active".to_string(),
            state_type: "normal".to_string(),
        }),
    };
    
    // Collect all events
    let events = vec![define_event, add_state_event];
    
    // 2. Build a projection from the events
    println!("Building projection from {} events...", events.len());
    let event_tuples: Vec<(GraphEvent, u64)> = events.iter()
        .enumerate()
        .map(|(i, e)| (e.clone(), i as u64 + 1))
        .collect();
    let projection = build_projection(event_tuples);
    
    // 3. Query the projection (read-only!)
    println!("\nProjection state:");
    println!("  - Components: {}", projection.components.len());
    println!("  - Relationships: {}", projection.relationships.len());
    println!("  - Version: {} (number of events processed)", projection.version);
    
    // 4. Show the event chain
    println!("\nEvent chain (causation):");
    for event in &events {
        match &event.payload {
            EventPayload::Workflow(workflow_payload) => {
                let event_type = match workflow_payload {
                    WorkflowPayload::WorkflowDefined { .. } => "WorkflowDefined",
                    WorkflowPayload::StateAdded { .. } => "StateAdded",
                    _ => "Other",
                };
                println!("  - {} (caused by: {:?})", event_type, event.causation_id);
            }
            _ => {}
        }
    }
    
    // 5. Key concepts demonstrated
    println!("\n=== Key Concepts ===");
    println!("1. Events are the ONLY way to change state");
    println!("2. Each event has causation (what caused it)");
    println!("3. Events share correlation (they're related)");
    println!("4. Projections are built from events");
    println!("5. Projections are read-only views");
    
    // 6. Save events for persistence
    println!("\nSaving events...");
    
    // Serialize to JSON
    let json = serialize_events(&events).unwrap();
    println!("Events as JSON: {} bytes", json.len());
    
    // Create event journal
    let journal = EventJournal::new(events);
    println!("Event journal created with {} events", journal.metadata.event_count);
    
    println!("\n=== Example Complete ===");
}