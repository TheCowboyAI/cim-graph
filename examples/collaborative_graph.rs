//! Collaborative graph editing through shared event streams
//! 
//! This demonstrates how CIM graphs enable real-time collaboration:
//! - Multiple clients subscribe to the same NATS JetStream subject
//! - Each client maintains a local projection of the graph
//! - Events from any client update all projections in real-time
//! - Everyone sees the same graph morphing as events occur

use cim_graph::core::event_driven::*;
use uuid::Uuid;
use cim_domain::{Subject, SubjectSegment};
use chrono::Utc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Simulated NATS JetStream connection
struct CollaborativeEventStream {
    subject: String,
    events: Arc<Mutex<Vec<GraphEvent>>>,
    subscribers: Arc<Mutex<Vec<Box<dyn Fn(&GraphEvent) + Send>>>>,
}

impl CollaborativeEventStream {
    fn new(subject: &str) -> Self {
        Self {
            subject: subject.to_string(),
            events: Arc::new(Mutex::new(Vec::new())),
            subscribers: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Publish event to stream - all subscribers will receive it
    fn publish(&self, mut event: GraphEvent) {
        // In real NATS, sequence would be assigned by the server
        let sequence = {
            let mut events = self.events.lock().unwrap();
            let seq = events.len() as u64 + 1;
            event.sequence = seq;
            event.subject = self.subject.clone();
            events.push(event.clone());
            seq
        };
        
        println!("📡 Published to {}: seq={} event={:?}", 
            self.subject, sequence, event.event_id);
        
        // Notify all subscribers
        let subscribers = self.subscribers.lock().unwrap();
        for subscriber in subscribers.iter() {
            subscriber(&event);
        }
    }
    
    /// Subscribe to events
    fn subscribe<F>(&self, handler: F) 
    where F: Fn(&GraphEvent) + Send + 'static
    {
        self.subscribers.lock().unwrap().push(Box::new(handler));
    }
    
    /// Get all events (for replay)
    fn replay(&self) -> Vec<GraphEvent> {
        self.events.lock().unwrap().clone()
    }
}

/// A collaborative client that can send commands and receive events
struct CollaborativeClient {
    client_id: String,
    projection: Arc<Mutex<GraphProjection>>,
    event_stream: Arc<CollaborativeEventStream>,
}

impl CollaborativeClient {
    fn new(client_id: &str, aggregate_id: Uuid, event_stream: Arc<CollaborativeEventStream>) -> Self {
        let client_id_owned = client_id.to_string();
        let projection = Arc::new(Mutex::new(GraphProjection::new(aggregate_id)));
        let projection_clone = projection.clone();
        
        // Subscribe to events and update local projection
        event_stream.subscribe(move |event| {
            let mut proj = projection_clone.lock().unwrap();
            proj.apply(event);
            println!("  📥 {} received event seq={}", client_id_owned, event.sequence);
        });
        
        Self {
            client_id: client_id.to_string(),
            projection,
            event_stream,
        }
    }
    
    /// Send a command (which will emit events if valid)
    fn send_command(&self, command: GraphCommand) {
        println!("\n👤 {} sending command: {:?}", self.client_id, 
            match &command {
                GraphCommand::AddNode { node_id, .. } => format!("AddNode({})", node_id),
                GraphCommand::AddEdge { edge_id, .. } => format!("AddEdge({})", edge_id),
                _ => "Other".to_string(),
            }
        );
        
        // In real system, this would go through command handler
        // For demo, we'll directly create events
        let event = match command {
            GraphCommand::AddNode { graph_id, node_id, node_type, data } => {
                GraphEvent {
                    event_id: Uuid::new_v4(),
                    sequence: 0, // Will be set by publish
                    subject: String::new(), // Will be set by publish
                    timestamp: Utc::now(),
                    aggregate_id: graph_id,
                    correlation_id: Uuid::new_v4(),
                    causation_id: None,
                    data: EventData::NodeAdded { node_id, node_type, data },
                }
            }
            GraphCommand::AddEdge { graph_id, edge_id, source_id, target_id, edge_type, data } => {
                GraphEvent {
                    event_id: Uuid::new_v4(),
                    sequence: 0,
                    subject: String::new(),
                    timestamp: Utc::now(),
                    aggregate_id: graph_id,
                    correlation_id: Uuid::new_v4(),
                    causation_id: None,
                    data: EventData::EdgeAdded { edge_id, source_id, target_id, edge_type, data },
                }
            }
            _ => return,
        };
        
        self.event_stream.publish(event);
    }
    
    /// Get current graph state
    fn get_state(&self) -> (usize, usize, u64) {
        let projection = self.projection.lock().unwrap();
        (projection.nodes.len(), projection.edges.len(), projection.version)
    }
    
    /// Display current graph
    fn display(&self) {
        let projection = self.projection.lock().unwrap();
        println!("\n📊 {} sees graph v{}: {} nodes, {} edges", 
            self.client_id, projection.version, 
            projection.nodes.len(), projection.edges.len()
        );
        
        for (id, node) in &projection.nodes {
            println!("   - Node {}: {}", id, node.node_type);
        }
        
        for (id, edge) in &projection.edges {
            println!("   - Edge {}: {} → {}", id, edge.source_id, edge.target_id);
        }
    }
}

fn main() {
    println!("=== Collaborative Graph Editing Demo ===\n");
    println!("Multiple clients working on the same graph through shared events\n");
    
    // Create shared event stream (would be NATS JetStream)
    let aggregate_id = Uuid::new_v4();
    let subject = Subject::from_segments(vec![
        SubjectSegment::new("cim").unwrap(),
        SubjectSegment::new("graph").unwrap(),
        SubjectSegment::new(aggregate_id.to_string()).unwrap(),
        SubjectSegment::new("events").unwrap(),
    ]).unwrap().to_string();
    let event_stream = Arc::new(CollaborativeEventStream::new(&subject));
    
    // Create multiple collaborative clients
    let alice = Arc::new(CollaborativeClient::new("Alice", aggregate_id, event_stream.clone()));
    let bob = Arc::new(CollaborativeClient::new("Bob", aggregate_id, event_stream.clone()));
    let charlie = Arc::new(CollaborativeClient::new("Charlie", aggregate_id, event_stream.clone()));
    
    // Initial state - everyone has empty graph
    println!("Initial state:");
    alice.display();
    bob.display();
    charlie.display();
    
    // Alice adds some nodes
    thread::sleep(Duration::from_millis(500));
    alice.send_command(GraphCommand::AddNode {
        graph_id: aggregate_id,
        node_id: "start".to_string(),
        node_type: "StartNode".to_string(),
        data: serde_json::json!({"label": "Begin"}),
    });
    
    thread::sleep(Duration::from_millis(200));
    alice.send_command(GraphCommand::AddNode {
        graph_id: aggregate_id,
        node_id: "process".to_string(),
        node_type: "ProcessNode".to_string(),
        data: serde_json::json!({"label": "Process Data"}),
    });
    
    // Bob adds a node and edge
    thread::sleep(Duration::from_millis(500));
    bob.send_command(GraphCommand::AddNode {
        graph_id: aggregate_id,
        node_id: "end".to_string(),
        node_type: "EndNode".to_string(),
        data: serde_json::json!({"label": "Complete"}),
    });
    
    thread::sleep(Duration::from_millis(200));
    bob.send_command(GraphCommand::AddEdge {
        graph_id: aggregate_id,
        edge_id: "e1".to_string(),
        source_id: "start".to_string(),
        target_id: "process".to_string(),
        edge_type: "Flow".to_string(),
        data: serde_json::json!({"condition": "valid"}),
    });
    
    // Charlie adds an edge
    thread::sleep(Duration::from_millis(500));
    charlie.send_command(GraphCommand::AddEdge {
        graph_id: aggregate_id,
        edge_id: "e2".to_string(),
        source_id: "process".to_string(),
        target_id: "end".to_string(),
        edge_type: "Flow".to_string(),
        data: serde_json::json!({"condition": "complete"}),
    });
    
    // Wait for all events to propagate
    thread::sleep(Duration::from_millis(500));
    
    // Everyone should see the same graph!
    println!("\n🎯 Final state - all clients see the same graph:");
    alice.display();
    bob.display();
    charlie.display();
    
    // Verify they all have identical state
    let alice_state = alice.get_state();
    let bob_state = bob.get_state();
    let charlie_state = charlie.get_state();
    
    println!("\n✅ Verification:");
    println!("   Alice:   {} nodes, {} edges, version {}", alice_state.0, alice_state.1, alice_state.2);
    println!("   Bob:     {} nodes, {} edges, version {}", bob_state.0, bob_state.1, bob_state.2);
    println!("   Charlie: {} nodes, {} edges, version {}", charlie_state.0, charlie_state.1, charlie_state.2);
    
    assert_eq!(alice_state, bob_state);
    assert_eq!(bob_state, charlie_state);
    println!("\n🎉 All clients have identical graphs!");
    
    // Demonstrate late-joining client
    println!("\n📱 David joins late and replays events...");
    let david = CollaborativeClient::new("David", aggregate_id, event_stream.clone());
    
    // Replay all events to catch up
    let events = event_stream.replay();
    for event in events {
        david.projection.lock().unwrap().apply(&event);
    }
    
    david.display();
    let david_state = david.get_state();
    assert_eq!(david_state, alice_state);
    println!("✅ David caught up and has the same graph!");
    
    println!("\n=== Key Points ===");
    println!("✓ All clients subscribe to the same NATS subject");
    println!("✓ Events from any client update all projections");
    println!("✓ Everyone sees the same graph morphing in real-time");
    println!("✓ Late-joining clients can replay events to catch up");
    println!("✓ Perfect consistency through event ordering");
    println!("✓ No conflicts - events are the single source of truth");
}
