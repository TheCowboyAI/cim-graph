//! Simple workflow example showing event-driven concepts
//! 
//! This example creates a basic workflow using events and shows
//! how to build projections.

use cim_graph::{
    core::cim_graph::{GraphEvent as CimGraphEvent, EventData},
    core::{ProjectionEngine, GraphProjection},
    core::projection_engine::GenericGraphProjection,
    graphs::workflow::{WorkflowNode, WorkflowEdge},
};
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;

fn main() {
    println!("=== Simple Workflow Example ===\n");
    
    // 1. Create workflow initialization event
    let workflow_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    
    let init_event = CimGraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id: workflow_id,
        sequence: 1,
        subject: "cim.graph.workflow.initialized".to_string(),
        timestamp: Utc::now(),
        correlation_id,
        causation_id: None,
        data: EventData::GraphInitialized {
            graph_type: "workflow".to_string(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("name".to_string(), serde_json::json!("Order Processing"));
                meta.insert("version".to_string(), serde_json::json!("1.0.0"));
                meta
            },
        },
    };
    
    // 2. Add workflow states as events
    let state_events = vec![
        CimGraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            sequence: 2,
            subject: "cim.graph.workflow.node.added".to_string(),
            timestamp: Utc::now(),
            correlation_id,
            causation_id: Some(init_event.event_id),
            data: EventData::NodeAdded {
                node_id: "start".to_string(),
                node_type: "start".to_string(),
                data: serde_json::json!({ "type": "initial" }),
            },
        },
        CimGraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            sequence: 3,
            subject: "cim.graph.workflow.node.added".to_string(),
            timestamp: Utc::now(),
            correlation_id,
            causation_id: Some(init_event.event_id),
            data: EventData::NodeAdded {
                node_id: "processing".to_string(),
                node_type: "state".to_string(),
                data: serde_json::json!({ "name": "Processing Order" }),
            },
        },
        CimGraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            sequence: 4,
            subject: "cim.graph.workflow.node.added".to_string(),
            timestamp: Utc::now(),
            correlation_id,
            causation_id: Some(init_event.event_id),
            data: EventData::NodeAdded {
                node_id: "completed".to_string(),
                node_type: "end".to_string(),
                data: serde_json::json!({ "type": "final" }),
            },
        },
    ];
    
    // 3. Add transitions between states
    let transition_events = vec![
        CimGraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            sequence: 5,
            subject: "cim.graph.workflow.edge.added".to_string(),
            timestamp: Utc::now(),
            correlation_id,
            causation_id: Some(state_events[0].event_id),
            data: EventData::EdgeAdded {
                edge_id: "start-to-processing".to_string(),
                source_id: "start".to_string(),
                target_id: "processing".to_string(),
                edge_type: "transition".to_string(),
                data: serde_json::json!({ "trigger": "submit" }),
            },
        },
        CimGraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            sequence: 6,
            subject: "cim.graph.workflow.edge.added".to_string(),
            timestamp: Utc::now(),
            correlation_id,
            causation_id: Some(state_events[1].event_id),
            data: EventData::EdgeAdded {
                edge_id: "processing-to-completed".to_string(),
                source_id: "processing".to_string(),
                target_id: "completed".to_string(),
                edge_type: "transition".to_string(),
                data: serde_json::json!({ "trigger": "finish" }),
            },
        },
    ];
    
    // 4. Collect all events
    let mut all_events = vec![init_event];
    all_events.extend(state_events);
    all_events.extend(transition_events);
    
    println!("Created {} events for workflow", all_events.len());
    
    // 5. Build projection from events
    let engine = ProjectionEngine::<GenericGraphProjection<WorkflowNode, WorkflowEdge>>::new();
    let projection = engine.project(all_events.clone());
    
    // 6. Query the projection
    println!("\nWorkflow Projection:");
    println!("  - Aggregate ID: {}", projection.aggregate_id());
    println!("  - Version: {} (events processed)", projection.version());
    println!("  - Nodes: {}", projection.node_count());
    println!("  - Edges: {}", projection.edge_count());
    
    // 7. Show event causation chain
    println!("\nEvent Causation Chain:");
    for event in &all_events {
        let event_type = match &event.data {
            EventData::GraphInitialized { .. } => "GraphInitialized",
            EventData::NodeAdded { .. } => "NodeAdded",
            EventData::EdgeAdded { .. } => "EdgeAdded",
            _ => "Other",
        };
        println!("  {} - Event {} caused by: {:?}", 
                 event.sequence,
                 event_type,
                 event.causation_id);
    }
    
    // 8. Key concepts
    println!("\n=== Key Concepts Demonstrated ===");
    println!("1. Events are the source of truth");
    println!("2. Each event has sequence number and timestamp");
    println!("3. Events use NATS subject patterns");
    println!("4. Projections are built by replaying events");
    println!("5. Causation tracks what caused each event");
    println!("6. Correlation groups related events");
}