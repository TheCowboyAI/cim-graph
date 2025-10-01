//! Example showing proper ECS + DDD graph implementation
//!
//! - Entities are just IDs
//! - Components are value objects
//! - Systems are functions
//! - State transitions through StateMachine only

use cim_graph::core::{GraphAggregateProjection, build_projection};
use cim_graph::core::aggregate_projection::{query_entities_with_component, get_entity_components};
use cim_graph::core::state_machine::{GraphStateMachine, process_command};
use cim_graph::events::{GraphEvent, GraphCommand, EventPayload, IpldPayload};
use uuid::Uuid;
use cim_domain::{Subject, SubjectSegment};

fn main() {
    println!("=== ECS + DDD Graph System Example ===\n");
    
    // Create aggregate ID (this is the graph)
    let aggregate_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    
    println!("📊 Graph Aggregate: {}", aggregate_id);
    println!("🔗 Correlation ID: {}\n", correlation_id);
    
    // Initialize state machine (enforces valid transitions)
    let mut state_machine = GraphStateMachine::new();
    
    // Start with empty projection
    let subject = Subject::from_segments(vec![
        SubjectSegment::new("cim").unwrap(),
        SubjectSegment::new("graph").unwrap(),
        SubjectSegment::new(aggregate_id.to_string()).unwrap(),
        SubjectSegment::new("events").unwrap(),
    ])
    .unwrap()
    .to_string();
    let mut projection = GraphAggregateProjection::new(aggregate_id, subject.clone());
    
    // Simulate events from JetStream (with sequence numbers)
    let mut events: Vec<(GraphEvent, u64)> = Vec::new();
    let mut sequence = 1;
    
    // Command 1: Initialize IPLD graph
    println!("🎯 Command: Initialize IPLD Graph");
    let init_command = GraphCommand::InitializeGraph {
        aggregate_id,
        graph_type: "ipld".to_string(),
        correlation_id,
    };
    
    // Process through state machine
    match process_command(&mut state_machine, init_command, &projection) {
        Ok(new_events) => {
            for event in new_events {
                println!("  ✅ Event generated: {:?}", event.event_id);
                events.push((event.clone(), sequence));
                projection.apply(&event, sequence);
                sequence += 1;
            }
        }
        Err(e) => println!("  ❌ Command rejected: {}", e),
    }
    
    // Command 2: Add a CID (entity with components)
    println!("\n🎯 Command: Add CID to graph");
    
    // Generate event directly (in real system, would go through command handler)
    let add_event = GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        correlation_id,
        causation_id: events.last().map(|(e, _)| e.event_id),
        payload: EventPayload::Ipld(IpldPayload::CidAdded {
            cid: "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_string(),
            codec: "dag-cbor".to_string(),
            size: 1024,
            data: serde_json::json!({
                "title": "Hello IPLD",
                "content": "This is content-addressed data"
            }),
        }),
    };
    
    println!("  ✅ Event generated: {:?}", add_event.event_id);
    events.push((add_event.clone(), sequence));
    projection.apply(&add_event, sequence);
    sequence += 1;
    
    // Command 3: Add another CID
    println!("\n🎯 Command: Add another CID");
    let add_event2 = GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        correlation_id,
        causation_id: Some(add_event.event_id),
        payload: EventPayload::Ipld(IpldPayload::CidAdded {
            cid: "QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco".to_string(),
            codec: "dag-cbor".to_string(),
            size: 512,
            data: serde_json::json!({
                "title": "Linked Data",
                "ref": "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG"
            }),
        }),
    };
    
    println!("  ✅ Event generated: {:?}", add_event2.event_id);
    events.push((add_event2.clone(), sequence));
    projection.apply(&add_event2, sequence);
    sequence += 1;
    
    // Command 4: Link CIDs
    println!("\n🎯 Command: Link CIDs");
    let link_event = GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        correlation_id,
        causation_id: Some(add_event2.event_id),
        payload: EventPayload::Ipld(IpldPayload::CidLinkAdded {
            cid: "QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco".to_string(),
            link_name: "previous".to_string(),
            target_cid: "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_string(),
        }),
    };
    
    println!("  ✅ Event generated: {:?}", link_event.event_id);
    events.push((link_event.clone(), sequence));
    projection.apply(&link_event, sequence);
    // sequence += 1; // Would increment in real system but not needed for demo
    
    // Now demonstrate querying the projection using Systems
    println!("\n📊 Current Projection State:");
    println!("  Version: {}", projection.version);
    println!("  Subject: {}", projection.subject);
    
    // System: Query all entities with CID component
    println!("\n🔍 System: Query entities with 'cid' component");
    let cid_entities = query_entities_with_component(&projection, "cid");
    for entity_id in &cid_entities {
        println!("  Entity: {}", entity_id);
        
        // System: Get components for entity
        if let Some(components) = get_entity_components(&projection, entity_id) {
            println!("    Components:");
            for (component_type, data) in components {
                println!("      - {}: {}", component_type, data);
            }
        }
    }
    
    // System: Query relationships
    println!("\n🔍 System: Query all relationships");
    for (edge_id, relationships) in &projection.relationships {
        println!("  Edge: {}", edge_id);
        for (source, target) in relationships {
            println!("    {} → {}", source, target);
        }
    }
    
    // Demonstrate rebuilding from events (left fold)
    println!("\n🔄 Rebuilding projection from event stream...");
    let rebuilt = build_projection(events.clone());
    println!("  ✅ Rebuilt to version: {}", rebuilt.version);
    println!("  ✅ Entities: {}", rebuilt.components.len());
    println!("  ✅ Relationships: {}", rebuilt.relationships.len());
    
    // Key principles demonstrated
    println!("\n💡 Key ECS + DDD Principles:");
    println!("  1. Graph is just an aggregate ID");
    println!("  2. Nodes/edges are entities (just IDs)");
    println!("  3. Properties are components (value objects)");
    println!("  4. State = left fold of events");
    println!("  5. Systems are functions that query/command");
    println!("  6. State transitions ONLY through StateMachine");
    println!("  7. Everything is immutable except the event log");
}
