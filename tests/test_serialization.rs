//! Event serialization tests
//!
//! Tests serialization of events and commands.
//! Projections are NOT serialized - they are ephemeral and rebuilt from events.

use cim_graph::{
    events::{GraphEvent, EventPayload, WorkflowPayload, IpldPayload},
    serde_support::{serialize_events, deserialize_events, EventJournal},
};
use uuid::Uuid;

#[test]
fn test_workflow_event_serialization() {
    let workflow_id = Uuid::new_v4();
    let events = vec![
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
                workflow_id,
                name: "Test Workflow".to_string(),
                version: "1.0.0".to_string(),
            }),
        },
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Workflow(WorkflowPayload::StateAdded {
                workflow_id,
                state_id: "start".to_string(),
                state_type: "initial".to_string(),
            }),
        },
    ];
    
    // Serialize events
    let json = serialize_events(&events).unwrap();
    assert!(json.contains("WorkflowDefined"));
    assert!(json.contains("StateAdded"));
    
    // Deserialize events
    let restored = deserialize_events(&json).unwrap();
    assert_eq!(restored.len(), 2);
    assert_eq!(restored[0].aggregate_id, workflow_id);
}

#[test]
fn test_ipld_event_serialization() {
    let aggregate_id = Uuid::new_v4();
    let event = GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        correlation_id: Uuid::new_v4(),
        causation_id: None,
        payload: EventPayload::Ipld(IpldPayload::CidAdded {
            cid: "Qm123".to_string(),
            codec: "dag-cbor".to_string(),
            size: 256,
            data: serde_json::json!({"test": "data"}),
        }),
    };
    
    // Serialize single event
    let json = serialize_events(&[event.clone()]).unwrap();
    assert!(json.contains("CidAdded"));
    assert!(json.contains("Qm123"));
    
    // Deserialize
    let restored = deserialize_events(&json).unwrap();
    assert_eq!(restored.len(), 1);
    
    match &restored[0].payload {
        EventPayload::Ipld(IpldPayload::CidAdded { cid, .. }) => {
            assert_eq!(cid, "Qm123");
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_event_journal() {
    let aggregate_id = Uuid::new_v4();
    let mut events: Vec<GraphEvent> = Vec::new();
    
    // Create a chain of events
    for i in 0..5 {
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id: Uuid::new_v4(),
            causation_id: if i > 0 { Some(events[i-1].event_id) } else { None },
            payload: EventPayload::Generic(cim_graph::events::GenericPayload {
                event_type: format!("Event{}", i),
                data: serde_json::json!({"index": i}),
            }),
        };
        events.push(event);
    }
    
    // Create journal
    let journal = EventJournal::new(events.clone());
    assert_eq!(journal.metadata.event_count, 5);
    assert_eq!(journal.metadata.aggregate_ids.len(), 1);
    assert_eq!(journal.metadata.aggregate_ids[0], aggregate_id);
    
    // Save and load from temp file
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    journal.save_to_file(temp_file.path()).unwrap();
    
    let loaded = EventJournal::load_from_file(temp_file.path()).unwrap();
    assert_eq!(loaded.metadata.event_count, 5);
    assert_eq!(loaded.events.len(), 5);
    
    // Verify causation chain
    for i in 1..5 {
        assert_eq!(loaded.events[i].causation_id, Some(loaded.events[i-1].event_id));
    }
}

#[test]
fn test_event_correlation() {
    let aggregate_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    
    let events = vec![
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id,
            causation_id: None,
            payload: EventPayload::Generic(cim_graph::events::GenericPayload {
                event_type: "Start".to_string(),
                data: serde_json::json!({}),
            }),
        },
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id, // Same correlation
            causation_id: None,
            payload: EventPayload::Generic(cim_graph::events::GenericPayload {
                event_type: "Continue".to_string(),
                data: serde_json::json!({}),
            }),
        },
    ];
    
    let json = serialize_events(&events).unwrap();
    let restored = deserialize_events(&json).unwrap();
    
    // Verify correlation is preserved
    assert_eq!(restored[0].correlation_id, restored[1].correlation_id);
}