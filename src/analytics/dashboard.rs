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
            let event_type = event.payload.event_type();
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

impl crate::events::EventPayload {
    fn event_type(&self) -> String {
        match self {
            Self::Generic(_) => "generic".to_string(),
            Self::Ipld(_) => "ipld".to_string(),
            Self::Context(_) => "context".to_string(),
            Self::Workflow(_) => "workflow".to_string(),
            Self::Concept(_) => "concept".to_string(),
            Self::Composed(_) => "composed".to_string(),
        }
    }
}

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
}