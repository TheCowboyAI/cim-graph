//! Demonstrates the ONLY way CIM graphs work - pure event-driven
//! 
//! This example shows:
//! - Commands are requests that may be rejected
//! - Events are facts that have happened
//! - State ONLY changes through events
//! - NATS JetStream is the event store
//! - cim-domain subject module defines the subject algebra

use cim_graph::core::event_driven::*;
use cim_graph::core::GraphType;
use uuid::Uuid;
use chrono::Utc;

/// This would normally connect to NATS JetStream
struct MockEventStream {
    events: Vec<GraphEvent>,
}

impl MockEventStream {
    fn new() -> Self {
        Self { events: Vec::new() }
    }
    
    /// Publish event to stream (would go to NATS)
    fn publish(&mut self, event: GraphEvent) {
        println!("📤 Publishing to NATS subject: {}", event.subject);
        println!("   Sequence: {}", event.sequence);
        println!("   Event ID: {}", event.event_id);
        self.events.push(event);
    }
    
    /// Subscribe to events (would come from NATS)
    fn subscribe(&self) -> Vec<GraphEvent> {
        self.events.clone()
    }
}

/// Example command handler
struct MyCommandHandler {
    next_sequence: u64,
}

impl CommandHandler for MyCommandHandler {
    fn handle(&self, command: GraphCommand, projection: &GraphProjection) -> Result<Vec<GraphEvent>, String> {
        match command {
            GraphCommand::CreateGraph { graph_id, graph_type, name } => {
                // Validate: graph must not already exist
                if projection.version > 0 {
                    return Err("Graph already exists".to_string());
                }
                
                // Emit event
                Ok(vec![GraphEvent {
                    event_id: Uuid::new_v4(),
                    sequence: self.next_sequence,
                    subject: build_subject(graph_type, "created"),
                    timestamp: Utc::now(),
                    aggregate_id: graph_id,
                    correlation_id: Uuid::new_v4(),
                    causation_id: None,
                    data: EventData::GraphCreated { graph_type, name },
                }])
            }
            
            GraphCommand::AddNode { graph_id, node_id, node_type, data } => {
                // Validate: node must not already exist
                if projection.nodes.contains_key(&node_id) {
                    return Err(format!("Node {} already exists", node_id));
                }
                
                // Emit event
                Ok(vec![GraphEvent {
                    event_id: Uuid::new_v4(),
                    sequence: projection.version + 1,
                    subject: build_subject(projection.graph_type, "node.added"),
                    timestamp: Utc::now(),
                    aggregate_id: graph_id,
                    correlation_id: Uuid::new_v4(),
                    causation_id: None,
                    data: EventData::NodeAdded { node_id, node_type, data },
                }])
            }
            
            GraphCommand::AddEdge { graph_id, edge_id, source_id, target_id, edge_type, data } => {
                // Validate: source and target must exist
                if !projection.nodes.contains_key(&source_id) {
                    return Err(format!("Source node {} does not exist", source_id));
                }
                if !projection.nodes.contains_key(&target_id) {
                    return Err(format!("Target node {} does not exist", target_id));
                }
                
                // Emit event
                Ok(vec![GraphEvent {
                    event_id: Uuid::new_v4(),
                    sequence: projection.version + 1,
                    subject: build_subject(projection.graph_type, "edge.added"),
                    timestamp: Utc::now(),
                    aggregate_id: graph_id,
                    correlation_id: Uuid::new_v4(),
                    causation_id: None,
                    data: EventData::EdgeAdded { edge_id, source_id, target_id, edge_type, data },
                }])
            }
            
            _ => Ok(vec![]),
        }
    }
}

fn main() {
    println!("=== CIM Graph: Pure Event-Driven Architecture ===\n");
    println!("Remember: Events are THE ONLY WAY to change state!\n");
    
    let graph_id = Uuid::new_v4();
    let mut event_stream = MockEventStream::new();
    let command_handler = MyCommandHandler { next_sequence: 1 };
    
    // Start with empty projection
    let mut projection = GraphProjection::new(graph_id);
    
    println!("1. Sending CreateGraph command...");
    let create_command = GraphCommand::CreateGraph {
        graph_id,
        graph_type: GraphType::WorkflowGraph,
        name: Some("Order Processing".to_string()),
    };
    
    // Command handler validates and produces events
    match command_handler.handle(create_command, &projection) {
        Ok(events) => {
            for event in events {
                event_stream.publish(event.clone());
                projection.apply(&event);
            }
        }
        Err(e) => println!("❌ Command rejected: {}", e),
    }
    
    println!("\n2. Sending AddNode commands...");
    let commands = vec![
        GraphCommand::AddNode {
            graph_id,
            node_id: "start".to_string(),
            node_type: "InitialState".to_string(),
            data: serde_json::json!({"label": "Order Received"}),
        },
        GraphCommand::AddNode {
            graph_id,
            node_id: "process".to_string(),
            node_type: "ProcessState".to_string(),
            data: serde_json::json!({"label": "Process Order"}),
        },
        GraphCommand::AddNode {
            graph_id,
            node_id: "complete".to_string(),
            node_type: "FinalState".to_string(),
            data: serde_json::json!({"label": "Order Complete"}),
        },
    ];
    
    for command in commands {
        match command_handler.handle(command, &projection) {
            Ok(events) => {
                for event in events {
                    event_stream.publish(event.clone());
                    projection.apply(&event);
                }
            }
            Err(e) => println!("❌ Command rejected: {}", e),
        }
    }
    
    println!("\n3. Attempting duplicate node (should be rejected)...");
    let duplicate_command = GraphCommand::AddNode {
        graph_id,
        node_id: "start".to_string(),
        node_type: "InitialState".to_string(),
        data: serde_json::json!({}),
    };
    
    match command_handler.handle(duplicate_command, &projection) {
        Ok(_) => println!("✅ Command accepted"),
        Err(e) => println!("❌ Command rejected: {}", e),
    }
    
    println!("\n4. Adding edges...");
    let edge_commands = vec![
        GraphCommand::AddEdge {
            graph_id,
            edge_id: "t1".to_string(),
            source_id: "start".to_string(),
            target_id: "process".to_string(),
            edge_type: "Transition".to_string(),
            data: serde_json::json!({"trigger": "order.validated"}),
        },
        GraphCommand::AddEdge {
            graph_id,
            edge_id: "t2".to_string(),
            source_id: "process".to_string(),
            target_id: "complete".to_string(),
            edge_type: "Transition".to_string(),
            data: serde_json::json!({"trigger": "order.processed"}),
        },
    ];
    
    for command in edge_commands {
        match command_handler.handle(command, &projection) {
            Ok(events) => {
                for event in events {
                    event_stream.publish(event.clone());
                    projection.apply(&event);
                }
            }
            Err(e) => println!("❌ Command rejected: {}", e),
        }
    }
    
    println!("\n5. Current projection state (built from events):");
    println!("   Version: {}", projection.version);
    println!("   Nodes: {}", projection.nodes.len());
    println!("   Edges: {}", projection.edges.len());
    
    println!("\n6. Rebuilding projection from event stream...");
    let events = event_stream.subscribe();
    let rebuilt = GraphProjection::from_events(graph_id, events.into_iter());
    println!("   Rebuilt version: {}", rebuilt.version);
    println!("   Rebuilt nodes: {}", rebuilt.nodes.len());
    println!("   Rebuilt edges: {}", rebuilt.edges.len());
    
    println!("\n=== Key Points ===");
    println!("✓ Commands can be rejected (validation)");
    println!("✓ Events are immutable facts");
    println!("✓ State ONLY changes through events");
    println!("✓ Projections can be rebuilt from events");
    println!("✓ NATS JetStream would persist the event stream");
    println!("✓ cim-domain subject module defines the subject hierarchy");
    println!("\n🚫 There is NO other way to modify graphs in CIM!");
}
