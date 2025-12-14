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

    // ========== Event Serialization Additional Tests ==========

    #[test]
    fn test_serialize_empty_events() {
        let events: Vec<GraphEvent> = vec![];
        let json = serialize_events(&events).unwrap();

        assert_eq!(json, "[]");

        let deserialized = deserialize_events(&json).unwrap();
        assert!(deserialized.is_empty());
    }

    #[test]
    fn test_serialize_multiple_events() {
        let agg_id = Uuid::new_v4();
        let events = vec![
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: agg_id,
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
                    workflow_id: Uuid::new_v4(),
                    name: "First".to_string(),
                    version: "1.0.0".to_string(),
                }),
            },
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: agg_id,
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Workflow(WorkflowPayload::StateAdded {
                    workflow_id: Uuid::new_v4(),
                    state_id: "start".to_string(),
                    state_type: "initial".to_string(),
                }),
            },
        ];

        let json = serialize_events(&events).unwrap();
        let deserialized = deserialize_events(&json).unwrap();

        assert_eq!(deserialized.len(), 2);
        assert_eq!(deserialized[0].aggregate_id, agg_id);
        assert_eq!(deserialized[1].aggregate_id, agg_id);
    }

    #[test]
    fn test_deserialize_invalid_json() {
        let result = deserialize_events("not valid json");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, crate::error::GraphError::SerializationError(_)));
    }

    #[test]
    fn test_deserialize_wrong_structure() {
        let result = deserialize_events(r#"{"wrong": "structure"}"#);
        assert!(result.is_err());
    }

    // ========== Command Serialization Tests ==========

    #[test]
    fn test_serialize_initialize_graph_command() {
        let command = GraphCommand::InitializeGraph {
            aggregate_id: Uuid::new_v4(),
            graph_type: "workflow".to_string(),
            correlation_id: Uuid::new_v4(),
        };

        let json = serialize_command(&command).unwrap();
        let deserialized = deserialize_command(&json).unwrap();

        match deserialized {
            GraphCommand::InitializeGraph { graph_type, .. } => {
                assert_eq!(graph_type, "workflow");
            }
            _ => panic!("Wrong command type"),
        }
    }

    #[test]
    fn test_serialize_ipld_command() {
        let command = GraphCommand::Ipld {
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            command: crate::events::IpldCommand::AddCid {
                cid: "QmTest123".to_string(),
                codec: "dag-cbor".to_string(),
                size: 1024,
                data: serde_json::json!({"key": "value"}),
            },
        };

        let json = serialize_command(&command).unwrap();
        let deserialized = deserialize_command(&json).unwrap();

        match deserialized {
            GraphCommand::Ipld { command: cmd, .. } => {
                match cmd {
                    crate::events::IpldCommand::AddCid { cid, codec, size, .. } => {
                        assert_eq!(cid, "QmTest123");
                        assert_eq!(codec, "dag-cbor");
                        assert_eq!(size, 1024);
                    }
                    _ => panic!("Wrong IPLD command type"),
                }
            }
            _ => panic!("Wrong command type"),
        }
    }

    #[test]
    fn test_serialize_workflow_command() {
        let command = GraphCommand::Workflow {
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            command: crate::events::WorkflowCommand::DefineWorkflow {
                workflow_id: Uuid::new_v4(),
                name: "OrderProcessing".to_string(),
                version: "2.0.0".to_string(),
            },
        };

        let json = serialize_command(&command).unwrap();
        let deserialized = deserialize_command(&json).unwrap();

        match deserialized {
            GraphCommand::Workflow { command: cmd, .. } => {
                match cmd {
                    crate::events::WorkflowCommand::DefineWorkflow { name, version, .. } => {
                        assert_eq!(name, "OrderProcessing");
                        assert_eq!(version, "2.0.0");
                    }
                    _ => panic!("Wrong workflow command type"),
                }
            }
            _ => panic!("Wrong command type"),
        }
    }

    #[test]
    fn test_serialize_archive_graph_command() {
        let agg_id = Uuid::new_v4();
        let corr_id = Uuid::new_v4();

        let command = GraphCommand::ArchiveGraph {
            aggregate_id: agg_id,
            correlation_id: corr_id,
        };

        let json = serialize_command(&command).unwrap();
        let deserialized = deserialize_command(&json).unwrap();

        match deserialized {
            GraphCommand::ArchiveGraph { aggregate_id, correlation_id } => {
                assert_eq!(aggregate_id, agg_id);
                assert_eq!(correlation_id, corr_id);
            }
            _ => panic!("Wrong command type"),
        }
    }

    #[test]
    fn test_serialize_generic_command() {
        let command = GraphCommand::Generic {
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            command: "custom_operation".to_string(),
            data: serde_json::json!({"param1": 42, "param2": "test"}),
        };

        let json = serialize_command(&command).unwrap();
        let deserialized = deserialize_command(&json).unwrap();

        match deserialized {
            GraphCommand::Generic { command: cmd, data, .. } => {
                assert_eq!(cmd, "custom_operation");
                assert_eq!(data["param1"], 42);
                assert_eq!(data["param2"], "test");
            }
            _ => panic!("Wrong command type"),
        }
    }

    #[test]
    fn test_deserialize_invalid_command() {
        let result = deserialize_command("not valid json");
        assert!(result.is_err());
    }

    // ========== Event Payload Tests ==========

    #[test]
    fn test_serialize_ipld_payload() {
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            causation_id: Some(Uuid::new_v4()),
            payload: EventPayload::Ipld(crate::events::IpldPayload::CidAdded {
                cid: "QmContent".to_string(),
                codec: "dag-json".to_string(),
                size: 512,
                data: serde_json::json!({"content": "test data"}),
            }),
        };

        let json = serialize_events(&[event.clone()]).unwrap();
        let deserialized = deserialize_events(&json).unwrap();

        assert_eq!(deserialized.len(), 1);
        assert!(deserialized[0].causation_id.is_some());

        match &deserialized[0].payload {
            EventPayload::Ipld(crate::events::IpldPayload::CidAdded { cid, codec, size, .. }) => {
                assert_eq!(cid, "QmContent");
                assert_eq!(codec, "dag-json");
                assert_eq!(*size, 512);
            }
            _ => panic!("Wrong payload type"),
        }
    }

    #[test]
    fn test_serialize_context_payload() {
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Context(crate::events::ContextPayload::BoundedContextCreated {
                context_id: "orders".to_string(),
                name: "Order Management".to_string(),
                description: "Handles order processing".to_string(),
            }),
        };

        let json = serialize_events(&[event]).unwrap();
        let deserialized = deserialize_events(&json).unwrap();

        match &deserialized[0].payload {
            EventPayload::Context(crate::events::ContextPayload::BoundedContextCreated {
                context_id,
                name,
                description,
            }) => {
                assert_eq!(context_id, "orders");
                assert_eq!(name, "Order Management");
                assert_eq!(description, "Handles order processing");
            }
            _ => panic!("Wrong payload type"),
        }
    }

    #[test]
    fn test_serialize_concept_payload() {
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Concept(crate::events::ConceptPayload::ConceptDefined {
                concept_id: "animal".to_string(),
                name: "Animal".to_string(),
                definition: "Living organism".to_string(),
            }),
        };

        let json = serialize_events(&[event]).unwrap();
        let deserialized = deserialize_events(&json).unwrap();

        match &deserialized[0].payload {
            EventPayload::Concept(crate::events::ConceptPayload::ConceptDefined {
                concept_id,
                name,
                definition,
            }) => {
                assert_eq!(concept_id, "animal");
                assert_eq!(name, "Animal");
                assert_eq!(definition, "Living organism");
            }
            _ => panic!("Wrong payload type"),
        }
    }

    #[test]
    fn test_serialize_composed_payload() {
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Composed(crate::events::ComposedPayload::SubGraphAdded {
                subgraph_id: Uuid::new_v4(),
                graph_type: "workflow".to_string(),
                namespace: "orders".to_string(),
            }),
        };

        let json = serialize_events(&[event]).unwrap();
        let deserialized = deserialize_events(&json).unwrap();

        match &deserialized[0].payload {
            EventPayload::Composed(crate::events::ComposedPayload::SubGraphAdded {
                graph_type,
                namespace,
                ..
            }) => {
                assert_eq!(graph_type, "workflow");
                assert_eq!(namespace, "orders");
            }
            _ => panic!("Wrong payload type"),
        }
    }

    // ========== EventStorageMetadata Tests ==========

    #[test]
    fn test_metadata_from_empty_events() {
        let events: Vec<GraphEvent> = vec![];
        let metadata = EventStorageMetadata::from_events(&events);

        assert_eq!(metadata.event_count, 0);
        assert!(metadata.aggregate_ids.is_empty());
        assert_eq!(metadata.version, "1.0.0");
        assert!(metadata.first_event_time.is_none());
        assert!(metadata.last_event_time.is_none());
    }

    #[test]
    fn test_metadata_from_multiple_aggregates() {
        let agg_id1 = Uuid::new_v4();
        let agg_id2 = Uuid::new_v4();

        let events = vec![
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: agg_id1,
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Generic(crate::events::GenericPayload {
                    event_type: "Test1".to_string(),
                    data: serde_json::json!({}),
                }),
            },
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: agg_id2,
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Generic(crate::events::GenericPayload {
                    event_type: "Test2".to_string(),
                    data: serde_json::json!({}),
                }),
            },
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: agg_id1,
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Generic(crate::events::GenericPayload {
                    event_type: "Test3".to_string(),
                    data: serde_json::json!({}),
                }),
            },
        ];

        let metadata = EventStorageMetadata::from_events(&events);

        assert_eq!(metadata.event_count, 3);
        assert_eq!(metadata.aggregate_ids.len(), 2);
        assert!(metadata.aggregate_ids.contains(&agg_id1));
        assert!(metadata.aggregate_ids.contains(&agg_id2));
    }

    #[test]
    fn test_metadata_serialization() {
        let metadata = EventStorageMetadata {
            version: "1.0.0".to_string(),
            event_count: 10,
            first_event_time: Some(chrono::Utc::now()),
            last_event_time: Some(chrono::Utc::now()),
            aggregate_ids: vec![Uuid::new_v4(), Uuid::new_v4()],
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: EventStorageMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.version, "1.0.0");
        assert_eq!(deserialized.event_count, 10);
        assert!(deserialized.first_event_time.is_some());
        assert_eq!(deserialized.aggregate_ids.len(), 2);
    }

    // ========== EventJournal Additional Tests ==========

    #[test]
    fn test_journal_with_multiple_events() {
        let agg_id = Uuid::new_v4();
        let events = vec![
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: agg_id,
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
                    workflow_id: Uuid::new_v4(),
                    name: "Test".to_string(),
                    version: "1.0.0".to_string(),
                }),
            },
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: agg_id,
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Workflow(WorkflowPayload::StateAdded {
                    workflow_id: Uuid::new_v4(),
                    state_id: "init".to_string(),
                    state_type: "start".to_string(),
                }),
            },
        ];

        let journal = EventJournal::new(events);

        assert_eq!(journal.metadata.event_count, 2);
        assert_eq!(journal.events.len(), 2);
        assert_eq!(journal.metadata.aggregate_ids.len(), 1);
    }

    #[test]
    fn test_journal_serialization() {
        let events = vec![GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Generic(crate::events::GenericPayload {
                event_type: "Test".to_string(),
                data: serde_json::json!({"key": "value"}),
            }),
        }];

        let journal = EventJournal::new(events);

        let json = serde_json::to_string_pretty(&journal).unwrap();
        let deserialized: EventJournal = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.metadata.event_count, 1);
        assert_eq!(deserialized.events.len(), 1);
    }

    // ========== File I/O Tests ==========

    #[test]
    fn test_save_and_load_events_file() {
        use std::env;
        let temp_dir = env::temp_dir();
        let temp_file = temp_dir.join(format!("test_events_{}.json", Uuid::new_v4()));

        let events = vec![GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
                workflow_id: Uuid::new_v4(),
                name: "FileTest".to_string(),
                version: "1.0.0".to_string(),
            }),
        }];

        // Save
        save_events_to_file(&events, &temp_file).unwrap();

        // Load
        let loaded = load_events_from_file(&temp_file).unwrap();

        assert_eq!(loaded.len(), 1);

        // Cleanup
        let _ = std::fs::remove_file(&temp_file);
    }

    #[test]
    fn test_load_events_file_not_found() {
        let result = load_events_from_file("/nonexistent/path/events.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_journal_save_and_load_file() {
        use std::env;
        let temp_dir = env::temp_dir();
        let temp_file = temp_dir.join(format!("test_journal_{}.json", Uuid::new_v4()));

        let events = vec![
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: Uuid::new_v4(),
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Generic(crate::events::GenericPayload {
                    event_type: "JournalTest".to_string(),
                    data: serde_json::json!({}),
                }),
            },
        ];

        let journal = EventJournal::new(events);

        // Save
        journal.save_to_file(&temp_file).unwrap();

        // Load
        let loaded = EventJournal::load_from_file(&temp_file).unwrap();

        assert_eq!(loaded.metadata.event_count, 1);
        assert_eq!(loaded.events.len(), 1);

        // Cleanup
        let _ = std::fs::remove_file(&temp_file);
    }

    #[test]
    fn test_journal_load_file_not_found() {
        let result = EventJournal::load_from_file("/nonexistent/path/journal.json");
        assert!(result.is_err());
    }

    // ========== Edge Cases ==========

    #[test]
    fn test_events_with_special_characters() {
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Generic(crate::events::GenericPayload {
                event_type: "Special \"Chars\" Test\n\t".to_string(),
                data: serde_json::json!({
                    "unicode": "Hello",
                    "emoji": "test",
                    "quotes": "He said \"hello\"",
                }),
            }),
        };

        let json = serialize_events(&[event]).unwrap();
        let deserialized = deserialize_events(&json).unwrap();

        match &deserialized[0].payload {
            EventPayload::Generic(p) => {
                assert!(p.event_type.contains("Special"));
            }
            _ => panic!("Wrong payload type"),
        }
    }

    #[test]
    fn test_events_with_nested_json() {
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Generic(crate::events::GenericPayload {
                event_type: "Nested".to_string(),
                data: serde_json::json!({
                    "level1": {
                        "level2": {
                            "level3": {
                                "value": [1, 2, 3]
                            }
                        }
                    }
                }),
            }),
        };

        let json = serialize_events(&[event]).unwrap();
        let deserialized = deserialize_events(&json).unwrap();

        match &deserialized[0].payload {
            EventPayload::Generic(p) => {
                let val = &p.data["level1"]["level2"]["level3"]["value"];
                assert!(val.is_array());
            }
            _ => panic!("Wrong payload type"),
        }
    }
}