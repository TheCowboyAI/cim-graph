//! Event batching for efficient processing

use crate::events::GraphEvent;
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// A batch of events ready for processing
#[derive(Debug, Clone)]
pub struct EventBatch {
    /// Events in this batch
    pub events: Vec<GraphEvent>,
    /// Batch ID for tracking
    pub batch_id: Uuid,
    /// When the batch was created
    pub created_at: Instant,
    /// Total size in bytes (approximate)
    pub size_bytes: usize,
}

impl EventBatch {
    /// Create a new batch from events
    pub fn new(events: Vec<GraphEvent>) -> Self {
        let size_bytes = events.iter()
            .map(|e| std::mem::size_of::<GraphEvent>() + 
                     serde_json::to_vec(e).map(|v| v.len()).unwrap_or(0))
            .sum();
        
        Self {
            events,
            batch_id: Uuid::new_v4(),
            created_at: Instant::now(),
            size_bytes,
        }
    }
    
    /// Get events by aggregate ID
    pub fn events_for_aggregate(&self, aggregate_id: Uuid) -> Vec<&GraphEvent> {
        self.events.iter()
            .filter(|e| e.aggregate_id == aggregate_id)
            .collect()
    }
    
    /// Get events by correlation ID
    pub fn events_for_correlation(&self, correlation_id: Uuid) -> Vec<&GraphEvent> {
        self.events.iter()
            .filter(|e| e.correlation_id == correlation_id)
            .collect()
    }
    
    /// Split batch by aggregate ID
    pub fn split_by_aggregate(self) -> Vec<EventBatch> {
        use std::collections::HashMap;
        
        let mut by_aggregate: HashMap<Uuid, Vec<GraphEvent>> = HashMap::new();
        
        for event in self.events {
            by_aggregate.entry(event.aggregate_id)
                .or_insert_with(Vec::new)
                .push(event);
        }
        
        by_aggregate.into_iter()
            .map(|(_, events)| EventBatch::new(events))
            .collect()
    }
}

/// Configuration for event batching
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum number of events per batch
    pub max_events: usize,
    /// Maximum batch size in bytes
    pub max_bytes: usize,
    /// Maximum time to wait before flushing
    pub max_wait: Duration,
    /// Whether to preserve event order within aggregates
    pub preserve_aggregate_order: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_events: 1000,
            max_bytes: 1024 * 1024, // 1MB
            max_wait: Duration::from_millis(100),
            preserve_aggregate_order: true,
        }
    }
}

/// Event batcher that groups events into efficient batches
#[derive(Debug)]
pub struct EventBatcher {
    config: BatchConfig,
    pending: VecDeque<GraphEvent>,
    pending_size: usize,
    batch_started: Option<Instant>,
}

impl EventBatcher {
    /// Create a new batcher with config
    pub fn new(config: BatchConfig) -> Self {
        Self {
            config,
            pending: VecDeque::new(),
            pending_size: 0,
            batch_started: None,
        }
    }
    
    /// Add an event to the batcher
    pub fn add_event(&mut self, event: GraphEvent) -> Option<EventBatch> {
        let event_size = std::mem::size_of::<GraphEvent>() + 
                        serde_json::to_vec(&event).map(|v| v.len()).unwrap_or(0);
        
        // Start batch timer if needed
        if self.batch_started.is_none() {
            self.batch_started = Some(Instant::now());
        }
        
        self.pending.push_back(event);
        self.pending_size += event_size;
        
        // Check if we should flush
        if self.should_flush() {
            Some(self.flush())
        } else {
            None
        }
    }
    
    /// Check if batch should be flushed
    pub fn should_flush(&self) -> bool {
        // Check size limits
        if self.pending.len() >= self.config.max_events {
            return true;
        }
        
        if self.pending_size >= self.config.max_bytes {
            return true;
        }
        
        // Check time limit
        if let Some(started) = self.batch_started {
            if started.elapsed() >= self.config.max_wait {
                return true;
            }
        }
        
        false
    }
    
    /// Flush pending events as a batch
    pub fn flush(&mut self) -> EventBatch {
        let events: Vec<GraphEvent> = self.pending.drain(..).collect();
        self.pending_size = 0;
        self.batch_started = None;
        
        let mut batch = EventBatch::new(events);
        
        // Sort by aggregate ID then sequence if preserving order
        if self.config.preserve_aggregate_order {
            batch.events.sort_by_key(|e| (e.aggregate_id, e.event_id));
        }
        
        batch
    }
    
    /// Get pending event count
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
    
    /// Get pending size in bytes
    pub fn pending_size(&self) -> usize {
        self.pending_size
    }
}

/// Adaptive batcher that adjusts batch size based on processing performance
#[derive(Debug)]
pub struct AdaptiveBatcher {
    _base_config: BatchConfig,
    current_config: BatchConfig,
    performance_history: VecDeque<BatchPerformance>,
    batcher: EventBatcher,
}

#[derive(Debug, Clone)]
struct BatchPerformance {
    _batch_size: usize,
    _processing_time: Duration,
    events_per_second: f64,
}

impl AdaptiveBatcher {
    /// Create a new adaptive batcher
    pub fn new(config: BatchConfig) -> Self {
        Self {
            _base_config: config.clone(),
            current_config: config.clone(),
            performance_history: VecDeque::with_capacity(10),
            batcher: EventBatcher::new(config),
        }
    }
    
    /// Add event with adaptive batching
    pub fn add_event(&mut self, event: GraphEvent) -> Option<EventBatch> {
        self.batcher.add_event(event)
    }
    
    /// Record batch processing performance
    pub fn record_performance(&mut self, batch_size: usize, processing_time: Duration) {
        let events_per_second = batch_size as f64 / processing_time.as_secs_f64();
        
        let perf = BatchPerformance {
            _batch_size: batch_size,
            _processing_time: processing_time,
            events_per_second,
        };
        
        self.performance_history.push_back(perf);
        if self.performance_history.len() > 10 {
            self.performance_history.pop_front();
        }
        
        // Adjust batch size based on performance
        self.adjust_batch_size();
    }
    
    /// Adjust batch size based on recent performance
    fn adjust_batch_size(&mut self) {
        if self.performance_history.len() < 3 {
            return; // Not enough data
        }
        
        let avg_events_per_second: f64 = self.performance_history.iter()
            .map(|p| p.events_per_second)
            .sum::<f64>() / self.performance_history.len() as f64;
        
        let recent_avg: f64 = self.performance_history.iter()
            .rev()
            .take(3)
            .map(|p| p.events_per_second)
            .sum::<f64>() / 3.0;
        
        // If recent performance is better, increase batch size
        if recent_avg > avg_events_per_second * 1.1 {
            self.current_config.max_events = 
                (self.current_config.max_events as f64 * 1.2).min(10000.0) as usize;
            self.current_config.max_bytes = 
                (self.current_config.max_bytes as f64 * 1.2).min(10_000_000.0) as usize;
        } 
        // If recent performance is worse, decrease batch size
        else if recent_avg < avg_events_per_second * 0.9 {
            self.current_config.max_events = 
                (self.current_config.max_events as f64 * 0.8).max(10.0) as usize;
            self.current_config.max_bytes = 
                (self.current_config.max_bytes as f64 * 0.8).max(1024.0) as usize;
        }
        
        // Update batcher config
        self.batcher = EventBatcher::new(self.current_config.clone());
    }
    
    /// Flush any pending events
    pub fn flush(&mut self) -> Option<EventBatch> {
        if self.batcher.pending_count() > 0 {
            Some(self.batcher.flush())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{EventPayload, IpldPayload};
    
    fn create_test_event(aggregate_id: Uuid) -> GraphEvent {
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Ipld(IpldPayload::CidAdded {
                cid: "QmTest".to_string(),
                codec: "dag-cbor".to_string(),
                size: 100,
                data: serde_json::json!({}),
            }),
        }
    }
    
    #[test]
    fn test_basic_batching() {
        let config = BatchConfig {
            max_events: 5,
            ..Default::default()
        };
        
        let mut batcher = EventBatcher::new(config);
        let aggregate_id = Uuid::new_v4();
        
        // Add 4 events - shouldn't flush
        for _ in 0..4 {
            let batch = batcher.add_event(create_test_event(aggregate_id));
            assert!(batch.is_none());
        }
        
        // 5th event should trigger flush
        let batch = batcher.add_event(create_test_event(aggregate_id));
        assert!(batch.is_some());
        
        let batch = batch.unwrap();
        assert_eq!(batch.events.len(), 5);
    }
    
    #[test]
    fn test_batch_splitting() {
        let agg1 = Uuid::new_v4();
        let agg2 = Uuid::new_v4();
        
        let events = vec![
            create_test_event(agg1),
            create_test_event(agg2),
            create_test_event(agg1),
            create_test_event(agg2),
        ];
        
        let batch = EventBatch::new(events);
        let split = batch.split_by_aggregate();
        
        assert_eq!(split.len(), 2);
        assert!(split.iter().all(|b| b.events.len() == 2));
    }
}