//! Type-specific graph implementations

pub mod ipld;
pub mod context;
pub mod workflow;
pub mod concept;
pub mod composed;

pub use self::ipld::IpldGraph;
pub use self::context::ContextGraph;
pub use self::workflow::WorkflowGraph;
pub use self::concept::ConceptGraph;
pub use self::composed::ComposedGraph;