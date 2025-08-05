//! Integration tests for NATS JetStream event store
//!
//! These tests require a running NATS server with JetStream enabled.
//! Run with: cargo test --features nats nats_integration -- --ignored

#![cfg(feature = "nats")]

use cim_graph::{
    events::{GraphEvent, EventPayload, GenericPayload},
    nats::{JetStreamEventStore, JetStreamConfig},
};
use uuid::Uuid;

#[tokio::test]
#[ignore = "Requires NATS server"]
async fn test_connect_to_jetstream() {
    let config = JetStreamConfig::default();
    let result = JetStreamEventStore::new(config).await;
    assert!(result.is_ok(), "Failed to connect to NATS: {:?}", result.err());
}

#[tokio::test]
#[ignore = "Requires NATS server"]
async fn test_publish_and_fetch_events() {
    let config = JetStreamConfig {
        stream_name: "CIM_TEST_STREAM".to_string(),
        ..Default::default()
    };
    
    let store = JetStreamEventStore::new(config).await.unwrap();
    let aggregate_id = Uuid::new_v4();
    
    // Create test events
    let events = vec![
        create_test_event(aggregate_id, "Event1"),
        create_test_event(aggregate_id, "Event2"),
        create_test_event(aggregate_id, "Event3"),
    ];
    
    // Publish events
    for event in &events {
        let seq = store.publish_event(event.clone(), None).await.unwrap();
        assert!(seq > 0);
    }
    
    // Small delay to ensure events are persisted
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Fetch events
    let fetched = store.fetch_events(aggregate_id).await.unwrap();
    assert!(!fetched.is_empty());
    
    // Verify we got all events
    let event_ids: Vec<_> = fetched.iter().map(|e| e.event_id).collect();
    for event in &events {
        assert!(event_ids.contains(&event.event_id));
    }
}

#[tokio::test]
#[ignore = "Requires NATS server"]
async fn test_event_subscription() {
    let config = JetStreamConfig {
        stream_name: "CIM_TEST_SUB_STREAM".to_string(),
        ..Default::default()
    };
    
    let store = JetStreamEventStore::new(config).await.unwrap();
    let aggregate_id = Uuid::new_v4();
    
    // Subscribe to events
    let mut subscription = store.subscribe_to_aggregate(aggregate_id).await.unwrap();
    
    // Publish event after subscription
    let event = create_test_event(aggregate_id, "SubscriptionTest");
    store.publish_event(event.clone(), None).await.unwrap();
    
    // Receive event
    let received = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        subscription.next()
    ).await;
    
    assert!(received.is_ok(), "Timeout waiting for event");
    let received_event = received.unwrap().unwrap().unwrap();
    assert_eq!(received_event.event_id, event.event_id);
}

#[tokio::test]
#[ignore = "Requires NATS server"]
async fn test_correlation_id_fetch() {
    let config = JetStreamConfig {
        stream_name: "CIM_TEST_CORR_STREAM".to_string(),
        ..Default::default()
    };
    
    let store = JetStreamEventStore::new(config).await.unwrap();
    let correlation_id = Uuid::new_v4();
    
    // Create events with same correlation ID but different aggregates
    let events = vec![
        create_correlated_event(Uuid::new_v4(), correlation_id, "Event1"),
        create_correlated_event(Uuid::new_v4(), correlation_id, "Event2"),
        create_correlated_event(Uuid::new_v4(), correlation_id, "Event3"),
    ];
    
    // Publish events
    for event in &events {
        store.publish_event(event.clone(), None).await.unwrap();
    }
    
    // Small delay
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Fetch by correlation ID
    let fetched = store.fetch_by_correlation(correlation_id).await.unwrap();
    assert_eq!(fetched.len(), events.len());
    
    // Verify all events have the same correlation ID
    for event in fetched {
        assert_eq!(event.correlation_id, correlation_id);
    }
}

#[tokio::test]
#[ignore = "Requires NATS server"]
async fn test_replay_consumer() {
    let config = JetStreamConfig {
        stream_name: "CIM_TEST_REPLAY_STREAM".to_string(),
        ..Default::default()
    };
    
    let store = JetStreamEventStore::new(config).await.unwrap();
    let aggregate_id = Uuid::new_v4();
    
    // Publish some events
    for i in 0..5 {
        let event = create_test_event(aggregate_id, &format!("Event{}", i));
        store.publish_event(event, None).await.unwrap();
    }
    
    // Small delay
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Create replay consumer
    let consumer_name = format!("test-replay-{}", Uuid::new_v4());
    let replay_consumer = store.create_replay_consumer(&consumer_name, None).await.unwrap();
    
    // Fetch batch
    let batch = replay_consumer.fetch_batch(10).await.unwrap();
    assert!(!batch.is_empty());
}

// Helper functions
fn create_test_event(aggregate_id: Uuid, data: &str) -> GraphEvent {
    GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        correlation_id: Uuid::new_v4(),
        causation_id: None,
        payload: EventPayload::Generic(GenericPayload {
            event_type: "TestEvent".to_string(),
            data: serde_json::json!({ "test": data }),
        }),
    }
}

fn create_correlated_event(aggregate_id: Uuid, correlation_id: Uuid, data: &str) -> GraphEvent {
    GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        correlation_id,
        causation_id: None,
        payload: EventPayload::Generic(GenericPayload {
            event_type: "CorrelatedEvent".to_string(),
            data: serde_json::json!({ "test": data }),
        }),
    }
}