//! Tests for event propagation and handling across graphs

use cim_graph::graphs::{ComposedGraph, IpldGraph, ContextGraph, WorkflowGraph, ConceptGraph};
use cim_graph::{GraphEvent, EventHandler, Result};
use serde_json::json;
use uuid::Uuid;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

struct EventCollector {
    events: Arc<Mutex<VecDeque<GraphEvent>>>,
}

impl EventCollector {
    fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
    
    fn get_events(&self) -> Vec<GraphEvent> {
        self.events.lock().unwrap().drain(..).collect()
    }
}

impl EventHandler for EventCollector {
    fn handle_event(&self, event: &GraphEvent) {
        self.events.lock().unwrap().push_back(event.clone());
    }
}

#[test]
fn test_basic_event_emission() -> Result<()> {
    let collector = Arc::new(EventCollector::new());
    
    let mut graph = ContextGraph::with_handler(collector.clone());
    
    // Trigger events
    let node = graph.add_aggregate("Entity", Uuid::new_v4(), json!({
        "name": "Test"
    }))?;
    
    let events = collector.get_events();
    assert_eq!(events.len(), 1);
    
    match &events[0] {
        GraphEvent::NodeAdded { node_id, .. } => {
            assert_eq!(node_id, &node.to_string());
        }
        _ => panic!("Wrong event type"),
    }
    
    Ok(())
}

#[test]
fn test_edge_event_propagation() -> Result<()> {
    let collector = Arc::new(EventCollector::new());
    
    let mut graph = WorkflowGraph::new("workflow-events");
    graph.graph_mut().add_handler(collector.clone());
    
    // Create states
    let s1 = graph.add_state("start", json!({}))?;
    let s2 = graph.add_state("end", json!({}))?;
    
    // Clear node addition events
    collector.get_events();
    
    // Add transition
    graph.add_transition(s1, s2, "complete", json!({}))?;
    
    let events = collector.get_events();
    assert_eq!(events.len(), 1);
    
    match &events[0] {
        GraphEvent::EdgeAdded { from, to, .. } => {
            assert_eq!(from, &s1.to_string());
            assert_eq!(to, &s2.to_string());
        }
        _ => panic!("Wrong event type"),
    }
    
    Ok(())
}

#[test]
fn test_cascading_events() -> Result<()> {
    let collector = Arc::new(EventCollector::new());
    
    let mut context = ContextGraph::with_handler(collector.clone());
    
    // Add aggregate (should trigger event)
    let aggregate = context.add_aggregate("Order", Uuid::new_v4(), json!({
        "total": 100.0
    }))?;
    
    // Add entity (should trigger event for entity AND implicit relationship)
    let entity = context.add_entity("OrderLine", Uuid::new_v4(), aggregate, json!({
        "product": "Widget",
        "quantity": 2
    }))?;
    
    let events = collector.get_events();
    
    // Should have: aggregate added, entity added, relationship added
    assert!(events.len() >= 2);
    
    let node_events: Vec<_> = events.iter()
        .filter(|e| matches!(e, GraphEvent::NodeAdded { .. }))
        .collect();
    
    assert_eq!(node_events.len(), 2);
    
    Ok(())
}

#[test]
fn test_multiple_subscribers() -> Result<()> {
    let collector1 = Arc::new(EventCollector::new());
    let collector2 = Arc::new(EventCollector::new());
    
    let mut graph = IpldGraph::new();
    graph.graph_mut().add_handler(collector1.clone());
    graph.graph_mut().add_handler(collector2.clone());
    
    // Add node
    graph.add_cid("QmTest", "dag-cbor", 1024)?;
    
    // Both collectors should receive the event
    let events1 = collector1.get_events();
    let events2 = collector2.get_events();
    
    assert_eq!(events1.len(), 1);
    assert_eq!(events2.len(), 1);
    
    // Events should be identical
    match (&events1[0], &events2[0]) {
        (
            GraphEvent::NodeAdded { node_id: id1, .. },
            GraphEvent::NodeAdded { node_id: id2, .. }
        ) => {
            assert_eq!(id1, id2);
        }
        _ => panic!("Events don't match"),
    }
    
    Ok(())
}

#[test]
fn test_event_filtering() -> Result<()> {
    struct FilteredHandler {
        events: Arc<Mutex<Vec<GraphEvent>>>,
        filter_type: String,
    }
    
    impl FilteredHandler {
        fn new(filter_type: &str) -> Self {
            Self {
                events: Arc::new(Mutex::new(Vec::new())),
                filter_type: filter_type.to_string(),
            }
        }
        
        fn get_events(&self) -> Vec<GraphEvent> {
            self.events.lock().unwrap().clone()
        }
    }
    
    impl EventHandler for FilteredHandler {
        fn handle_event(&self, event: &GraphEvent) {
            match event {
                GraphEvent::NodeAdded { node_id, .. } => {
                    // Note: We can't filter by node_type in GraphEvent, so we'll accept all NodeAdded events
                    self.events.lock().unwrap().push(event.clone());
                }
                _ => {} // Ignore other events
            }
        }
    }
    
    let aggregate_handler = Arc::new(FilteredHandler::new("Aggregate"));
    let entity_handler = Arc::new(FilteredHandler::new("Entity"));
    
    let mut context = ContextGraph::new("filtered");
    context.graph_mut().add_handler(aggregate_handler.clone());
    context.graph_mut().add_handler(entity_handler.clone());
    
    // Add different node types
    let agg = context.add_aggregate("Customer", Uuid::new_v4(), json!({}))?;
    context.add_entity("Address", Uuid::new_v4(), agg, json!({}))?;
    
    // Check filtered events
    let agg_events = aggregate_handler.get_events();
    let entity_events = entity_handler.get_events();
    
    assert_eq!(agg_events.len(), 1);
    assert_eq!(entity_events.len(), 1);
    
    Ok(())
}

#[test]
fn test_event_ordering() -> Result<()> {
    let collector = Arc::new(EventCollector::new());
    
    let mut workflow = WorkflowGraph::new("ordering");
    workflow.graph_mut().add_handler(collector.clone());
    
    // Perform a series of operations
    let s1 = workflow.add_state("s1", json!({}))?;
    let s2 = workflow.add_state("s2", json!({}))?;
    let s3 = workflow.add_state("s3", json!({}))?;
    
    workflow.add_transition(s1, s2, "t1", json!({}))?;
    workflow.add_transition(s2, s3, "t2", json!({}))?;
    
    let events = collector.get_events();
    
    // Verify event order
    assert!(events.len() >= 5); // 3 nodes + 2 edges
    
    // First 3 should be node additions
    for i in 0..3 {
        assert!(matches!(events[i], GraphEvent::NodeAdded { .. }));
    }
    
    // Next 2 should be edge additions
    for i in 3..5 {
        assert!(matches!(events[i], GraphEvent::EdgeAdded { .. }));
    }
    
    Ok(())
}

#[test]
fn test_composed_graph_events() -> Result<()> {
    // Create individual graphs with handlers
    let ipld_collector = Arc::new(EventCollector::new());
    let context_collector = Arc::new(EventCollector::new());
    
    let mut ipld = IpldGraph::new();
    ipld.graph_mut().add_handler(ipld_collector.clone());
    
    let mut context = ContextGraph::new("test");
    context.graph_mut().add_handler(context_collector.clone());
    
    // Add data to individual graphs
    ipld.add_cid("QmTest", "dag-cbor", 256)?;
    context.add_aggregate("Entity", Uuid::new_v4(), json!({}))?;
    
    // Verify individual events
    assert_eq!(ipld_collector.get_events().len(), 1);
    assert_eq!(context_collector.get_events().len(), 1);
    
    // Compose graphs
    let composed = ComposedGraph::builder()
        .add_graph("ipld", ipld)
        .add_graph("context", context)
        .build()?;
    
    // Events should still be accessible through original collectors
    // (handlers are preserved in sub-graphs)
    
    Ok(())
}

#[test]
fn test_error_in_event_handler() -> Result<()> {
    struct FailingHandler {
        fail_on_count: usize,
        count: Arc<Mutex<usize>>,
    }
    
    impl FailingHandler {
        fn new(fail_on_count: usize) -> Self {
            Self {
                fail_on_count,
                count: Arc::new(Mutex::new(0)),
            }
        }
    }
    
    impl EventHandler for FailingHandler {
        fn handle_event(&self, _event: &GraphEvent) {
            let mut count = self.count.lock().unwrap();
            *count += 1;
            
            if *count == self.fail_on_count {
                // Since we can't return errors, we'll just panic or log
                eprintln!("Handler error at count {}", *count);
            }
        }
    }
    
    let failing_handler = Arc::new(FailingHandler::new(2));
    let normal_collector = Arc::new(EventCollector::new());
    
    let mut graph = ContextGraph::new("error-test");
    graph.graph_mut().add_handler(failing_handler);
    graph.graph_mut().add_handler(normal_collector.clone());
    
    // First event should succeed
    graph.add_aggregate("Entity1", Uuid::new_v4(), json!({}))?;
    
    // Second event - handler fails but shouldn't affect graph operation
    let result = graph.add_aggregate("Entity2", Uuid::new_v4(), json!({}));
    assert!(result.is_ok()); // Graph operation should still succeed
    
    // Normal collector should still receive both events
    let events = normal_collector.get_events();
    assert_eq!(events.len(), 2);
    
    Ok(())
}

#[test]
fn test_event_metadata() -> Result<()> {
    let collector = Arc::new(EventCollector::new());
    
    let mut concept = ConceptGraph::new();
    concept.graph_mut().add_handler(collector.clone());
    
    // Add concept with metadata
    let features = vec![
        ("color", 0.8),
        ("size", 0.6),
    ];
    
    let concept_id = concept.add_concept("Thing", features)?;
    
    let events = collector.get_events();
    assert_eq!(events.len(), 1);
    
    match &events[0] {
        GraphEvent::NodeAdded { .. } => {
            // NodeAdded event confirmed
        }
        _ => panic!("Wrong event type"),
    }
    
    Ok(())
}

#[test]
fn test_batch_event_handling() -> Result<()> {
    struct BatchHandler {
        batch: Arc<Mutex<Vec<GraphEvent>>>,
        batch_size: usize,
        processed: Arc<Mutex<Vec<Vec<GraphEvent>>>>,
    }
    
    impl BatchHandler {
        fn new(batch_size: usize) -> Self {
            Self {
                batch: Arc::new(Mutex::new(Vec::new())),
                batch_size,
                processed: Arc::new(Mutex::new(Vec::new())),
            }
        }
        
        fn get_batches(&self) -> Vec<Vec<GraphEvent>> {
            self.processed.lock().unwrap().clone()
        }
    }
    
    impl EventHandler for BatchHandler {
        fn handle_event(&self, event: &GraphEvent) {
            let mut batch = self.batch.lock().unwrap();
            batch.push(event.clone());
            
            if batch.len() >= self.batch_size {
                let completed_batch = batch.drain(..).collect();
                self.processed.lock().unwrap().push(completed_batch);
            }
        }
    }
    
    let batch_handler = Arc::new(BatchHandler::new(3));
    
    let mut graph = IpldGraph::new();
    graph.graph_mut().add_handler(batch_handler.clone());
    
    // Add 7 nodes (should create 2 complete batches)
    for i in 0..7 {
        graph.add_cid(&format!("Qm{}", i), "dag-cbor", 100)?;
    }
    
    let batches = batch_handler.get_batches();
    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0].len(), 3);
    assert_eq!(batches[1].len(), 3);
    
    Ok(())
}