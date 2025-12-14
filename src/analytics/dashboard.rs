//! Metrics dashboard for real-time event monitoring

use crate::analytics::metrics::{EventMetrics, MetricsCollector};
use crate::events::GraphEvent;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Configuration for the metrics dashboard
#[derive(Debug, Clone)]
pub struct DashboardConfig {
    /// Number of time windows to track
    pub window_count: usize,
    /// Duration of each time window
    pub window_duration: Duration,
    /// Maximum number of events to keep in history
    pub max_history: usize,
    /// Enable detailed per-aggregate tracking
    pub track_aggregates: bool,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            window_count: 10,
            window_duration: Duration::from_secs(60), // 1 minute windows
            max_history: 1000,
            track_aggregates: true,
        }
    }
}

/// Time window for metrics aggregation
#[derive(Debug, Clone)]
struct TimeWindow {
    start_time: Instant,
    end_time: Instant,
    metrics: EventMetrics,
    collector: MetricsCollector,
}

/// Real-time metrics dashboard
#[derive(Debug)]
pub struct MetricsDashboard {
    config: DashboardConfig,
    windows: Arc<RwLock<VecDeque<TimeWindow>>>,
    current_window: Arc<RwLock<TimeWindow>>,
    event_history: Arc<RwLock<VecDeque<EventSnapshot>>>,
}

/// Snapshot of an event for history tracking
#[derive(Debug, Clone)]
struct EventSnapshot {
    event_id: uuid::Uuid,
    aggregate_id: uuid::Uuid,
    event_type: String,
    timestamp: Instant,
    processing_time: Duration,
}

impl MetricsDashboard {
    /// Create a new metrics dashboard
    pub fn new(config: DashboardConfig) -> Self {
        let now = Instant::now();
        let current_window = TimeWindow {
            start_time: now,
            end_time: now + config.window_duration,
            metrics: EventMetrics::default(),
            collector: MetricsCollector::new(),
        };
        
        Self {
            config,
            windows: Arc::new(RwLock::new(VecDeque::new())),
            current_window: Arc::new(RwLock::new(current_window)),
            event_history: Arc::new(RwLock::new(VecDeque::new())),
        }
    }
    
    /// Record an event in the dashboard
    pub fn record_event(&self, event: &GraphEvent, processing_time: Duration) {
        let now = Instant::now();
        
        // Check if we need to rotate windows
        self.rotate_windows_if_needed(now);
        
        // Record in current window
        {
            let mut window = self.current_window.write().unwrap();
            window.collector.record_event(event, processing_time);
        }
        
        // Add to history
        if self.config.max_history > 0 {
            let event_type = event.payload.event_type().to_string();
            let snapshot = EventSnapshot {
                event_id: event.event_id,
                aggregate_id: event.aggregate_id,
                event_type,
                timestamp: now,
                processing_time,
            };
            
            let mut history = self.event_history.write().unwrap();
            history.push_back(snapshot);
            
            // Trim history if needed
            while history.len() > self.config.max_history {
                history.pop_front();
            }
        }
    }
    
    /// Record an error
    pub fn record_error(&self) {
        let mut window = self.current_window.write().unwrap();
        window.collector.record_error();
    }
    
    /// Rotate windows if current window has expired
    fn rotate_windows_if_needed(&self, now: Instant) {
        let mut current = self.current_window.write().unwrap();
        
        if now >= current.end_time {
            // Finalize current window
            let finalized_metrics = std::mem::replace(
                &mut current.collector,
                MetricsCollector::new()
            ).finalize();
            
            current.metrics = finalized_metrics;
            
            // Move to historical windows
            let completed_window = current.clone();
            drop(current); // Release write lock
            
            let mut windows = self.windows.write().unwrap();
            windows.push_back(completed_window);
            
            // Trim old windows
            while windows.len() > self.config.window_count {
                windows.pop_front();
            }
            
            // Create new current window
            let mut current = self.current_window.write().unwrap();
            *current = TimeWindow {
                start_time: now,
                end_time: now + self.config.window_duration,
                metrics: EventMetrics::default(),
                collector: MetricsCollector::new(),
            };
        }
    }
    
    /// Get current metrics (real-time)
    pub fn current_metrics(&self) -> EventMetrics {
        let window = self.current_window.read().unwrap();
        window.collector.current_metrics().clone()
    }
    
    /// Get metrics for a specific time range
    pub fn metrics_for_range(&self, duration: Duration) -> EventMetrics {
        let cutoff = Instant::now() - duration;
        
        let windows = self.windows.read().unwrap();
        let mut relevant_metrics = Vec::new();
        
        // Include historical windows in range
        for window in windows.iter() {
            if window.end_time > cutoff {
                relevant_metrics.push(window.metrics.clone());
            }
        }
        
        // Include current window
        let current = self.current_window.read().unwrap();
        if current.start_time > cutoff {
            relevant_metrics.push(current.collector.current_metrics().clone());
        }
        
        crate::analytics::metrics::aggregate_metrics(&relevant_metrics)
    }
    
    /// Get throughput over time (events per second for each window)
    pub fn throughput_history(&self) -> Vec<(Instant, f64)> {
        let mut history = Vec::new();
        
        let windows = self.windows.read().unwrap();
        for window in windows.iter() {
            history.push((
                window.start_time,
                window.metrics.processing_times.events_per_second
            ));
        }
        
        // Add current window
        let current = self.current_window.read().unwrap();
        let current_metrics = current.collector.current_metrics();
        history.push((
            current.start_time,
            current_metrics.processing_times.events_per_second
        ));
        
        history
    }
    
    /// Get recent events from history
    pub fn recent_events(&self, limit: usize) -> Vec<(Instant, String, Duration)> {
        let history = self.event_history.read().unwrap();
        history.iter()
            .rev()
            .take(limit)
            .map(|s| (s.timestamp, s.event_type.clone(), s.processing_time))
            .collect()
    }
    
    /// Get events by aggregate from history
    pub fn events_by_aggregate(&self, aggregate_id: uuid::Uuid) -> Vec<(Instant, String, Duration)> {
        let history = self.event_history.read().unwrap();
        history.iter()
            .filter(|s| s.aggregate_id == aggregate_id)
            .map(|s| (s.timestamp, s.event_type.clone(), s.processing_time))
            .collect()
    }
    
    /// Get detailed event history with IDs
    pub fn detailed_event_history(&self, limit: usize) -> Vec<(uuid::Uuid, uuid::Uuid, String, Instant, Duration)> {
        let history = self.event_history.read().unwrap();
        history.iter()
            .rev()
            .take(limit)
            .map(|s| (s.event_id, s.aggregate_id, s.event_type.clone(), s.timestamp, s.processing_time))
            .collect()
    }
    
    /// Get processing time percentiles
    pub fn processing_time_percentiles(&self) -> ProcessingTimePercentiles {
        let history = self.event_history.read().unwrap();
        
        if history.is_empty() {
            return ProcessingTimePercentiles::default();
        }
        
        let mut times: Vec<Duration> = history.iter()
            .map(|s| s.processing_time)
            .collect();
        
        times.sort();
        
        let len = times.len();
        ProcessingTimePercentiles {
            p50: times[len / 2],
            p90: times[len * 9 / 10],
            p95: times[len * 95 / 100],
            p99: times[len * 99 / 100],
            max: times[len - 1],
        }
    }
    
    /// Get top aggregates by event count
    pub fn top_aggregates(&self, limit: usize) -> Vec<(uuid::Uuid, u64)> {
        let current_metrics = self.current_metrics();
        
        let mut aggregates: Vec<_> = current_metrics.events_per_aggregate
            .into_iter()
            .collect();
        
        aggregates.sort_by(|a, b| b.1.cmp(&a.1));
        aggregates.truncate(limit);
        
        aggregates
    }
    
    /// Generate a text summary of current metrics
    pub fn summary(&self) -> String {
        let metrics = self.current_metrics();
        let percentiles = self.processing_time_percentiles();
        
        format!(
            r#"=== Event Processing Dashboard ===
Total Events: {}
Error Rate: {:.2}%
Throughput: {:.2} events/sec

Event Types:
{}

Processing Times:
  p50: {:?}
  p90: {:?}
  p95: {:?}
  p99: {:?}
  max: {:?}

Correlation Stats:
  Unique Correlations: {}
  Avg Events/Correlation: {:.2}
  Events with Causation: {} ({:.2}%)
  Avg Causation Depth: {:.2}
"#,
            metrics.total_events,
            if metrics.total_events > 0 {
                (metrics.error_count as f64 / metrics.total_events as f64) * 100.0
            } else {
                0.0
            },
            metrics.processing_times.events_per_second,
            metrics.events_by_type.iter()
                .map(|(t, c)| format!("  {}: {}", t, c))
                .collect::<Vec<_>>()
                .join("\n"),
            percentiles.p50,
            percentiles.p90,
            percentiles.p95,
            percentiles.p99,
            percentiles.max,
            metrics.correlation_stats.unique_correlations,
            metrics.correlation_stats.avg_events_per_correlation,
            metrics.correlation_stats.events_with_causation,
            if metrics.total_events > 0 {
                (metrics.correlation_stats.events_with_causation as f64 / metrics.total_events as f64) * 100.0
            } else {
                0.0
            },
            metrics.correlation_stats.avg_causation_depth
        )
    }
}

/// Processing time percentiles
#[derive(Debug, Clone, Default)]
pub struct ProcessingTimePercentiles {
    /// 50th percentile (median)
    pub p50: Duration,
    /// 90th percentile
    pub p90: Duration,
    /// 95th percentile
    pub p95: Duration,
    /// 99th percentile
    pub p99: Duration,
    /// Maximum observed
    pub max: Duration,
}

// event_type() is defined in crate::events::graph_events

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{EventPayload, IpldPayload};
    
    fn create_test_event() -> GraphEvent {
        GraphEvent {
            event_id: uuid::Uuid::new_v4(),
            aggregate_id: uuid::Uuid::new_v4(),
            correlation_id: uuid::Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Ipld(IpldPayload::CidLinkAdded {
                cid: "QmTest".to_string(),
                link_name: "test".to_string(),
                target_cid: "QmTarget".to_string(),
            }),
        }
    }
    
    #[test]
    fn test_dashboard_basic_metrics() {
        let config = DashboardConfig {
            window_duration: Duration::from_millis(100),
            ..Default::default()
        };
        
        let dashboard = MetricsDashboard::new(config);
        
        // Record some events
        for _ in 0..10 {
            let event = create_test_event();
            dashboard.record_event(&event, Duration::from_millis(5));
        }
        
        dashboard.record_error();
        
        let metrics = dashboard.current_metrics();
        assert_eq!(metrics.total_events, 10);
        assert_eq!(metrics.error_count, 1);
    }
    
    #[test]
    fn test_window_rotation() {
        let config = DashboardConfig {
            window_duration: Duration::from_millis(50),
            window_count: 3,
            ..Default::default()
        };
        
        let dashboard = MetricsDashboard::new(config);
        
        // First window
        for _ in 0..5 {
            dashboard.record_event(&create_test_event(), Duration::from_millis(1));
        }
        
        // Wait for rotation
        std::thread::sleep(Duration::from_millis(60));
        
        // Second window
        for _ in 0..3 {
            dashboard.record_event(&create_test_event(), Duration::from_millis(1));
        }
        
        let history = dashboard.throughput_history();
        assert!(history.len() >= 2);
    }
    
    #[test]
    fn test_dashboard_summary() {
        let dashboard = MetricsDashboard::new(DashboardConfig::default());
        
        for i in 0..20 {
            let mut event = create_test_event();
            if i > 10 {
                event.causation_id = Some(uuid::Uuid::new_v4());
            }
            dashboard.record_event(&event, Duration::from_millis(i as u64));
        }
        
        let summary = dashboard.summary();
        assert!(summary.contains("Total Events: 20"));
        assert!(summary.contains("ipld: 20"));
    }
    
    #[test]
    fn test_event_history_tracking() {
        let dashboard = MetricsDashboard::new(DashboardConfig::default());

        let agg1 = uuid::Uuid::new_v4();
        let agg2 = uuid::Uuid::new_v4();

        // Record events for different aggregates
        for i in 0..5 {
            let mut event = create_test_event();
            event.aggregate_id = agg1;
            dashboard.record_event(&event, Duration::from_millis(i * 10));
        }

        for i in 0..3 {
            let mut event = create_test_event();
            event.aggregate_id = agg2;
            dashboard.record_event(&event, Duration::from_millis(i * 20));
        }

        // Test recent events
        let recent = dashboard.recent_events(5);
        assert_eq!(recent.len(), 5);
        assert!(recent.iter().all(|(_, event_type, _)| event_type == "ipld"));

        // Test events by aggregate
        let agg1_events = dashboard.events_by_aggregate(agg1);
        assert_eq!(agg1_events.len(), 5);

        let agg2_events = dashboard.events_by_aggregate(agg2);
        assert_eq!(agg2_events.len(), 3);
    }

    // ========== DashboardConfig Tests ==========

    #[test]
    fn test_dashboard_config_default() {
        let config = DashboardConfig::default();
        assert_eq!(config.window_count, 10);
        assert_eq!(config.window_duration, Duration::from_secs(60));
        assert_eq!(config.max_history, 1000);
        assert!(config.track_aggregates);
    }

    #[test]
    fn test_dashboard_config_clone() {
        let config = DashboardConfig {
            window_count: 5,
            window_duration: Duration::from_secs(30),
            max_history: 500,
            track_aggregates: false,
        };

        let cloned = config.clone();
        assert_eq!(cloned.window_count, 5);
        assert_eq!(cloned.window_duration, Duration::from_secs(30));
        assert_eq!(cloned.max_history, 500);
        assert!(!cloned.track_aggregates);
    }

    #[test]
    fn test_dashboard_config_debug() {
        let config = DashboardConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("DashboardConfig"));
        assert!(debug_str.contains("window_count"));
    }

    // ========== MetricsDashboard Tests ==========

    #[test]
    fn test_metrics_dashboard_new() {
        let config = DashboardConfig::default();
        let dashboard = MetricsDashboard::new(config);
        let metrics = dashboard.current_metrics();
        assert_eq!(metrics.total_events, 0);
    }

    #[test]
    fn test_metrics_dashboard_debug() {
        let dashboard = MetricsDashboard::new(DashboardConfig::default());
        let debug_str = format!("{:?}", dashboard);
        assert!(debug_str.contains("MetricsDashboard"));
    }

    #[test]
    fn test_metrics_dashboard_record_event_with_history_disabled() {
        let config = DashboardConfig {
            max_history: 0, // Disable history
            ..Default::default()
        };
        let dashboard = MetricsDashboard::new(config);

        let event = create_test_event();
        dashboard.record_event(&event, Duration::from_millis(5));

        // Events are still recorded in metrics
        let metrics = dashboard.current_metrics();
        assert_eq!(metrics.total_events, 1);

        // But no history is kept
        let recent = dashboard.recent_events(10);
        assert!(recent.is_empty());
    }

    #[test]
    fn test_metrics_dashboard_history_trimming() {
        let config = DashboardConfig {
            max_history: 5, // Small history limit
            ..Default::default()
        };
        let dashboard = MetricsDashboard::new(config);

        // Add more events than history limit
        for _ in 0..10 {
            let event = create_test_event();
            dashboard.record_event(&event, Duration::from_millis(1));
        }

        // History should be trimmed to max_history
        let recent = dashboard.recent_events(20);
        assert_eq!(recent.len(), 5);
    }

    #[test]
    fn test_metrics_dashboard_metrics_for_range() {
        let config = DashboardConfig {
            window_duration: Duration::from_millis(50),
            window_count: 5,
            ..Default::default()
        };
        let dashboard = MetricsDashboard::new(config);

        // Record events
        for _ in 0..10 {
            let event = create_test_event();
            dashboard.record_event(&event, Duration::from_millis(1));
        }

        // Get metrics for a duration
        let metrics = dashboard.metrics_for_range(Duration::from_secs(60));
        assert_eq!(metrics.total_events, 10);
    }

    #[test]
    fn test_metrics_dashboard_throughput_history() {
        let config = DashboardConfig {
            window_duration: Duration::from_millis(50),
            window_count: 3,
            ..Default::default()
        };
        let dashboard = MetricsDashboard::new(config);

        // Record events
        for _ in 0..5 {
            let event = create_test_event();
            dashboard.record_event(&event, Duration::from_millis(1));
        }

        let history = dashboard.throughput_history();
        // At least current window should be in history
        assert!(!history.is_empty());
    }

    #[test]
    fn test_metrics_dashboard_detailed_event_history() {
        let dashboard = MetricsDashboard::new(DashboardConfig::default());

        let event = create_test_event();
        let event_id = event.event_id;
        let aggregate_id = event.aggregate_id;
        dashboard.record_event(&event, Duration::from_millis(5));

        let detailed = dashboard.detailed_event_history(10);
        assert_eq!(detailed.len(), 1);

        let (stored_event_id, stored_agg_id, event_type, _, _) = &detailed[0];
        assert_eq!(*stored_event_id, event_id);
        assert_eq!(*stored_agg_id, aggregate_id);
        assert_eq!(event_type, "ipld");
    }

    // ========== ProcessingTimePercentiles Tests ==========

    #[test]
    fn test_processing_time_percentiles_default() {
        let percentiles = ProcessingTimePercentiles::default();
        assert_eq!(percentiles.p50, Duration::ZERO);
        assert_eq!(percentiles.p90, Duration::ZERO);
        assert_eq!(percentiles.p95, Duration::ZERO);
        assert_eq!(percentiles.p99, Duration::ZERO);
        assert_eq!(percentiles.max, Duration::ZERO);
    }

    #[test]
    fn test_processing_time_percentiles_clone() {
        let percentiles = ProcessingTimePercentiles {
            p50: Duration::from_millis(10),
            p90: Duration::from_millis(50),
            p95: Duration::from_millis(75),
            p99: Duration::from_millis(95),
            max: Duration::from_millis(100),
        };

        let cloned = percentiles.clone();
        assert_eq!(cloned.p50, Duration::from_millis(10));
        assert_eq!(cloned.p90, Duration::from_millis(50));
        assert_eq!(cloned.p99, Duration::from_millis(95));
    }

    #[test]
    fn test_processing_time_percentiles_debug() {
        let percentiles = ProcessingTimePercentiles::default();
        let debug_str = format!("{:?}", percentiles);
        assert!(debug_str.contains("ProcessingTimePercentiles"));
    }

    #[test]
    fn test_metrics_dashboard_processing_time_percentiles_empty() {
        let dashboard = MetricsDashboard::new(DashboardConfig::default());
        let percentiles = dashboard.processing_time_percentiles();
        // Empty history should return default percentiles
        assert_eq!(percentiles.p50, Duration::ZERO);
    }

    #[test]
    fn test_metrics_dashboard_processing_time_percentiles_with_data() {
        let dashboard = MetricsDashboard::new(DashboardConfig::default());

        // Add events with varying processing times (100 events)
        for i in 1..=100 {
            let event = create_test_event();
            dashboard.record_event(&event, Duration::from_millis(i));
        }

        let percentiles = dashboard.processing_time_percentiles();

        // p50 should be around 50ms (middle value)
        assert!(percentiles.p50 >= Duration::from_millis(40));
        assert!(percentiles.p50 <= Duration::from_millis(60));

        // p99 should be around 99ms
        assert!(percentiles.p99 >= Duration::from_millis(90));

        // max should be 100ms
        assert_eq!(percentiles.max, Duration::from_millis(100));
    }

    #[test]
    fn test_metrics_dashboard_top_aggregates() {
        let dashboard = MetricsDashboard::new(DashboardConfig::default());

        let agg1 = uuid::Uuid::new_v4();
        let agg2 = uuid::Uuid::new_v4();
        let agg3 = uuid::Uuid::new_v4();

        // agg1: 10 events, agg2: 5 events, agg3: 3 events
        for _ in 0..10 {
            let mut event = create_test_event();
            event.aggregate_id = agg1;
            dashboard.record_event(&event, Duration::from_millis(1));
        }
        for _ in 0..5 {
            let mut event = create_test_event();
            event.aggregate_id = agg2;
            dashboard.record_event(&event, Duration::from_millis(1));
        }
        for _ in 0..3 {
            let mut event = create_test_event();
            event.aggregate_id = agg3;
            dashboard.record_event(&event, Duration::from_millis(1));
        }

        let top = dashboard.top_aggregates(2);
        assert_eq!(top.len(), 2);
        // Top aggregate should be agg1 with 10 events
        assert_eq!(top[0].0, agg1);
        assert_eq!(top[0].1, 10);
        // Second should be agg2 with 5 events
        assert_eq!(top[1].0, agg2);
        assert_eq!(top[1].1, 5);
    }

    #[test]
    fn test_metrics_dashboard_summary_error_rate() {
        let dashboard = MetricsDashboard::new(DashboardConfig::default());

        // Add events with some errors
        for _ in 0..10 {
            let event = create_test_event();
            dashboard.record_event(&event, Duration::from_millis(1));
        }
        dashboard.record_error();
        dashboard.record_error();

        let summary = dashboard.summary();
        assert!(summary.contains("Error Rate:"));
        assert!(summary.contains("Total Events: 10"));
    }

    #[test]
    fn test_metrics_dashboard_summary_empty() {
        let dashboard = MetricsDashboard::new(DashboardConfig::default());
        let summary = dashboard.summary();

        // Should handle zero events gracefully (no divide by zero)
        assert!(summary.contains("Total Events: 0"));
        assert!(summary.contains("Error Rate: 0.00%"));
    }

    #[test]
    fn test_metrics_dashboard_events_by_nonexistent_aggregate() {
        let dashboard = MetricsDashboard::new(DashboardConfig::default());

        // Add some events
        let event = create_test_event();
        dashboard.record_event(&event, Duration::from_millis(1));

        // Query for a non-existent aggregate
        let random_agg = uuid::Uuid::new_v4();
        let events = dashboard.events_by_aggregate(random_agg);
        assert!(events.is_empty());
    }

    #[test]
    fn test_metrics_dashboard_recent_events_limit() {
        let dashboard = MetricsDashboard::new(DashboardConfig::default());

        // Add 20 events
        for _ in 0..20 {
            let event = create_test_event();
            dashboard.record_event(&event, Duration::from_millis(1));
        }

        // Request only 5
        let recent = dashboard.recent_events(5);
        assert_eq!(recent.len(), 5);

        // Request more than available
        let all_recent = dashboard.recent_events(100);
        assert_eq!(all_recent.len(), 20);
    }

    // ========== EventPayload::event_type Tests ==========

    #[test]
    fn test_event_payload_type_generic() {
        use crate::events::GenericPayload;
        let payload = EventPayload::Generic(GenericPayload {
            event_type: "Test".to_string(),
            data: serde_json::json!({}),
        });
        assert_eq!(payload.event_type(), "generic");
    }

    #[test]
    fn test_event_payload_type_ipld() {
        let payload = EventPayload::Ipld(IpldPayload::CidLinkAdded {
            cid: "Qm".to_string(),
            link_name: "test".to_string(),
            target_cid: "QmT".to_string(),
        });
        assert_eq!(payload.event_type(), "ipld");
    }

    #[test]
    fn test_event_payload_type_context() {
        use crate::events::ContextPayload;
        let payload = EventPayload::Context(ContextPayload::BoundedContextCreated {
            context_id: "ctx".to_string(),
            name: "Test".to_string(),
            description: "Desc".to_string(),
        });
        assert_eq!(payload.event_type(), "context");
    }

    #[test]
    fn test_event_payload_type_workflow() {
        use crate::events::WorkflowPayload;
        let payload = EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
            workflow_id: uuid::Uuid::new_v4(),
            name: "Test".to_string(),
            version: "1.0".to_string(),
        });
        assert_eq!(payload.event_type(), "workflow");
    }

    #[test]
    fn test_event_payload_type_concept() {
        use crate::events::ConceptPayload;
        let payload = EventPayload::Concept(ConceptPayload::ConceptDefined {
            concept_id: "c1".to_string(),
            name: "Test".to_string(),
            definition: "Def".to_string(),
        });
        assert_eq!(payload.event_type(), "concept");
    }

    #[test]
    fn test_event_payload_type_composed() {
        use crate::events::ComposedPayload;
        let payload = EventPayload::Composed(ComposedPayload::SubGraphAdded {
            subgraph_id: uuid::Uuid::new_v4(),
            graph_type: "test".to_string(),
            namespace: "ns".to_string(),
        });
        assert_eq!(payload.event_type(), "composed");
    }

    // ========== Concurrent Access Tests ==========

    #[test]
    fn test_metrics_dashboard_concurrent_record() {
        use std::sync::Arc;
        use std::thread;

        let dashboard = Arc::new(MetricsDashboard::new(DashboardConfig::default()));

        let handles: Vec<_> = (0..4)
            .map(|_| {
                let dashboard_clone = Arc::clone(&dashboard);
                thread::spawn(move || {
                    for _ in 0..25 {
                        let event = GraphEvent {
                            event_id: uuid::Uuid::new_v4(),
                            aggregate_id: uuid::Uuid::new_v4(),
                            correlation_id: uuid::Uuid::new_v4(),
                            causation_id: None,
                            payload: EventPayload::Ipld(IpldPayload::CidLinkAdded {
                                cid: "QmTest".to_string(),
                                link_name: "test".to_string(),
                                target_cid: "QmTarget".to_string(),
                            }),
                        };
                        dashboard_clone.record_event(&event, Duration::from_millis(1));
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let metrics = dashboard.current_metrics();
        assert_eq!(metrics.total_events, 100);
    }

    #[test]
    fn test_metrics_dashboard_concurrent_record_and_error() {
        use std::sync::Arc;
        use std::thread;

        let dashboard = Arc::new(MetricsDashboard::new(DashboardConfig::default()));

        let dashboard_events = Arc::clone(&dashboard);
        let dashboard_errors = Arc::clone(&dashboard);

        let event_thread = thread::spawn(move || {
            for _ in 0..50 {
                let event = GraphEvent {
                    event_id: uuid::Uuid::new_v4(),
                    aggregate_id: uuid::Uuid::new_v4(),
                    correlation_id: uuid::Uuid::new_v4(),
                    causation_id: None,
                    payload: EventPayload::Ipld(IpldPayload::CidLinkAdded {
                        cid: "QmTest".to_string(),
                        link_name: "test".to_string(),
                        target_cid: "QmTarget".to_string(),
                    }),
                };
                dashboard_events.record_event(&event, Duration::from_millis(1));
            }
        });

        let error_thread = thread::spawn(move || {
            for _ in 0..10 {
                dashboard_errors.record_error();
            }
        });

        event_thread.join().unwrap();
        error_thread.join().unwrap();

        let metrics = dashboard.current_metrics();
        assert_eq!(metrics.total_events, 50);
        assert_eq!(metrics.error_count, 10);
    }
}