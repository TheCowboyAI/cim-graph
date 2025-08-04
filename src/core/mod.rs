//! Core graph abstractions and traits

pub mod builder;
pub mod edge;
pub mod event;
pub mod graph;
pub mod node;
pub mod petgraph_impl;

pub use self::builder::GraphBuilder;
pub use self::edge::{Edge, GenericEdge};
pub use self::event::{EventHandler, GraphEvent, MemoryEventHandler};
pub use self::graph::{Graph, GraphId, GraphMetadata, GraphType};
pub use self::node::{GenericNode, Node};
pub use self::petgraph_impl::EventGraph;
