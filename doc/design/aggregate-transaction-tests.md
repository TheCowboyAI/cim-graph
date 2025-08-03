# CIM Graph Aggregate Transaction Acceptance Tests

## Overview

This document defines comprehensive acceptance tests for each aggregate transaction, ensuring that correct state transitions occur and expected events are produced.

## Test Framework

### Base Test Structure

```rust
/// Framework for testing aggregate transactions
pub trait AggregateTransactionTest {
    type Aggregate;
    type Command;
    type Event;
    type State;
    
    /// Execute command and verify events
    fn test_transaction(
        initial_state: Self::State,
        command: Self::Command,
        expected_events: Vec<Self::Event>,
        expected_state: Self::State,
    ) {
        // Arrange
        let mut aggregate = Self::Aggregate::from_state(initial_state);
        let mut event_bus = MockEventBus::new();
        
        // Act
        let result = aggregate.handle_command(command, &mut event_bus);
        
        // Assert
        assert!(result.is_ok());
        let produced_events = event_bus.captured_events();
        assert_events_equal(&produced_events, &expected_events);
        assert_eq!(aggregate.state(), expected_state);
        
        // Verify event sourcing
        let restored = Self::Aggregate::from_events(&produced_events);
        assert_eq!(restored.state(), expected_state);
    }
}
```

## Graph Aggregate Transaction Tests

### Test: Create Graph Transaction

```rust
#[test]
fn test_create_graph_transaction_produces_correct_events() {
    // Given: No existing graph
    let initial_state = GraphState::Uninitialized;
    
    // When: CreateGraph command is executed
    let command = CreateGraphCommand {
        graph_type: GraphType::IpldGraph,
        constraints: vec![
            Constraint::Acyclic,
            Constraint::MaxNodes(1000),
        ],
        metadata: GraphMetadata {
            name: "test-graph".to_string(),
            description: Some("Test IPLD graph".to_string()),
            tags: vec!["test", "ipld"],
        },
    };
    
    // Then: Expected events are produced
    let expected_events = vec![
        GraphEvent::GraphCreated {
            graph_id: GraphId::new(),
            graph_type: GraphType::IpldGraph,
            timestamp: Timestamp::now(),
        },
        GraphEvent::MetadataInitialized {
            graph_id: GraphId::new(),
            metadata: command.metadata.clone(),
            version: Version::new(1, 0, 0),
        },
        GraphEvent::ConstraintsApplied {
            graph_id: GraphId::new(),
            constraints: command.constraints.clone(),
            validation_result: ValidationResult::Valid,
        },
        GraphEvent::GraphActivated {
            graph_id: GraphId::new(),
            ready_at: Timestamp::now(),
        },
    ];
    
    // And: Final state is Active
    let expected_state = GraphState::Active {
        id: GraphId::new(),
        graph_type: GraphType::IpldGraph,
        node_count: 0,
        edge_count: 0,
        constraints: command.constraints.clone(),
        version: Version::new(1, 0, 0),
    };
    
    test_graph_transaction(initial_state, command, expected_events, expected_state);
}
```

### Test: Add Node Transaction

```rust
#[test]
fn test_add_node_transaction_with_constraints() {
    // Given: Active graph with constraints
    let graph_id = GraphId::new();
    let initial_state = GraphState::Active {
        id: graph_id.clone(),
        graph_type: GraphType::ContextGraph,
        node_count: 5,
        edge_count: 4,
        constraints: vec![
            Constraint::MaxNodes(10),
            Constraint::NodeValidator(Box::new(|node| {
                node.data().contains_key("entity_type")
            })),
        ],
        version: Version::new(1, 0, 5),
    };
    
    // When: Valid node is added
    let command = AddNodeCommand {
        graph_id: graph_id.clone(),
        node_data: NodeData::from_json(json!({
            "entity_type": "User",
            "name": "John Doe",
            "id": "user-123"
        })),
    };
    
    // Then: Node addition events are produced
    let node_id = NodeId::new();
    let expected_events = vec![
        GraphEvent::NodeAdded {
            graph_id: graph_id.clone(),
            node_id: node_id.clone(),
            node_data: command.node_data.clone(),
            timestamp: Timestamp::now(),
        },
        GraphEvent::GraphStructureUpdated {
            graph_id: graph_id.clone(),
            node_count: 6,
            edge_count: 4,
            version: Version::new(1, 0, 6),
        },
        GraphEvent::IndexesUpdated {
            graph_id: graph_id.clone(),
            index_type: IndexType::NodeIndex,
            affected_nodes: vec![node_id.clone()],
        },
        GraphEvent::MetricsRecalculated {
            graph_id: graph_id.clone(),
            metrics: GraphMetrics {
                node_count: 6,
                edge_count: 4,
                density: 0.27, // 4 edges / (6 * 5 / 2) possible
                diameter: None,
                clustering_coefficient: 0.0,
            },
        },
    ];
    
    // And: State is updated
    let expected_state = GraphState::Active {
        id: graph_id,
        graph_type: GraphType::ContextGraph,
        node_count: 6,
        edge_count: 4,
        constraints: initial_state.constraints.clone(),
        version: Version::new(1, 0, 6),
    };
    
    test_graph_transaction(initial_state, command, expected_events, expected_state);
}
```

### Test: Connect Nodes Transaction

```rust
#[test]
fn test_connect_nodes_creates_edge_and_updates_paths() {
    // Given: Graph with two disconnected nodes
    let graph_id = GraphId::new();
    let node_a = NodeId::from("node-a");
    let node_b = NodeId::from("node-b");
    
    let initial_state = GraphState::Active {
        id: graph_id.clone(),
        graph_type: GraphType::WorkflowGraph,
        node_count: 2,
        edge_count: 0,
        constraints: vec![Constraint::Acyclic],
        version: Version::new(1, 0, 2),
    };
    
    // When: Nodes are connected
    let command = ConnectNodesCommand {
        graph_id: graph_id.clone(),
        source: node_a.clone(),
        target: node_b.clone(),
        edge_data: EdgeData::from_json(json!({
            "transition": "approve",
            "guard": "has_permission",
        })),
    };
    
    // Then: Edge creation events are produced
    let edge_id = EdgeId::new();
    let expected_events = vec![
        GraphEvent::EdgeAdded {
            graph_id: graph_id.clone(),
            edge_id: edge_id.clone(),
            source: node_a.clone(),
            target: node_b.clone(),
            edge_data: command.edge_data.clone(),
            timestamp: Timestamp::now(),
        },
        GraphEvent::NodesConnected {
            graph_id: graph_id.clone(),
            source: node_a.clone(),
            target: node_b.clone(),
            connection_type: ConnectionType::Directed,
        },
        GraphEvent::PathsUpdated {
            graph_id: graph_id.clone(),
            new_paths: vec![
                GraphPath {
                    nodes: vec![node_a.clone(), node_b.clone()],
                    total_weight: 1.0,
                    is_cycle: false,
                }
            ],
            removed_paths: vec![],
        },
        GraphEvent::ComponentsRecalculated {
            graph_id: graph_id.clone(),
            component_count: 1, // Now connected
            largest_component_size: 2,
        },
    ];
    
    test_graph_transaction(initial_state, command, expected_events, expected_state);
}
```

### Test: Constraint Violation Transaction

```rust
#[test]
fn test_constraint_violation_rejects_operation() {
    // Given: Graph with acyclic constraint
    let graph_id = GraphId::new();
    let nodes = vec!["A", "B", "C"].into_iter()
        .map(NodeId::from)
        .collect::<Vec<_>>();
    
    let initial_state = GraphState::Active {
        id: graph_id.clone(),
        graph_type: GraphType::WorkflowGraph,
        node_count: 3,
        edge_count: 2, // A->B->C
        constraints: vec![Constraint::Acyclic],
        version: Version::new(1, 0, 5),
    };
    
    // When: Attempting to create cycle C->A
    let command = ConnectNodesCommand {
        graph_id: graph_id.clone(),
        source: nodes[2].clone(), // C
        target: nodes[0].clone(), // A
        edge_data: EdgeData::default(),
    };
    
    // Then: Operation is rejected with violation event
    let expected_events = vec![
        GraphEvent::ConstraintViolated {
            graph_id: graph_id.clone(),
            constraint: Constraint::Acyclic,
            violation: ConstraintViolation {
                constraint_type: "Acyclic",
                message: "Adding edge would create cycle: A -> B -> C -> A",
                severity: Severity::Error,
                affected_elements: vec![
                    Element::Node(nodes[0].clone()),
                    Element::Node(nodes[1].clone()),
                    Element::Node(nodes[2].clone()),
                ],
            },
            timestamp: Timestamp::now(),
        },
        GraphEvent::OperationRejected {
            graph_id: graph_id.clone(),
            operation: Operation::AddEdge {
                source: nodes[2].clone(),
                target: nodes[0].clone(),
            },
            reason: "Constraint violation: Acyclic",
        },
    ];
    
    // And: State remains unchanged
    let expected_state = initial_state.clone();
    
    test_graph_transaction(initial_state, command, expected_events, expected_state);
}
```

## ComposedGraph Aggregate Transaction Tests

### Test: Complete Composition Flow

```rust
#[test]
fn test_graph_composition_transaction_flow() {
    // Given: Two compatible graphs to compose
    let ipld_graph_id = GraphId::from("ipld-graph");
    let context_graph_id = GraphId::from("context-graph");
    
    let initial_state = ComposedGraphState::Planning;
    
    // When: Composition is initiated
    let command = ComposeGraphsCommand {
        source_graphs: vec![ipld_graph_id.clone(), context_graph_id.clone()],
        mappings: vec![
            GraphMapping {
                source: MappingEndpoint {
                    graph_id: ipld_graph_id.clone(),
                    node_id: NodeId::from("cid-123"),
                },
                target: MappingEndpoint {
                    graph_id: context_graph_id.clone(),
                    node_id: NodeId::from("entity-456"),
                },
                mapping_type: MappingType::References,
                bidirectional: false,
            },
        ],
        composition_metadata: CompositionMetadata {
            name: "IPLD-Context Bridge",
            purpose: "Link content to domain entities",
        },
    };
    
    // Then: Full composition event sequence is produced
    let composition_id = GraphId::new();
    let expected_events = vec![
        ComposedGraphEvent::CompositionStarted {
            composition_id: composition_id.clone(),
            source_graphs: vec![ipld_graph_id.clone(), context_graph_id.clone()],
            timestamp: Timestamp::now(),
        },
        ComposedGraphEvent::GraphsValidated {
            composition_id: composition_id.clone(),
            validation_results: vec![
                ValidationResult::valid(ipld_graph_id.clone()),
                ValidationResult::valid(context_graph_id.clone()),
            ],
        },
        ComposedGraphEvent::ReadyForMapping {
            composition_id: composition_id.clone(),
        },
        ComposedGraphEvent::MappingsCreated {
            composition_id: composition_id.clone(),
            mappings: command.mappings.clone(),
            validated_count: 1,
        },
        ComposedGraphEvent::CrossGraphEdgesEstablished {
            composition_id: composition_id.clone(),
            edge_count: 1,
            failed_mappings: vec![],
        },
        ComposedGraphEvent::CompositionCompleted {
            graph_id: composition_id.clone(),
            subgraph_count: 2,
            total_mappings: 1,
            composition_type: CompositionType::Heterogeneous,
        },
        ComposedGraphEvent::ComposedGraphActivated {
            graph_id: composition_id.clone(),
            metadata: command.composition_metadata.clone(),
        },
    ];
    
    // And: Final state is Active
    let expected_state = ComposedGraphState::Active {
        id: composition_id,
        subgraphs: vec![ipld_graph_id, context_graph_id],
        mappings: command.mappings,
        metadata: command.composition_metadata,
    };
    
    test_composed_graph_transaction(initial_state, command, expected_events, expected_state);
}
```

## Node Aggregate Transaction Tests

### Test: Node Lifecycle Transaction

```rust
#[test]
fn test_node_lifecycle_from_creation_to_removal() {
    // Test complete node lifecycle
    let node_id = NodeId::new();
    let graph_id = GraphId::new();
    
    // Phase 1: Node Creation
    let create_events = test_transaction(
        NodeState::Created,
        AddToGraphCommand { graph_id: graph_id.clone(), node_data: test_data() },
        vec![
            NodeEvent::NodeAddedToGraph {
                node_id: node_id.clone(),
                graph_id: graph_id.clone(),
                node_data: test_data(),
            },
        ],
        NodeState::Active { id: node_id.clone(), graph_id: graph_id.clone() },
    );
    
    // Phase 2: Node Connection
    let target_node = NodeId::new();
    let connect_events = test_transaction(
        NodeState::Active { id: node_id.clone(), graph_id: graph_id.clone() },
        ConnectToNodeCommand { target: target_node.clone(), edge_data: edge_data() },
        vec![
            NodeEvent::NodeConnected {
                source: node_id.clone(),
                target: target_node.clone(),
                edge_id: EdgeId::new(),
            },
            NodeEvent::NodeDegreeChanged {
                node_id: node_id.clone(),
                old_degree: 0,
                new_degree: 1,
            },
        ],
        NodeState::Connected { 
            id: node_id.clone(), 
            graph_id: graph_id.clone(),
            degree: 1 
        },
    );
    
    // Phase 3: Node Removal
    let remove_events = test_transaction(
        NodeState::Connected { id: node_id.clone(), graph_id: graph_id.clone(), degree: 1 },
        RemoveFromGraphCommand { cascade: true },
        vec![
            NodeEvent::EdgesCascadeRemoved {
                node_id: node_id.clone(),
                removed_edges: vec![EdgeId::new()],
            },
            NodeEvent::NodeRemovedFromGraph {
                node_id: node_id.clone(),
                graph_id: graph_id.clone(),
            },
        ],
        NodeState::Removed,
    );
    
    // Verify complete event sequence
    let all_events = [create_events, connect_events, remove_events].concat();
    assert_can_replay_lifecycle(&all_events, node_id);
}
```

## Edge Aggregate Transaction Tests

### Test: Edge Validation and Establishment

```rust
#[test]
fn test_edge_establishment_with_validation() {
    // Given: Proposed edge
    let edge_id = EdgeId::new();
    let source = NodeId::from("source");
    let target = NodeId::from("target");
    
    let initial_state = EdgeState::Proposed {
        id: edge_id.clone(),
        source: source.clone(),
        target: target.clone(),
    };
    
    // When: Edge is validated and established
    let validation_events = test_transaction(
        initial_state,
        ValidateEdgeCommand {
            check_nodes_exist: true,
            check_constraints: true,
        },
        vec![
            EdgeEvent::EdgeValidated {
                edge_id: edge_id.clone(),
                source: source.clone(),
                target: target.clone(),
                validation_results: ValidationResults {
                    nodes_exist: true,
                    constraints_satisfied: true,
                    warnings: vec![],
                },
            },
        ],
        EdgeState::Validated {
            id: edge_id.clone(),
            source: source.clone(),
            target: target.clone(),
        },
    );
    
    let establish_events = test_transaction(
        EdgeState::Validated { id: edge_id.clone(), source: source.clone(), target: target.clone() },
        EstablishEdgeCommand {
            edge_data: EdgeData::weighted(1.5),
        },
        vec![
            EdgeEvent::EdgeEstablished {
                edge_id: edge_id.clone(),
                source: source.clone(),
                target: target.clone(),
                edge_data: EdgeData::weighted(1.5),
            },
            EdgeEvent::GraphConnectivityChanged {
                graph_id: GraphId::new(),
                components_before: 2,
                components_after: 1,
            },
        ],
        EdgeState::Active {
            id: edge_id.clone(),
            source: source.clone(),
            target: target.clone(),
            weight: 1.5,
        },
    );
}
```

## Constraint Aggregate Transaction Tests

### Test: Constraint Lifecycle

```rust
#[test]
fn test_constraint_enforcement_lifecycle() {
    // Given: New constraint to be applied
    let constraint_id = ConstraintId::new();
    let graph_id = GraphId::new();
    
    // Phase 1: Apply Constraint
    let apply_events = test_transaction(
        ConstraintState::Defined {
            id: constraint_id.clone(),
            constraint_type: ConstraintType::MaxDegree(3),
        },
        ApplyConstraintCommand {
            graph_id: graph_id.clone(),
        },
        vec![
            ConstraintEvent::ConstraintApplied {
                constraint_id: constraint_id.clone(),
                graph_id: graph_id.clone(),
                constraint_type: ConstraintType::MaxDegree(3),
            },
        ],
        ConstraintState::Active {
            id: constraint_id.clone(),
            graph_id: graph_id.clone(),
            constraint_type: ConstraintType::MaxDegree(3),
        },
    );
    
    // Phase 2: Constraint Violation
    let violation_events = test_transaction(
        ConstraintState::Active { 
            id: constraint_id.clone(),
            graph_id: graph_id.clone(),
            constraint_type: ConstraintType::MaxDegree(3),
        },
        EvaluateConstraintCommand {
            operation: Operation::AddEdge {
                to_node: NodeId::from("high-degree-node"),
            },
        },
        vec![
            ConstraintEvent::ConstraintEvaluationStarted {
                constraint_id: constraint_id.clone(),
                operation: Operation::AddEdge {
                    to_node: NodeId::from("high-degree-node"),
                },
            },
            ConstraintEvent::ConstraintViolated {
                constraint_id: constraint_id.clone(),
                violation: ConstraintViolation {
                    constraint_type: "MaxDegree",
                    message: "Node would exceed maximum degree of 3",
                    severity: Severity::Error,
                    affected_elements: vec![Element::Node(NodeId::from("high-degree-node"))],
                },
            },
            ConstraintEvent::GraphOperationRejected {
                graph_id: graph_id.clone(),
                operation: Operation::AddEdge {
                    to_node: NodeId::from("high-degree-node"),
                },
                reason: "MaxDegree constraint violation",
            },
        ],
        ConstraintState::Violated {
            id: constraint_id.clone(),
            graph_id: graph_id.clone(),
            constraint_type: ConstraintType::MaxDegree(3),
            violation: Some(violation_details()),
        },
    );
}
```

## Transformation Aggregate Transaction Tests

### Test: Complete Transformation Flow

```rust
#[test]
fn test_graph_transformation_transaction() {
    // Given: Configured transformation
    let transformation_id = TransformationId::new();
    let source_graph = GraphId::from("workflow-graph");
    let target_type = GraphType::ConceptGraph;
    
    // Complete transformation flow
    let events = test_transformation_flow(
        TransformationState::Configured {
            id: transformation_id.clone(),
            source_graph: source_graph.clone(),
            target_type: target_type.clone(),
        },
        TransformGraphCommand {
            node_mapper: Box::new(|workflow_node| {
                ConceptNode::from_workflow(workflow_node)
            }),
            edge_mapper: Box::new(|workflow_edge| {
                SemanticRelation::from_transition(workflow_edge)
            }),
        },
        vec![
            // Analysis Phase
            TransformationEvent::TransformationStarted {
                transformation_id: transformation_id.clone(),
                source_graph: source_graph.clone(),
                target_type: target_type.clone(),
            },
            TransformationEvent::TransformationAnalysisCompleted {
                transformation_id: transformation_id.clone(),
                feasibility: FeasibilityReport::feasible(),
                node_mappings: vec![/* mapping details */],
                edge_mappings: vec![/* mapping details */],
            },
            
            // Mapping Phase
            TransformationEvent::MappingsEstablished {
                transformation_id: transformation_id.clone(),
                node_mapping_count: 10,
                edge_mapping_count: 15,
            },
            
            // Transformation Phase
            TransformationEvent::NodesTransformed {
                transformation_id: transformation_id.clone(),
                nodes_processed: 10,
                nodes_transformed: 10,
                failures: vec![],
            },
            TransformationEvent::EdgesTransformed {
                transformation_id: transformation_id.clone(),
                edges_processed: 15,
                edges_transformed: 15,
                failures: vec![],
            },
            
            // Completion Phase
            TransformationEvent::TransformationCompleted {
                transformation_id: transformation_id.clone(),
                nodes_transformed: 10,
                edges_transformed: 15,
            },
            TransformationEvent::TargetGraphCreated {
                source_graph: source_graph.clone(),
                target_graph: GraphId::new(),
                provenance: TransformationProvenance {
                    transformation_id: transformation_id.clone(),
                    source_type: GraphType::WorkflowGraph,
                    target_type: GraphType::ConceptGraph,
                    timestamp: Timestamp::now(),
                },
            },
        ],
    );
}
```

## Property-Based Transaction Tests

### Test: Event Sourcing Properties

```rust
#[property_test]
fn prop_events_can_rebuild_aggregate_state(
    commands: Vec<GraphCommand>
) {
    // Property: Any sequence of valid commands produces events
    // that can rebuild the exact same state
    
    let mut aggregate = GraphAggregate::new();
    let mut all_events = Vec::new();
    
    // Execute all commands
    for command in commands {
        if let Ok(events) = aggregate.handle_command(command) {
            all_events.extend(events);
        }
    }
    
    // Rebuild from events
    let rebuilt = GraphAggregate::from_events(&all_events);
    
    // States must be identical
    assert_eq!(aggregate.state(), rebuilt.state());
    assert_eq!(aggregate.version(), rebuilt.version());
}

#[property_test]
fn prop_invalid_commands_produce_no_events(
    aggregate: GraphAggregate,
    invalid_command: InvalidCommand
) {
    // Property: Invalid commands never produce events
    let initial_state = aggregate.state().clone();
    let result = aggregate.handle_command(invalid_command.into());
    
    assert!(result.is_err());
    assert_eq!(aggregate.state(), initial_state);
}

#[property_test]
fn prop_events_maintain_causality(
    commands: Vec<GraphCommand>
) {
    // Property: Events maintain causal ordering
    let mut aggregate = GraphAggregate::new();
    let mut previous_timestamp = Timestamp::zero();
    
    for command in commands {
        if let Ok(events) = aggregate.handle_command(command) {
            for event in events {
                assert!(event.timestamp > previous_timestamp);
                previous_timestamp = event.timestamp;
            }
        }
    }
}
```

## Integration Test Helpers

### Event Assertion Utilities

```rust
/// Assert that events match expected patterns
pub fn assert_events_match<E: Event>(
    actual: &[E],
    expected: &[E],
) {
    assert_eq!(actual.len(), expected.len(), 
        "Event count mismatch: expected {}, got {}", 
        expected.len(), actual.len()
    );
    
    for (i, (actual_event, expected_event)) in 
        actual.iter().zip(expected.iter()).enumerate() 
    {
        assert_event_equal(actual_event, expected_event)
            .unwrap_or_else(|e| {
                panic!("Event {} mismatch: {}", i, e)
            });
    }
}

/// Assert event equality with helpful error messages
pub fn assert_event_equal<E: Event>(
    actual: &E,
    expected: &E,
) -> Result<(), String> {
    // Compare event types
    if actual.event_type() != expected.event_type() {
        return Err(format!(
            "Event type mismatch: expected {:?}, got {:?}",
            expected.event_type(),
            actual.event_type()
        ));
    }
    
    // Compare payloads (ignoring timestamps and IDs)
    let actual_payload = normalize_event_payload(actual);
    let expected_payload = normalize_event_payload(expected);
    
    if actual_payload != expected_payload {
        return Err(format!(
            "Event payload mismatch:\nExpected: {:?}\nActual: {:?}",
            expected_payload,
            actual_payload
        ));
    }
    
    Ok(())
}
```

### Test Data Builders

```rust
/// Builder for test graphs
pub struct TestGraphBuilder {
    graph_type: GraphType,
    nodes: Vec<(NodeId, NodeData)>,
    edges: Vec<(NodeId, NodeId, EdgeData)>,
    constraints: Vec<Constraint>,
}

impl TestGraphBuilder {
    pub fn workflow_graph() -> Self {
        Self {
            graph_type: GraphType::WorkflowGraph,
            ..Default::default()
        }
    }
    
    pub fn with_cycle(mut self) -> Self {
        self.nodes = vec![
            (NodeId::from("A"), NodeData::default()),
            (NodeId::from("B"), NodeData::default()),
            (NodeId::from("C"), NodeData::default()),
        ];
        self.edges = vec![
            (NodeId::from("A"), NodeId::from("B"), EdgeData::default()),
            (NodeId::from("B"), NodeId::from("C"), EdgeData::default()),
            (NodeId::from("C"), NodeId::from("A"), EdgeData::default()),
        ];
        self
    }
    
    pub fn build(self) -> GraphAggregate {
        let mut aggregate = GraphAggregate::new();
        // Build graph from components
        aggregate
    }
}
```

## Next Steps

1. Implement test framework infrastructure
2. Create mock event bus for testing
3. Build test data generators
4. Set up property-based testing
5. Create CI pipeline for transaction tests