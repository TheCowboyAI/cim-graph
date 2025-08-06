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