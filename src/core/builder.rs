//! Graph builder for fluent graph construction

use crate::core::graph::BasicGraph;
use crate::core::petgraph_impl::EventGraph;
use crate::core::{Edge, EventHandler, Graph, GraphType, Node};
use crate::error::Result;
use std::marker::PhantomData;
use std::sync::Arc;

/// Builder for creating graphs with proper initialization
pub struct GraphBuilder<N: Node, E: Edge> {
    graph_type: GraphType,
    name: Option<String>,
    description: Option<String>,
    event_handlers: Vec<Arc<dyn EventHandler>>,
    _phantom: PhantomData<(N, E)>,
}

impl<N: Node, E: Edge> std::fmt::Debug for GraphBuilder<N, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphBuilder")
            .field("graph_type", &self.graph_type)
            .field("name", &self.name)
            .field("description", &self.description)
            .field("event_handlers_count", &self.event_handlers.len())
            .finish()
    }
}

impl<N: Node, E: Edge> GraphBuilder<N, E> {
    /// Create a new graph builder
    pub fn new() -> Self {
        Self {
            graph_type: GraphType::Generic,
            name: None,
            description: None,
            event_handlers: Vec::new(),
            _phantom: PhantomData,
        }
    }

    /// Set the graph type
    pub fn graph_type(mut self, graph_type: GraphType) -> Self {
        self.graph_type = graph_type;
        self
    }

    /// Set the graph name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the graph description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    
    /// Add an event handler
    pub fn add_handler(mut self, handler: Arc<dyn EventHandler>) -> Self {
        self.event_handlers.push(handler);
        self
    }

    /// Build a basic graph (for backwards compatibility)
    pub fn build(self) -> Result<BasicGraph<N, E>> {
        let mut graph = BasicGraph::new(self.graph_type);

        if let Some(name) = self.name {
            graph.metadata_mut().name = Some(name);
        }

        if let Some(description) = self.description {
            graph.metadata_mut().description = Some(description);
        }

        Ok(graph)
    }
    
    /// Build an event-driven graph using petgraph
    pub fn build_event(self) -> Result<EventGraph<N, E>> 
    where 
        N: Clone,
        E: Clone,
    {
        let mut graph = EventGraph::new(self.graph_type);

        if let Some(name) = self.name {
            graph.metadata_mut().name = Some(name);
        }

        if let Some(description) = self.description {
            graph.metadata_mut().description = Some(description);
        }
        
        // Add event handlers
        for handler in self.event_handlers {
            graph.add_handler(handler);
        }

        Ok(graph)
    }
}

impl<N: Node, E: Edge> Default for GraphBuilder<N, E> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::edge::GenericEdge;
    use crate::core::node::GenericNode;

    type TestGraph = BasicGraph<GenericNode<String>, GenericEdge<()>>;

    #[test]
    fn test_builder_default() {
        let graph: TestGraph = GraphBuilder::new().build().unwrap();
        assert_eq!(graph.graph_type(), GraphType::Generic);
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_builder_with_metadata() {
        let graph: TestGraph = GraphBuilder::new()
            .name("Test Graph")
            .description("A graph for testing")
            .graph_type(GraphType::WorkflowGraph)
            .build()
            .unwrap();

        assert_eq!(graph.graph_type(), GraphType::WorkflowGraph);
        assert_eq!(graph.metadata().name, Some("Test Graph".to_string()));
        assert_eq!(
            graph.metadata().description,
            Some("A graph for testing".to_string())
        );
    }

    #[test]
    fn test_builder_creates_unique_ids() {
        let graph1: TestGraph = GraphBuilder::new().build().unwrap();
        let graph2: TestGraph = GraphBuilder::new().build().unwrap();

        assert_ne!(graph1.id(), graph2.id());
    }
}
