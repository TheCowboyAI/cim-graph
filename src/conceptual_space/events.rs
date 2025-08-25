/*
 * Copyright (c) 2025 - Cowboy AI, LLC.
 */

//! Event sourcing for Conceptual Spaces
//!
//! All state changes in conceptual spaces are driven by events,
//! ensuring immutability and auditability.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Events that can modify conceptual spaces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConceptualSpaceEvent {
    /// Create new conceptual space
    SpaceCreated {
        space_id: String,
        initial_radius: f64,
        timestamp: u64,
        creator: String,
    },
    /// Add concept to space
    ConceptAdded {
        concept_id: String,
        properties: HashMap<String, serde_json::Value>,
        timestamp: u64,
    },
    /// Remove concept from space
    ConceptRemoved {
        concept_id: String,
        timestamp: u64,
    },
    /// Add edge between concepts
    EdgeAdded {
        edge_id: String,
        from_node: String,
        to_node: String,
        edge_type: String,
        properties: HashMap<String, serde_json::Value>,
        timestamp: u64,
    },
    /// Remove edge between concepts
    EdgeRemoved {
        edge_id: String,
        timestamp: u64,
    },
    /// Topology changed
    TopologyChanged {
        space_id: String,
        new_topology: String,
        timestamp: u64,
    },
    /// Tessellation updated
    TessellationUpdated {
        space_id: String,
        cell_count: usize,
        timestamp: u64,
    },
    /// Pattern emerged
    PatternEmerged {
        pattern_id: String,
        pattern_type: String,
        involved_concepts: Vec<String>,
        timestamp: u64,
    },
    /// Pattern dissolved
    PatternDissolved {
        pattern_id: String,
        timestamp: u64,
    },
    /// Quality dimension added
    QualityDimensionAdded {
        dimension_id: String,
        dimension_type: String,
        origin_concept: String,
        target_concept: String,
        timestamp: u64,
    },
    /// Quality dimension removed
    QualityDimensionRemoved {
        dimension_id: String,
        timestamp: u64,
    },
}

impl ConceptualSpaceEvent {
    /// Get the timestamp of the event
    pub fn timestamp(&self) -> u64 {
        match self {
            Self::SpaceCreated { timestamp, .. } |
            Self::ConceptAdded { timestamp, .. } |
            Self::ConceptRemoved { timestamp, .. } |
            Self::EdgeAdded { timestamp, .. } |
            Self::EdgeRemoved { timestamp, .. } |
            Self::TopologyChanged { timestamp, .. } |
            Self::TessellationUpdated { timestamp, .. } |
            Self::PatternEmerged { timestamp, .. } |
            Self::PatternDissolved { timestamp, .. } |
            Self::QualityDimensionAdded { timestamp, .. } |
            Self::QualityDimensionRemoved { timestamp, .. } => *timestamp,
        }
    }

    /// Get a human-readable description of the event
    pub fn description(&self) -> String {
        match self {
            Self::SpaceCreated { space_id, .. } => {
                format!("Conceptual space '{}' created", space_id)
            }
            Self::ConceptAdded { concept_id, .. } => {
                format!("Concept '{}' added to space", concept_id)
            }
            Self::ConceptRemoved { concept_id, .. } => {
                format!("Concept '{}' removed from space", concept_id)
            }
            Self::EdgeAdded { from_node, to_node, edge_type, .. } => {
                format!("Edge of type '{}' added from '{}' to '{}'", edge_type, from_node, to_node)
            }
            Self::EdgeRemoved { edge_id, .. } => {
                format!("Edge '{}' removed", edge_id)
            }
            Self::TopologyChanged { new_topology, .. } => {
                format!("Topology changed to {}", new_topology)
            }
            Self::TessellationUpdated { cell_count, .. } => {
                format!("Tessellation updated with {} cells", cell_count)
            }
            Self::PatternEmerged { pattern_type, involved_concepts, .. } => {
                format!("Pattern of type '{}' emerged involving {} concepts", 
                    pattern_type, involved_concepts.len())
            }
            Self::PatternDissolved { pattern_id, .. } => {
                format!("Pattern '{}' dissolved", pattern_id)
            }
            Self::QualityDimensionAdded { dimension_type, origin_concept, target_concept, .. } => {
                format!("Quality dimension of type '{}' added from '{}' to '{}'", 
                    dimension_type, origin_concept, target_concept)
            }
            Self::QualityDimensionRemoved { dimension_id, .. } => {
                format!("Quality dimension '{}' removed", dimension_id)
            }
        }
    }

    /// Check if this event affects a specific concept
    pub fn affects_concept(&self, concept_id: &str) -> bool {
        match self {
            Self::ConceptAdded { concept_id: cid, .. } |
            Self::ConceptRemoved { concept_id: cid, .. } => cid == concept_id,
            Self::EdgeAdded { from_node, to_node, .. } => {
                from_node == concept_id || to_node == concept_id
            }
            Self::PatternEmerged { involved_concepts, .. } => {
                involved_concepts.contains(&concept_id.to_string())
            }
            Self::QualityDimensionAdded { origin_concept, target_concept, .. } => {
                origin_concept == concept_id || target_concept == concept_id
            }
            _ => false,
        }
    }

    /// Create a timestamp for now
    pub fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}

/// Event store for managing conceptual space events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStore {
    events: Vec<ConceptualSpaceEvent>,
    version: u64,
}

impl EventStore {
    /// Create a new event store
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            version: 0,
        }
    }

    /// Add an event to the store
    pub fn append(&mut self, event: ConceptualSpaceEvent) {
        self.events.push(event);
        self.version += 1;
    }

    /// Get all events
    pub fn all_events(&self) -> &[ConceptualSpaceEvent] {
        &self.events
    }

    /// Get events since a specific version
    pub fn events_since(&self, version: u64) -> Vec<ConceptualSpaceEvent> {
        self.events
            .iter()
            .skip(version as usize)
            .cloned()
            .collect()
    }

    /// Get current version
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Find events affecting a specific concept
    pub fn events_for_concept(&self, concept_id: &str) -> Vec<ConceptualSpaceEvent> {
        self.events
            .iter()
            .filter(|e| e.affects_concept(concept_id))
            .cloned()
            .collect()
    }

    /// Find events within a time range
    pub fn events_in_range(&self, start: u64, end: u64) -> Vec<ConceptualSpaceEvent> {
        self.events
            .iter()
            .filter(|e| {
                let ts = e.timestamp();
                ts >= start && ts <= end
            })
            .cloned()
            .collect()
    }
}

impl Default for EventStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_store() {
        let mut store = EventStore::new();
        assert_eq!(store.version(), 0);

        // Add some events
        store.append(ConceptualSpaceEvent::SpaceCreated {
            space_id: "test".to_string(),
            initial_radius: 1.0,
            timestamp: 1000,
            creator: "test_user".to_string(),
        });

        store.append(ConceptualSpaceEvent::ConceptAdded {
            concept_id: "cat".to_string(),
            properties: HashMap::new(),
            timestamp: 2000,
        });

        assert_eq!(store.version(), 2);
        assert_eq!(store.all_events().len(), 2);

        // Test events_since
        let recent = store.events_since(1);
        assert_eq!(recent.len(), 1);
        assert!(matches!(recent[0], ConceptualSpaceEvent::ConceptAdded { .. }));

        // Test events_for_concept
        let cat_events = store.events_for_concept("cat");
        assert_eq!(cat_events.len(), 1);

        // Test events_in_range
        let range_events = store.events_in_range(1500, 2500);
        assert_eq!(range_events.len(), 1);
    }

    #[test]
    fn test_event_affects_concept() {
        let event = ConceptualSpaceEvent::EdgeAdded {
            edge_id: "edge1".to_string(),
            from_node: "cat".to_string(),
            to_node: "dog".to_string(),
            edge_type: "similar".to_string(),
            properties: HashMap::new(),
            timestamp: 1000,
        };

        assert!(event.affects_concept("cat"));
        assert!(event.affects_concept("dog"));
        assert!(!event.affects_concept("bird"));
    }
}