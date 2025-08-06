//! Comprehensive error handling tests

use cim_graph::core::{build_projection, GraphStateMachine};
use cim_graph::events::{GraphEvent, EventPayload, IpldPayload, WorkflowPayload};
use uuid::Uuid;

#[test]
fn test_empty_event_stream_panic() {
    // build_projection should panic on empty event stream
    let result = std::panic::catch_unwind(|| {
        let events: Vec<(GraphEvent, u64)> = Vec::new();
        build_projection(events);
    });
    
    assert!(result.is_err(), "build_projection should panic on empty event stream");
}

#[test]
fn test_duplicate_cid_behavior() {
    let aggregate_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    let cid = "QmTest123".to_string();
    
    // Create events with duplicate CID
    let events = vec![
        (GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id,
            causation_id: None,
            payload: EventPayload::Ipld(IpldPayload::CidAdded {
                cid: cid.clone(),
                codec: "dag-cbor".to_string(),
                size: 100,
                data: serde_json::json!({ "first": true }),
            }),
        }, 1),
        (GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id,
            causation_id: None,
            payload: EventPayload::Ipld(IpldPayload::CidAdded {
                cid: cid.clone(),
                codec: "dag-cbor".to_string(),
                size: 200,
                data: serde_json::json!({ "second": true }),
            }),
        }, 2),
    ];
    
    let projection = build_projection(events);
    
    // Last write wins - should have second event's data
    assert_eq!(projection.components.len(), 1);
    assert!(projection.components.contains_key(&cid));
    assert_eq!(projection.components[&cid]["size"], serde_json::json!(200));
}

#[test]
fn test_state_machine_tracks_state() {
    let aggregate_id = Uuid::new_v4();
    let mut state_machine = GraphStateMachine::new();
    
    // Initially should be uninitialized
    let initial_state = state_machine.get_state(&aggregate_id);
    assert!(matches!(initial_state, cim_graph::core::state_machine::GraphState::Uninitialized));
    
    // Apply initialization event
    let init_event = GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        correlation_id: Uuid::new_v4(),
        causation_id: None,
        payload: EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
            workflow_id: aggregate_id,
            name: "Test".to_string(),
            version: "1.0".to_string(),
        }),
    };
    
    state_machine.apply_event(&init_event);
    
    // Should now be initialized
    let new_state = state_machine.get_state(&aggregate_id);
    assert!(matches!(new_state, cim_graph::core::state_machine::GraphState::Initialized { .. }));
}

#[test]
fn test_event_deserialization_errors() {
    // Test malformed event JSON
    let bad_json = r#"{
        "event_id": "not-a-uuid",
        "aggregate_id": "also-not-a-uuid",
        "payload": {
            "missing": "required fields"
        }
    }"#;
    
    let result: Result<GraphEvent, _> = serde_json::from_str(bad_json);
    assert!(result.is_err(), "Should fail to deserialize malformed event");
}

#[test]
fn test_causation_chain_validation() {
    let aggregate_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    let future_event_id = Uuid::new_v4();
    
    // Create event with causation ID that doesn't exist yet
    let event = GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        correlation_id,
        causation_id: Some(future_event_id), // References non-existent event
        payload: EventPayload::Ipld(IpldPayload::CidAdded {
            cid: "QmTest".to_string(),
            codec: "dag-cbor".to_string(),
            size: 100,
            data: serde_json::json!({}),
        }),
    };
    
    // This should still build (we don't validate causation in projection)
    let projection = build_projection(vec![(event, 1)]);
    assert_eq!(projection.version, 1);
}

#[test] 
fn test_concurrent_modification_handling() {
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    let aggregate_id = Uuid::new_v4();
    let events = Arc::new(Mutex::new(Vec::new()));
    
    // Simulate concurrent event additions
    let mut handles = vec![];
    
    for i in 0..5 {
        let events_clone = Arc::clone(&events);
        let handle = thread::spawn(move || {
            let event = GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id,
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Ipld(IpldPayload::CidAdded {
                    cid: format!("Qm{}", i),
                    codec: "dag-cbor".to_string(),
                    size: 100,
                    data: serde_json::json!({ "thread": i }),
                }),
            };
            
            events_clone.lock().unwrap().push((event, (i + 1) as u64));
        });
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Events might be out of order, sort by sequence
    let mut final_events = events.lock().unwrap().clone();
    final_events.sort_by_key(|(_, seq)| *seq);
    
    // Should still build valid projection
    let projection = build_projection(final_events);
    assert_eq!(projection.version, 5);
    assert_eq!(projection.components.len(), 5);
}

#[cfg(test)]
mod policy_error_tests {
    use super::*;
    use cim_graph::core::{PolicyEngine, PolicyContext};
    use std::collections::HashMap;
    
    #[test]
    fn test_policy_execution_continues_on_error() {
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Ipld(IpldPayload::CidAdded {
                cid: "QmTest".to_string(),
                codec: "dag-cbor".to_string(),
                size: 100,
                data: serde_json::json!({}),
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
        
        // Execute policies - should not panic even if individual policies fail
        let result = policy_engine.execute_policies(&event, &mut context);
        assert!(result.is_ok());
        
        // Should have executed some policies successfully
        let actions = result.unwrap();
        assert!(!actions.is_empty());
    }
}

#[test]
fn test_invalid_event_sequence() {
    let aggregate_id = Uuid::new_v4();
    
    // Create events with non-sequential sequence numbers
    let events = vec![
        (GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Ipld(IpldPayload::CidAdded {
                cid: "Qm1".to_string(),
                codec: "dag-cbor".to_string(),
                size: 100,
                data: serde_json::json!({}),
            }),
        }, 1),
        (GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Ipld(IpldPayload::CidAdded {
                cid: "Qm3".to_string(),
                codec: "dag-cbor".to_string(),
                size: 100,
                data: serde_json::json!({}),
            }),
        }, 3), // Gap in sequence
    ];
    
    // Should still build - projection uses provided sequence
    let projection = build_projection(events);
    assert_eq!(projection.version, 3); // Uses highest sequence
    assert_eq!(projection.components.len(), 2);
}

#[test]
fn test_mixed_aggregate_ids() {
    let aggregate_id1 = Uuid::new_v4();
    let aggregate_id2 = Uuid::new_v4();
    
    // Create events with different aggregate IDs
    let events = vec![
        (GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: aggregate_id1,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Ipld(IpldPayload::CidAdded {
                cid: "Qm1".to_string(),
                codec: "dag-cbor".to_string(),
                size: 100,
                data: serde_json::json!({}),
            }),
        }, 1),
        (GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: aggregate_id2, // Different aggregate
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Ipld(IpldPayload::CidAdded {
                cid: "Qm2".to_string(),
                codec: "dag-cbor".to_string(),
                size: 100,
                data: serde_json::json!({}),
            }),
        }, 2),
    ];
    
    let projection = build_projection(events);
    
    // Projection uses first event's aggregate ID
    assert_eq!(projection.aggregate_id, aggregate_id1);
    // But only includes events for that aggregate
    assert_eq!(projection.components.len(), 1);
    assert!(projection.components.contains_key("Qm1"));
    assert!(!projection.components.contains_key("Qm2"));
}