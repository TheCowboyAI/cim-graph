//! Complete example demonstrating the event-driven architecture
//! 
//! This example shows how to:
//! - Create events for different graph types
//! - Build projections from event streams
//! - Use state machines for validation
//! - Apply policies for automated behavior
//! - Store events in IPLD chains
//! - Query projections

use cim_graph::{
    core::{
        PolicyEngine, PolicyContext,
        CidGenerationPolicy, ProjectionUpdatePolicy, StateValidationPolicy,
        GraphStateMachine, GraphAggregateProjection,
    },
    events::{
        GraphEvent, EventPayload, 
        WorkflowPayload, ConceptPayload, ContextPayload, ComposedPayload,
    },
    contexts::{BoundedContext, ContextMap},
};
use std::collections::HashMap;
use uuid::Uuid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CIM Graph Event-Driven Architecture Demo ===\n");
    
    // 1. Set up the policy engine with automated behaviors
    let mut policy_engine = PolicyEngine::new();
    policy_engine.add_policy(Box::new(CidGenerationPolicy));
    policy_engine.add_policy(Box::new(ProjectionUpdatePolicy));
    policy_engine.add_policy(Box::new(StateValidationPolicy));
    
    // 2. Create state machine and IPLD chains for validation
    let mut state_machine = GraphStateMachine::new();
    let mut ipld_chains = HashMap::new();
    
    // 3. Demonstrate Workflow Context
    println!("--- Workflow Context ---");
    let workflow_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    
    // Create workflow definition event
    let workflow_event = GraphEvent {
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
    
    // Create workflow state events
    let state_events = vec![
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            correlation_id,
            causation_id: Some(workflow_event.event_id),
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
            causation_id: Some(workflow_event.event_id),
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
            causation_id: Some(workflow_event.event_id),
            payload: EventPayload::Workflow(WorkflowPayload::StateAdded {
                workflow_id,
                state_id: "completed".to_string(),
                state_type: "final".to_string(),
            }),
        },
    ];
    
    // Create transition events
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
    
    // Collect all workflow events
    let mut all_workflow_events = vec![workflow_event.clone()];
    all_workflow_events.extend(state_events.clone());
    all_workflow_events.extend(transition_events.clone());
    
    // Apply policies to generate additional events (CIDs, etc.)
    let mut policy_context = PolicyContext {
        state_machine: &mut state_machine,
        ipld_chains: &mut ipld_chains,
        metrics: Default::default(),
    };
    
    for event in &all_workflow_events {
        let actions = policy_engine.execute_policies(event, &mut policy_context)?;
        println!("Policy actions for event {}: {} actions", 
                 event.event_id, actions.len());
    }
    
    // Build workflow projection
    use cim_graph::core::build_projection;
    let event_tuples: Vec<(GraphEvent, u64)> = all_workflow_events.iter()
        .enumerate()
        .map(|(i, e)| (e.clone(), i as u64 + 1))
        .collect();
    let workflow_projection = build_projection(event_tuples);
    
    println!("Workflow projection built:");
    println!("  - Components: {}", workflow_projection.components.len());
    println!("  - Relationships: {}", workflow_projection.relationships.len());
    println!("  - Version: {}", workflow_projection.version);
    
    // 4. Demonstrate Concept Context
    println!("\n--- Concept Context ---");
    let concept_id = Uuid::new_v4();
    
    let concept_events = vec![
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: concept_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Concept(ConceptPayload::ConceptDefined {
                concept_id: "order".to_string(),
                name: "Order".to_string(),
                definition: "A customer order with attributes: id, customer, items, total".to_string(),
            }),
        },
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: concept_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Concept(ConceptPayload::ConceptDefined {
                concept_id: "customer".to_string(),
                name: "Customer".to_string(),
                definition: "A customer who places orders with attributes: id, name, email".to_string(),
            }),
        },
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: concept_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Concept(ConceptPayload::RelationAdded {
                source_concept: "order".to_string(),
                target_concept: "customer".to_string(),
                relation_type: "placed_by".to_string(),
                strength: 1.0,
            }),
        },
    ];
    
    // Build concept projection
    let event_tuples: Vec<(GraphEvent, u64)> = concept_events.iter()
        .enumerate()
        .map(|(i, e)| (e.clone(), i as u64 + 1))
        .collect();
    let concept_projection = build_projection(event_tuples);
    
    println!("Concept projection built:");
    println!("  - Components: {}", concept_projection.components.len());
    println!("  - Relationships: {}", concept_projection.relationships.len());
    
    // 5. Demonstrate Context Context (DDD)
    println!("\n--- Context Context (DDD) ---");
    let context_id = Uuid::new_v4();
    
    let context_events = vec![
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: context_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Context(ContextPayload::BoundedContextCreated {
                context_id: "orders".to_string(),
                name: "Order Management".to_string(),
                description: "Handles order lifecycle".to_string(),
            }),
        },
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: context_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Context(ContextPayload::AggregateAdded {
                context_id: "orders".to_string(),
                aggregate_id: Uuid::new_v4(),
                aggregate_type: "Order".to_string(),
            }),
        },
    ];
    
    // Build context projection
    let event_tuples: Vec<(GraphEvent, u64)> = context_events.iter()
        .enumerate()
        .map(|(i, e)| (e.clone(), i as u64 + 1))
        .collect();
    let context_projection = build_projection(event_tuples);
    
    println!("Context projection built:");
    println!("  - DDD elements: {}", context_projection.components.len());
    
    // 6. Demonstrate Composed Context
    println!("\n--- Composed Context ---");
    let composed_id = Uuid::new_v4();
    
    let composed_events = vec![
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: composed_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Composed(ComposedPayload::SubGraphAdded {
                subgraph_id: workflow_id,
                graph_type: "workflow".to_string(),
                namespace: "processes".to_string(),
            }),
        },
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: composed_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Composed(ComposedPayload::SubGraphAdded {
                subgraph_id: concept_id,
                graph_type: "concept".to_string(),
                namespace: "domain".to_string(),
            }),
        },
    ];
    
    // Build composed projection
    let event_tuples: Vec<(GraphEvent, u64)> = composed_events.iter()
        .enumerate()
        .map(|(i, e)| (e.clone(), i as u64 + 1))
        .collect();
    let composed_projection = build_projection(event_tuples);
    
    println!("Composed projection built:");
    println!("  - Sub-graphs: {}", composed_projection.components.len());
    
    // 7. Show bounded contexts and their relationships
    println!("\n--- Bounded Contexts ---");
    let context_map = ContextMap::new();
    
    // Show relationships from the context map
    println!("Context relationships:");
    let workflow_context = BoundedContext::Workflow;
    let relationships = context_map.relationships_for(&workflow_context);
    println!("  Workflow context has {} relationships", relationships.len());
    for rel in relationships {
        println!("    - {} -> {}: {}", 
                 rel.from_context.name(), 
                 rel.to_context.name(), 
                 rel.description);
    }
    
    println!("\nContext dependencies:");
    for context in &[
        BoundedContext::Workflow,
        BoundedContext::Concept,
        BoundedContext::Context,
        BoundedContext::Composed,
    ] {
        let upstream = context.upstream_contexts();
        let downstream = context.downstream_contexts();
        println!("  {} context:", context.name());
        println!("    - Depends on: {:?}", upstream.iter().map(|c| c.name()).collect::<Vec<_>>());
        println!("    - Used by: {:?}", downstream.iter().map(|c| c.name()).collect::<Vec<_>>());
    }
    
    // 8. Demonstrate event correlation and causation
    println!("\n--- Event Correlation & Causation ---");
    println!("Workflow event chain:");
    for (i, event) in all_workflow_events.iter().enumerate() {
        println!("  {}. Event {} caused by: {:?}", 
                 i + 1,
                 event.event_id,
                 event.causation_id);
    }
    
    // 9. Show IPLD chain construction
    println!("\n--- IPLD Chains ---");
    if let Some(chain) = ipld_chains.get(&workflow_id) {
        println!("Workflow {} has IPLD chain with {} blocks", 
                 workflow_id, 
                 chain.metadata.length);
    }
    
    // 10. Demonstrate aggregate projection
    println!("\n--- Aggregate Projection ---");
    let mut aggregate_projection = GraphAggregateProjection::new(
        workflow_id,
        format!("graph.{}.events", workflow_id),
    );
    
    // Apply events to aggregate projection
    for (index, event) in all_workflow_events.iter().enumerate() {
        aggregate_projection.apply(event, (index + 1) as u64);
    }
    
    println!("Aggregate projection state:");
    println!("  - Aggregate ID: {}", aggregate_projection.aggregate_id);
    println!("  - Version: {}", aggregate_projection.version);
    println!("  - Components: {}", aggregate_projection.components.len());
    println!("  - Relationships: {}", aggregate_projection.relationships.len());
    
    // Show some component details
    if let Some((entity_id, components)) = aggregate_projection.components.iter().next() {
        println!("  - Sample entity '{}' has {} components", entity_id, components.len());
    }
    
    println!("\n=== Demo Complete ===");
    println!("This example demonstrated:");
    println!("- Creating events for different graph types");
    println!("- Building projections from event streams");
    println!("- Using policies for automated behavior");
    println!("- Event correlation and causation tracking");
    println!("- Bounded contexts and their relationships");
    println!("- IPLD chain construction for event storage");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_complete_example() {
        main().unwrap();
    }
}