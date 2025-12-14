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

    // ========== GraphType Tests ==========

    #[test]
    fn test_graph_type_as_str() {
        assert_eq!(GraphType::Ipld.as_str(), "ipld");
        assert_eq!(GraphType::Context.as_str(), "context");
        assert_eq!(GraphType::Workflow.as_str(), "workflow");
        assert_eq!(GraphType::Concept.as_str(), "concept");
        assert_eq!(GraphType::Composed.as_str(), "composed");
    }

    #[test]
    fn test_graph_type_equality() {
        assert_eq!(GraphType::Ipld, GraphType::Ipld);
        assert_eq!(GraphType::Context, GraphType::Context);
        assert_ne!(GraphType::Ipld, GraphType::Context);
        assert_ne!(GraphType::Workflow, GraphType::Concept);
    }

    #[test]
    fn test_graph_type_clone() {
        let original = GraphType::Workflow;
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_graph_type_copy() {
        let original = GraphType::Concept;
        let copied: GraphType = original; // Uses Copy trait
        assert_eq!(original, copied);
    }

    #[test]
    fn test_graph_type_debug() {
        let debug_str = format!("{:?}", GraphType::Composed);
        assert_eq!(debug_str, "Composed");
    }

    // ========== EventType Tests ==========

    #[test]
    fn test_event_type_as_str() {
        assert_eq!(EventType::Created.as_str(), "created");
        assert_eq!(EventType::Updated.as_str(), "updated");
        assert_eq!(EventType::Deleted.as_str(), "deleted");
        assert_eq!(EventType::NodeAdded.as_str(), "node_added");
        assert_eq!(EventType::NodeRemoved.as_str(), "node_removed");
        assert_eq!(EventType::EdgeAdded.as_str(), "edge_added");
        assert_eq!(EventType::EdgeRemoved.as_str(), "edge_removed");
        assert_eq!(EventType::StateChanged.as_str(), "state_changed");
    }

    #[test]
    fn test_event_type_equality() {
        assert_eq!(EventType::Created, EventType::Created);
        assert_eq!(EventType::NodeAdded, EventType::NodeAdded);
        assert_ne!(EventType::Created, EventType::Deleted);
        assert_ne!(EventType::EdgeAdded, EventType::EdgeRemoved);
    }

    #[test]
    fn test_event_type_clone() {
        let original = EventType::StateChanged;
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_event_type_copy() {
        let original = EventType::Updated;
        let copied: EventType = original;
        assert_eq!(original, copied);
    }

    #[test]
    fn test_event_type_debug() {
        let debug_str = format!("{:?}", EventType::NodeAdded);
        assert_eq!(debug_str, "NodeAdded");
    }

    // ========== build_event_subject Tests ==========

    #[test]
    fn test_build_event_subject_all_graph_types() {
        let aggregate_id = Uuid::new_v4();

        let subjects = vec![
            (GraphType::Ipld, "ipld"),
            (GraphType::Context, "context"),
            (GraphType::Workflow, "workflow"),
            (GraphType::Concept, "concept"),
            (GraphType::Composed, "composed"),
        ];

        for (graph_type, type_str) in subjects {
            let subject = build_event_subject(graph_type, aggregate_id, EventType::Created);
            assert!(subject.contains(type_str), "Subject should contain '{}'", type_str);
            assert!(subject.contains(&aggregate_id.to_string()));
            assert!(subject.contains("created"));
        }
    }

    #[test]
    fn test_build_event_subject_all_event_types() {
        let aggregate_id = Uuid::new_v4();

        let event_types = vec![
            (EventType::Created, "created"),
            (EventType::Updated, "updated"),
            (EventType::Deleted, "deleted"),
            (EventType::NodeAdded, "node_added"),
            (EventType::NodeRemoved, "node_removed"),
            (EventType::EdgeAdded, "edge_added"),
            (EventType::EdgeRemoved, "edge_removed"),
            (EventType::StateChanged, "state_changed"),
        ];

        for (event_type, type_str) in event_types {
            let subject = build_event_subject(GraphType::Workflow, aggregate_id, event_type);
            assert!(subject.ends_with(type_str), "Subject should end with '{}'", type_str);
        }
    }

    #[test]
    fn test_build_event_subject_format() {
        let aggregate_id = Uuid::new_v4();
        let subject = build_event_subject(GraphType::Ipld, aggregate_id, EventType::NodeAdded);

        // Verify format: cim.graph.{type}.{uuid}.{event}
        let parts: Vec<&str> = subject.split('.').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0], "cim");
        assert_eq!(parts[1], "graph");
        assert_eq!(parts[2], "ipld");
        assert_eq!(parts[3], aggregate_id.to_string());
        assert_eq!(parts[4], "node_added");
    }

    // ========== build_graph_subscription Tests ==========

    #[test]
    fn test_build_graph_subscription_all_types() {
        let aggregate_id = Uuid::new_v4();

        for graph_type in [GraphType::Ipld, GraphType::Context, GraphType::Workflow, GraphType::Concept, GraphType::Composed] {
            let sub = build_graph_subscription(graph_type, aggregate_id);
            assert!(sub.contains(&aggregate_id.to_string()));
            assert!(sub.ends_with(".*"));
        }
    }

    #[test]
    fn test_build_graph_subscription_format() {
        let aggregate_id = Uuid::new_v4();
        let sub = build_graph_subscription(GraphType::Context, aggregate_id);

        let expected = format!("cim.graph.context.{}.*", aggregate_id);
        assert_eq!(sub, expected);
    }

    // ========== build_type_subscription Tests ==========

    #[test]
    fn test_build_type_subscription_all_types() {
        let subscriptions = vec![
            (GraphType::Ipld, "cim.graph.ipld.>"),
            (GraphType::Context, "cim.graph.context.>"),
            (GraphType::Workflow, "cim.graph.workflow.>"),
            (GraphType::Concept, "cim.graph.concept.>"),
            (GraphType::Composed, "cim.graph.composed.>"),
        ];

        for (graph_type, expected) in subscriptions {
            let sub = build_type_subscription(graph_type);
            assert_eq!(sub, expected);
        }
    }

    // ========== build_all_events_subscription Tests ==========

    #[test]
    fn test_build_all_events_subscription() {
        let sub = build_all_events_subscription();
        assert_eq!(sub, "cim.graph.>");
    }

    // ========== parse_event_subject Tests ==========

    #[test]
    fn test_parse_event_subject_roundtrip() {
        let aggregate_id = Uuid::new_v4();
        let subject = build_event_subject(GraphType::Workflow, aggregate_id, EventType::StateChanged);

        let (parsed_type, parsed_id, parsed_event) = parse_event_subject(&subject).unwrap();

        assert_eq!(parsed_type, GraphType::Workflow);
        assert_eq!(parsed_id, aggregate_id);
        assert_eq!(parsed_event, EventType::StateChanged);
    }

    #[test]
    fn test_parse_event_subject_all_graph_types() {
        let aggregate_id = Uuid::new_v4();

        for graph_type in [GraphType::Ipld, GraphType::Context, GraphType::Workflow, GraphType::Concept, GraphType::Composed] {
            let subject = build_event_subject(graph_type, aggregate_id, EventType::Created);
            let (parsed_type, _, _) = parse_event_subject(&subject).unwrap();
            assert_eq!(parsed_type, graph_type);
        }
    }

    #[test]
    fn test_parse_event_subject_all_event_types() {
        let aggregate_id = Uuid::new_v4();

        let event_types = [
            EventType::Created,
            EventType::Updated,
            EventType::Deleted,
            EventType::NodeAdded,
            EventType::NodeRemoved,
            EventType::EdgeAdded,
            EventType::EdgeRemoved,
            EventType::StateChanged,
        ];

        for event_type in event_types {
            let subject = build_event_subject(GraphType::Ipld, aggregate_id, event_type);
            let (_, _, parsed_event) = parse_event_subject(&subject).unwrap();
            assert_eq!(parsed_event, event_type);
        }
    }

    #[test]
    fn test_parse_event_subject_invalid_format_too_few_segments() {
        let result = parse_event_subject("cim.graph.workflow");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid subject format"));
    }

    #[test]
    fn test_parse_event_subject_invalid_format_wrong_prefix() {
        let aggregate_id = Uuid::new_v4();
        let result = parse_event_subject(&format!("other.graph.workflow.{}.created", aggregate_id));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_event_subject_invalid_format_wrong_second_segment() {
        let aggregate_id = Uuid::new_v4();
        let result = parse_event_subject(&format!("cim.other.workflow.{}.created", aggregate_id));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_event_subject_unknown_graph_type() {
        let aggregate_id = Uuid::new_v4();
        let result = parse_event_subject(&format!("cim.graph.unknown.{}.created", aggregate_id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown graph type"));
    }

    #[test]
    fn test_parse_event_subject_invalid_uuid() {
        let result = parse_event_subject("cim.graph.workflow.invalid-uuid.created");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid UUID"));
    }

    #[test]
    fn test_parse_event_subject_unknown_event_type() {
        let aggregate_id = Uuid::new_v4();
        let result = parse_event_subject(&format!("cim.graph.workflow.{}.unknown_event", aggregate_id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown event type"));
    }

    // ========== GRAPH_ROOT Constant Test ==========

    #[test]
    fn test_graph_root_constant() {
        assert_eq!(GRAPH_ROOT, "cim.graph");
    }
}
