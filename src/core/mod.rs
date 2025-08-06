//! Core graph abstractions and traits

pub mod edge;
pub mod event;
pub mod event_sourcing;
pub mod event_driven;
pub mod cim_graph;
pub mod graph;
pub mod node;
pub mod projection_engine;
pub mod aggregate_projection;
pub mod state_machine;
pub mod ipld_chain;
pub mod policies;

#[cfg(test)]
mod event_tests;

pub use self::edge::{Edge, GenericEdge};
pub use self::event::{EventHandler, GraphEvent, MemoryEventHandler};
pub use self::event_sourcing::{
    EventMetadata, GraphEvent as SourcingEvent, GraphEventPayload, 
    EventStore, MemoryEventStore, GraphAggregate as EventSourcedAggregate
};
pub use self::graph::{GraphId, GraphMetadata, GraphType};
pub use self::node::{GenericNode, Node};
pub use self::cim_graph::{GraphProjection, GraphEvent as CimGraphEvent, EventData, GraphCommand};
pub use self::projection_engine::{ProjectionEngine, GenericGraphProjection, ProjectionCache};
pub use self::aggregate_projection::{GraphAggregateProjection, build_projection};
pub use self::ipld_chain::{IpldChainAggregate, Cid, IpldChainCommand, IpldChainEvent};
pub use self::state_machine::{GraphStateMachine, GraphState, WorkflowState};
pub use self::policies::{
    Policy, PolicyEngine, PolicyContext, PolicyAction, PolicyMetrics,
    CidGenerationPolicy, ProjectionUpdatePolicy, StateValidationPolicy,
    ChainValidationPolicy, CollaborationPolicy
};
