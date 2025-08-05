//! Example demonstrating event-driven graph operations with NATS JetStream
//!
//! This example shows how to:
//! - Connect to NATS JetStream
//! - Publish graph events
//! - Build projections from event streams
//! - Apply automated policies
//! - Handle real-time subscriptions

use cim_graph::{
    events::{GraphEvent, EventPayload, GenericPayload, WorkflowPayload},
    nats::{JetStreamEventStore, JetStreamConfig},
    core::{
        GraphStateMachine, IpldChainAggregate, PolicyEngine, PolicyContext,
    },
    Result,
};
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    println!("CIM Graph - Event-Driven Architecture with NATS JetStream\n");
    
    // 1. Setup NATS JetStream connection
    println!("1. Connecting to NATS JetStream...");
    let config = JetStreamConfig {
        server_url: "localhost:4222".to_string(),
        stream_name: "CIM_GRAPH_DEMO".to_string(),
        subject_prefix: "cim.demo".to_string(),
        max_age_secs: 3600, // 1 hour for demo
        enable_dedup: true,
    };
    
    let event_store = match JetStreamEventStore::new(config).await {
        Ok(store) => {
            println!("   ✓ Connected to NATS at localhost:4222");
            store
        }
        Err(e) => {
            eprintln!("   ✗ Failed to connect: {}", e);
            eprintln!("\n   Please ensure NATS is running:");
            eprintln!("   docker run -p 4222:4222 nats:latest -js");
            return Err(e.into());
        }
    };
    
    // 2. Create a workflow aggregate
    let workflow_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    println!("\n2. Creating workflow aggregate: {}", workflow_id);
    
    // 3. Publish workflow events
    println!("\n3. Publishing workflow events...");
    
    // Event 1: Define workflow
    let event1 = GraphEvent {
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
    
    let seq1 = event_store.publish_event(event1.clone(), None).await?;
    println!("   ✓ Published WorkflowDefined (seq: {})", seq1);
    
    // Event 2: Add states
    let states = vec!["pending", "processing", "shipped", "delivered", "cancelled"];
    for state in states {
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            correlation_id,
            causation_id: Some(event1.event_id),
            payload: EventPayload::Workflow(WorkflowPayload::StateAdded {
                workflow_id,
                state_id: state.to_string(),
                state_type: "state".to_string(),
            }),
        };
        
        let seq = event_store.publish_event(event, None).await?;
        println!("   ✓ Published StateAdded: {} (seq: {})", state, seq);
    }
    
    // Event 3: Add transitions
    let transitions = vec![
        ("pending", "processing", "process"),
        ("processing", "shipped", "ship"),
        ("shipped", "delivered", "deliver"),
        ("pending", "cancelled", "cancel"),
        ("processing", "cancelled", "cancel"),
    ];
    
    for (from, to, trigger) in transitions {
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            correlation_id,
            causation_id: Some(event1.event_id),
            payload: EventPayload::Workflow(WorkflowPayload::TransitionAdded {
                workflow_id,
                from_state: from.to_string(),
                to_state: to.to_string(),
                trigger: trigger.to_string(),
            }),
        };
        
        let seq = event_store.publish_event(event, None).await?;
        println!("   ✓ Published TransitionAdded: {} -> {} (seq: {})", from, to, seq);
    }
    
    // 4. Fetch events and build projection
    println!("\n4. Fetching events and building projection...");
    let events = event_store.fetch_events(workflow_id).await?;
    println!("   ✓ Fetched {} events", events.len());
    
    // 5. Apply policies
    println!("\n5. Applying automated policies...");
    let mut policy_engine = PolicyEngine::new();
    let mut state_machine = GraphStateMachine::new();
    let mut ipld_chains = HashMap::new();
    
    for event in &events {
        let mut context = PolicyContext {
            state_machine: &mut state_machine,
            ipld_chains: &mut ipld_chains,
            metrics: Default::default(),
        };
        
        let actions = policy_engine.execute_policies(event, &mut context)?;
        println!("   ✓ Event {} triggered {} policy actions", 
                 event.event_id, actions.len());
    }
    
    let metrics = policy_engine.get_metrics();
    println!("\n   Policy Metrics:");
    println!("   - CIDs generated: {}", metrics.cids_generated);
    println!("   - Projections updated: {}", metrics.projections_updated);
    println!("   - Chains validated: {}", metrics.chains_validated);
    
    // 6. Create workflow instance
    println!("\n6. Creating workflow instance...");
    let instance_id = Uuid::new_v4();
    let instance_event = GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id: workflow_id,
        correlation_id: Uuid::new_v4(),
        causation_id: None,
        payload: EventPayload::Workflow(WorkflowPayload::InstanceCreated {
            workflow_id,
            instance_id,
            initial_state: "pending".to_string(),
        }),
    };
    
    let seq = event_store.publish_event(instance_event.clone(), None).await?;
    println!("   ✓ Created instance {} in 'pending' state (seq: {})", instance_id, seq);
    
    // 7. Subscribe to real-time events
    println!("\n7. Setting up real-time subscription...");
    let mut subscription = event_store.subscribe_to_aggregate(workflow_id).await?;
    
    // Trigger a state transition
    let transition_event = GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id: workflow_id,
        correlation_id: Uuid::new_v4(),
        causation_id: Some(instance_event.event_id),
        payload: EventPayload::Workflow(WorkflowPayload::StateTransitioned {
            instance_id,
            from_state: "pending".to_string(),
            to_state: "processing".to_string(),
        }),
    };
    
    event_store.publish_event(transition_event, None).await?;
    
    // Receive the event via subscription
    if let Some(event) = subscription.next().await? {
        println!("   ✓ Received real-time event: {:?}", event.payload);
    }
    
    // 8. Demonstrate event replay
    println!("\n8. Creating replay consumer...");
    let replay_consumer = event_store
        .create_replay_consumer("demo-replay", Some(1))
        .await?;
    
    let replayed_events = replay_consumer.fetch_batch(5).await?;
    println!("   ✓ Replayed {} events from sequence 1", replayed_events.len());
    
    // 9. Show IPLD chain
    println!("\n9. IPLD Chain Summary:");
    if let Some(chain) = ipld_chains.get(&workflow_id) {
        println!("   - Root CID: {}", chain.root_cid);
        println!("   - Chain length: {}", chain.chain.len());
        println!("   - Total size: {} bytes", chain.metadata.total_size);
        
        // Verify chain integrity
        match chain.verify_chain() {
            Ok(_) => println!("   ✓ Chain integrity verified"),
            Err(e) => println!("   ✗ Chain verification failed: {}", e),
        }
    }
    
    // 10. Demonstrate event correlation
    println!("\n10. Fetching correlated events...");
    let correlated = event_store.fetch_by_correlation(correlation_id).await?;
    println!("   ✓ Found {} events with correlation ID {}", 
             correlated.len(), correlation_id);
    
    println!("\n✨ Event-driven architecture demonstration complete!");
    println!("\nKey takeaways:");
    println!("- All state changes happen through events");
    println!("- Events are persisted in NATS JetStream");
    println!("- Projections are built by replaying events");
    println!("- Policies automate CID generation and validation");
    println!("- Real-time subscriptions enable reactive systems");
    
    Ok(())
}