//! Event schemas for all graph types in CIM
//!
//! Following CIM's event-sourcing patterns:
//! - All events have correlation and causation IDs
//! - Events are the ONLY way to change state
//! - Sequence and timestamp come from NATS JetStream
//! - Payload is added once, metadata accumulates in subsequent events

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

/// Base event structure - minimal metadata since JetStream provides the rest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEvent {
    /// Unique event ID
    pub event_id: Uuid,
    /// Aggregate this event belongs to
    pub aggregate_id: Uuid,
    /// Correlation ID - links related events across aggregates
    pub correlation_id: Uuid,
    /// Causation ID - the event that caused this one (replaces triggered_by)
    pub causation_id: Option<Uuid>,
    /// The actual event payload
    pub payload: EventPayload,
}

/// Event payloads - the actual data that gets CID'd in IPLD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventPayload {
    /// Generic graph operations
    Generic(GenericPayload),
    /// IPLD graph operations
    Ipld(IpldPayload),
    /// Context graph operations
    Context(ContextPayload),
    /// Workflow operations
    Workflow(WorkflowPayload),
    /// Concept graph operations
    Concept(ConceptPayload),
    /// Composed graph operations
    Composed(ComposedPayload),
}

/// Generic payload for basic graph operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericPayload {
    /// Event type name
    pub event_type: String,
    /// Event data
    pub data: serde_json::Value,
}

/// IPLD payloads - content-addressed operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpldPayload {
    /// A new CID was added to the graph
    CidAdded {
        /// Content Identifier (CID) of the data being added
        cid: String,
        /// IPLD codec used to encode the data (e.g., "dag-cbor", "dag-json")
        codec: String,
        /// Size of the data in bytes
        size: u64,
        /// The actual data content as JSON value
        data: serde_json::Value,
    },
    /// Link metadata for existing CID (subsequent event)
    CidLinkAdded {
        /// Source CID that the link originates from
        cid: String,
        /// Name of the link relationship
        link_name: String,
        /// Target CID that the link points to
        target_cid: String,
    },
    /// Pin metadata for existing CID (subsequent event)
    CidPinned {
        /// CID to pin in the local IPLD store
        cid: String,
        /// Whether to recursively pin all linked CIDs
        recursive: bool,
    },
    /// Unpin metadata for existing CID (subsequent event)
    CidUnpinned {
        /// CID to unpin from the local IPLD store
        cid: String,
    },
}

/// Context graph payloads - Domain-Driven Design
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextPayload {
    /// Initial bounded context creation
    BoundedContextCreated {
        /// Unique identifier for the bounded context
        context_id: String,
        /// Human-readable name of the bounded context
        name: String,
        /// Description of the bounded context's purpose and scope
        description: String,
    },
    /// Add aggregate to existing context (subsequent event)
    AggregateAdded {
        /// ID of the bounded context to add the aggregate to
        context_id: String,
        /// Unique identifier for the aggregate
        aggregate_id: Uuid,
        /// Type of the aggregate (e.g., "Order", "Customer")
        aggregate_type: String,
    },
    /// Add entity to existing aggregate (subsequent event)
    EntityAdded {
        /// ID of the aggregate this entity belongs to
        aggregate_id: Uuid,
        /// Unique identifier for the entity
        entity_id: Uuid,
        /// Type of the entity (e.g., "OrderItem", "Address")
        entity_type: String,
        /// Initial properties of the entity as JSON
        properties: serde_json::Value,
    },
    /// Add value object (subsequent event)
    ValueObjectAttached {
        /// ID of the parent entity or aggregate
        parent_id: Uuid,
        /// Type of the value object (e.g., "Money", "Email")
        value_type: String,
        /// Immutable data of the value object as JSON
        value_data: serde_json::Value,
    },
    /// Add relationship metadata (subsequent event)
    RelationshipEstablished {
        /// ID of the source entity in the relationship
        source_id: Uuid,
        /// ID of the target entity in the relationship
        target_id: Uuid,
        /// Type of relationship (e.g., "owns", "references")
        relationship_type: String,
    },
}

/// Workflow payloads - state machines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowPayload {
    /// Initial workflow definition
    WorkflowDefined {
        /// Unique identifier for the workflow definition
        workflow_id: Uuid,
        /// Human-readable name of the workflow
        name: String,
        /// Version string for the workflow definition
        version: String,
    },
    /// Add state to workflow (subsequent event)
    StateAdded {
        /// ID of the workflow to add the state to
        workflow_id: Uuid,
        /// Unique identifier for the state within the workflow
        state_id: String,
        /// Type of state (e.g., "initial", "final", "intermediate")
        state_type: String,
    },
    /// Add transition (subsequent event)
    TransitionAdded {
        /// ID of the workflow containing the transition
        workflow_id: Uuid,
        /// ID of the state to transition from
        from_state: String,
        /// ID of the state to transition to
        to_state: String,
        /// Event or condition that triggers the transition
        trigger: String,
    },
    /// Instance created from workflow
    InstanceCreated {
        /// ID of the workflow definition used
        workflow_id: Uuid,
        /// Unique identifier for the workflow instance
        instance_id: Uuid,
        /// Initial state of the workflow instance
        initial_state: String,
    },
    /// State transition in instance (subsequent event)
    StateTransitioned {
        /// ID of the workflow instance
        instance_id: Uuid,
        /// State transitioning from
        from_state: String,
        /// State transitioning to
        to_state: String,
    },
}

/// Concept graph payloads - semantic reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConceptPayload {
    /// Initial concept definition
    ConceptDefined {
        /// Unique identifier for the concept
        concept_id: String,
        /// Human-readable name of the concept
        name: String,
        /// Formal definition or description of the concept
        definition: String,
    },
    /// Add properties to concept (subsequent event)
    PropertiesAdded {
        /// ID of the concept to add properties to
        concept_id: String,
        /// List of property name-value pairs with confidence scores
        properties: Vec<(String, f64)>,
    },
    /// Add semantic relation (subsequent event)
    RelationAdded {
        /// ID of the source concept in the relation
        source_concept: String,
        /// ID of the target concept in the relation
        target_concept: String,
        /// Type of semantic relation (e.g., "is-a", "part-of")
        relation_type: String,
        /// Strength or confidence of the relation (0.0 to 1.0)
        strength: f64,
    },
    /// Inference result (subsequent event)
    PropertyInferred {
        /// ID of the concept with inferred property
        concept_id: String,
        /// Name of the inferred property
        property_name: String,
        /// Inferred value for the property
        inferred_value: f64,
        /// Confidence level of the inference (0.0 to 1.0)
        confidence: f64,
    },
}

/// Composed graph payloads - multi-graph operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComposedPayload {
    /// Add sub-graph to composition
    SubGraphAdded {
        /// Unique identifier for the sub-graph
        subgraph_id: Uuid,
        /// Type of graph (e.g., "ipld", "context", "workflow")
        graph_type: String,
        /// Namespace for isolating the sub-graph
        namespace: String,
    },
    /// Link across graphs (subsequent event)
    CrossGraphLinkCreated {
        /// ID of the source graph
        source_graph: Uuid,
        /// ID of the node in the source graph
        source_node: String,
        /// ID of the target graph
        target_graph: Uuid,
        /// ID of the node in the target graph
        target_node: String,
    },
}

/// Commands that request state changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphCommand {
    /// Initialize a graph
    InitializeGraph {
        /// Unique identifier for the graph aggregate
        aggregate_id: Uuid,
        /// Type of graph to initialize
        graph_type: String,
        /// Correlation ID for tracking related operations
        correlation_id: Uuid,
    },
    /// IPLD-specific commands
    Ipld {
        /// ID of the IPLD graph aggregate
        aggregate_id: Uuid,
        /// Correlation ID for tracking related operations
        correlation_id: Uuid,
        /// The specific IPLD command to execute
        command: IpldCommand,
    },
    /// Context-specific commands
    Context {
        /// ID of the context graph aggregate
        aggregate_id: Uuid,
        /// Correlation ID for tracking related operations
        correlation_id: Uuid,
        /// The specific context command to execute
        command: ContextCommand,
    },
    /// Workflow-specific commands
    Workflow {
        /// ID of the workflow graph aggregate
        aggregate_id: Uuid,
        /// Correlation ID for tracking related operations
        correlation_id: Uuid,
        /// The specific workflow command to execute
        command: WorkflowCommand,
    },
    /// Concept-specific commands
    Concept {
        /// ID of the concept graph aggregate
        aggregate_id: Uuid,
        /// Correlation ID for tracking related operations
        correlation_id: Uuid,
        /// The specific concept command to execute
        command: ConceptCommand,
    },
    /// Composed-specific commands
    Composed {
        /// ID of the composed graph aggregate
        aggregate_id: Uuid,
        /// Correlation ID for tracking related operations
        correlation_id: Uuid,
        /// The specific composed command to execute
        command: ComposedCommand,
    },
    /// Archive a graph
    ArchiveGraph {
        /// ID of the graph aggregate to archive
        aggregate_id: Uuid,
        /// Correlation ID for tracking related operations
        correlation_id: Uuid,
    },
    /// Generic command
    Generic {
        /// ID of the graph aggregate
        aggregate_id: Uuid,
        /// Correlation ID for tracking related operations
        correlation_id: Uuid,
        /// Command name as string
        command: String,
        /// Command data as JSON value
        data: serde_json::Value,
    },
}

/// IPLD-specific commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpldCommand {
    /// Add a new CID to the IPLD graph
    AddCid {
        /// Content Identifier to add
        cid: String,
        /// IPLD codec used (e.g., "dag-cbor", "dag-json")
        codec: String,
        /// Size of the content in bytes
        size: u64,
        /// The content data as JSON
        data: serde_json::Value,
    },
    /// Create a link between two CIDs
    LinkCids {
        /// Source CID to link from
        source_cid: String,
        /// Target CID to link to
        target_cid: String,
        /// Name of the link relationship
        link_name: String,
    },
    /// Pin a CID to prevent garbage collection
    PinCid {
        /// CID to pin
        cid: String,
        /// Whether to recursively pin linked CIDs
        recursive: bool,
    },
    /// Unpin a CID to allow garbage collection
    UnpinCid {
        /// CID to unpin
        cid: String,
    },
}

/// Context-specific commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextCommand {
    /// Create a new bounded context
    CreateBoundedContext {
        /// Unique identifier for the context
        context_id: String,
        /// Human-readable name
        name: String,
        /// Description of purpose and scope
        description: String,
    },
    /// Add an aggregate to a bounded context
    AddAggregate {
        /// ID of the context to add to
        context_id: String,
        /// Unique identifier for the aggregate
        aggregate_id: Uuid,
        /// Type of aggregate (e.g., "Order", "Customer")
        aggregate_type: String,
    },
    /// Add an entity to an aggregate
    AddEntity {
        /// ID of the parent aggregate
        aggregate_id: Uuid,
        /// Unique identifier for the entity
        entity_id: Uuid,
        /// Type of entity (e.g., "OrderItem")
        entity_type: String,
        /// Initial entity properties as JSON
        properties: serde_json::Value,
    },
}

/// Workflow-specific commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowCommand {
    /// Define a new workflow template
    DefineWorkflow {
        /// Unique identifier for the workflow
        workflow_id: Uuid,
        /// Human-readable workflow name
        name: String,
        /// Version of the workflow definition
        version: String,
    },
    /// Add a state to a workflow definition
    AddState {
        /// ID of the workflow to modify
        workflow_id: Uuid,
        /// Unique identifier for the state
        state_id: String,
        /// Type of state (e.g., "initial", "final")
        state_type: String,
    },
    /// Add a transition between states
    AddTransition {
        /// ID of the workflow containing the transition
        workflow_id: Uuid,
        /// State to transition from
        from_state: String,
        /// State to transition to
        to_state: String,
        /// Event that triggers the transition
        trigger: String,
    },
    /// Create an instance of a workflow
    CreateInstance {
        /// ID of the workflow template to instantiate
        workflow_id: Uuid,
        /// Unique identifier for the new instance
        instance_id: Uuid,
    },
    /// Trigger a state transition in an instance
    TriggerTransition {
        /// ID of the workflow instance
        instance_id: Uuid,
        /// Trigger event name
        trigger: String,
    },
}

/// Concept-specific commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConceptCommand {
    /// Define a new concept in the graph
    DefineConcept {
        /// Unique identifier for the concept
        concept_id: String,
        /// Human-readable concept name
        name: String,
        /// Formal definition of the concept
        definition: String,
    },
    /// Add properties to an existing concept
    AddProperties {
        /// ID of the concept to modify
        concept_id: String,
        /// Property name-value pairs with confidence
        properties: Vec<(String, f64)>,
    },
    /// Add a semantic relation between concepts
    AddRelation {
        /// Source concept ID
        source_concept: String,
        /// Target concept ID
        target_concept: String,
        /// Type of relation (e.g., "is-a", "part-of")
        relation_type: String,
        /// Strength of the relation (0.0 to 1.0)
        strength: f64,
    },
}

/// Composed-specific commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComposedCommand {
    /// Add a sub-graph to the composition
    AddSubGraph {
        /// Unique identifier for the sub-graph
        subgraph_id: Uuid,
        /// Type of graph (e.g., "ipld", "context")
        graph_type: String,
        /// Namespace for graph isolation
        namespace: String,
    },
    /// Create a link between nodes in different graphs
    LinkAcrossGraphs {
        /// ID of the source graph
        source_graph: Uuid,
        /// Node ID in the source graph
        source_node: String,
        /// ID of the target graph
        target_graph: Uuid,
        /// Node ID in the target graph
        target_node: String,
    },
}

/// Helper to create NATS headers from an event
pub fn event_to_nats_headers(event: &GraphEvent) -> HashMap<String, String> {
    let mut headers = HashMap::new();

    // These are the only headers we need - JetStream provides the rest
    headers.insert("Nats-Msg-Id".to_string(), event.event_id.to_string());
    headers.insert("Correlation-Id".to_string(), event.correlation_id.to_string());
    if let Some(causation_id) = event.causation_id {
        headers.insert("Causation-Id".to_string(), causation_id.to_string());
    }
    headers.insert("Aggregate-Id".to_string(), event.aggregate_id.to_string());

    headers
}

impl EventPayload {
    /// Get the event type as a string
    pub fn event_type(&self) -> &str {
        match self {
            EventPayload::Generic(_) => "generic",
            EventPayload::Ipld(_) => "ipld",
            EventPayload::Context(_) => "context",
            EventPayload::Workflow(_) => "workflow",
            EventPayload::Concept(_) => "concept",
            EventPayload::Composed(_) => "composed",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== GraphEvent Tests ==========

    #[test]
    fn test_graph_event_creation() {
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Generic(GenericPayload {
                event_type: "Test".to_string(),
                data: serde_json::json!({}),
            }),
        };

        assert!(event.causation_id.is_none());
    }

    #[test]
    fn test_graph_event_with_causation() {
        let cause_id = Uuid::new_v4();
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            causation_id: Some(cause_id),
            payload: EventPayload::Generic(GenericPayload {
                event_type: "Effect".to_string(),
                data: serde_json::json!({}),
            }),
        };

        assert_eq!(event.causation_id, Some(cause_id));
    }

    #[test]
    fn test_graph_event_clone() {
        let original = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            causation_id: Some(Uuid::new_v4()),
            payload: EventPayload::Generic(GenericPayload {
                event_type: "Clone".to_string(),
                data: serde_json::json!({"key": "value"}),
            }),
        };

        let cloned = original.clone();
        assert_eq!(original.event_id, cloned.event_id);
        assert_eq!(original.aggregate_id, cloned.aggregate_id);
    }

    #[test]
    fn test_graph_event_debug() {
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Generic(GenericPayload {
                event_type: "Debug".to_string(),
                data: serde_json::json!({}),
            }),
        };

        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("GraphEvent"));
    }

    #[test]
    fn test_graph_event_serialize_deserialize() {
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            causation_id: Some(Uuid::new_v4()),
            payload: EventPayload::Generic(GenericPayload {
                event_type: "Serialize".to_string(),
                data: serde_json::json!({"test": true}),
            }),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: GraphEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.event_id, deserialized.event_id);
        assert_eq!(event.aggregate_id, deserialized.aggregate_id);
        assert_eq!(event.causation_id, deserialized.causation_id);
    }

    // ========== EventPayload Tests ==========

    #[test]
    fn test_event_payload_generic() {
        let payload = EventPayload::Generic(GenericPayload {
            event_type: "Test".to_string(),
            data: serde_json::json!({"key": "value"}),
        });

        assert_eq!(payload.event_type(), "generic");
    }

    #[test]
    fn test_event_payload_ipld() {
        let payload = EventPayload::Ipld(IpldPayload::CidAdded {
            cid: "QmTest".to_string(),
            codec: "dag-cbor".to_string(),
            size: 100,
            data: serde_json::json!({}),
        });

        assert_eq!(payload.event_type(), "ipld");
    }

    #[test]
    fn test_event_payload_context() {
        let payload = EventPayload::Context(ContextPayload::BoundedContextCreated {
            context_id: "ctx1".to_string(),
            name: "Test Context".to_string(),
            description: "A test bounded context".to_string(),
        });

        assert_eq!(payload.event_type(), "context");
    }

    #[test]
    fn test_event_payload_workflow() {
        let payload = EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
            workflow_id: Uuid::new_v4(),
            name: "Order Processing".to_string(),
            version: "1.0.0".to_string(),
        });

        assert_eq!(payload.event_type(), "workflow");
    }

    #[test]
    fn test_event_payload_concept() {
        let payload = EventPayload::Concept(ConceptPayload::ConceptDefined {
            concept_id: "c1".to_string(),
            name: "Customer".to_string(),
            definition: "A person who buys products".to_string(),
        });

        assert_eq!(payload.event_type(), "concept");
    }

    #[test]
    fn test_event_payload_composed() {
        let payload = EventPayload::Composed(ComposedPayload::SubGraphAdded {
            subgraph_id: Uuid::new_v4(),
            graph_type: "workflow".to_string(),
            namespace: "orders".to_string(),
        });

        assert_eq!(payload.event_type(), "composed");
    }

    // ========== GenericPayload Tests ==========

    #[test]
    fn test_generic_payload() {
        let payload = GenericPayload {
            event_type: "NodeCreated".to_string(),
            data: serde_json::json!({
                "node_id": "n1",
                "label": "Test Node"
            }),
        };

        assert_eq!(payload.event_type, "NodeCreated");
        assert_eq!(payload.data.get("node_id").unwrap().as_str(), Some("n1"));
    }

    #[test]
    fn test_generic_payload_clone() {
        let original = GenericPayload {
            event_type: "Clone".to_string(),
            data: serde_json::json!({"value": 42}),
        };

        let cloned = original.clone();
        assert_eq!(original.event_type, cloned.event_type);
    }

    // ========== IpldPayload Tests ==========

    #[test]
    fn test_ipld_payload_cid_added() {
        let payload = IpldPayload::CidAdded {
            cid: "QmAbcd".to_string(),
            codec: "dag-cbor".to_string(),
            size: 256,
            data: serde_json::json!({"content": "test"}),
        };

        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("CidAdded"));
        assert!(debug_str.contains("QmAbcd"));
    }

    #[test]
    fn test_ipld_payload_cid_link_added() {
        let payload = IpldPayload::CidLinkAdded {
            cid: "QmSource".to_string(),
            link_name: "child".to_string(),
            target_cid: "QmTarget".to_string(),
        };

        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("CidLinkAdded"));
    }

    #[test]
    fn test_ipld_payload_cid_pinned() {
        let payload = IpldPayload::CidPinned {
            cid: "QmPinMe".to_string(),
            recursive: true,
        };

        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("CidPinned"));
        assert!(debug_str.contains("recursive: true"));
    }

    #[test]
    fn test_ipld_payload_cid_unpinned() {
        let payload = IpldPayload::CidUnpinned {
            cid: "QmUnpinMe".to_string(),
        };

        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("CidUnpinned"));
    }

    // ========== ContextPayload Tests ==========

    #[test]
    fn test_context_payload_bounded_context_created() {
        let payload = ContextPayload::BoundedContextCreated {
            context_id: "orders".to_string(),
            name: "Order Management".to_string(),
            description: "Handles order lifecycle".to_string(),
        };

        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("BoundedContextCreated"));
    }

    #[test]
    fn test_context_payload_aggregate_added() {
        let payload = ContextPayload::AggregateAdded {
            context_id: "orders".to_string(),
            aggregate_id: Uuid::new_v4(),
            aggregate_type: "Order".to_string(),
        };

        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("AggregateAdded"));
    }

    #[test]
    fn test_context_payload_entity_added() {
        let payload = ContextPayload::EntityAdded {
            aggregate_id: Uuid::new_v4(),
            entity_id: Uuid::new_v4(),
            entity_type: "OrderItem".to_string(),
            properties: serde_json::json!({"quantity": 5}),
        };

        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("EntityAdded"));
    }

    #[test]
    fn test_context_payload_value_object_attached() {
        let payload = ContextPayload::ValueObjectAttached {
            parent_id: Uuid::new_v4(),
            value_type: "Money".to_string(),
            value_data: serde_json::json!({"amount": 100, "currency": "USD"}),
        };

        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("ValueObjectAttached"));
    }

    #[test]
    fn test_context_payload_relationship_established() {
        let payload = ContextPayload::RelationshipEstablished {
            source_id: Uuid::new_v4(),
            target_id: Uuid::new_v4(),
            relationship_type: "owns".to_string(),
        };

        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("RelationshipEstablished"));
    }

    // ========== WorkflowPayload Tests ==========

    #[test]
    fn test_workflow_payload_workflow_defined() {
        let payload = WorkflowPayload::WorkflowDefined {
            workflow_id: Uuid::new_v4(),
            name: "Order Approval".to_string(),
            version: "2.0.0".to_string(),
        };

        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("WorkflowDefined"));
    }

    #[test]
    fn test_workflow_payload_state_added() {
        let payload = WorkflowPayload::StateAdded {
            workflow_id: Uuid::new_v4(),
            state_id: "pending".to_string(),
            state_type: "intermediate".to_string(),
        };

        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("StateAdded"));
    }

    #[test]
    fn test_workflow_payload_transition_added() {
        let payload = WorkflowPayload::TransitionAdded {
            workflow_id: Uuid::new_v4(),
            from_state: "pending".to_string(),
            to_state: "approved".to_string(),
            trigger: "approve_button".to_string(),
        };

        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("TransitionAdded"));
    }

    #[test]
    fn test_workflow_payload_instance_created() {
        let payload = WorkflowPayload::InstanceCreated {
            workflow_id: Uuid::new_v4(),
            instance_id: Uuid::new_v4(),
            initial_state: "start".to_string(),
        };

        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("InstanceCreated"));
    }

    #[test]
    fn test_workflow_payload_state_transitioned() {
        let payload = WorkflowPayload::StateTransitioned {
            instance_id: Uuid::new_v4(),
            from_state: "pending".to_string(),
            to_state: "approved".to_string(),
        };

        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("StateTransitioned"));
    }

    // ========== ConceptPayload Tests ==========

    #[test]
    fn test_concept_payload_concept_defined() {
        let payload = ConceptPayload::ConceptDefined {
            concept_id: "vehicle".to_string(),
            name: "Vehicle".to_string(),
            definition: "A thing used for transporting people or cargo".to_string(),
        };

        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("ConceptDefined"));
    }

    #[test]
    fn test_concept_payload_properties_added() {
        let payload = ConceptPayload::PropertiesAdded {
            concept_id: "car".to_string(),
            properties: vec![
                ("wheels".to_string(), 4.0),
                ("engine".to_string(), 1.0),
            ],
        };

        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("PropertiesAdded"));
    }

    #[test]
    fn test_concept_payload_relation_added() {
        let payload = ConceptPayload::RelationAdded {
            source_concept: "car".to_string(),
            target_concept: "vehicle".to_string(),
            relation_type: "is-a".to_string(),
            strength: 1.0,
        };

        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("RelationAdded"));
    }

    #[test]
    fn test_concept_payload_property_inferred() {
        let payload = ConceptPayload::PropertyInferred {
            concept_id: "car".to_string(),
            property_name: "transportable".to_string(),
            inferred_value: 1.0,
            confidence: 0.95,
        };

        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("PropertyInferred"));
    }

    // ========== ComposedPayload Tests ==========

    #[test]
    fn test_composed_payload_subgraph_added() {
        let payload = ComposedPayload::SubGraphAdded {
            subgraph_id: Uuid::new_v4(),
            graph_type: "workflow".to_string(),
            namespace: "orders".to_string(),
        };

        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("SubGraphAdded"));
    }

    #[test]
    fn test_composed_payload_cross_graph_link_created() {
        let payload = ComposedPayload::CrossGraphLinkCreated {
            source_graph: Uuid::new_v4(),
            source_node: "n1".to_string(),
            target_graph: Uuid::new_v4(),
            target_node: "n2".to_string(),
        };

        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("CrossGraphLinkCreated"));
    }

    // ========== GraphCommand Tests ==========

    #[test]
    fn test_graph_command_initialize() {
        let cmd = GraphCommand::InitializeGraph {
            aggregate_id: Uuid::new_v4(),
            graph_type: "workflow".to_string(),
            correlation_id: Uuid::new_v4(),
        };

        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("InitializeGraph"));
    }

    #[test]
    fn test_graph_command_ipld() {
        let cmd = GraphCommand::Ipld {
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            command: IpldCommand::AddCid {
                cid: "QmTest".to_string(),
                codec: "dag-cbor".to_string(),
                size: 100,
                data: serde_json::json!({}),
            },
        };

        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("Ipld"));
    }

    #[test]
    fn test_graph_command_context() {
        let cmd = GraphCommand::Context {
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            command: ContextCommand::CreateBoundedContext {
                context_id: "ctx".to_string(),
                name: "Test".to_string(),
                description: "Desc".to_string(),
            },
        };

        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("Context"));
    }

    #[test]
    fn test_graph_command_workflow() {
        let cmd = GraphCommand::Workflow {
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            command: WorkflowCommand::DefineWorkflow {
                workflow_id: Uuid::new_v4(),
                name: "Test".to_string(),
                version: "1.0".to_string(),
            },
        };

        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("Workflow"));
    }

    #[test]
    fn test_graph_command_concept() {
        let cmd = GraphCommand::Concept {
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            command: ConceptCommand::DefineConcept {
                concept_id: "c1".to_string(),
                name: "Test".to_string(),
                definition: "Def".to_string(),
            },
        };

        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("Concept"));
    }

    #[test]
    fn test_graph_command_composed() {
        let cmd = GraphCommand::Composed {
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            command: ComposedCommand::AddSubGraph {
                subgraph_id: Uuid::new_v4(),
                graph_type: "ipld".to_string(),
                namespace: "ns".to_string(),
            },
        };

        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("Composed"));
    }

    #[test]
    fn test_graph_command_archive() {
        let cmd = GraphCommand::ArchiveGraph {
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
        };

        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("ArchiveGraph"));
    }

    #[test]
    fn test_graph_command_generic() {
        let cmd = GraphCommand::Generic {
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            command: "custom_command".to_string(),
            data: serde_json::json!({"param": "value"}),
        };

        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("Generic"));
    }

    // ========== IpldCommand Tests ==========

    #[test]
    fn test_ipld_command_add_cid() {
        let cmd = IpldCommand::AddCid {
            cid: "QmNew".to_string(),
            codec: "dag-json".to_string(),
            size: 50,
            data: serde_json::json!({}),
        };

        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("AddCid"));
    }

    #[test]
    fn test_ipld_command_link_cids() {
        let cmd = IpldCommand::LinkCids {
            source_cid: "QmSource".to_string(),
            target_cid: "QmTarget".to_string(),
            link_name: "related".to_string(),
        };

        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("LinkCids"));
    }

    #[test]
    fn test_ipld_command_pin_cid() {
        let cmd = IpldCommand::PinCid {
            cid: "QmPin".to_string(),
            recursive: false,
        };

        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("PinCid"));
    }

    #[test]
    fn test_ipld_command_unpin_cid() {
        let cmd = IpldCommand::UnpinCid {
            cid: "QmUnpin".to_string(),
        };

        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("UnpinCid"));
    }

    // ========== ContextCommand Tests ==========

    #[test]
    fn test_context_command_create_bounded_context() {
        let cmd = ContextCommand::CreateBoundedContext {
            context_id: "billing".to_string(),
            name: "Billing".to_string(),
            description: "Handles billing operations".to_string(),
        };

        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("CreateBoundedContext"));
    }

    #[test]
    fn test_context_command_add_aggregate() {
        let cmd = ContextCommand::AddAggregate {
            context_id: "billing".to_string(),
            aggregate_id: Uuid::new_v4(),
            aggregate_type: "Invoice".to_string(),
        };

        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("AddAggregate"));
    }

    #[test]
    fn test_context_command_add_entity() {
        let cmd = ContextCommand::AddEntity {
            aggregate_id: Uuid::new_v4(),
            entity_id: Uuid::new_v4(),
            entity_type: "LineItem".to_string(),
            properties: serde_json::json!({"amount": 99.99}),
        };

        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("AddEntity"));
    }

    // ========== WorkflowCommand Tests ==========

    #[test]
    fn test_workflow_command_define_workflow() {
        let cmd = WorkflowCommand::DefineWorkflow {
            workflow_id: Uuid::new_v4(),
            name: "Approval Flow".to_string(),
            version: "3.0.0".to_string(),
        };

        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("DefineWorkflow"));
    }

    #[test]
    fn test_workflow_command_add_state() {
        let cmd = WorkflowCommand::AddState {
            workflow_id: Uuid::new_v4(),
            state_id: "review".to_string(),
            state_type: "intermediate".to_string(),
        };

        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("AddState"));
    }

    #[test]
    fn test_workflow_command_add_transition() {
        let cmd = WorkflowCommand::AddTransition {
            workflow_id: Uuid::new_v4(),
            from_state: "draft".to_string(),
            to_state: "review".to_string(),
            trigger: "submit".to_string(),
        };

        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("AddTransition"));
    }

    #[test]
    fn test_workflow_command_create_instance() {
        let cmd = WorkflowCommand::CreateInstance {
            workflow_id: Uuid::new_v4(),
            instance_id: Uuid::new_v4(),
        };

        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("CreateInstance"));
    }

    #[test]
    fn test_workflow_command_trigger_transition() {
        let cmd = WorkflowCommand::TriggerTransition {
            instance_id: Uuid::new_v4(),
            trigger: "approve".to_string(),
        };

        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("TriggerTransition"));
    }

    // ========== ConceptCommand Tests ==========

    #[test]
    fn test_concept_command_define_concept() {
        let cmd = ConceptCommand::DefineConcept {
            concept_id: "animal".to_string(),
            name: "Animal".to_string(),
            definition: "A living organism".to_string(),
        };

        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("DefineConcept"));
    }

    #[test]
    fn test_concept_command_add_properties() {
        let cmd = ConceptCommand::AddProperties {
            concept_id: "dog".to_string(),
            properties: vec![("legs".to_string(), 4.0)],
        };

        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("AddProperties"));
    }

    #[test]
    fn test_concept_command_add_relation() {
        let cmd = ConceptCommand::AddRelation {
            source_concept: "dog".to_string(),
            target_concept: "animal".to_string(),
            relation_type: "is-a".to_string(),
            strength: 1.0,
        };

        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("AddRelation"));
    }

    // ========== ComposedCommand Tests ==========

    #[test]
    fn test_composed_command_add_subgraph() {
        let cmd = ComposedCommand::AddSubGraph {
            subgraph_id: Uuid::new_v4(),
            graph_type: "context".to_string(),
            namespace: "orders".to_string(),
        };

        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("AddSubGraph"));
    }

    #[test]
    fn test_composed_command_link_across_graphs() {
        let cmd = ComposedCommand::LinkAcrossGraphs {
            source_graph: Uuid::new_v4(),
            source_node: "order-1".to_string(),
            target_graph: Uuid::new_v4(),
            target_node: "invoice-1".to_string(),
        };

        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("LinkAcrossGraphs"));
    }

    // ========== event_to_nats_headers Tests ==========

    #[test]
    fn test_event_to_nats_headers_basic() {
        let event = GraphEvent {
            event_id: Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
            aggregate_id: Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap(),
            correlation_id: Uuid::parse_str("33333333-3333-3333-3333-333333333333").unwrap(),
            causation_id: None,
            payload: EventPayload::Generic(GenericPayload {
                event_type: "Test".to_string(),
                data: serde_json::json!({}),
            }),
        };

        let headers = event_to_nats_headers(&event);

        assert_eq!(headers.get("Nats-Msg-Id"), Some(&"11111111-1111-1111-1111-111111111111".to_string()));
        assert_eq!(headers.get("Correlation-Id"), Some(&"33333333-3333-3333-3333-333333333333".to_string()));
        assert_eq!(headers.get("Aggregate-Id"), Some(&"22222222-2222-2222-2222-222222222222".to_string()));
        assert!(headers.get("Causation-Id").is_none());
    }

    #[test]
    fn test_event_to_nats_headers_with_causation() {
        let causation = Uuid::parse_str("44444444-4444-4444-4444-444444444444").unwrap();
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            causation_id: Some(causation),
            payload: EventPayload::Generic(GenericPayload {
                event_type: "Test".to_string(),
                data: serde_json::json!({}),
            }),
        };

        let headers = event_to_nats_headers(&event);

        assert!(headers.get("Causation-Id").is_some());
        assert_eq!(headers.get("Causation-Id"), Some(&"44444444-4444-4444-4444-444444444444".to_string()));
    }

    #[test]
    fn test_event_to_nats_headers_header_count() {
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Generic(GenericPayload {
                event_type: "Test".to_string(),
                data: serde_json::json!({}),
            }),
        };

        let headers = event_to_nats_headers(&event);
        assert_eq!(headers.len(), 3); // event_id, correlation_id, aggregate_id

        let event_with_causation = GraphEvent {
            causation_id: Some(Uuid::new_v4()),
            ..event
        };

        let headers_with_causation = event_to_nats_headers(&event_with_causation);
        assert_eq!(headers_with_causation.len(), 4); // +causation_id
    }
}