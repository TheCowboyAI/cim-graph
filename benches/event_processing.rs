//! Performance benchmarks for event processing

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use cim_graph::core::{build_projection, PolicyEngine, PolicyContext, GraphStateMachine};
use cim_graph::events::{GraphEvent, EventPayload, IpldPayload};
use cim_graph::serde_support::EventJournal;
use uuid::Uuid;
use std::collections::HashMap;

/// Generate test events
fn generate_events(count: usize) -> Vec<GraphEvent> {
    let aggregate_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    let mut events: Vec<GraphEvent> = Vec::with_capacity(count);
    
    for i in 0..count {
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id,
            causation_id: if i > 0 { Some(events[i-1].event_id) } else { None },
            payload: EventPayload::Ipld(IpldPayload::CidAdded {
                cid: format!("Qm{:032x}", i),
                codec: "dag-cbor".to_string(),
                size: 1024,
                data: serde_json::json!({
                    "index": i,
                    "timestamp": i * 1000,
                    "data": format!("Event data {}", i),
                }),
            }),
        };
        events.push(event);
    }
    
    events
}

/// Benchmark projection building
fn bench_projection_building(c: &mut Criterion) {
    let mut group = c.benchmark_group("projection_building");
    
    for count in [10, 100, 1000, 5000].iter() {
        let events = generate_events(*count);
        let events_with_seq: Vec<_> = events.into_iter()
            .enumerate()
            .map(|(i, e)| (e, (i + 1) as u64))
            .collect();
        
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            &events_with_seq,
            |b, events| {
                b.iter(|| {
                    let projection = build_projection(black_box(events.clone()));
                    black_box(projection);
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark event serialization
fn bench_event_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_serialization");
    
    for count in [10, 100, 1000].iter() {
        let events = generate_events(*count);
        
        group.bench_with_input(
            BenchmarkId::new("serialize", count),
            &events,
            |b, events| {
                b.iter(|| {
                    let journal = EventJournal::new(black_box(events.clone()));
                    let json = serde_json::to_string(&journal).unwrap();
                    black_box(json);
                });
            },
        );
        
        let journal = EventJournal::new(events);
        let json = serde_json::to_string(&journal).unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("deserialize", count),
            &json,
            |b, json| {
                b.iter(|| {
                    let journal: EventJournal = serde_json::from_str(black_box(json)).unwrap();
                    black_box(journal);
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark policy execution
fn bench_policy_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("policy_execution");
    
    for count in [1, 10, 100].iter() {
        let events = generate_events(*count);
        
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            &events,
            |b, events| {
                b.iter(|| {
                    let mut policy_engine = PolicyEngine::new();
                    let mut state_machine = GraphStateMachine::new();
                    let mut ipld_chains = HashMap::new();
                    
                    for event in events {
                        let mut context = PolicyContext {
                            state_machine: &mut state_machine,
                            ipld_chains: &mut ipld_chains,
                            metrics: Default::default(),
                        };
                        
                        let actions = policy_engine.execute_policies(event, &mut context).unwrap();
                        black_box(actions);
                    }
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark causation chain traversal
fn bench_causation_chain(c: &mut Criterion) {
    let mut group = c.benchmark_group("causation_chain");
    
    for depth in [10, 50, 100, 500].iter() {
        let events = generate_events(*depth);
        
        group.bench_with_input(
            BenchmarkId::new("traverse", depth),
            &events,
            |b, events| {
                b.iter(|| {
                    // Traverse causation chain from last to first
                    let mut count = 0;
                    let mut current = events.last();
                    
                    while let Some(event) = current {
                        count += 1;
                        current = event.causation_id
                            .and_then(|id| events.iter().find(|e| e.event_id == id));
                    }
                    
                    black_box(count);
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark concurrent event processing
fn bench_concurrent_processing(c: &mut Criterion) {
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    let mut group = c.benchmark_group("concurrent_processing");
    
    for thread_count in [1, 2, 4, 8].iter() {
        let events_per_thread = 100;
        let aggregate_id = Uuid::new_v4();
        
        group.bench_with_input(
            BenchmarkId::new("threads", thread_count),
            thread_count,
            |b, &thread_count| {
                b.iter(|| {
                    let events = Arc::new(Mutex::new(Vec::new()));
                    let mut handles = vec![];
                    
                    for t in 0..thread_count {
                        let events_clone = Arc::clone(&events);
                        let handle = thread::spawn(move || {
                            for i in 0..events_per_thread {
                                let event = GraphEvent {
                                    event_id: Uuid::new_v4(),
                                    aggregate_id,
                                    correlation_id: Uuid::new_v4(),
                                    causation_id: None,
                                    payload: EventPayload::Ipld(IpldPayload::CidAdded {
                                        cid: format!("Qm_t{}_i{}", t, i),
                                        codec: "dag-cbor".to_string(),
                                        size: 256,
                                        data: serde_json::json!({ "thread": t, "index": i }),
                                    }),
                                };
                                
                                events_clone.lock().unwrap().push(event);
                            }
                        });
                        handles.push(handle);
                    }
                    
                    for handle in handles {
                        handle.join().unwrap();
                    }
                    
                    let final_events = events.lock().unwrap();
                    black_box(final_events.len());
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_projection_building,
    bench_event_serialization,
    bench_policy_execution,
    bench_causation_chain,
    bench_concurrent_processing
);
criterion_main!(benches);