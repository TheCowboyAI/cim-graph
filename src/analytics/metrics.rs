//! Event metrics collection and analysis

use crate::events::{GraphEvent, EventPayload};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Metrics collected for event processing
#[derive(Debug, Clone, Default)]
pub struct EventMetrics {
    /// Total number of events processed
    pub total_events: u64,
    /// Events grouped by type
    pub events_by_type: HashMap<String, u64>,
    /// Events per aggregate
    pub events_per_aggregate: HashMap<Uuid, u64>,
    /// Processing time statistics
    pub processing_times: ProcessingStats,
    /// Event correlation statistics
    pub correlation_stats: CorrelationStats,
    /// Error counts
    pub error_count: u64,
    /// Start time of metrics collection
    pub start_time: Option<Instant>,
}

/// Processing time statistics
#[derive(Debug, Clone, Default)]
pub struct ProcessingStats {
    /// Total processing time
    pub total_time: Duration,
    /// Average processing time per event
    pub avg_time_per_event: Duration,
    /// Minimum processing time
    pub min_time: Option<Duration>,
    /// Maximum processing time
    pub max_time: Option<Duration>,
    /// Events per second
    pub events_per_second: f64,
}

/// Event correlation statistics
#[derive(Debug, Clone, Default)]
pub struct CorrelationStats {
    /// Number of unique correlation IDs
    pub unique_correlations: usize,
    /// Average events per correlation
    pub avg_events_per_correlation: f64,
    /// Number of events with causation
    pub events_with_causation: u64,
    /// Average causation chain depth
    pub avg_causation_depth: f64,
}

/// Metrics collector for tracking event processing
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    metrics: EventMetrics,
    correlation_events: HashMap<Uuid, Vec<Uuid>>,
    causation_chains: HashMap<Uuid, u32>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: EventMetrics {
                start_time: Some(Instant::now()),
                ..Default::default()
            },
            correlation_events: HashMap::new(),
            causation_chains: HashMap::new(),
        }
    }
    
    /// Record an event being processed
    pub fn record_event(&mut self, event: &GraphEvent, processing_time: Duration) {
        self.metrics.total_events += 1;
        
        // Track event type
        let event_type = match &event.payload {
            EventPayload::Generic(_) => "generic",
            EventPayload::Ipld(_) => "ipld",
            EventPayload::Context(_) => "context",
            EventPayload::Workflow(_) => "workflow",
            EventPayload::Concept(_) => "concept",
            EventPayload::Composed(_) => "composed",
        };
        *self.metrics.events_by_type.entry(event_type.to_string()).or_insert(0) += 1;
        
        // Track per-aggregate
        *self.metrics.events_per_aggregate.entry(event.aggregate_id).or_insert(0) += 1;
        
        // Track correlation
        self.correlation_events
            .entry(event.correlation_id)
            .or_insert_with(Vec::new)
            .push(event.event_id);
        
        // Track causation depth
        if let Some(causation_id) = event.causation_id {
            let parent_depth = self.causation_chains.get(&causation_id).copied().unwrap_or(0);
            self.causation_chains.insert(event.event_id, parent_depth + 1);
            self.metrics.correlation_stats.events_with_causation += 1;
        } else {
            self.causation_chains.insert(event.event_id, 0);
        }
        
        // Update processing times
        self.update_processing_stats(processing_time);
    }
    
    /// Record an error
    pub fn record_error(&mut self) {
        self.metrics.error_count += 1;
    }
    
    /// Update processing statistics
    fn update_processing_stats(&mut self, duration: Duration) {
        let stats = &mut self.metrics.processing_times;
        
        stats.total_time += duration;
        stats.avg_time_per_event = stats.total_time / self.metrics.total_events.max(1) as u32;
        
        // Update min/max
        match stats.min_time {
            Some(min) if duration < min => stats.min_time = Some(duration),
            None => stats.min_time = Some(duration),
            _ => {}
        }
        
        match stats.max_time {
            Some(max) if duration > max => stats.max_time = Some(duration),
            None => stats.max_time = Some(duration),
            _ => {}
        }
        
        // Calculate events per second
        if let Some(start) = self.metrics.start_time {
            let elapsed = start.elapsed().as_secs_f64();
            if elapsed > 0.0 {
                stats.events_per_second = self.metrics.total_events as f64 / elapsed;
            }
        }
    }
    
    /// Finalize and return metrics
    pub fn finalize(mut self) -> EventMetrics {
        // Calculate correlation statistics
        let stats = &mut self.metrics.correlation_stats;
        stats.unique_correlations = self.correlation_events.len();
        
        if !self.correlation_events.is_empty() {
            let total_correlated: usize = self.correlation_events.values()
                .map(|v| v.len())
                .sum();
            stats.avg_events_per_correlation = total_correlated as f64 / stats.unique_correlations as f64;
        }
        
        if stats.events_with_causation > 0 {
            let total_depth: u32 = self.causation_chains.values().sum();
            stats.avg_causation_depth = total_depth as f64 / self.causation_chains.len() as f64;
        }
        
        self.metrics
    }
    
    /// Get current metrics without finalizing
    pub fn current_metrics(&self) -> &EventMetrics {
        &self.metrics
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate aggregate statistics across multiple metrics
pub fn aggregate_metrics(metrics_list: &[EventMetrics]) -> EventMetrics {
    if metrics_list.is_empty() {
        return EventMetrics::default();
    }
    
    let mut aggregated = EventMetrics::default();
    
    // Sum basic counters
    for metrics in metrics_list {
        aggregated.total_events += metrics.total_events;
        aggregated.error_count += metrics.error_count;
        
        // Merge event types
        for (event_type, count) in &metrics.events_by_type {
            *aggregated.events_by_type.entry(event_type.clone()).or_insert(0) += count;
        }
        
        // Merge per-aggregate counts
        for (agg_id, count) in &metrics.events_per_aggregate {
            *aggregated.events_per_aggregate.entry(*agg_id).or_insert(0) += count;
        }
    }
    
    // Calculate processing stats
    let total_time: Duration = metrics_list.iter()
        .map(|m| m.processing_times.total_time)
        .sum();
    
    aggregated.processing_times = ProcessingStats {
        total_time,
        avg_time_per_event: if aggregated.total_events > 0 {
            total_time / aggregated.total_events as u32
        } else {
            Duration::default()
        },
        min_time: metrics_list.iter()
            .filter_map(|m| m.processing_times.min_time)
            .min(),
        max_time: metrics_list.iter()
            .filter_map(|m| m.processing_times.max_time)
            .max(),
        events_per_second: metrics_list.iter()
            .map(|m| m.processing_times.events_per_second)
            .sum::<f64>() / metrics_list.len() as f64,
    };
    
    // Average correlation stats
    let avg_correlations: f64 = metrics_list.iter()
        .map(|m| m.correlation_stats.unique_correlations as f64)
        .sum::<f64>() / metrics_list.len() as f64;
    
    let avg_events_per_corr: f64 = metrics_list.iter()
        .map(|m| m.correlation_stats.avg_events_per_correlation)
        .sum::<f64>() / metrics_list.len() as f64;
    
    let total_causation_events: u64 = metrics_list.iter()
        .map(|m| m.correlation_stats.events_with_causation)
        .sum();
    
    let avg_causation_depth: f64 = metrics_list.iter()
        .map(|m| m.correlation_stats.avg_causation_depth)
        .sum::<f64>() / metrics_list.len() as f64;
    
    aggregated.correlation_stats = CorrelationStats {
        unique_correlations: avg_correlations as usize,
        avg_events_per_correlation: avg_events_per_corr,
        events_with_causation: total_causation_events,
        avg_causation_depth,
    };
    
    aggregated
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{IpldPayload, WorkflowPayload};
    
    fn create_test_event(event_type: &str) -> GraphEvent {
        let event_id = Uuid::new_v4();
        let payload = match event_type {
            "ipld" => EventPayload::Ipld(IpldPayload::CidLinkAdded {
                cid: "QmTest".to_string(),
                link_name: "test".to_string(),
                target_cid: "QmTarget".to_string(),
            }),
            "workflow" => EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
                workflow_id: Uuid::new_v4(),
                name: "Test".to_string(),
                version: "1.0".to_string(),
            }),
            _ => panic!("Unknown event type"),
        };
        
        GraphEvent {
            event_id,
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload,
        }
    }
    
    #[test]
    fn test_metrics_collection() {
        let mut collector = MetricsCollector::new();
        
        // Record some events
        for _ in 0..5 {
            let event = create_test_event("ipld");
            collector.record_event(&event, Duration::from_millis(10));
        }
        
        for _ in 0..3 {
            let event = create_test_event("workflow");
            collector.record_event(&event, Duration::from_millis(20));
        }
        
        collector.record_error();
        
        let metrics = collector.finalize();
        
        assert_eq!(metrics.total_events, 8);
        assert_eq!(metrics.error_count, 1);
        assert_eq!(metrics.events_by_type.get("ipld"), Some(&5));
        assert_eq!(metrics.events_by_type.get("workflow"), Some(&3));
        assert!(metrics.processing_times.events_per_second > 0.0);
    }
    
    #[test]
    fn test_causation_tracking() {
        let mut collector = MetricsCollector::new();
        
        let root_event = create_test_event("ipld");
        collector.record_event(&root_event, Duration::from_millis(5));
        
        // Create causation chain
        let mut parent_id = root_event.event_id;
        for _ in 0..3 {
            let mut event = create_test_event("ipld");
            event.causation_id = Some(parent_id);
            event.correlation_id = root_event.correlation_id;
            collector.record_event(&event, Duration::from_millis(5));
            parent_id = event.event_id;
        }
        
        let metrics = collector.finalize();
        
        assert_eq!(metrics.correlation_stats.events_with_causation, 3);
        assert!(metrics.correlation_stats.avg_causation_depth > 0.0);
        assert_eq!(metrics.correlation_stats.unique_correlations, 1);
        assert_eq!(metrics.correlation_stats.avg_events_per_correlation, 4.0);
    }
}