# Graph Domain API Specification

> The API that brings the Graph Domain to life through event-driven, semantically-aware operations.

## API Design Principles

1. **Event-First**: All mutations through events
2. **Semantic Clarity**: APIs express domain meaning
3. **Type Safety**: Leverage Rust's type system
4. **Composability**: Operations can be combined
5. **Observable**: All operations emit events for monitoring

## Core API Structure

```rust
/// Main entry point for Graph Domain
pub struct GraphDomain {
    registry: GraphRegistry,
    event_store: EventStore,
    composer: GraphComposer,
    validator: SemanticValidator,
}

impl GraphDomain {
    /// Create a new graph
    pub async fn create_graph<G: CimGraph>(
        &mut self,
        spec: GraphSpecification<G>,
    ) -> Result<GraphHandle<G>, GraphError> {
        // Emit GraphCreationRequested event
        let event = GraphCreationRequested {
            correlation_id: Uuid::new_v4(),
            spec: spec.clone(),
            timestamp: Utc::now(),
        };
        
        self.event_store.append(event).await?;
        
        // Create graph through event handling
        let graph = G::from_specification(spec)?;
        let handle = self.registry.register(graph).await?;
        
        // Emit GraphCreated event
        self.event_store.append(GraphCreated {
            graph_id: handle.id(),
            graph_type: G::semantic_type(),
        }).await?;
        
        Ok(handle)
    }
}
```

## Graph Handles

Graphs are accessed through handles that provide type-safe operations:

```rust
/// Type-safe handle to a living graph
pub struct GraphHandle<G: CimGraph> {
    id: GraphId,
    graph_type: PhantomData<G>,
    event_channel: EventChannel,
}

impl<G: CimGraph> GraphHandle<G> {
    /// Add a node to the graph
    pub async fn add_node(
        &mut self,
        node: G::Node,
        context: SemanticContext,
    ) -> Result<NodeId, GraphError> {
        let event = NodeAdditionRequested {
            graph_id: self.id,
            node_data: node.into(),
            semantic_context: context,
        };
        
        self.event_channel.send(event).await?;
        
        // Wait for confirmation
        let confirmation = self.event_channel
            .receive::<NodeAdded>()
            .await?;
            
        Ok(confirmation.node_id)
    }
    
    /// Query the graph
    pub async fn query<Q: GraphQuery>(
        &self,
        query: Q,
    ) -> Result<Q::Result, QueryError> {
        let event = QueryRequested {
            graph_id: self.id,
            query: query.into(),
        };
        
        self.event_channel.send(event).await?;
        
        let result = self.event_channel
            .receive::<QueryCompleted<Q::Result>>()
            .await?;
            
        Ok(result.data)
    }
}
```

## Fluent API for Graph Construction

```rust
/// Builder pattern for graph construction
pub struct GraphBuilder<G: CimGraph> {
    nodes: Vec<G::Node>,
    edges: Vec<G::Edge>,
    metadata: G::Metadata,
}

impl<G: CimGraph> GraphBuilder<G> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            metadata: G::Metadata::default(),
        }
    }
    
    pub fn with_node(mut self, node: G::Node) -> Self {
        self.nodes.push(node);
        self
    }
    
    pub fn with_edge(mut self, from: NodeId, to: NodeId, relationship: G::Edge) -> Self {
        self.edges.push(relationship);
        self
    }
    
    pub fn with_metadata(mut self, metadata: G::Metadata) -> Self {
        self.metadata = metadata;
        self
    }
    
    pub async fn build(self, domain: &mut GraphDomain) -> Result<GraphHandle<G>, GraphError> {
        let spec = GraphSpecification {
            nodes: self.nodes,
            edges: self.edges,
            metadata: self.metadata,
        };
        
        domain.create_graph(spec).await
    }
}
```

## Semantic Graph Operations

### IPLD Graph API

```rust
/// API specific to IPLD graphs
impl GraphHandle<IpldGraph> {
    /// Add content-addressed data
    pub async fn add_content<T: Serialize>(
        &mut self,
        content: T,
    ) -> Result<Cid, IpldError> {
        let bytes = serde_ipld::to_vec(&content)?;
        let cid = Cid::from_bytes(&bytes);
        
        let node = IpldNode::new(cid.clone(), content);
        self.add_node(node, SemanticContext::content_addressing()).await?;
        
        Ok(cid)
    }
    
    /// Resolve an IPLD path
    pub async fn resolve_path(
        &self,
        path: &str,
    ) -> Result<ResolvedContent, IpldError> {
        let query = IpldPathQuery::new(path);
        self.query(query).await
    }
}
```

### Context Graph API

```rust
/// API specific to Context graphs
impl GraphHandle<ContextGraph> {
    /// Define a bounded context
    pub async fn define_context(
        &mut self,
        context: BoundedContext,
    ) -> Result<ContextId, ContextError> {
        let node = ContextNode::BoundedContext(context);
        self.add_node(node, SemanticContext::ddd()).await
    }
    
    /// Add an aggregate
    pub async fn add_aggregate<A: AggregateRoot>(
        &mut self,
        aggregate: A,
        context_id: ContextId,
    ) -> Result<AggregateId, ContextError> {
        let node = ContextNode::Aggregate(aggregate.into());
        let id = self.add_node(node, SemanticContext::ddd()).await?;
        
        // Connect to context
        self.add_edge(
            id,
            context_id,
            ContextRelationship::BelongsTo,
        ).await?;
        
        Ok(AggregateId(id))
    }
}
```

### Workflow Graph API

```rust
/// API specific to Workflow graphs
impl GraphHandle<WorkflowGraph> {
    /// Define a workflow state
    pub async fn add_state(
        &mut self,
        state: WorkflowState,
    ) -> Result<StateId, WorkflowError> {
        let node = WorkflowNode::State(state);
        self.add_node(node, SemanticContext::workflow()).await
    }
    
    /// Define a transition
    pub async fn add_transition(
        &mut self,
        from: StateId,
        to: StateId,
        event: WorkflowEvent,
    ) -> Result<TransitionId, WorkflowError> {
        let edge = WorkflowEdge::Transition(event);
        self.add_edge(from.into(), to.into(), edge).await
    }
    
    /// Execute workflow
    pub async fn execute(
        &mut self,
        instance: WorkflowInstance,
    ) -> Result<ExecutionHandle, WorkflowError> {
        let execution = WorkflowExecution::start(instance, self.id);
        Ok(execution.handle())
    }
}
```

### Concept Graph API

```rust
/// API specific to Concept graphs
impl GraphHandle<ConceptGraph> {
    /// Add a concept
    pub async fn add_concept(
        &mut self,
        concept: Concept,
        position: ConceptualPosition,
    ) -> Result<ConceptId, ConceptError> {
        let node = ConceptNode::new(concept, position);
        self.add_node(node, SemanticContext::conceptual()).await
    }
    
    /// Find similar concepts
    pub async fn find_similar(
        &self,
        concept: &Concept,
        threshold: f64,
    ) -> Result<Vec<(Concept, f64)>, ConceptError> {
        let query = SemanticSimilarityQuery::new(concept, threshold);
        self.query(query).await
    }
}
```

## Graph Composition API

```rust
/// Compose multiple graphs
pub struct GraphComposer {
    domain: Arc<GraphDomain>,
}

impl GraphComposer {
    /// Merge two graphs
    pub async fn merge<A, B>(
        &self,
        graph_a: GraphHandle<A>,
        graph_b: GraphHandle<B>,
        strategy: MergeStrategy,
    ) -> Result<GraphHandle<MergedGraph>, CompositionError>
    where
        A: CimGraph + Mergeable,
        B: CimGraph + Mergeable,
    {
        let event = GraphMergeRequested {
            graph_a: graph_a.id,
            graph_b: graph_b.id,
            strategy,
        };
        
        self.domain.handle_event(event).await?;
        
        // Return handle to merged graph
        self.domain.get_merged_graph(graph_a.id, graph_b.id).await
    }
    
    /// Create semantic bridge between graphs
    pub async fn bridge<A, B>(
        &self,
        graph_a: &GraphHandle<A>,
        graph_b: &GraphHandle<B>,
        mapping: SemanticMapping,
    ) -> Result<BridgeHandle, CompositionError>
    where
        A: CimGraph,
        B: CimGraph,
    {
        let bridge = SemanticBridge::new(graph_a.id, graph_b.id, mapping);
        self.domain.register_bridge(bridge).await
    }
}
```

## Query API

```rust
/// Fluent query builder
pub struct QueryBuilder<G: CimGraph> {
    graph: GraphId,
    _phantom: PhantomData<G>,
}

impl<G: CimGraph> QueryBuilder<G> {
    /// Start traversal from nodes
    pub fn from_nodes(self, nodes: Vec<NodeId>) -> TraversalBuilder<G> {
        TraversalBuilder::new(self.graph, nodes)
    }
    
    /// Search by pattern
    pub fn match_pattern(self, pattern: GraphPattern) -> PatternBuilder<G> {
        PatternBuilder::new(self.graph, pattern)
    }
    
    /// Semantic search
    pub fn semantic_search(self, query: SemanticQuery) -> SemanticSearchBuilder<G> {
        SemanticSearchBuilder::new(self.graph, query)
    }
}

/// Example traversal builder
pub struct TraversalBuilder<G: CimGraph> {
    graph: GraphId,
    start: Vec<NodeId>,
    filters: Vec<TraversalFilter>,
    _phantom: PhantomData<G>,
}

impl<G: CimGraph> TraversalBuilder<G> {
    pub fn follow_edges(mut self, predicate: EdgePredicate) -> Self {
        self.filters.push(TraversalFilter::Edge(predicate));
        self
    }
    
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.filters.push(TraversalFilter::MaxDepth(depth));
        self
    }
    
    pub async fn execute(self) -> Result<TraversalResult<G>, QueryError> {
        let query = TraversalQuery {
            graph: self.graph,
            start_nodes: self.start,
            filters: self.filters,
        };
        
        // Execute through domain
        todo!()
    }
}
```

## Subscription API

```rust
/// Subscribe to graph events
pub struct GraphSubscription<G: CimGraph> {
    graph: GraphId,
    filters: Vec<EventFilter>,
    _phantom: PhantomData<G>,
}

impl<G: CimGraph> GraphSubscription<G> {
    /// Subscribe to specific event types
    pub fn on_node_added(mut self) -> Self {
        self.filters.push(EventFilter::EventType("NodeAdded".into()));
        self
    }
    
    /// Subscribe to semantic changes
    pub fn on_semantic_change(mut self, threshold: f64) -> Self {
        self.filters.push(EventFilter::SemanticChange(threshold));
        self
    }
    
    /// Start receiving events
    pub async fn subscribe(self) -> EventStream<GraphDomainEvent> {
        // Connect to event stream
        todo!()
    }
}
```

## Error Handling

```rust
/// Comprehensive error types
#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error("Graph not found: {0}")]
    NotFound(GraphId),
    
    #[error("Semantic validation failed: {0}")]
    SemanticValidation(String),
    
    #[error("Event handling failed: {0}")]
    EventHandling(#[from] EventError),
    
    #[error("Composition not allowed: {reason}")]
    CompositionDenied { reason: String },
}
```

## Example Usage

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut domain = GraphDomain::new().await?;
    
    // Create a workflow graph
    let workflow = GraphBuilder::<WorkflowGraph>::new()
        .with_node(WorkflowNode::State(WorkflowState::Start))
        .with_node(WorkflowNode::State(WorkflowState::Processing))
        .with_node(WorkflowNode::State(WorkflowState::Complete))
        .build(&mut domain)
        .await?;
    
    // Create a context graph
    let context = GraphBuilder::<ContextGraph>::new()
        .with_node(ContextNode::BoundedContext(
            BoundedContext::new("OrderManagement")
        ))
        .build(&mut domain)
        .await?;
    
    // Bridge them semantically
    let bridge = domain.composer()
        .bridge(&workflow, &context, SemanticMapping::default())
        .await?;
    
    // Query across both graphs
    let results = workflow
        .query(CrossGraphQuery::new(bridge))
        .await?;
    
    Ok(())
}
```

This API provides:
1. **Event-Driven Operations**: All mutations flow through events
2. **Type-Safe Handles**: Graph operations are type-checked
3. **Fluent Interfaces**: Intuitive API design
4. **Semantic Awareness**: Operations preserve domain meaning
5. **Composition Support**: Graphs can be combined and bridged