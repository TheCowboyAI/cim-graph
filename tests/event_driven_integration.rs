//! Integration tests for event-driven architecture

use cim_graph::core::{build_projection, PolicyEngine, PolicyContext, GraphStateMachine};
use cim_graph::events::{GraphEvent, EventPayload, GenericPayload, WorkflowPayload, ContextPayload, IpldPayload};
use cim_graph::serde_support::EventJournal;
use std::collections::HashMap;
use uuid::Uuid;

#[test]
fn test_event_causation_chain() {
    let aggregate_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    let mut events = Vec::new();
    
    // First event has no causation
    let event1 = GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        correlation_id,
        causation_id: None,
        payload: EventPayload::Ipld(IpldPayload::CidAdded {
            cid: "QmNode1".to_string(),
            codec: "dag-cbor".to_string(),
            size: 100,
            data: serde_json::json!({ "label": "Node 1" }),
        }),
    };
    let event1_id = event1.event_id;
    events.push((event1, 1));
    
    // Second event caused by first
    let event2 = GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        correlation_id,
        causation_id: Some(event1_id),
        payload: EventPayload::Ipld(IpldPayload::CidAdded {
            cid: "QmNode2".to_string(),
            codec: "dag-cbor".to_string(),
            size: 100,
            data: serde_json::json!({ "label": "Node 2" }),
        }),
    };
    let event2_id = event2.event_id;
    events.push((event2, 2));
    
    // Third event caused by second
    let event3 = GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        correlation_id,
        causation_id: Some(event2_id),
        payload: EventPayload::Ipld(IpldPayload::CidLinkAdded {
            cid: "QmNode1".to_string(),
            link_name: "connects_to".to_string(),
            target_cid: "QmNode2".to_string(),
        }),
    };
    events.push((event3, 3));
    
    // Build projection and verify
    let projection = build_projection(events.clone());
    assert_eq!(projection.version, 3);
    assert_eq!(projection.components.len(), 2);
    assert_eq!(projection.relationships.len(), 1);
    
    // Verify causation chain
    assert_eq!(events[0].0.causation_id, None);
    assert_eq!(events[1].0.causation_id, Some(event1_id));
    assert_eq!(events[2].0.causation_id, Some(event2_id));
}

#[test]
fn test_mixed_event_types() {
    let aggregate_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    let mut events = Vec::new();
    
    // Workflow event
    events.push((GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        correlation_id,
        causation_id: None,
        payload: EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
            workflow_id: Uuid::new_v4(),
            name: "Test Workflow".to_string(),
            version: "1.0.0".to_string(),
        }),
    }, 1));
    
    // Context event
    events.push((GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        correlation_id,
        causation_id: Some(events[0].0.event_id),
        payload: EventPayload::Context(ContextPayload::BoundedContextCreated {
            context_id: "test_context".to_string(),
            name: "Test Context".to_string(),
            description: "A test bounded context".to_string(),
        }),
    }, 2));
    
    // Generic event
    events.push((GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        correlation_id,
        causation_id: Some(events[1].0.event_id),
        payload: EventPayload::Generic(GenericPayload {
            event_type: "CustomEvent".to_string(),
            data: serde_json::json!({ "custom": "data" }),
        }),
    }, 3));
    
    let projection = build_projection(events);
    assert_eq!(projection.version, 3);
}

#[test]
fn test_event_persistence_and_replay() {
    let aggregate_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    let mut events = Vec::new();
    
    // Create some events
    let mut event_ids = Vec::new();
    for i in 0..5 {
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id,
            causation_id: if i > 0 { Some(event_ids[i-1]) } else { None },
            payload: EventPayload::Ipld(IpldPayload::CidAdded {
                cid: format!("QmNode{}", i),
                codec: "dag-cbor".to_string(),
                size: 100,
                data: serde_json::json!({ "index": i }),
            }),
        };
        event_ids.push(event.event_id);
        events.push(event);
    }
    
    // Create journal and test serialization
    let journal = EventJournal::new(events.clone());
    let json = serde_json::to_string(&journal).unwrap();
    let loaded_journal: EventJournal = serde_json::from_str(&json).unwrap();
    
    assert_eq!(loaded_journal.events.len(), 5);
    assert_eq!(loaded_journal.metadata.version, "1.0.0");
    
    // Build projection from loaded events
    let projection = build_projection(
        loaded_journal.events.into_iter()
            .enumerate()
            .map(|(i, e)| (e, i as u64 + 1))
            .collect()
    );
    
    assert_eq!(projection.version, 5);
    assert_eq!(projection.components.len(), 5);
}

#[test]
fn test_policy_engine_integration() {
    let aggregate_id = Uuid::new_v4();
    let event = GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        correlation_id: Uuid::new_v4(),
        causation_id: None,
        payload: EventPayload::Generic(GenericPayload {
            event_type: "TestEvent".to_string(),
            data: serde_json::json!({ "test": true }),
        }),
    };
    
    let mut policy_engine = PolicyEngine::new();
    let mut state_machine = GraphStateMachine::new();
    let mut ipld_chains = HashMap::new();
    
    let mut context = PolicyContext {
        state_machine: &mut state_machine,
        ipld_chains: &mut ipld_chains,
        metrics: Default::default(),
    };
    
    // Execute policies
    let actions = policy_engine.execute_policies(&event, &mut context).unwrap();
    
    // Should have at least CID generation and projection update actions
    assert!(actions.len() >= 2);
    assert!(context.metrics.cids_generated > 0);
}

#[test]
fn test_concurrent_event_correlation() {
    // Simulate multiple correlated event streams
    let aggregate_id = Uuid::new_v4();
    let mut all_events = Vec::new();
    
    // Stream 1: User actions
    let user_correlation = Uuid::new_v4();
    for i in 0..3 {
        all_events.push((GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id: user_correlation,
            causation_id: None,
            payload: EventPayload::Generic(GenericPayload {
                event_type: "UserAction".to_string(),
                data: serde_json::json!({ "action": format!("action_{}", i) }),
            }),
        }, (i * 2 + 1) as u64)); // Odd sequence numbers
    }
    
    // Stream 2: System reactions
    let system_correlation = Uuid::new_v4();
    for i in 0..3 {
        all_events.push((GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id: system_correlation,
            causation_id: None,
            payload: EventPayload::Generic(GenericPayload {
                event_type: "SystemReaction".to_string(),
                data: serde_json::json!({ "reaction": format!("reaction_{}", i) }),
            }),
        }, (i * 2 + 2) as u64)); // Even sequence numbers
    }
    
    // Sort by sequence number
    all_events.sort_by_key(|(_, seq)| *seq);
    
    let projection = build_projection(all_events.clone());
    assert_eq!(projection.version, 6);
    
    // Verify both correlation IDs are present
    let correlations: std::collections::HashSet<_> = all_events.iter()
        .map(|(e, _)| e.correlation_id)
        .collect();
    assert_eq!(correlations.len(), 2);
}