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
        cid: String,
        codec: String,
        size: u64,
        data: serde_json::Value,
    },
    /// Link metadata for existing CID (subsequent event)
    CidLinkAdded {
        cid: String,
        link_name: String,
        target_cid: String,
    },
    /// Pin metadata for existing CID (subsequent event)
    CidPinned {
        cid: String,
        recursive: bool,
    },
    /// Unpin metadata for existing CID (subsequent event)
    CidUnpinned {
        cid: String,
    },
}

/// Context graph payloads - Domain-Driven Design
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextPayload {
    /// Initial bounded context creation
    BoundedContextCreated {
        context_id: String,
        name: String,
        description: String,
    },
    /// Add aggregate to existing context (subsequent event)
    AggregateAdded {
        context_id: String,
        aggregate_id: Uuid,
        aggregate_type: String,
    },
    /// Add entity to existing aggregate (subsequent event)
    EntityAdded {
        aggregate_id: Uuid,
        entity_id: Uuid,
        entity_type: String,
        properties: serde_json::Value,
    },
    /// Add value object (subsequent event)
    ValueObjectAttached {
        parent_id: Uuid,
        value_type: String,
        value_data: serde_json::Value,
    },
    /// Add relationship metadata (subsequent event)
    RelationshipEstablished {
        source_id: Uuid,
        target_id: Uuid,
        relationship_type: String,
    },
}

/// Workflow payloads - state machines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowPayload {
    /// Initial workflow definition
    WorkflowDefined {
        workflow_id: Uuid,
        name: String,
        version: String,
    },
    /// Add state to workflow (subsequent event)
    StateAdded {
        workflow_id: Uuid,
        state_id: String,
        state_type: String,
    },
    /// Add transition (subsequent event)
    TransitionAdded {
        workflow_id: Uuid,
        from_state: String,
        to_state: String,
        trigger: String,
    },
    /// Instance created from workflow
    InstanceCreated {
        workflow_id: Uuid,
        instance_id: Uuid,
        initial_state: String,
    },
    /// State transition in instance (subsequent event)
    StateTransitioned {
        instance_id: Uuid,
        from_state: String,
        to_state: String,
    },
}

/// Concept graph payloads - semantic reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConceptPayload {
    /// Initial concept definition
    ConceptDefined {
        concept_id: String,
        name: String,
        definition: String,
    },
    /// Add properties to concept (subsequent event)
    PropertiesAdded {
        concept_id: String,
        properties: Vec<(String, f64)>,
    },
    /// Add semantic relation (subsequent event)
    RelationAdded {
        source_concept: String,
        target_concept: String,
        relation_type: String,
        strength: f64,
    },
    /// Inference result (subsequent event)
    PropertyInferred {
        concept_id: String,
        property_name: String,
        inferred_value: f64,
        confidence: f64,
    },
}

/// Composed graph payloads - multi-graph operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComposedPayload {
    /// Add sub-graph to composition
    SubGraphAdded {
        subgraph_id: Uuid,
        graph_type: String,
        namespace: String,
    },
    /// Link across graphs (subsequent event)
    CrossGraphLinkCreated {
        source_graph: Uuid,
        source_node: String,
        target_graph: Uuid,
        target_node: String,
    },
}

/// Commands that request state changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphCommand {
    /// Initialize a graph
    InitializeGraph {
        aggregate_id: Uuid,
        graph_type: String,
        correlation_id: Uuid,
    },
    /// IPLD-specific commands
    Ipld {
        aggregate_id: Uuid,
        correlation_id: Uuid,
        command: IpldCommand,
    },
    /// Context-specific commands
    Context {
        aggregate_id: Uuid,
        correlation_id: Uuid,
        command: ContextCommand,
    },
    /// Workflow-specific commands
    Workflow {
        aggregate_id: Uuid,
        correlation_id: Uuid,
        command: WorkflowCommand,
    },
    /// Concept-specific commands
    Concept {
        aggregate_id: Uuid,
        correlation_id: Uuid,
        command: ConceptCommand,
    },
    /// Composed-specific commands
    Composed {
        aggregate_id: Uuid,
        correlation_id: Uuid,
        command: ComposedCommand,
    },
    /// Archive a graph
    ArchiveGraph {
        aggregate_id: Uuid,
        correlation_id: Uuid,
    },
    /// Generic command
    Generic {
        aggregate_id: Uuid,
        correlation_id: Uuid,
        command: String,
        data: serde_json::Value,
    },
}

/// IPLD-specific commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpldCommand {
    AddCid {
        cid: String,
        codec: String,
        size: u64,
        data: serde_json::Value,
    },
    LinkCids {
        source_cid: String,
        target_cid: String,
        link_name: String,
    },
    PinCid {
        cid: String,
        recursive: bool,
    },
    UnpinCid {
        cid: String,
    },
}

/// Context-specific commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextCommand {
    CreateBoundedContext {
        context_id: String,
        name: String,
        description: String,
    },
    AddAggregate {
        context_id: String,
        aggregate_id: Uuid,
        aggregate_type: String,
    },
    AddEntity {
        aggregate_id: Uuid,
        entity_id: Uuid,
        entity_type: String,
        properties: serde_json::Value,
    },
}

/// Workflow-specific commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowCommand {
    DefineWorkflow {
        workflow_id: Uuid,
        name: String,
        version: String,
    },
    AddState {
        workflow_id: Uuid,
        state_id: String,
        state_type: String,
    },
    AddTransition {
        workflow_id: Uuid,
        from_state: String,
        to_state: String,
        trigger: String,
    },
    CreateInstance {
        workflow_id: Uuid,
        instance_id: Uuid,
    },
    TriggerTransition {
        instance_id: Uuid,
        trigger: String,
    },
}

/// Concept-specific commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConceptCommand {
    DefineConcept {
        concept_id: String,
        name: String,
        definition: String,
    },
    AddProperties {
        concept_id: String,
        properties: Vec<(String, f64)>,
    },
    AddRelation {
        source_concept: String,
        target_concept: String,
        relation_type: String,
        strength: f64,
    },
}

/// Composed-specific commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComposedCommand {
    AddSubGraph {
        subgraph_id: Uuid,
        graph_type: String,
        namespace: String,
    },
    LinkAcrossGraphs {
        source_graph: Uuid,
        source_node: String,
        target_graph: Uuid,
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