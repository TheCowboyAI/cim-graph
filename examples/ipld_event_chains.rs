//! Demonstrates IPLD CID chains - the heart of CIM's storage system
//!
//! Shows how:
//! - Event payloads are given CIDs
//! - CIDs form Merkle DAGs
//! - Entire aggregate transactions can be referenced by a single CID
//! - Event streams can be retrieved from JetStream with one CID request

use cim_graph::graphs::{Cid, CidChain, EventChainBuilder, CidGenerator};
use cim_graph::core::{CimGraphEvent, EventData, GraphCommand};
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;

/// Example CID generator that shows the concept
/// (Real implementation would use cim-ipld)
struct ExampleCidGenerator;

impl CidGenerator for ExampleCidGenerator {
    fn generate_cid(&self, data: &EventData) -> Cid {
        // Simulate CID generation from event data
        let json = serde_json::to_string(data).unwrap();
        let hash = sha256_mock(&json);
        Cid::new(format!("Qm{}", hash))
    }
    
    fn verify_cid(&self, cid: &Cid, data: &EventData) -> bool {
        self.generate_cid(data) == *cid
    }
}

/// Mock SHA256 for demo (first 16 chars of a simple hash)
fn sha256_mock(data: &str) -> String {
    data.bytes()
        .take(8)
        .map(|b| format!("{:02x}", b))
        .collect()
}

fn main() {
    println!("=== IPLD CID Chains - The Heart of CIM Storage ===\n");
    
    // Create CID generator (would use cim-ipld in production)
    let cid_gen = ExampleCidGenerator;
    let chain_builder = EventChainBuilder::new(cid_gen);
    
    // Simulate an aggregate's event stream
    let aggregate_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    
    println!("📊 Creating event stream for aggregate: {}", aggregate_id);
    println!("🔗 Correlation ID: {}\n", correlation_id);
    
    // Build a sequence of events
    let mut events = Vec::new();
    
    // Event 1: Initialize graph
    let event1 = CimGraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        sequence: 1,
        subject: format!("graph.{}.events", aggregate_id),
        timestamp: Utc::now(),
        correlation_id,
        causation_id: None,
        data: EventData::GraphInitialized {
            graph_type: "workflow".to_string(),
            metadata: HashMap::from([
                ("name".to_string(), serde_json::json!("Order Processing")),
                ("version".to_string(), serde_json::json!("1.0.0")),
            ]),
        },
    };
    events.push(event1.clone());
    
    // Event 2: Add first node
    let event2 = CimGraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        sequence: 2,
        subject: format!("graph.{}.events", aggregate_id),
        timestamp: Utc::now(),
        correlation_id,
        causation_id: Some(event1.event_id),
        data: EventData::NodeAdded {
            node_id: "order_received".to_string(),
            node_type: "state".to_string(),
            data: serde_json::json!({
                "label": "Order Received",
                "initial": true,
            }),
        },
    };
    events.push(event2.clone());
    
    // Event 3: Add second node
    let event3 = CimGraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        sequence: 3,
        subject: format!("graph.{}.events", aggregate_id),
        timestamp: Utc::now(),
        correlation_id,
        causation_id: Some(event2.event_id),
        data: EventData::NodeAdded {
            node_id: "payment_pending".to_string(),
            node_type: "state".to_string(),
            data: serde_json::json!({
                "label": "Payment Pending",
                "timeout": "PT1H",
            }),
        },
    };
    events.push(event3.clone());
    
    // Event 4: Add edge
    let event4 = CimGraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        sequence: 4,
        subject: format!("graph.{}.events", aggregate_id),
        timestamp: Utc::now(),
        correlation_id,
        causation_id: Some(event3.event_id),
        data: EventData::EdgeAdded {
            edge_id: "validate_order".to_string(),
            source_id: "order_received".to_string(),
            target_id: "payment_pending".to_string(),
            edge_type: "transition".to_string(),
            data: serde_json::json!({
                "action": "validate",
                "guard": "order.isValid()",
            }),
        },
    };
    events.push(event4);
    
    // Build the CID chain
    println!("🔨 Building CID chain from {} events...\n", events.len());
    let chain = chain_builder.build_chain(&events);
    
    // Display the Merkle DAG structure
    println!("📊 Merkle DAG Structure:");
    println!("└─ Root CID: {}", chain.root.as_str());
    
    for (seq, cid) in &chain.cids {
        println!("   ├─ Event {}: CID = {}", seq, cid.as_str());
        
        // Show what each CID represents
        let event = &events[(*seq as usize) - 1];
        match &event.data {
            EventData::GraphInitialized { graph_type, .. } => {
                println!("   │  └─ Type: Initialize {} graph", graph_type);
            }
            EventData::NodeAdded { node_id, node_type, .. } => {
                println!("   │  └─ Type: Add {} node '{}'", node_type, node_id);
            }
            EventData::EdgeAdded { source_id, target_id, .. } => {
                println!("   │  └─ Type: Add edge {} → {}", source_id, target_id);
            }
            _ => {}
        }
    }
    
    println!("\n🔗 Chain Properties:");
    println!("   • Aggregate ID: {}", chain.aggregate_id);
    println!("   • Total Events: {}", chain.length);
    println!("   • Root CID: {}", chain.root.as_str());
    println!("   • Latest Time: {}", chain.latest_timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
    
    // Demonstrate how a single CID can retrieve the entire stream
    println!("\n🚀 With NATS JetStream Integration:");
    println!("   1. Store each event payload in IPLD with its CID");
    println!("   2. Each event links to previous CID (forming Merkle DAG)");
    println!("   3. Root CID '{}' identifies entire transaction", chain.root.as_str());
    println!("   4. Single request retrieves complete event stream!");
    
    // Show the power of CID chains for verification
    println!("\n🔐 CID Chain Benefits:");
    println!("   ✓ Immutable history - CIDs change if data changes");
    println!("   ✓ Efficient sync - only fetch missing CIDs");
    println!("   ✓ Cryptographic proof - verify entire history");
    println!("   ✓ Content addressing - deduplication across aggregates");
    
    // Demonstrate transaction references
    println!("\n💡 Transaction References:");
    println!("   • Share this CID to reference entire workflow: {}", chain.root.as_str());
    println!("   • Others can retrieve & verify the complete history");
    println!("   • Perfect for audit trails and compliance");
    
    // Show how this integrates with the larger system
    println!("\n🏗️  System Integration:");
    println!("   cim-graph     → Defines graph events and projections");
    println!("   cim-ipld      → Generates CIDs and manages Merkle DAGs");
    println!("   cim-subject   → Defines NATS subject algebra");
    println!("   NATS JetStream → Persists events with CID indexing");
    println!("\n   Result: Complete event history retrievable by single CID!");
}

// Type alias for clarity
type CimGraphEvent = cim_graph::core::CimGraphEvent;