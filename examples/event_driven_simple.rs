//! Simple event-driven example without complex dependencies
//! 
//! This example shows the core event-driven concepts without policies or state machines.

use cim_graph::{
    events::{GraphEvent, EventPayload, WorkflowPayload, ConceptPayload},
    contexts::BoundedContext,
    serde_support::{serialize_events, EventJournal},
};
use uuid::Uuid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Event-Driven Architecture Example ===\n");
    
    // 1. Create workflow events
    let workflow_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    
    println!("Creating workflow events...");
    
    let workflow_events = vec![
        // Define workflow
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            correlation_id,
            causation_id: None,
            payload: EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
                workflow_id,
                name: "Order Processing".to_string(),
                version: "1.0.0".to_string(),
            }),
        },
        // Add states
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            correlation_id,
            causation_id: None,
            payload: EventPayload::Workflow(WorkflowPayload::StateAdded {
                workflow_id,
                state_id: "new".to_string(),
                state_type: "initial".to_string(),
            }),
        },
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            correlation_id,
            causation_id: None,
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
            causation_id: None,
            payload: EventPayload::Workflow(WorkflowPayload::StateAdded {
                workflow_id,
                state_id: "shipped".to_string(),
                state_type: "final".to_string(),
            }),
        },
    ];
    
    // 2. Create concept events
    let concept_id = Uuid::new_v4();
    let concept_correlation_id = Uuid::new_v4();
    
    println!("\nCreating concept events...");
    
    let concept_events = vec![
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: concept_id,
            correlation_id: concept_correlation_id,
            causation_id: None,
            payload: EventPayload::Concept(ConceptPayload::ConceptDefined {
                concept_id: "order".to_string(),
                name: "Order".to_string(),
                definition: "A customer order in the system".to_string(),
            }),
        },
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: concept_id,
            correlation_id: concept_correlation_id,
            causation_id: None,
            payload: EventPayload::Concept(ConceptPayload::ConceptDefined {
                concept_id: "customer".to_string(),
                name: "Customer".to_string(),
                definition: "A customer who places orders".to_string(),
            }),
        },
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: concept_id,
            correlation_id: concept_correlation_id,
            causation_id: None,
            payload: EventPayload::Concept(ConceptPayload::RelationAdded {
                source_concept: "order".to_string(),
                target_concept: "customer".to_string(),
                relation_type: "placed_by".to_string(),
                strength: 1.0,
            }),
        },
    ];
    
    // 3. Show event counts
    println!("\nEvent Summary:");
    println!("  - Workflow events: {}", workflow_events.len());
    println!("  - Concept events: {}", concept_events.len());
    
    // 4. Demonstrate bounded contexts
    println!("\nBounded Contexts:");
    for context in &[
        BoundedContext::Workflow,
        BoundedContext::Concept,
        BoundedContext::Context,
        BoundedContext::Ipld,
        BoundedContext::Composed,
    ] {
        println!("  - {}", context.name());
        println!("    Aggregate type: {}", context.aggregate_type());
        println!("    Depends on: {:?}", 
                 context.upstream_contexts()
                     .iter()
                     .map(|c| c.name())
                     .collect::<Vec<_>>());
    }
    
    // 5. Show event correlation
    println!("\nEvent Correlation:");
    println!("  - All workflow events share correlation ID: {}", correlation_id);
    println!("  - All concept events share correlation ID: {}", concept_correlation_id);
    
    // 6. Serialize and save events
    let mut all_events = workflow_events;
    all_events.extend(concept_events);
    
    // Serialize to JSON
    let json = serialize_events(&all_events)?;
    println!("\nSerialized {} events to {} bytes of JSON", all_events.len(), json.len());
    
    // Create event journal
    let journal = EventJournal::new(all_events);
    println!("Event journal metadata:");
    println!("  - Event count: {}", journal.metadata.event_count);
    println!("  - Version: {}", journal.metadata.version);
    println!("  - Aggregate IDs: {}", journal.metadata.aggregate_ids.len());
    
    // Save to file
    let filename = "example_events.json";
    journal.save_to_file(filename)?;
    println!("\nSaved events to {}", filename);
    
    // 7. Key concepts summary
    println!("\n=== Key Concepts ===");
    println!("1. Events are the ONLY way to change state");
    println!("2. Each event belongs to an aggregate (identified by aggregate_id)");
    println!("3. Events can be correlated using correlation_id");
    println!("4. Events can track causation with causation_id");
    println!("5. Events are serialized and persisted as JSON");
    println!("6. Different bounded contexts have different event types");
    println!("7. Projections are built by replaying events (not shown here)");
    
    Ok(())
}