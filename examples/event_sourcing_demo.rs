//! Demonstrates event sourcing with correlation, causation, and deterministic ordering

use cim_graph::core::event_sourcing::*;
use cim_graph::core::GraphType;
use uuid::Uuid;

fn main() {
    println!("=== CIM Graph Event Sourcing Demo ===\n");
    
    // Create IDs for our demonstration
    let aggregate_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    let user = "demo-user";
    
    // Initialize event store
    let mut store = MemoryEventStore::default();
    
    println!("1. Creating a workflow graph with correlated events...");
    
    // Event 1: Create graph
    let create_event = GraphEvent {
        metadata: EventMetadata::new(aggregate_id, correlation_id, 1, user.to_string()),
        payload: GraphEventPayload::GraphCreated {
            graph_type: GraphType::WorkflowGraph,
            name: Some("Order Processing Workflow".to_string()),
            description: Some("Handles order lifecycle".to_string()),
        },
    };
    
    println!("   - Event: GraphCreated (v1)");
    println!("   - Correlation ID: {}", correlation_id);
    
    // Event 2: Add start node (caused by graph creation)
    let add_start = GraphEvent {
        metadata: EventMetadata::new(aggregate_id, correlation_id, 2, user.to_string())
            .with_causation(create_event.metadata.event_id),
        payload: GraphEventPayload::NodeAdded {
            node_id: "start".to_string(),
            node_type: "StartState".to_string(),
            data: serde_json::json!({"label": "Order Received"}),
        },
    };
    
    println!("   - Event: NodeAdded 'start' (v2)");
    println!("   - Caused by: GraphCreated event");
    
    // Event 3: Add process node
    let add_process = GraphEvent {
        metadata: EventMetadata::new(aggregate_id, correlation_id, 3, user.to_string())
            .with_causation(add_start.metadata.event_id),
        payload: GraphEventPayload::NodeAdded {
            node_id: "process".to_string(),
            node_type: "ProcessState".to_string(),
            data: serde_json::json!({"label": "Process Order", "timeout": "5m"}),
        },
    };
    
    // Event 4: Connect nodes
    let add_transition = GraphEvent {
        metadata: EventMetadata::new(aggregate_id, correlation_id, 4, user.to_string())
            .with_causation(add_process.metadata.event_id),
        payload: GraphEventPayload::EdgeAdded {
            edge_id: "t1".to_string(),
            source_id: "start".to_string(),
            target_id: "process".to_string(),
            edge_type: "Transition".to_string(),
            data: serde_json::json!({"trigger": "order.validated"}),
        },
    };
    
    // Store all events
    store.append(vec![
        create_event.clone(),
        add_start.clone(),
        add_process.clone(),
        add_transition.clone(),
    ]).unwrap();
    
    println!("\n2. Demonstrating correlation - all events in the same workflow:");
    let correlated = store.get_correlated_events(correlation_id);
    for event in &correlated {
        let event_type = match &event.payload {
            GraphEventPayload::GraphCreated { .. } => "GraphCreated".to_string(),
            GraphEventPayload::NodeAdded { node_id, .. } => format!("NodeAdded({})", node_id),
            GraphEventPayload::EdgeAdded { edge_id, .. } => format!("EdgeAdded({})", edge_id),
            _ => "Other".to_string(),
        };
        println!("   - v{}: {}", event.metadata.version, event_type);
    }
    
    println!("\n3. Demonstrating causation chain:");
    let chain = store.get_causation_chain(add_transition.metadata.event_id);
    println!("   Causation chain for 'add_transition' event:");
    for (i, event) in chain.iter().enumerate() {
        let event_type = match &event.payload {
            GraphEventPayload::GraphCreated { .. } => "GraphCreated".to_string(),
            GraphEventPayload::NodeAdded { node_id, .. } => format!("NodeAdded({})", node_id),
            GraphEventPayload::EdgeAdded { edge_id, .. } => format!("EdgeAdded({})", edge_id),
            _ => "Other".to_string(),
        };
        println!("   {}→ v{}: {}", 
            "  ".repeat(i),
            event.metadata.version,
            event_type
        );
    }
    
    println!("\n4. Demonstrating deterministic ordering:");
    println!("   Events are always returned in version order:");
    let all_events = store.get_events(aggregate_id);
    for event in &all_events {
        println!("   - Version {}: occurred at {}", 
            event.metadata.version,
            event.metadata.occurred_at.format("%H:%M:%S%.3f")
        );
    }
    
    println!("\n5. Demonstrating aggregate replay:");
    let aggregate = GraphAggregate::replay(aggregate_id, all_events);
    println!("   Aggregate state after replay:");
    println!("   - Version: {}", aggregate.version());
    println!("   - Nodes: {}", aggregate.node_count());
    println!("   - Edges: {}", aggregate.edge_count());
    
    println!("\n6. Demonstrating version range queries:");
    let range = store.get_events_in_range(aggregate_id, 2, 3);
    println!("   Events between version 2 and 3:");
    for event in &range {
        let event_type = match &event.payload {
            GraphEventPayload::NodeAdded { node_id, .. } => format!("NodeAdded({})", node_id),
            _ => "Other".to_string(),
        };
        println!("   - v{}: {}", event.metadata.version, event_type);
    }
    
    println!("\n=== Key Benefits ===");
    println!("1. Correlation IDs group related events across aggregates");
    println!("2. Causation IDs show the chain of events that led to each action");
    println!("3. Version numbers provide deterministic ordering within aggregates");
    println!("4. Event replay allows rebuilding state from any point in time");
    println!("5. All events are immutable and form an audit log");
}