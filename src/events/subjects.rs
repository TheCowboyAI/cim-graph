//! Event subjects using cim-domain's subject module
//!
//! This module defines all NATS subjects used in the event-driven graph system
//! using cim-domain's subject module for type-safe subject algebra.

use uuid::Uuid;
use cim_domain::{Subject, SubjectPattern};

/// Root subject component for all graph events
pub const GRAPH_ROOT: &str = "cim.graph";

/// Subject components for different graph types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphType {
    /// InterPlanetary Linked Data graphs
    Ipld,
    /// Domain-driven design context graphs
    Context,
    /// State machine workflow graphs
    Workflow,
    /// Knowledge representation concept graphs
    Concept,
    /// Graphs composed of multiple sub-graphs
    Composed,
}

impl GraphType {
    /// Convert graph type to string representation
    fn as_str(&self) -> &str {
        match self {
            GraphType::Ipld => "ipld",
            GraphType::Context => "context",
            GraphType::Workflow => "workflow",
            GraphType::Concept => "concept",
            GraphType::Composed => "composed",
        }
    }
}

/// Event type components
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    /// Graph or entity was created
    Created,
    /// Graph or entity was updated
    Updated,
    /// Graph or entity was deleted
    Deleted,
    /// Node was added to graph
    NodeAdded,
    /// Node was removed from graph
    NodeRemoved,
    /// Edge was added between nodes
    EdgeAdded,
    /// Edge was removed from graph
    EdgeRemoved,
    /// State machine state changed
    StateChanged,
}

impl EventType {
    /// Convert event type to string representation
    fn as_str(&self) -> &str {
        match self {
            EventType::Created => "created",
            EventType::Updated => "updated",
            EventType::Deleted => "deleted",
            EventType::NodeAdded => "node_added",
            EventType::NodeRemoved => "node_removed",
            EventType::EdgeAdded => "edge_added",
            EventType::EdgeRemoved => "edge_removed",
            EventType::StateChanged => "state_changed",
        }
    }
}

/// Build a subject for a specific graph event
pub fn build_event_subject(
    graph_type: GraphType,
    aggregate_id: Uuid,
    event_type: EventType,
) -> String {
    // Build via cim-domain's Subject to enforce validation
    let subject = Subject::from_segments([
        // Graph root segments
        cim_domain::SubjectSegment::new("cim").unwrap(),
        cim_domain::SubjectSegment::new("graph").unwrap(),
        // Graph type
        cim_domain::SubjectSegment::new(graph_type.as_str()).unwrap(),
        // Aggregate as segment
        cim_domain::SubjectSegment::new(aggregate_id.to_string()).unwrap(),
        // Event type
        cim_domain::SubjectSegment::new(event_type.as_str()).unwrap(),
    ].into_iter()).expect("valid subject segments");
    subject.to_string()
}

/// Build a subject for subscribing to all events of a specific graph
pub fn build_graph_subscription(graph_type: GraphType, aggregate_id: Uuid) -> String {
    // Validate via SubjectPattern semantics
    let pattern_str = format!("{}.{}.{}.*", GRAPH_ROOT, graph_type.as_str(), aggregate_id);
    let _ = SubjectPattern::parse(&pattern_str).expect("valid subject pattern");
    pattern_str
}

/// Build a subject for subscribing to all events of a graph type
pub fn build_type_subscription(graph_type: GraphType) -> String {
    let pattern_str = format!("{}.{}.>", GRAPH_ROOT, graph_type.as_str());
    let _ = SubjectPattern::parse(&pattern_str).expect("valid subject pattern");
    pattern_str
}

/// Build a subject for subscribing to all graph events
pub fn build_all_events_subscription() -> String {
    let pattern_str = format!("{}.>", GRAPH_ROOT);
    let _ = SubjectPattern::parse(&pattern_str).expect("valid subject pattern");
    pattern_str
}

/// Parse an event subject back into its components
pub fn parse_event_subject(subject: &str) -> Result<(GraphType, Uuid, EventType), String> {
    let parsed = Subject::parse(subject).map_err(|e| e.to_string())?;
    let segs: Vec<_> = parsed.segments().map(|s| s.as_str()).collect();
    // Expected: cim.graph.{type}.{uuid}.{event}
    if segs.len() != 5 || segs[0] != "cim" || segs[1] != "graph" {
        return Err("Invalid subject format".to_string());
    }
    let graph_type = match segs[2] {
        "ipld" => GraphType::Ipld,
        "context" => GraphType::Context,
        "workflow" => GraphType::Workflow,
        "concept" => GraphType::Concept,
        "composed" => GraphType::Composed,
        _ => return Err("Unknown graph type".to_string()),
    };
    let aggregate_id = Uuid::parse_str(segs[3]).map_err(|_| "Invalid UUID in subject".to_string())?;
    let event_type = match segs[4] {
        "created" => EventType::Created,
        "updated" => EventType::Updated,
        "deleted" => EventType::Deleted,
        "node_added" => EventType::NodeAdded,
        "node_removed" => EventType::NodeRemoved,
        "edge_added" => EventType::EdgeAdded,
        "edge_removed" => EventType::EdgeRemoved,
        "state_changed" => EventType::StateChanged,
        _ => return Err("Unknown event type".to_string()),
    };
    Ok((graph_type, aggregate_id, event_type))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_subject_building() {
        let aggregate_id = Uuid::new_v4();
        let subject = build_event_subject(
            GraphType::Workflow,
            aggregate_id,
            EventType::Created
        );
        
        let expected = format!("cim.graph.workflow.{}.created", aggregate_id);
        assert_eq!(subject.to_string(), expected);
    }
    
    #[test]
    fn test_subscription_subjects() {
        let aggregate_id = Uuid::new_v4();
        
        // Graph-specific subscription
        let graph_sub = build_graph_subscription(GraphType::Ipld, aggregate_id);
        assert_eq!(
            graph_sub.to_string(),
            format!("cim.graph.ipld.{}.*", aggregate_id)
        );
        
        // Type-wide subscription
        let type_sub = build_type_subscription(GraphType::Context);
        assert_eq!(type_sub.to_string(), "cim.graph.context.>");
        
        // All events subscription
        let all_sub = build_all_events_subscription();
        assert_eq!(all_sub.to_string(), "cim.graph.>");
    }
}
