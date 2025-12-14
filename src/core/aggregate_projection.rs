//! Aggregate projections - left fold of events for an aggregate
//!
//! Following DDD + ECS principles:
//! - Entities are just IDs
//! - Components are value objects attached via events
//! - Systems are functions (queries, commands, event handlers)
//! - State transitions ONLY through StateMachine

use crate::events::{GraphEvent, EventPayload};
use cim_domain::{Subject, SubjectSegment};
use uuid::Uuid;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// A graph aggregate projection - the result of folding all events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphAggregateProjection {
    /// The aggregate root ID
    pub aggregate_id: Uuid,
    
    /// Current version (sequence number from JetStream)
    pub version: u64,
    
    /// Components attached to entities (entity_id -> component_type -> data)
    pub components: HashMap<String, HashMap<String, serde_json::Value>>,
    
    /// Relationships between entities (stored as components)
    pub relationships: HashMap<String, Vec<(String, String)>>, // edge_id -> (source, target)
    
    /// Subject this aggregate belongs to (from NATS subject algebra)
    pub subject: String,
}

impl GraphAggregateProjection {
    /// Create an empty projection for an aggregate
    pub fn new(aggregate_id: Uuid, subject: String) -> Self {
        Self {
            aggregate_id,
            version: 0,
            components: HashMap::new(),
            relationships: HashMap::new(),
            subject,
        }
    }
    
    /// Apply an event to the projection (left fold)
    pub fn apply(&mut self, event: &GraphEvent, sequence: u64) {
        // Only process events for this aggregate
        if event.aggregate_id != self.aggregate_id {
            return;
        }
        
        self.version = sequence;
        
        // Process the event payload
        match &event.payload {
            EventPayload::Ipld(ipld) => {
                // IPLD events attach CID components to entities
                match ipld {
                    crate::events::IpldPayload::CidAdded { cid, codec, size, data } => {
                        // The CID is the entity ID
                        let entity_id = cid.clone();
                        
                        // Attach components
                        let mut entity_components = HashMap::new();
                        entity_components.insert("cid".to_string(), serde_json::json!(cid));
                        entity_components.insert("codec".to_string(), serde_json::json!(codec));
                        entity_components.insert("size".to_string(), serde_json::json!(size));
                        entity_components.insert("data".to_string(), data.clone());
                        
                        self.components.insert(entity_id, entity_components);
                    }
                    crate::events::IpldPayload::CidLinkAdded { cid, link_name, target_cid } => {
                        // Add relationship component
                        let edge_id = format!("{}->{}:{}", cid, target_cid, link_name);
                        self.relationships.insert(edge_id, vec![(cid.clone(), target_cid.clone())]);
                    }
                    _ => {} // Other events don't change structure
                }
            }
            EventPayload::Context(ctx) => {
                // Context events attach DDD components
                match ctx {
                    crate::events::ContextPayload::BoundedContextCreated { context_id, name, description } => {
                        let mut components = HashMap::new();
                        components.insert("type".to_string(), serde_json::json!("bounded_context"));
                        components.insert("name".to_string(), serde_json::json!(name));
                        components.insert("description".to_string(), serde_json::json!(description));
                        self.components.insert(context_id.clone(), components);
                    }
                    crate::events::ContextPayload::AggregateAdded { context_id, aggregate_id, aggregate_type } => {
                        let mut components = HashMap::new();
                        components.insert("type".to_string(), serde_json::json!("aggregate"));
                        components.insert("aggregate_type".to_string(), serde_json::json!(aggregate_type));
                        self.components.insert(aggregate_id.to_string(), components);
                        
                        // Add relationship
                        let edge_id = format!("{}-contains-{}", context_id, aggregate_id);
                        self.relationships.insert(edge_id, vec![(context_id.clone(), aggregate_id.to_string())]);
                    }
                    _ => {}
                }
            }
            EventPayload::Workflow(wf) => {
                // Workflow events create state machine components
                match wf {
                    crate::events::WorkflowPayload::WorkflowDefined { workflow_id, name, version } => {
                        let mut components = HashMap::new();
                        components.insert("type".to_string(), serde_json::json!("workflow"));
                        components.insert("name".to_string(), serde_json::json!(name));
                        components.insert("version".to_string(), serde_json::json!(version));
                        self.components.insert(workflow_id.to_string(), components);
                    }
                    crate::events::WorkflowPayload::StateAdded { workflow_id, state_id, state_type } => {
                        let mut components = HashMap::new();
                        components.insert("type".to_string(), serde_json::json!("state"));
                        components.insert("state_type".to_string(), serde_json::json!(state_type));
                        self.components.insert(state_id.clone(), components);
                        
                        // Add relationship
                        let edge_id = format!("{}-has-state-{}", workflow_id, state_id);
                        self.relationships.insert(edge_id, vec![(workflow_id.to_string(), state_id.clone())]);
                    }
                    _ => {}
                }
            }
            _ => {} // Other event types
        }
    }
}

/// System: Query entities with specific components
pub fn query_entities_with_component(
    projection: &GraphAggregateProjection,
    component_type: &str,
) -> Vec<String> {
    projection.components
        .iter()
        .filter(|(_, components)| components.contains_key(component_type))
        .map(|(entity_id, _)| entity_id.clone())
        .collect()
}

/// System: Query relationships of a specific type
pub fn query_relationships(
    projection: &GraphAggregateProjection,
    source: Option<&str>,
    target: Option<&str>,
) -> Vec<(String, String, String)> { // (edge_id, source, target)
    projection.relationships
        .iter()
        .filter(|(_, edges)| {
            edges.iter().any(|(s, t)| {
                (source.is_none() || s == source.unwrap()) &&
                (target.is_none() || t == target.unwrap())
            })
        })
        .flat_map(|(edge_id, edges)| {
            edges.iter().map(|(s, t)| (edge_id.clone(), s.clone(), t.clone()))
        })
        .collect()
}

/// System: Get component data for an entity
pub fn get_entity_components<'a>(
    projection: &'a GraphAggregateProjection,
    entity_id: &str,
) -> Option<&'a HashMap<String, serde_json::Value>> {
    projection.components.get(entity_id)
}

/// Build a projection by folding events from JetStream
pub fn build_projection(events: Vec<(GraphEvent, u64)>) -> GraphAggregateProjection {
    if events.is_empty() {
        panic!("Cannot build projection from empty event stream");
    }

    let first_event = &events[0].0;
    let aggregate_id = first_event.aggregate_id;
    let subject = Subject::from_segments(vec![
        SubjectSegment::new("cim").unwrap(),
        SubjectSegment::new("graph").unwrap(),
        SubjectSegment::new(aggregate_id.to_string()).unwrap(),
        SubjectSegment::new("events").unwrap(),
    ])
    .expect("valid subject segments")
    .to_string();

    let mut projection = GraphAggregateProjection::new(aggregate_id, subject);

    // Left fold over all events
    for (event, sequence) in events {
        projection.apply(&event, sequence);
    }

    projection
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{IpldPayload, ContextPayload, WorkflowPayload};

    // ========== GraphAggregateProjection Tests ==========

    #[test]
    fn test_new_projection() {
        let agg_id = Uuid::new_v4();
        let projection = GraphAggregateProjection::new(agg_id, "test.subject".to_string());

        assert_eq!(projection.aggregate_id, agg_id);
        assert_eq!(projection.version, 0);
        assert!(projection.components.is_empty());
        assert!(projection.relationships.is_empty());
        assert_eq!(projection.subject, "test.subject");
    }

    #[test]
    fn test_apply_ipld_cid_added() {
        let agg_id = Uuid::new_v4();
        let mut projection = GraphAggregateProjection::new(agg_id, "test".to_string());

        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: agg_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Ipld(IpldPayload::CidAdded {
                cid: "QmTest123".to_string(),
                codec: "dag-cbor".to_string(),
                size: 256,
                data: serde_json::json!({"test": "data"}),
            }),
        };

        projection.apply(&event, 1);

        assert_eq!(projection.version, 1);
        assert!(projection.components.contains_key("QmTest123"));

        let components = projection.components.get("QmTest123").unwrap();
        assert_eq!(components.get("cid").unwrap(), "QmTest123");
        assert_eq!(components.get("codec").unwrap(), "dag-cbor");
        assert_eq!(components.get("size").unwrap(), 256);
    }

    #[test]
    fn test_apply_ipld_cid_link_added() {
        let agg_id = Uuid::new_v4();
        let mut projection = GraphAggregateProjection::new(agg_id, "test".to_string());

        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: agg_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Ipld(IpldPayload::CidLinkAdded {
                cid: "QmSource".to_string(),
                link_name: "child".to_string(),
                target_cid: "QmTarget".to_string(),
            }),
        };

        projection.apply(&event, 1);

        assert_eq!(projection.version, 1);
        assert_eq!(projection.relationships.len(), 1);

        let edge_id = "QmSource->QmTarget:child".to_string();
        assert!(projection.relationships.contains_key(&edge_id));
    }

    #[test]
    fn test_apply_context_bounded_context_created() {
        let agg_id = Uuid::new_v4();
        let mut projection = GraphAggregateProjection::new(agg_id, "test".to_string());

        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: agg_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Context(ContextPayload::BoundedContextCreated {
                context_id: "orders".to_string(),
                name: "Orders Context".to_string(),
                description: "Manages order processing".to_string(),
            }),
        };

        projection.apply(&event, 1);

        assert!(projection.components.contains_key("orders"));
        let components = projection.components.get("orders").unwrap();
        assert_eq!(components.get("type").unwrap(), "bounded_context");
        assert_eq!(components.get("name").unwrap(), "Orders Context");
    }

    #[test]
    fn test_apply_context_aggregate_added() {
        let agg_id = Uuid::new_v4();
        let agg_uuid = Uuid::new_v4();
        let mut projection = GraphAggregateProjection::new(agg_id, "test".to_string());

        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: agg_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Context(ContextPayload::AggregateAdded {
                context_id: "orders".to_string(),
                aggregate_id: agg_uuid,
                aggregate_type: "Order".to_string(),
            }),
        };

        projection.apply(&event, 1);

        // Check aggregate component was added
        assert!(projection.components.contains_key(&agg_uuid.to_string()));
        let components = projection.components.get(&agg_uuid.to_string()).unwrap();
        assert_eq!(components.get("type").unwrap(), "aggregate");
        assert_eq!(components.get("aggregate_type").unwrap(), "Order");

        // Check relationship was added
        let edge_id = format!("orders-contains-{}", agg_uuid);
        assert!(projection.relationships.contains_key(&edge_id));
    }

    #[test]
    fn test_apply_workflow_defined() {
        let agg_id = Uuid::new_v4();
        let wf_id = Uuid::new_v4();
        let mut projection = GraphAggregateProjection::new(agg_id, "test".to_string());

        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: agg_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
                workflow_id: wf_id,
                name: "OrderProcessing".to_string(),
                version: "1.0.0".to_string(),
            }),
        };

        projection.apply(&event, 1);

        assert!(projection.components.contains_key(&wf_id.to_string()));
        let components = projection.components.get(&wf_id.to_string()).unwrap();
        assert_eq!(components.get("type").unwrap(), "workflow");
        assert_eq!(components.get("name").unwrap(), "OrderProcessing");
        assert_eq!(components.get("version").unwrap(), "1.0.0");
    }

    #[test]
    fn test_apply_workflow_state_added() {
        let agg_id = Uuid::new_v4();
        let wf_id = Uuid::new_v4();
        let mut projection = GraphAggregateProjection::new(agg_id, "test".to_string());

        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: agg_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Workflow(WorkflowPayload::StateAdded {
                workflow_id: wf_id,
                state_id: "pending".to_string(),
                state_type: "initial".to_string(),
            }),
        };

        projection.apply(&event, 1);

        // Check state component was added
        assert!(projection.components.contains_key("pending"));
        let components = projection.components.get("pending").unwrap();
        assert_eq!(components.get("type").unwrap(), "state");
        assert_eq!(components.get("state_type").unwrap(), "initial");

        // Check relationship was added
        let edge_id = format!("{}-has-state-pending", wf_id);
        assert!(projection.relationships.contains_key(&edge_id));
    }

    #[test]
    fn test_apply_ignores_other_aggregate_events() {
        let agg_id = Uuid::new_v4();
        let other_agg_id = Uuid::new_v4();
        let mut projection = GraphAggregateProjection::new(agg_id, "test".to_string());

        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: other_agg_id, // Different aggregate
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Ipld(IpldPayload::CidAdded {
                cid: "QmForeign".to_string(),
                codec: "raw".to_string(),
                size: 100,
                data: serde_json::json!({}),
            }),
        };

        projection.apply(&event, 10);

        // Should not have applied the event
        assert_eq!(projection.version, 0);
        assert!(!projection.components.contains_key("QmForeign"));
    }

    #[test]
    fn test_apply_multiple_events() {
        let agg_id = Uuid::new_v4();
        let mut projection = GraphAggregateProjection::new(agg_id, "test".to_string());

        let events = vec![
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: agg_id,
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Ipld(IpldPayload::CidAdded {
                    cid: "QmRoot".to_string(),
                    codec: "dag-pb".to_string(),
                    size: 512,
                    data: serde_json::json!({"name": "root"}),
                }),
            },
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: agg_id,
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Ipld(IpldPayload::CidAdded {
                    cid: "QmChild".to_string(),
                    codec: "dag-pb".to_string(),
                    size: 256,
                    data: serde_json::json!({"name": "child"}),
                }),
            },
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: agg_id,
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Ipld(IpldPayload::CidLinkAdded {
                    cid: "QmRoot".to_string(),
                    link_name: "child".to_string(),
                    target_cid: "QmChild".to_string(),
                }),
            },
        ];

        for (i, event) in events.iter().enumerate() {
            projection.apply(event, (i + 1) as u64);
        }

        assert_eq!(projection.version, 3);
        assert_eq!(projection.components.len(), 2);
        assert_eq!(projection.relationships.len(), 1);
    }

    // ========== Query Function Tests ==========

    #[test]
    fn test_query_entities_with_component() {
        let agg_id = Uuid::new_v4();
        let mut projection = GraphAggregateProjection::new(agg_id, "test".to_string());

        // Add entities with different components
        let mut cid_components = HashMap::new();
        cid_components.insert("type".to_string(), serde_json::json!("cid"));
        cid_components.insert("cid".to_string(), serde_json::json!("QmTest"));

        let mut state_components = HashMap::new();
        state_components.insert("type".to_string(), serde_json::json!("state"));
        state_components.insert("state_type".to_string(), serde_json::json!("initial"));

        projection.components.insert("entity1".to_string(), cid_components);
        projection.components.insert("entity2".to_string(), state_components);

        // Query for entities with "cid" component
        let cid_entities = query_entities_with_component(&projection, "cid");
        assert_eq!(cid_entities.len(), 1);
        assert!(cid_entities.contains(&"entity1".to_string()));

        // Query for entities with "state_type" component
        let state_entities = query_entities_with_component(&projection, "state_type");
        assert_eq!(state_entities.len(), 1);
        assert!(state_entities.contains(&"entity2".to_string()));

        // Query for entities with "type" component (both have it)
        let type_entities = query_entities_with_component(&projection, "type");
        assert_eq!(type_entities.len(), 2);

        // Query for non-existent component
        let none_entities = query_entities_with_component(&projection, "nonexistent");
        assert!(none_entities.is_empty());
    }

    #[test]
    fn test_query_relationships_all() {
        let agg_id = Uuid::new_v4();
        let mut projection = GraphAggregateProjection::new(agg_id, "test".to_string());

        projection.relationships.insert(
            "e1".to_string(),
            vec![("A".to_string(), "B".to_string())]
        );
        projection.relationships.insert(
            "e2".to_string(),
            vec![("B".to_string(), "C".to_string())]
        );
        projection.relationships.insert(
            "e3".to_string(),
            vec![("A".to_string(), "C".to_string())]
        );

        let all = query_relationships(&projection, None, None);
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_query_relationships_by_source() {
        let agg_id = Uuid::new_v4();
        let mut projection = GraphAggregateProjection::new(agg_id, "test".to_string());

        projection.relationships.insert(
            "e1".to_string(),
            vec![("A".to_string(), "B".to_string())]
        );
        projection.relationships.insert(
            "e2".to_string(),
            vec![("B".to_string(), "C".to_string())]
        );
        projection.relationships.insert(
            "e3".to_string(),
            vec![("A".to_string(), "C".to_string())]
        );

        let from_a = query_relationships(&projection, Some("A"), None);
        assert_eq!(from_a.len(), 2);

        let from_b = query_relationships(&projection, Some("B"), None);
        assert_eq!(from_b.len(), 1);

        let from_x = query_relationships(&projection, Some("X"), None);
        assert!(from_x.is_empty());
    }

    #[test]
    fn test_query_relationships_by_target() {
        let agg_id = Uuid::new_v4();
        let mut projection = GraphAggregateProjection::new(agg_id, "test".to_string());

        projection.relationships.insert(
            "e1".to_string(),
            vec![("A".to_string(), "B".to_string())]
        );
        projection.relationships.insert(
            "e2".to_string(),
            vec![("B".to_string(), "C".to_string())]
        );
        projection.relationships.insert(
            "e3".to_string(),
            vec![("A".to_string(), "C".to_string())]
        );

        let to_c = query_relationships(&projection, None, Some("C"));
        assert_eq!(to_c.len(), 2);

        let to_b = query_relationships(&projection, None, Some("B"));
        assert_eq!(to_b.len(), 1);
    }

    #[test]
    fn test_query_relationships_by_both() {
        let agg_id = Uuid::new_v4();
        let mut projection = GraphAggregateProjection::new(agg_id, "test".to_string());

        projection.relationships.insert(
            "e1".to_string(),
            vec![("A".to_string(), "B".to_string())]
        );
        projection.relationships.insert(
            "e2".to_string(),
            vec![("A".to_string(), "C".to_string())]
        );

        let a_to_b = query_relationships(&projection, Some("A"), Some("B"));
        assert_eq!(a_to_b.len(), 1);

        let a_to_c = query_relationships(&projection, Some("A"), Some("C"));
        assert_eq!(a_to_c.len(), 1);

        let a_to_x = query_relationships(&projection, Some("A"), Some("X"));
        assert!(a_to_x.is_empty());
    }

    #[test]
    fn test_get_entity_components() {
        let agg_id = Uuid::new_v4();
        let mut projection = GraphAggregateProjection::new(agg_id, "test".to_string());

        let mut components = HashMap::new();
        components.insert("type".to_string(), serde_json::json!("test"));
        components.insert("value".to_string(), serde_json::json!(42));

        projection.components.insert("entity1".to_string(), components);

        let result = get_entity_components(&projection, "entity1");
        assert!(result.is_some());

        let comps = result.unwrap();
        assert_eq!(comps.get("type").unwrap(), "test");
        assert_eq!(comps.get("value").unwrap(), 42);

        let missing = get_entity_components(&projection, "nonexistent");
        assert!(missing.is_none());
    }

    // ========== Build Projection Tests ==========

    #[test]
    fn test_build_projection_single_event() {
        let agg_id = Uuid::new_v4();

        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: agg_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Ipld(IpldPayload::CidAdded {
                cid: "QmSingle".to_string(),
                codec: "raw".to_string(),
                size: 100,
                data: serde_json::json!({}),
            }),
        };

        let projection = build_projection(vec![(event, 1)]);

        assert_eq!(projection.aggregate_id, agg_id);
        assert_eq!(projection.version, 1);
        assert!(projection.components.contains_key("QmSingle"));
    }

    #[test]
    fn test_build_projection_multiple_events() {
        let agg_id = Uuid::new_v4();
        let wf_id = Uuid::new_v4();

        let events = vec![
            (GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: agg_id,
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
                    workflow_id: wf_id,
                    name: "TestWorkflow".to_string(),
                    version: "1.0".to_string(),
                }),
            }, 1),
            (GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: agg_id,
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Workflow(WorkflowPayload::StateAdded {
                    workflow_id: wf_id,
                    state_id: "start".to_string(),
                    state_type: "initial".to_string(),
                }),
            }, 2),
            (GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: agg_id,
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Workflow(WorkflowPayload::StateAdded {
                    workflow_id: wf_id,
                    state_id: "end".to_string(),
                    state_type: "terminal".to_string(),
                }),
            }, 3),
        ];

        let projection = build_projection(events);

        assert_eq!(projection.aggregate_id, agg_id);
        assert_eq!(projection.version, 3);
        assert!(projection.components.contains_key(&wf_id.to_string()));
        assert!(projection.components.contains_key("start"));
        assert!(projection.components.contains_key("end"));
        assert_eq!(projection.relationships.len(), 2);
    }

    #[test]
    #[should_panic(expected = "Cannot build projection from empty event stream")]
    fn test_build_projection_empty_panics() {
        build_projection(vec![]);
    }

    #[test]
    fn test_build_projection_subject_format() {
        let agg_id = Uuid::new_v4();

        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: agg_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Ipld(IpldPayload::CidAdded {
                cid: "QmTest".to_string(),
                codec: "raw".to_string(),
                size: 50,
                data: serde_json::json!({}),
            }),
        };

        let projection = build_projection(vec![(event, 1)]);

        assert!(projection.subject.starts_with("cim.graph."));
        assert!(projection.subject.contains(&agg_id.to_string()));
        assert!(projection.subject.ends_with(".events"));
    }

    // ========== Serialization Tests ==========

    #[test]
    fn test_projection_serialization() {
        let agg_id = Uuid::new_v4();
        let mut projection = GraphAggregateProjection::new(agg_id, "test.subject".to_string());

        let mut components = HashMap::new();
        components.insert("key".to_string(), serde_json::json!("value"));
        projection.components.insert("entity1".to_string(), components);

        // Serialize to JSON
        let json = serde_json::to_string(&projection).unwrap();
        assert!(json.contains("aggregate_id"));
        assert!(json.contains("test.subject"));

        // Deserialize back
        let deserialized: GraphAggregateProjection = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.aggregate_id, agg_id);
        assert_eq!(deserialized.subject, "test.subject");
        assert!(deserialized.components.contains_key("entity1"));
    }

    #[test]
    fn test_projection_clone() {
        let agg_id = Uuid::new_v4();
        let mut projection = GraphAggregateProjection::new(agg_id, "test".to_string());
        projection.version = 10;

        let cloned = projection.clone();

        assert_eq!(cloned.aggregate_id, projection.aggregate_id);
        assert_eq!(cloned.version, projection.version);
        assert_eq!(cloned.subject, projection.subject);
    }

    #[test]
    fn test_projection_debug() {
        let agg_id = Uuid::new_v4();
        let projection = GraphAggregateProjection::new(agg_id, "test".to_string());

        let debug_str = format!("{:?}", projection);
        assert!(debug_str.contains("GraphAggregateProjection"));
        assert!(debug_str.contains("aggregate_id"));
    }
}
