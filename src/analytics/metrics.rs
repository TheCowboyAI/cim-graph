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

    // ========== EventMetrics Tests ==========

    #[test]
    fn test_event_metrics_default() {
        let metrics = EventMetrics::default();
        assert_eq!(metrics.total_events, 0);
        assert_eq!(metrics.error_count, 0);
        assert!(metrics.events_by_type.is_empty());
        assert!(metrics.events_per_aggregate.is_empty());
        assert!(metrics.start_time.is_none());
    }

    #[test]
    fn test_event_metrics_clone() {
        let mut metrics = EventMetrics::default();
        metrics.total_events = 100;
        metrics.error_count = 5;
        metrics.events_by_type.insert("test".to_string(), 50);

        let cloned = metrics.clone();
        assert_eq!(cloned.total_events, 100);
        assert_eq!(cloned.error_count, 5);
        assert_eq!(cloned.events_by_type.get("test"), Some(&50));
    }

    // ========== ProcessingStats Tests ==========

    #[test]
    fn test_processing_stats_default() {
        let stats = ProcessingStats::default();
        assert_eq!(stats.total_time, Duration::ZERO);
        assert_eq!(stats.avg_time_per_event, Duration::ZERO);
        assert!(stats.min_time.is_none());
        assert!(stats.max_time.is_none());
        assert_eq!(stats.events_per_second, 0.0);
    }

    #[test]
    fn test_processing_stats_clone() {
        let stats = ProcessingStats {
            total_time: Duration::from_millis(100),
            avg_time_per_event: Duration::from_millis(10),
            min_time: Some(Duration::from_millis(5)),
            max_time: Some(Duration::from_millis(20)),
            events_per_second: 10.0,
        };

        let cloned = stats.clone();
        assert_eq!(cloned.total_time, Duration::from_millis(100));
        assert_eq!(cloned.avg_time_per_event, Duration::from_millis(10));
        assert_eq!(cloned.min_time, Some(Duration::from_millis(5)));
        assert_eq!(cloned.max_time, Some(Duration::from_millis(20)));
        assert_eq!(cloned.events_per_second, 10.0);
    }

    // ========== CorrelationStats Tests ==========

    #[test]
    fn test_correlation_stats_default() {
        let stats = CorrelationStats::default();
        assert_eq!(stats.unique_correlations, 0);
        assert_eq!(stats.avg_events_per_correlation, 0.0);
        assert_eq!(stats.events_with_causation, 0);
        assert_eq!(stats.avg_causation_depth, 0.0);
    }

    #[test]
    fn test_correlation_stats_clone() {
        let stats = CorrelationStats {
            unique_correlations: 5,
            avg_events_per_correlation: 3.5,
            events_with_causation: 10,
            avg_causation_depth: 2.0,
        };

        let cloned = stats.clone();
        assert_eq!(cloned.unique_correlations, 5);
        assert_eq!(cloned.avg_events_per_correlation, 3.5);
        assert_eq!(cloned.events_with_causation, 10);
        assert_eq!(cloned.avg_causation_depth, 2.0);
    }

    // ========== MetricsCollector Tests ==========

    #[test]
    fn test_metrics_collector_new() {
        let collector = MetricsCollector::new();
        assert_eq!(collector.metrics.total_events, 0);
        assert!(collector.metrics.start_time.is_some());
    }

    #[test]
    fn test_metrics_collector_default() {
        let collector = MetricsCollector::default();
        assert_eq!(collector.metrics.total_events, 0);
    }

    #[test]
    fn test_metrics_collector_record_error() {
        let mut collector = MetricsCollector::new();
        assert_eq!(collector.metrics.error_count, 0);

        collector.record_error();
        assert_eq!(collector.metrics.error_count, 1);

        collector.record_error();
        collector.record_error();
        assert_eq!(collector.metrics.error_count, 3);
    }

    #[test]
    fn test_metrics_collector_current_metrics() {
        let mut collector = MetricsCollector::new();

        let event = create_test_event("ipld");
        collector.record_event(&event, Duration::from_millis(10));

        let current = collector.current_metrics();
        assert_eq!(current.total_events, 1);
        assert_eq!(current.events_by_type.get("ipld"), Some(&1));
    }

    #[test]
    fn test_metrics_collector_processing_stats() {
        let mut collector = MetricsCollector::new();

        // Record events with different processing times
        for i in 1..=5 {
            let event = create_test_event("ipld");
            collector.record_event(&event, Duration::from_millis(i * 10));
        }

        let metrics = collector.finalize();

        // Check min/max times
        assert_eq!(metrics.processing_times.min_time, Some(Duration::from_millis(10)));
        assert_eq!(metrics.processing_times.max_time, Some(Duration::from_millis(50)));

        // Total time = 10 + 20 + 30 + 40 + 50 = 150ms
        assert_eq!(metrics.processing_times.total_time, Duration::from_millis(150));
    }

    #[test]
    fn test_metrics_collector_per_aggregate_tracking() {
        let mut collector = MetricsCollector::new();

        let agg1 = Uuid::new_v4();
        let agg2 = Uuid::new_v4();

        // Create events for aggregate 1
        for _ in 0..3 {
            let mut event = create_test_event("ipld");
            event.aggregate_id = agg1;
            collector.record_event(&event, Duration::from_millis(1));
        }

        // Create events for aggregate 2
        for _ in 0..2 {
            let mut event = create_test_event("workflow");
            event.aggregate_id = agg2;
            collector.record_event(&event, Duration::from_millis(1));
        }

        let metrics = collector.finalize();

        assert_eq!(metrics.events_per_aggregate.get(&agg1), Some(&3));
        assert_eq!(metrics.events_per_aggregate.get(&agg2), Some(&2));
    }

    #[test]
    fn test_metrics_collector_multiple_correlations() {
        let mut collector = MetricsCollector::new();

        // Create events with different correlation IDs
        for _ in 0..3 {
            let event = create_test_event("ipld"); // Each creates new correlation_id
            collector.record_event(&event, Duration::from_millis(1));
        }

        let metrics = collector.finalize();
        assert_eq!(metrics.correlation_stats.unique_correlations, 3);
        assert_eq!(metrics.correlation_stats.avg_events_per_correlation, 1.0);
    }

    #[test]
    fn test_metrics_collector_events_by_type_all_types() {
        use crate::events::{ContextPayload, ConceptPayload, ComposedPayload, GenericPayload};

        let mut collector = MetricsCollector::new();

        // Test all event types
        let events = vec![
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: Uuid::new_v4(),
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Generic(GenericPayload {
                    event_type: "Test".to_string(),
                    data: serde_json::json!({}),
                }),
            },
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: Uuid::new_v4(),
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Ipld(IpldPayload::CidLinkAdded {
                    cid: "QmTest".to_string(),
                    link_name: "test".to_string(),
                    target_cid: "QmTarget".to_string(),
                }),
            },
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: Uuid::new_v4(),
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Context(ContextPayload::BoundedContextCreated {
                    context_id: "ctx1".to_string(),
                    name: "Test".to_string(),
                    description: "Desc".to_string(),
                }),
            },
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: Uuid::new_v4(),
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
                    workflow_id: Uuid::new_v4(),
                    name: "Test".to_string(),
                    version: "1.0".to_string(),
                }),
            },
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: Uuid::new_v4(),
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Concept(ConceptPayload::ConceptDefined {
                    concept_id: "c1".to_string(),
                    name: "Test".to_string(),
                    definition: "Def".to_string(),
                }),
            },
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: Uuid::new_v4(),
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Composed(ComposedPayload::SubGraphAdded {
                    subgraph_id: Uuid::new_v4(),
                    graph_type: "test".to_string(),
                    namespace: "ns".to_string(),
                }),
            },
        ];

        for event in &events {
            collector.record_event(event, Duration::from_millis(1));
        }

        let metrics = collector.finalize();

        assert_eq!(metrics.events_by_type.get("generic"), Some(&1));
        assert_eq!(metrics.events_by_type.get("ipld"), Some(&1));
        assert_eq!(metrics.events_by_type.get("context"), Some(&1));
        assert_eq!(metrics.events_by_type.get("workflow"), Some(&1));
        assert_eq!(metrics.events_by_type.get("concept"), Some(&1));
        assert_eq!(metrics.events_by_type.get("composed"), Some(&1));
    }

    #[test]
    fn test_metrics_collector_clone() {
        let mut collector = MetricsCollector::new();
        let event = create_test_event("ipld");
        collector.record_event(&event, Duration::from_millis(10));

        let cloned = collector.clone();
        assert_eq!(cloned.metrics.total_events, 1);
    }

    #[test]
    fn test_metrics_collector_debug() {
        let collector = MetricsCollector::new();
        let debug_str = format!("{:?}", collector);
        assert!(debug_str.contains("MetricsCollector"));
    }

    // ========== aggregate_metrics Tests ==========

    #[test]
    fn test_aggregate_metrics_empty() {
        let result = aggregate_metrics(&[]);
        assert_eq!(result.total_events, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_aggregate_metrics_single() {
        let mut metrics = EventMetrics::default();
        metrics.total_events = 10;
        metrics.error_count = 2;
        metrics.events_by_type.insert("ipld".to_string(), 10);

        let result = aggregate_metrics(&[metrics]);
        assert_eq!(result.total_events, 10);
        assert_eq!(result.error_count, 2);
        assert_eq!(result.events_by_type.get("ipld"), Some(&10));
    }

    #[test]
    fn test_aggregate_metrics_multiple() {
        let mut metrics1 = EventMetrics::default();
        metrics1.total_events = 10;
        metrics1.error_count = 1;
        metrics1.events_by_type.insert("ipld".to_string(), 8);
        metrics1.events_by_type.insert("workflow".to_string(), 2);
        metrics1.processing_times = ProcessingStats {
            total_time: Duration::from_millis(100),
            avg_time_per_event: Duration::from_millis(10),
            min_time: Some(Duration::from_millis(5)),
            max_time: Some(Duration::from_millis(20)),
            events_per_second: 10.0,
        };

        let mut metrics2 = EventMetrics::default();
        metrics2.total_events = 5;
        metrics2.error_count = 0;
        metrics2.events_by_type.insert("ipld".to_string(), 3);
        metrics2.events_by_type.insert("context".to_string(), 2);
        metrics2.processing_times = ProcessingStats {
            total_time: Duration::from_millis(50),
            avg_time_per_event: Duration::from_millis(10),
            min_time: Some(Duration::from_millis(2)),
            max_time: Some(Duration::from_millis(15)),
            events_per_second: 5.0,
        };

        let result = aggregate_metrics(&[metrics1, metrics2]);

        assert_eq!(result.total_events, 15);
        assert_eq!(result.error_count, 1);
        assert_eq!(result.events_by_type.get("ipld"), Some(&11));
        assert_eq!(result.events_by_type.get("workflow"), Some(&2));
        assert_eq!(result.events_by_type.get("context"), Some(&2));
        assert_eq!(result.processing_times.total_time, Duration::from_millis(150));
        assert_eq!(result.processing_times.min_time, Some(Duration::from_millis(2)));
        assert_eq!(result.processing_times.max_time, Some(Duration::from_millis(20)));
    }

    #[test]
    fn test_aggregate_metrics_per_aggregate_merge() {
        let agg1 = Uuid::new_v4();
        let agg2 = Uuid::new_v4();
        let agg3 = Uuid::new_v4();

        let mut metrics1 = EventMetrics::default();
        metrics1.events_per_aggregate.insert(agg1, 5);
        metrics1.events_per_aggregate.insert(agg2, 3);

        let mut metrics2 = EventMetrics::default();
        metrics2.events_per_aggregate.insert(agg1, 2);
        metrics2.events_per_aggregate.insert(agg3, 4);

        let result = aggregate_metrics(&[metrics1, metrics2]);

        assert_eq!(result.events_per_aggregate.get(&agg1), Some(&7));
        assert_eq!(result.events_per_aggregate.get(&agg2), Some(&3));
        assert_eq!(result.events_per_aggregate.get(&agg3), Some(&4));
    }

    #[test]
    fn test_aggregate_metrics_correlation_stats() {
        let mut metrics1 = EventMetrics::default();
        metrics1.correlation_stats = CorrelationStats {
            unique_correlations: 10,
            avg_events_per_correlation: 2.0,
            events_with_causation: 5,
            avg_causation_depth: 1.5,
        };

        let mut metrics2 = EventMetrics::default();
        metrics2.correlation_stats = CorrelationStats {
            unique_correlations: 20,
            avg_events_per_correlation: 3.0,
            events_with_causation: 15,
            avg_causation_depth: 2.5,
        };

        let result = aggregate_metrics(&[metrics1, metrics2]);

        // Averages
        assert_eq!(result.correlation_stats.unique_correlations, 15); // Average of 10, 20
        assert_eq!(result.correlation_stats.avg_events_per_correlation, 2.5); // Average of 2.0, 3.0
        assert_eq!(result.correlation_stats.events_with_causation, 20); // Sum
        assert_eq!(result.correlation_stats.avg_causation_depth, 2.0); // Average of 1.5, 2.5
    }

    #[test]
    fn test_aggregate_metrics_processing_stats_none_times() {
        let mut metrics1 = EventMetrics::default();
        metrics1.processing_times.min_time = None;
        metrics1.processing_times.max_time = Some(Duration::from_millis(10));

        let mut metrics2 = EventMetrics::default();
        metrics2.processing_times.min_time = Some(Duration::from_millis(5));
        metrics2.processing_times.max_time = None;

        let result = aggregate_metrics(&[metrics1, metrics2]);

        // Should pick the Some values
        assert_eq!(result.processing_times.min_time, Some(Duration::from_millis(5)));
        assert_eq!(result.processing_times.max_time, Some(Duration::from_millis(10)));
    }

    #[test]
    fn test_aggregate_metrics_events_per_second() {
        let mut metrics1 = EventMetrics::default();
        metrics1.processing_times.events_per_second = 10.0;

        let mut metrics2 = EventMetrics::default();
        metrics2.processing_times.events_per_second = 20.0;

        let result = aggregate_metrics(&[metrics1, metrics2]);

        // Average events per second
        assert_eq!(result.processing_times.events_per_second, 15.0);
    }
}