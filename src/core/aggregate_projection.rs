//! Aggregate projections - left fold of events for an aggregate
//!
//! Following DDD + ECS principles:
//! - Entities are just IDs
//! - Components are value objects attached via events
//! - Systems are functions (queries, commands, event handlers)
//! - State transitions ONLY through StateMachine

use crate::events::{GraphEvent, EventPayload};
use uuid::Uuid;
use std::collections::HashMap;

/// A graph aggregate projection - the result of folding all events
#[derive(Debug, Clone)]
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
    let subject = format!("graph.{}.events", aggregate_id); // Would come from cim-subject
    
    let mut projection = GraphAggregateProjection::new(aggregate_id, subject);
    
    // Left fold over all events
    for (event, sequence) in events {
        projection.apply(&event, sequence);
    }
    
    projection
}