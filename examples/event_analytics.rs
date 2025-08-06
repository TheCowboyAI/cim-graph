//! Example demonstrating event analytics and metrics dashboard

use cim_graph::{
    analytics::{MetricsDashboard, DashboardConfig},
    events::{GraphEvent, EventPayload, WorkflowPayload, IpldPayload},
};
use std::time::{Duration, Instant};
use uuid::Uuid;

fn main() {
    println!("=== Event Analytics Example ===\n");
    
    // Configure dashboard with short windows for demo
    let config = DashboardConfig {
        window_duration: Duration::from_secs(5), // 5 second windows
        window_count: 6, // Keep 6 windows (30 seconds of history)
        max_history: 100,
        track_aggregates: true,
    };
    
    let dashboard = MetricsDashboard::new(config);
    
    // Simulate different event patterns
    println!("Simulating event processing...\n");
    
    // Phase 1: Steady workflow events
    println!("Phase 1: Workflow events (10 events)");
    let workflow_aggregate = Uuid::new_v4();
    for i in 0..10 {
        let event = create_workflow_event(workflow_aggregate, i);
        let processing_time = Duration::from_millis(5 + (i % 3) * 2);
        dashboard.record_event(&event, processing_time);
        std::thread::sleep(Duration::from_millis(100));
    }
    
    // Phase 2: Burst of IPLD events
    println!("Phase 2: IPLD event burst (20 events)");
    let ipld_aggregate = Uuid::new_v4();
    let mut parent_id = None;
    for i in 0..20 {
        let event = create_ipld_event(ipld_aggregate, parent_id);
        parent_id = Some(event.event_id);
        let processing_time = Duration::from_millis(2 + (i % 5));
        dashboard.record_event(&event, processing_time);
        std::thread::sleep(Duration::from_millis(50));
    }
    
    // Phase 3: Mixed events with some errors
    println!("Phase 3: Mixed events with errors (15 events)\n");
    for i in 0..15 {
        let event = if i % 3 == 0 {
            create_workflow_event(workflow_aggregate, i + 10)
        } else {
            create_ipld_event(ipld_aggregate, None)
        };
        
        let processing_time = Duration::from_millis(10 + (i % 10) * 3);
        dashboard.record_event(&event, processing_time);
        
        // Simulate some errors
        if i % 7 == 0 {
            dashboard.record_error();
        }
        
        std::thread::sleep(Duration::from_millis(200));
    }
    
    // Display dashboard summary
    println!("{}", dashboard.summary());
    
    // Show throughput history
    println!("\n=== Throughput History ===");
    let throughput = dashboard.throughput_history();
    let start = throughput.first().map(|(t, _)| *t).unwrap_or(Instant::now());
    
    for (timestamp, events_per_sec) in throughput {
        let elapsed = timestamp.duration_since(start).as_secs();
        println!("  +{}s: {:.2} events/sec", elapsed, events_per_sec);
    }
    
    // Show top aggregates
    println!("\n=== Top Aggregates by Event Count ===");
    let top_aggregates = dashboard.top_aggregates(5);
    for (i, (aggregate_id, count)) in top_aggregates.iter().enumerate() {
        println!("  {}. {} - {} events", i + 1, aggregate_id, count);
    }
    
    // Show metrics for different time ranges
    println!("\n=== Metrics by Time Range ===");
    let ranges = vec![
        ("Last 5 seconds", Duration::from_secs(5)),
        ("Last 10 seconds", Duration::from_secs(10)),
        ("Last 30 seconds", Duration::from_secs(30)),
    ];
    
    for (label, duration) in ranges {
        let metrics = dashboard.metrics_for_range(duration);
        println!("\n{}:", label);
        println!("  Total Events: {}", metrics.total_events);
        println!("  Error Rate: {:.2}%", 
            if metrics.total_events > 0 {
                (metrics.error_count as f64 / metrics.total_events as f64) * 100.0
            } else {
                0.0
            }
        );
        println!("  Avg Processing Time: {:?}", metrics.processing_times.avg_time_per_event);
    }
}

fn create_workflow_event(aggregate_id: Uuid, index: u64) -> GraphEvent {
    GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        correlation_id: Uuid::new_v4(),
        causation_id: None,
        payload: EventPayload::Workflow(WorkflowPayload::StateTransitioned {
            instance_id: aggregate_id,
            from_state: format!("state_{}", index),
            to_state: format!("state_{}", index + 1),
        }),
    }
}

fn create_ipld_event(aggregate_id: Uuid, parent_id: Option<Uuid>) -> GraphEvent {
    GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        correlation_id: aggregate_id, // Use aggregate as correlation for chains
        causation_id: parent_id,
        payload: EventPayload::Ipld(IpldPayload::CidLinkAdded {
            cid: format!("Qm{}", Uuid::new_v4().to_string().chars().take(8).collect::<String>()),
            link_name: "next".to_string(),
            target_cid: format!("Qm{}", Uuid::new_v4().to_string().chars().take(8).collect::<String>()),
        }),
    }
}