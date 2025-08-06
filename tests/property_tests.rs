//! Property-based tests for event sourcing invariants

use cim_graph::core::{build_projection, PolicyEngine, PolicyContext, GraphStateMachine};
use cim_graph::events::{GraphEvent, EventPayload, IpldPayload};
use proptest::prelude::*;
use uuid::Uuid;
use std::collections::HashMap;

/// Generate a random event payload
fn arb_event_payload() -> impl Strategy<Value = EventPayload> {
    prop_oneof![
        // IPLD events
        (any::<String>(), any::<u64>()).prop_map(|(cid_suffix, size)| {
            EventPayload::Ipld(IpldPayload::CidAdded {
                cid: format!("Qm{}", cid_suffix.chars().take(10).collect::<String>()),
                codec: "dag-cbor".to_string(),
                size,
                data: serde_json::json!({ "test": true }),
            })
        }),
        // IPLD link events
        (any::<String>(), any::<String>(), any::<String>()).prop_map(|(cid1, cid2, link)| {
            EventPayload::Ipld(IpldPayload::CidLinkAdded {
                cid: format!("Qm{}", cid1.chars().take(10).collect::<String>()),
                link_name: link.chars().take(20).collect::<String>(),
                target_cid: format!("Qm{}", cid2.chars().take(10).collect::<String>()),
            })
        }),
    ]
}

/// Generate a random GraphEvent
fn arb_graph_event(aggregate_id: Uuid, correlation_id: Uuid) -> impl Strategy<Value = GraphEvent> {
    arb_event_payload().prop_map(move |payload| {
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id,
            causation_id: None,
            payload,
        }
    })
}

proptest! {
    /// Test: Projection version always equals number of events
    #[test]
    fn test_projection_version_invariant(
        num_events in 1usize..50
    ) {
        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let mut events: Vec<(GraphEvent, u64)> = Vec::new();
        
        for i in 0..num_events {
            let event = GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id,
                correlation_id,
                causation_id: if i > 0 { Some(events[i-1].0.event_id) } else { None },
                payload: EventPayload::Ipld(IpldPayload::CidAdded {
                    cid: format!("Qm{}", i),
                    codec: "dag-cbor".to_string(),
                    size: 100,
                    data: serde_json::json!({ "index": i }),
                }),
            };
            events.push((event, (i + 1) as u64));
        }
        
        let projection = build_projection(events.clone());
        prop_assert_eq!(projection.version, num_events as u64);
    }
    
    /// Test: Events are immutable - replaying same events produces same projection
    #[test]
    fn test_projection_determinism(
        events in prop::collection::vec(
            arb_graph_event(Uuid::new_v4(), Uuid::new_v4()),
            1..20
        )
    ) {
        let events_with_seq: Vec<_> = events.iter()
            .enumerate()
            .map(|(i, e)| (e.clone(), (i + 1) as u64))
            .collect();
        
        let projection1 = build_projection(events_with_seq.clone());
        let projection2 = build_projection(events_with_seq);
        
        // Projections should be identical
        prop_assert_eq!(projection1.version, projection2.version);
        prop_assert_eq!(projection1.components.len(), projection2.components.len());
        prop_assert_eq!(projection1.relationships.len(), projection2.relationships.len());
    }
    
    /// Test: Causation chains are valid (no forward references)
    #[test]
    fn test_causation_chain_validity(
        num_events in 2usize..30
    ) {
        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let mut events: Vec<(GraphEvent, u64)> = Vec::new();
        let mut event_ids = Vec::new();
        
        for i in 0..num_events {
            let causation_id = if i > 0 {
                // Can only reference previous events
                let ref_idx = i.saturating_sub(1);
                Some(event_ids[ref_idx])
            } else {
                None
            };
            
            let event = GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id,
                correlation_id,
                causation_id,
                payload: EventPayload::Ipld(IpldPayload::CidAdded {
                    cid: format!("Qm{}", i),
                    codec: "dag-cbor".to_string(),
                    size: 100,
                    data: serde_json::json!({ "index": i }),
                }),
            };
            
            event_ids.push(event.event_id);
            events.push((event, (i + 1) as u64));
        }
        
        // Verify all causation IDs reference earlier events
        for (i, (event, _)) in events.iter().enumerate() {
            if let Some(causation_id) = event.causation_id {
                let found = event_ids[0..i].iter().any(|id| *id == causation_id);
                prop_assert!(found, "Causation ID must reference an earlier event");
            }
        }
        
        // Should build without panic
        let projection = build_projection(events);
        prop_assert!(projection.version > 0);
    }
    
    /// Test: Adding components is idempotent for same CID
    #[test]
    fn test_component_idempotency(
        cid in "[A-Za-z0-9]{10}",
        data1 in prop::collection::hash_map(any::<String>(), any::<String>(), 0..5),
        data2 in prop::collection::hash_map(any::<String>(), any::<String>(), 0..5),
    ) {
        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let cid = format!("Qm{}", cid);
        
        // Add same CID twice with different data
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
                    data: serde_json::to_value(&data1).unwrap(),
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
                    data: serde_json::to_value(&data2).unwrap(),
                }),
            }, 2),
        ];
        
        let projection = build_projection(events);
        
        // Should have only one component (last write wins)
        prop_assert_eq!(projection.components.len(), 1);
        prop_assert!(projection.components.contains_key(&cid));
        
        // Should have the data from the second event
        let component = &projection.components[&cid];
        prop_assert_eq!(&component["size"], &serde_json::json!(200));
    }
    
    /// Test: Policy engine always produces valid actions
    #[test]
    fn test_policy_engine_invariants(
        events in prop::collection::vec(
            arb_graph_event(Uuid::new_v4(), Uuid::new_v4()),
            1..10
        )
    ) {
        let mut policy_engine = PolicyEngine::new();
        let mut state_machine = GraphStateMachine::new();
        let mut ipld_chains = HashMap::new();
        
        for event in &events {
            let mut context = PolicyContext {
                state_machine: &mut state_machine,
                ipld_chains: &mut ipld_chains,
                metrics: Default::default(),
            };
            
            let result = policy_engine.execute_policies(event, &mut context);
            
            // Policy engine should never fail
            prop_assert!(result.is_ok());
            
            let actions = result.unwrap();
            
            // Should produce at least some actions for IPLD events
            match &event.payload {
                EventPayload::Ipld(_) => prop_assert!(!actions.is_empty()),
                _ => {} // Other events might not trigger policies
            }
            
            // Metrics should be updated for IPLD events
            if matches!(&event.payload, EventPayload::Ipld(_)) {
                prop_assert!(context.metrics.cids_generated > 0);
            }
        }
    }
}

#[cfg(test)]
mod event_ordering_tests {
    use super::*;
    
    /// Test that events maintain ordering through serialization
    #[test]
    fn test_event_ordering_preserved() {
        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let mut original_events: Vec<GraphEvent> = Vec::new();
        
        for i in 0..10 {
            let event = GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id,
                correlation_id,
                causation_id: if i > 0 { Some(original_events[i-1].event_id) } else { None },
                payload: EventPayload::Ipld(IpldPayload::CidAdded {
                    cid: format!("Qm{}", i),
                    codec: "dag-cbor".to_string(),
                    size: 100,
                    data: serde_json::json!({ "order": i }),
                }),
            };
            original_events.push(event);
        }
        
        // Serialize and deserialize
        let json = serde_json::to_string(&original_events).unwrap();
        let restored_events: Vec<GraphEvent> = serde_json::from_str(&json).unwrap();
        
        // Verify order is preserved
        assert_eq!(original_events.len(), restored_events.len());
        for (i, (orig, restored)) in original_events.iter().zip(restored_events.iter()).enumerate() {
            assert_eq!(orig.event_id, restored.event_id);
            assert_eq!(orig.causation_id, restored.causation_id);
            
            // Verify the order field in data
            if let (EventPayload::Ipld(IpldPayload::CidAdded { data: orig_data, .. }),
                    EventPayload::Ipld(IpldPayload::CidAdded { data: restored_data, .. })) = 
                (&orig.payload, &restored.payload) {
                assert_eq!(orig_data["order"], restored_data["order"]);
                assert_eq!(orig_data["order"], i);
            }
        }
    }
}