//! Serialization support for events only
//!
//! IMPORTANT: Projections are ephemeral and should NOT be serialized.
//! Only events should be persisted. Projections are rebuilt from events.
//!
//! This module now only provides serialization for:
//! - Events (GraphEvent)
//! - Commands (GraphCommand)
//! - Event metadata
//!
//! The old graph serialization has been removed as it contradicts
//! the event-sourcing pattern.

use crate::events::{GraphEvent, GraphCommand};
use crate::error::{GraphError, Result};
use serde::{Serialize, Deserialize};
use std::path::Path;

/// Serializes events to JSON
pub fn serialize_events(events: &[GraphEvent]) -> Result<String> {
    serde_json::to_string_pretty(events)
        .map_err(|e| GraphError::SerializationError(format!("Failed to serialize events: {}", e)))
}

/// Deserializes events from JSON
pub fn deserialize_events(json: &str) -> Result<Vec<GraphEvent>> {
    serde_json::from_str(json)
        .map_err(|e| GraphError::SerializationError(format!("Failed to deserialize events: {}", e)))
}

/// Saves events to a file
pub fn save_events_to_file<P: AsRef<Path>>(events: &[GraphEvent], path: P) -> Result<()> {
    let json = serialize_events(events)?;
    std::fs::write(path, json)
        .map_err(|e| GraphError::SerializationError(format!("Failed to write file: {}", e)))
}

/// Loads events from a file
pub fn load_events_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<GraphEvent>> {
    let json = std::fs::read_to_string(path)
        .map_err(|e| GraphError::SerializationError(format!("Failed to read file: {}", e)))?;
    deserialize_events(&json)
}

/// Serializes a command to JSON
pub fn serialize_command(command: &GraphCommand) -> Result<String> {
    serde_json::to_string_pretty(command)
        .map_err(|e| GraphError::SerializationError(format!("Failed to serialize command: {}", e)))
}

/// Deserializes a command from JSON
pub fn deserialize_command(json: &str) -> Result<GraphCommand> {
    serde_json::from_str(json)
        .map_err(|e| GraphError::SerializationError(format!("Failed to deserialize command: {}", e)))
}

/// Event storage metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStorageMetadata {
    /// Storage version
    pub version: String,
    /// Number of events
    pub event_count: usize,
    /// First event timestamp
    pub first_event_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Last event timestamp
    pub last_event_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Aggregate IDs in this storage
    pub aggregate_ids: Vec<uuid::Uuid>,
}

impl EventStorageMetadata {
    /// Create metadata from a slice of events
    pub fn from_events(events: &[GraphEvent]) -> Self {
        let aggregate_ids: Vec<uuid::Uuid> = events.iter()
            .map(|e| e.aggregate_id)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        
        Self {
            version: "1.0.0".to_string(),
            event_count: events.len(),
            first_event_time: None, // Would need timestamps on events
            last_event_time: None,
            aggregate_ids,
        }
    }
}

/// Event journal for persistent storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventJournal {
    /// Metadata about the journal
    pub metadata: EventStorageMetadata,
    /// The events in order
    pub events: Vec<GraphEvent>,
}

impl EventJournal {
    /// Create a new event journal
    pub fn new(events: Vec<GraphEvent>) -> Self {
        let metadata = EventStorageMetadata::from_events(&events);
        Self {
            metadata,
            events,
        }
    }
    
    /// Save journal to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| GraphError::SerializationError(format!("Failed to serialize journal: {}", e)))?;
        std::fs::write(path, json)
            .map_err(|e| GraphError::SerializationError(format!("Failed to write file: {}", e)))
    }
    
    /// Load journal from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| GraphError::SerializationError(format!("Failed to read file: {}", e)))?;
        serde_json::from_str(&json)
            .map_err(|e| GraphError::SerializationError(format!("Failed to deserialize journal: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{EventPayload, WorkflowPayload};
    use uuid::Uuid;
    
    #[test]
    fn test_event_serialization() {
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
                workflow_id: Uuid::new_v4(),
                name: "Test".to_string(),
                version: "1.0.0".to_string(),
            }),
        };
        
        let json = serialize_events(&[event.clone()]).unwrap();
        let deserialized = deserialize_events(&json).unwrap();
        
        assert_eq!(deserialized.len(), 1);
        assert_eq!(deserialized[0].event_id, event.event_id);
    }
    
    #[test]
    fn test_event_journal() {
        let events = vec![
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: Uuid::new_v4(),
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Generic(crate::events::GenericPayload {
                    event_type: "Test".to_string(),
                    data: serde_json::json!({}),
                }),
            },
        ];
        
        let journal = EventJournal::new(events.clone());
        assert_eq!(journal.metadata.event_count, 1);
        assert_eq!(journal.events.len(), 1);
    }
}