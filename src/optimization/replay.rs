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
}