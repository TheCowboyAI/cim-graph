//! Event replay optimization strategies

use crate::events::GraphEvent;
use crate::core::{build_projection, GraphAggregateProjection};
use crate::error::Result;
use std::collections::{HashMap, BTreeMap};
use uuid::Uuid;
use std::sync::{Arc, RwLock};

/// Strategy for replaying events
#[derive(Debug, Clone, Copy)]
pub enum ReplayStrategy {
    /// Replay all events from the beginning
    Full,
    /// Replay from a snapshot point
    FromSnapshot {
        /// Sequence number to start replay from
        sequence: u64
    },
    /// Replay only recent events
    Recent {
        /// Maximum number of recent events to replay
        max_events: usize
    },
    /// Replay events within a time window
    TimeWindow {
        /// Time window in seconds
        seconds: u64
    },
}

/// Snapshot of projection state at a point in time
#[derive(Debug, Clone)]
pub struct ProjectionSnapshot {
    /// Sequence number of last event in snapshot
    pub sequence: u64,
    /// Aggregate ID
    pub aggregate_id: Uuid,
    /// Serialized projection state
    pub state: Vec<u8>,
    /// When snapshot was taken
    pub timestamp: std::time::SystemTime,
}

/// Replay optimizer that manages snapshots and efficient replay
#[derive(Debug)]
pub struct ReplayOptimizer {
    /// Snapshots by aggregate ID
    snapshots: Arc<RwLock<HashMap<Uuid, Vec<ProjectionSnapshot>>>>,
    /// Configuration
    config: ReplayConfig,
}

/// Configuration for replay optimization
#[derive(Debug, Clone)]
pub struct ReplayConfig {
    /// How often to take snapshots (every N events)
    pub snapshot_interval: u64,
    /// Maximum snapshots to keep per aggregate
    pub max_snapshots: usize,
    /// Enable parallel replay for multiple aggregates
    pub parallel_replay: bool,
}

impl Default for ReplayConfig {
    fn default() -> Self {
        Self {
            snapshot_interval: 100,
            max_snapshots: 5,
            parallel_replay: true,
        }
    }
}

impl ReplayOptimizer {
    /// Create a new replay optimizer
    pub fn new(config: ReplayConfig) -> Self {
        Self {
            snapshots: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    /// Determine optimal replay strategy for an aggregate
    pub fn determine_strategy(
        &self,
        aggregate_id: Uuid,
        current_sequence: u64,
        total_events: usize,
    ) -> ReplayStrategy {
        let snapshots = self.snapshots.read().unwrap();
        
        if let Some(aggregate_snapshots) = snapshots.get(&aggregate_id) {
            if let Some(latest_snapshot) = aggregate_snapshots.last() {
                let events_since_snapshot = current_sequence - latest_snapshot.sequence;
                
                // If we have a recent snapshot and few events since, use it
                if events_since_snapshot < self.config.snapshot_interval * 2 {
                    return ReplayStrategy::FromSnapshot {
                        sequence: latest_snapshot.sequence,
                    };
                }
            }
        }
        
        // For small event counts, just replay all
        if total_events < 1000 {
            return ReplayStrategy::Full;
        }
        
        // For large event counts without snapshots, replay recent
        ReplayStrategy::Recent {
            max_events: 1000,
        }
    }
    
    /// Replay events with optimal strategy
    pub fn replay_events(
        &self,
        events: Vec<(GraphEvent, u64)>,
        strategy: ReplayStrategy,
    ) -> Result<GraphAggregateProjection> {
        let filtered_events = match strategy {
            ReplayStrategy::Full => events,
            
            ReplayStrategy::FromSnapshot { sequence } => {
                events.into_iter()
                    .filter(|(_, seq)| *seq > sequence)
                    .collect()
            }
            
            ReplayStrategy::Recent { max_events } => {
                let skip = events.len().saturating_sub(max_events);
                events.into_iter().skip(skip).collect()
            }
            
            ReplayStrategy::TimeWindow { seconds: _window_seconds } => {
                // Would filter by timestamp if events had them
                events
            }
        };
        
        Ok(build_projection(filtered_events))
    }
    
    /// Create a snapshot of current projection state
    pub fn create_snapshot(
        &self,
        projection: &GraphAggregateProjection,
        sequence: u64,
    ) -> Result<()> {
        let snapshot = ProjectionSnapshot {
            sequence,
            aggregate_id: projection.aggregate_id,
            state: bincode::serialize(projection)
                .map_err(|e| crate::error::GraphError::SerializationError(e.to_string()))?,
            timestamp: std::time::SystemTime::now(),
        };
        
        let mut snapshots = self.snapshots.write().unwrap();
        let aggregate_snapshots = snapshots
            .entry(projection.aggregate_id)
            .or_insert_with(Vec::new);
        
        aggregate_snapshots.push(snapshot);
        
        // Keep only max_snapshots most recent
        if aggregate_snapshots.len() > self.config.max_snapshots {
            let remove_count = aggregate_snapshots.len() - self.config.max_snapshots;
            aggregate_snapshots.drain(0..remove_count);
        }
        
        Ok(())
    }
    
    /// Check if a snapshot should be created
    pub fn should_snapshot(&self, current_sequence: u64, aggregate_id: Uuid) -> bool {
        if current_sequence % self.config.snapshot_interval != 0 {
            return false;
        }
        
        let snapshots = self.snapshots.read().unwrap();
        if let Some(aggregate_snapshots) = snapshots.get(&aggregate_id) {
            if let Some(latest) = aggregate_snapshots.last() {
                // Don't snapshot if we recently did
                return current_sequence > latest.sequence + self.config.snapshot_interval / 2;
            }
        }
        
        true
    }
    
    /// Get snapshot statistics
    pub fn snapshot_stats(&self) -> SnapshotStats {
        let snapshots = self.snapshots.read().unwrap();
        
        let total_snapshots: usize = snapshots.values().map(|v| v.len()).sum();
        let total_size: usize = snapshots.values()
            .flat_map(|v| v.iter())
            .map(|s| s.state.len())
            .sum();
        
        SnapshotStats {
            aggregate_count: snapshots.len(),
            total_snapshots,
            total_size_bytes: total_size,
            avg_snapshot_size: if total_snapshots > 0 {
                total_size / total_snapshots
            } else {
                0
            },
        }
    }
}

/// Snapshot statistics for monitoring
#[derive(Debug, Clone)]
pub struct SnapshotStats {
    /// Number of unique aggregates with snapshots
    pub aggregate_count: usize,
    /// Total number of snapshots across all aggregates
    pub total_snapshots: usize,
    /// Total size of all snapshots in bytes
    pub total_size_bytes: usize,
    /// Average size of a snapshot in bytes
    pub avg_snapshot_size: usize,
}

/// Parallel event replayer for multiple aggregates
#[derive(Debug)]
pub struct ParallelReplayer {
    _thread_count: usize,
}

impl ParallelReplayer {
    /// Create a new parallel replayer
    pub fn new(thread_count: usize) -> Self {
        Self {
            _thread_count: thread_count.max(1),
        }
    }
    
    /// Replay events for multiple aggregates in parallel
    pub fn replay_multiple(
        &self,
        events_by_aggregate: HashMap<Uuid, Vec<(GraphEvent, u64)>>,
    ) -> Result<HashMap<Uuid, GraphAggregateProjection>> {
        use rayon::prelude::*;
        
        let results: Vec<(Uuid, Result<GraphAggregateProjection>)> = events_by_aggregate
            .into_par_iter()
            .map(|(aggregate_id, events)| {
                let projection = build_projection(events);
                (aggregate_id, Ok(projection))
            })
            .collect();
        
        let mut projections = HashMap::new();
        for (id, result) in results {
            projections.insert(id, result?);
        }
        
        Ok(projections)
    }
}

/// Event index for fast replay queries
#[derive(Debug)]
pub struct EventIndex {
    /// Events by aggregate ID and sequence
    by_aggregate: BTreeMap<(Uuid, u64), usize>,
    /// Events by correlation ID
    by_correlation: HashMap<Uuid, Vec<usize>>,
    /// Events by causation ID
    _by_causation: HashMap<Uuid, Vec<usize>>,
}

impl EventIndex {
    /// Build index from events
    pub fn build(events: &[(GraphEvent, u64)]) -> Self {
        let mut by_aggregate = BTreeMap::new();
        let mut by_correlation = HashMap::new();
        let mut by_causation = HashMap::new();
        
        for (idx, (event, seq)) in events.iter().enumerate() {
            by_aggregate.insert((event.aggregate_id, *seq), idx);
            
            by_correlation
                .entry(event.correlation_id)
                .or_insert_with(Vec::new)
                .push(idx);
            
            if let Some(causation_id) = event.causation_id {
                by_causation
                    .entry(causation_id)
                    .or_insert_with(Vec::new)
                    .push(idx);
            }
        }
        
        Self {
            by_aggregate,
            by_correlation,
            _by_causation: by_causation,
        }
    }
    
    /// Get events for an aggregate after a sequence number
    pub fn events_after(&self, aggregate_id: Uuid, after_sequence: u64) -> Vec<usize> {
        self.by_aggregate
            .range((aggregate_id, after_sequence + 1)..)
            .take_while(|((id, _), _)| *id == aggregate_id)
            .map(|(_, idx)| *idx)
            .collect()
    }
    
    /// Get events for a correlation ID
    pub fn events_for_correlation(&self, correlation_id: Uuid) -> Option<&[usize]> {
        self.by_correlation.get(&correlation_id).map(|v| v.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{EventPayload, IpldPayload};
    
    fn create_test_events(aggregate_id: Uuid, count: usize) -> Vec<(GraphEvent, u64)> {
        (0..count).map(|i| {
            let event = GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id,
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Ipld(IpldPayload::CidAdded {
                    cid: format!("Qm{}", i),
                    codec: "dag-cbor".to_string(),
                    size: 100,
                    data: serde_json::json!({ "index": i }),
                }),
            };
            (event, (i + 1) as u64)
        }).collect()
    }
    
    #[test]
    fn test_replay_strategy_selection() {
        let optimizer = ReplayOptimizer::new(ReplayConfig::default());
        let aggregate_id = Uuid::new_v4();
        
        // Small event count should use full replay
        let strategy = optimizer.determine_strategy(aggregate_id, 50, 50);
        assert!(matches!(strategy, ReplayStrategy::Full));
        
        // Large event count without snapshots should use recent
        let strategy = optimizer.determine_strategy(aggregate_id, 5000, 5000);
        assert!(matches!(strategy, ReplayStrategy::Recent { .. }));
    }
    
    #[test]
    fn test_event_index() {
        let aggregate_id = Uuid::new_v4();
        let events = create_test_events(aggregate_id, 10);

        let index = EventIndex::build(&events);

        // Test getting events after sequence
        let after_5 = index.events_after(aggregate_id, 5);
        assert_eq!(after_5.len(), 5); // Events 6-10
        assert_eq!(after_5[0], 5); // Index 5 is sequence 6
    }

    // ========== Additional Coverage Tests ==========

    #[test]
    fn test_replay_config_default() {
        let config = ReplayConfig::default();
        assert_eq!(config.snapshot_interval, 100);
        assert_eq!(config.max_snapshots, 5);
        assert!(config.parallel_replay);
    }

    #[test]
    fn test_replay_optimizer_new() {
        let config = ReplayConfig {
            snapshot_interval: 50,
            max_snapshots: 3,
            parallel_replay: false,
        };
        let optimizer = ReplayOptimizer::new(config);

        let debug_str = format!("{:?}", optimizer);
        assert!(debug_str.contains("ReplayOptimizer"));
    }

    #[test]
    fn test_determine_strategy_full_for_small() {
        let optimizer = ReplayOptimizer::new(ReplayConfig::default());
        let aggregate_id = Uuid::new_v4();

        // Small number of events should use full replay
        let strategy = optimizer.determine_strategy(aggregate_id, 50, 50);
        assert!(matches!(strategy, ReplayStrategy::Full));

        let strategy = optimizer.determine_strategy(aggregate_id, 999, 999);
        assert!(matches!(strategy, ReplayStrategy::Full));
    }

    #[test]
    fn test_determine_strategy_recent_for_large() {
        let optimizer = ReplayOptimizer::new(ReplayConfig::default());
        let aggregate_id = Uuid::new_v4();

        // Large number of events without snapshots should use recent replay
        let strategy = optimizer.determine_strategy(aggregate_id, 5000, 5000);
        assert!(matches!(strategy, ReplayStrategy::Recent { max_events: 1000 }));
    }

    #[test]
    fn test_replay_events_full_strategy() {
        let optimizer = ReplayOptimizer::new(ReplayConfig::default());
        let aggregate_id = Uuid::new_v4();
        let events = create_test_events(aggregate_id, 10);

        let projection = optimizer.replay_events(events.clone(), ReplayStrategy::Full).unwrap();

        assert_eq!(projection.aggregate_id, aggregate_id);
    }

    #[test]
    fn test_replay_events_from_snapshot_strategy() {
        let optimizer = ReplayOptimizer::new(ReplayConfig::default());
        let aggregate_id = Uuid::new_v4();
        let events = create_test_events(aggregate_id, 10);

        // Replay from sequence 5 (should only include events 6-10)
        let projection = optimizer.replay_events(
            events.clone(),
            ReplayStrategy::FromSnapshot { sequence: 5 }
        ).unwrap();

        assert_eq!(projection.aggregate_id, aggregate_id);
    }

    #[test]
    fn test_replay_events_recent_strategy() {
        let optimizer = ReplayOptimizer::new(ReplayConfig::default());
        let aggregate_id = Uuid::new_v4();
        let events = create_test_events(aggregate_id, 20);

        // Only replay last 5 events
        let projection = optimizer.replay_events(
            events.clone(),
            ReplayStrategy::Recent { max_events: 5 }
        ).unwrap();

        assert_eq!(projection.aggregate_id, aggregate_id);
    }

    #[test]
    fn test_replay_events_time_window_strategy() {
        let optimizer = ReplayOptimizer::new(ReplayConfig::default());
        let aggregate_id = Uuid::new_v4();
        let events = create_test_events(aggregate_id, 10);

        // Time window strategy (currently just returns all events)
        let projection = optimizer.replay_events(
            events.clone(),
            ReplayStrategy::TimeWindow { seconds: 3600 }
        ).unwrap();

        assert_eq!(projection.aggregate_id, aggregate_id);
    }

    #[test]
    fn test_create_snapshot() {
        let optimizer = ReplayOptimizer::new(ReplayConfig::default());
        let aggregate_id = Uuid::new_v4();
        let events = create_test_events(aggregate_id, 10);

        // Build projection first
        let projection = build_projection(events);

        // Create snapshot
        let result = optimizer.create_snapshot(&projection, 10);
        assert!(result.is_ok());

        // Verify snapshot was created
        let stats = optimizer.snapshot_stats();
        assert_eq!(stats.aggregate_count, 1);
        assert_eq!(stats.total_snapshots, 1);
    }

    #[test]
    fn test_create_multiple_snapshots() {
        let config = ReplayConfig {
            snapshot_interval: 5,
            max_snapshots: 3,
            parallel_replay: true,
        };
        let optimizer = ReplayOptimizer::new(config);
        let aggregate_id = Uuid::new_v4();

        // Create multiple snapshots
        for i in 1..=5 {
            let events = create_test_events(aggregate_id, i * 5);
            let projection = build_projection(events);
            optimizer.create_snapshot(&projection, (i * 5) as u64).unwrap();
        }

        // Only max_snapshots (3) should be kept
        let stats = optimizer.snapshot_stats();
        assert_eq!(stats.total_snapshots, 3);
    }

    #[test]
    fn test_should_snapshot() {
        let config = ReplayConfig {
            snapshot_interval: 10,
            max_snapshots: 5,
            parallel_replay: true,
        };
        let optimizer = ReplayOptimizer::new(config);
        let aggregate_id = Uuid::new_v4();

        // Should snapshot at interval boundaries
        assert!(optimizer.should_snapshot(10, aggregate_id));
        assert!(optimizer.should_snapshot(20, aggregate_id));
        assert!(optimizer.should_snapshot(100, aggregate_id));

        // Should not snapshot at non-interval points
        assert!(!optimizer.should_snapshot(5, aggregate_id));
        assert!(!optimizer.should_snapshot(15, aggregate_id));
        assert!(!optimizer.should_snapshot(99, aggregate_id));
    }

    #[test]
    fn test_should_snapshot_after_recent() {
        let config = ReplayConfig {
            snapshot_interval: 10,
            max_snapshots: 5,
            parallel_replay: true,
        };
        let optimizer = ReplayOptimizer::new(config);
        let aggregate_id = Uuid::new_v4();

        // Create a snapshot at sequence 10
        let events = create_test_events(aggregate_id, 10);
        let projection = build_projection(events);
        optimizer.create_snapshot(&projection, 10).unwrap();

        // Should not snapshot at same sequence
        assert!(!optimizer.should_snapshot(10, aggregate_id)); // Same sequence: 10 > 10 + 5 = 15 is false

        // Should snapshot at 20 since 20 > 10 + 5 = 15
        assert!(optimizer.should_snapshot(20, aggregate_id));

        // Update snapshot to 20
        let events = create_test_events(aggregate_id, 20);
        let projection = build_projection(events);
        optimizer.create_snapshot(&projection, 20).unwrap();

        // Now at sequence 30: 30 > 20 + 5 = 25, should be true
        assert!(optimizer.should_snapshot(30, aggregate_id));
    }

    #[test]
    fn test_snapshot_stats_empty() {
        let optimizer = ReplayOptimizer::new(ReplayConfig::default());
        let stats = optimizer.snapshot_stats();

        assert_eq!(stats.aggregate_count, 0);
        assert_eq!(stats.total_snapshots, 0);
        assert_eq!(stats.total_size_bytes, 0);
        assert_eq!(stats.avg_snapshot_size, 0);
    }

    #[test]
    fn test_snapshot_stats_with_snapshots() {
        let optimizer = ReplayOptimizer::new(ReplayConfig::default());
        let aggregate_id = Uuid::new_v4();

        let events = create_test_events(aggregate_id, 10);
        let projection = build_projection(events);
        optimizer.create_snapshot(&projection, 10).unwrap();

        let stats = optimizer.snapshot_stats();
        assert_eq!(stats.aggregate_count, 1);
        assert_eq!(stats.total_snapshots, 1);
        assert!(stats.total_size_bytes > 0);
        assert!(stats.avg_snapshot_size > 0);
    }

    #[test]
    fn test_parallel_replayer() {
        let replayer = ParallelReplayer::new(4);

        let debug_str = format!("{:?}", replayer);
        assert!(debug_str.contains("ParallelReplayer"));
    }

    #[test]
    fn test_parallel_replayer_with_zero_threads() {
        // Should clamp to at least 1 thread
        let replayer = ParallelReplayer::new(0);
        let debug_str = format!("{:?}", replayer);
        assert!(debug_str.contains("1")); // thread_count should be 1
    }

    #[test]
    fn test_parallel_replay_multiple() {
        let replayer = ParallelReplayer::new(2);

        let mut events_by_aggregate = HashMap::new();

        // Create events for 3 different aggregates
        for _ in 0..3 {
            let aggregate_id = Uuid::new_v4();
            let events = create_test_events(aggregate_id, 5);
            events_by_aggregate.insert(aggregate_id, events);
        }

        let projections = replayer.replay_multiple(events_by_aggregate.clone()).unwrap();

        assert_eq!(projections.len(), 3);

        for (aggregate_id, projection) in &projections {
            assert_eq!(*aggregate_id, projection.aggregate_id);
        }
    }

    #[test]
    fn test_parallel_replay_empty() {
        let replayer = ParallelReplayer::new(2);
        let events_by_aggregate = HashMap::new();

        let projections = replayer.replay_multiple(events_by_aggregate).unwrap();
        assert!(projections.is_empty());
    }

    #[test]
    fn test_parallel_replay_single_aggregate() {
        let replayer = ParallelReplayer::new(2);

        let aggregate_id = Uuid::new_v4();
        let events = create_test_events(aggregate_id, 10);

        let mut events_by_aggregate = HashMap::new();
        events_by_aggregate.insert(aggregate_id, events);

        let projections = replayer.replay_multiple(events_by_aggregate).unwrap();

        assert_eq!(projections.len(), 1);
        assert!(projections.contains_key(&aggregate_id));
    }

    #[test]
    fn test_event_index_empty() {
        let events: Vec<(GraphEvent, u64)> = vec![];
        let index = EventIndex::build(&events);

        let result = index.events_after(Uuid::new_v4(), 0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_event_index_correlation() {
        let aggregate_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();

        let mut events = vec![];
        for i in 0..5 {
            let event = GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id,
                correlation_id, // Same correlation for all
                causation_id: None,
                payload: EventPayload::Ipld(IpldPayload::CidAdded {
                    cid: format!("Qm{}", i),
                    codec: "dag-cbor".to_string(),
                    size: 100,
                    data: serde_json::json!({ "index": i }),
                }),
            };
            events.push((event, (i + 1) as u64));
        }

        let index = EventIndex::build(&events);

        // All events should be found by correlation ID
        let correlated = index.events_for_correlation(correlation_id).unwrap();
        assert_eq!(correlated.len(), 5);
    }

    #[test]
    fn test_event_index_correlation_not_found() {
        let aggregate_id = Uuid::new_v4();
        let events = create_test_events(aggregate_id, 5);
        let index = EventIndex::build(&events);

        let result = index.events_for_correlation(Uuid::new_v4());
        assert!(result.is_none());
    }

    #[test]
    fn test_event_index_with_causation() {
        let aggregate_id = Uuid::new_v4();
        let causation_id = Uuid::new_v4();

        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id: Uuid::new_v4(),
            causation_id: Some(causation_id),
            payload: EventPayload::Ipld(IpldPayload::CidAdded {
                cid: "QmTest".to_string(),
                codec: "dag-cbor".to_string(),
                size: 100,
                data: serde_json::json!({}),
            }),
        };

        let events = vec![(event, 1)];
        let _index = EventIndex::build(&events);

        // Just verify the build doesn't panic with causation IDs
    }

    #[test]
    fn test_replay_strategy_debug() {
        let strategies = vec![
            ReplayStrategy::Full,
            ReplayStrategy::FromSnapshot { sequence: 100 },
            ReplayStrategy::Recent { max_events: 50 },
            ReplayStrategy::TimeWindow { seconds: 3600 },
        ];

        for strategy in strategies {
            let debug_str = format!("{:?}", strategy);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_projection_snapshot_debug() {
        let snapshot = ProjectionSnapshot {
            sequence: 100,
            aggregate_id: Uuid::new_v4(),
            state: vec![1, 2, 3, 4],
            timestamp: std::time::SystemTime::now(),
        };

        let debug_str = format!("{:?}", snapshot);
        assert!(debug_str.contains("ProjectionSnapshot"));
        assert!(debug_str.contains("100"));
    }

    #[test]
    fn test_snapshot_stats_debug() {
        let stats = SnapshotStats {
            aggregate_count: 5,
            total_snapshots: 10,
            total_size_bytes: 1024,
            avg_snapshot_size: 102,
        };

        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("SnapshotStats"));
    }

    #[test]
    fn test_event_index_events_after_boundary() {
        let aggregate_id = Uuid::new_v4();
        let events = create_test_events(aggregate_id, 10);
        let index = EventIndex::build(&events);

        // Events after sequence 0 should return all
        let after_0 = index.events_after(aggregate_id, 0);
        assert_eq!(after_0.len(), 10);

        // Events after max sequence should return none
        let after_max = index.events_after(aggregate_id, 10);
        assert!(after_max.is_empty());
    }

    #[test]
    fn test_determine_strategy_with_snapshot() {
        let config = ReplayConfig {
            snapshot_interval: 10,
            max_snapshots: 5,
            parallel_replay: true,
        };
        let optimizer = ReplayOptimizer::new(config);
        let aggregate_id = Uuid::new_v4();

        // Create a snapshot
        let events = create_test_events(aggregate_id, 50);
        let projection = build_projection(events);
        optimizer.create_snapshot(&projection, 50).unwrap();

        // Determine strategy when close to snapshot
        let strategy = optimizer.determine_strategy(aggregate_id, 55, 55);

        // Should use FromSnapshot since we have a recent snapshot
        assert!(matches!(strategy, ReplayStrategy::FromSnapshot { sequence: 50 }));
    }
}