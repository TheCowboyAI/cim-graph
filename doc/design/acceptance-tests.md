# CIM Graph Acceptance Tests

## Overview

This document defines acceptance tests as executable graph specifications. Each test is represented as a graph transformation that can be validated programmatically.

## Test Graph Structure

```rust
struct AcceptanceTest {
    id: String,
    story_id: String,
    criterion_id: String,
    test_graph: TestGraph,
    assertions: Vec<Assertion>,
}

struct TestGraph {
    initial_state: Graph,
    actions: Vec<Action>,
    expected_state: Graph,
    invariants: Vec<Invariant>,
}

struct Assertion {
    path: GraphPath,
    condition: Condition,
    expected: Value,
}
```

## Acceptance Test Specifications

### AT-001: Empty Graph Creation

```graph
AcceptanceTest {
    id: "AT-001-1",
    story_id: "US-001",
    criterion_id: "AC-001-1",
    test_graph: {
        initial_state: null,
        actions: [
            Action::CreateGraph { type: "Generic" }
        ],
        expected_state: {
            nodes: [],
            edges: [],
            metadata: {
                created: "${timestamp}",
                version: "1.0.0",
                type: "Generic"
            }
        },
        invariants: [
            "node_count == 0",
            "edge_count == 0",
            "metadata.created <= now()"
        ]
    },
    assertions: [
        {
            path: "$.nodes.length",
            condition: "equals",
            expected: 0
        },
        {
            path: "$.edges.length",
            condition: "equals",
            expected: 0
        },
        {
            path: "$.metadata.type",
            condition: "equals",
            expected: "Generic"
        }
    ]
}
```

### AT-002: Typed Graph Creation

```graph
AcceptanceTest {
    id: "AT-001-2",
    story_id: "US-001",
    criterion_id: "AC-001-2",
    test_graph: {
        initial_state: null,
        actions: [
            Action::CreateGraph { 
                type: "IpldGraph",
                constraints: {
                    node_type: "Cid",
                    edge_type: "Transition"
                }
            }
        ],
        expected_state: {
            type: "IpldGraph",
            constraints: {
                node_type: "Cid",
                edge_type: "Transition",
                validation: "strict"
            }
        },
        invariants: [
            "all_nodes.type == Cid",
            "all_edges.type == Transition"
        ]
    },
    assertions: [
        {
            path: "$.type",
            condition: "equals",
            expected: "IpldGraph"
        },
        {
            path: "$.constraints.node_type",
            condition: "equals",
            expected: "Cid"
        }
    ]
}
```

### AT-003: Node Addition

```graph
AcceptanceTest {
    id: "AT-002-1",
    story_id: "US-002",
    criterion_id: "AC-002-1",
    test_graph: {
        initial_state: {
            nodes: [],
            edges: []
        },
        actions: [
            Action::AddNode {
                data: { value: "test_node", metadata: {} }
            }
        ],
        expected_state: {
            nodes: [
                {
                    id: "${generated_id}",
                    data: { value: "test_node", metadata: {} }
                }
            ],
            edges: []
        },
        invariants: [
            "node_count == 1",
            "edge_count == 0",
            "nodes[0].id != null",
            "nodes[0].id matches /^[a-zA-Z0-9-]+$/"
        ]
    },
    assertions: [
        {
            path: "$.nodes.length",
            condition: "equals",
            expected: 1
        },
        {
            path: "$.nodes[0].data.value",
            condition: "equals",
            expected: "test_node"
        },
        {
            path: "$.nodes[0].id",
            condition: "matches",
            expected: "^[a-zA-Z0-9-]+$"
        }
    ]
}
```

### AT-004: Type Validation on Node Addition

```graph
AcceptanceTest {
    id: "AT-002-2",
    story_id: "US-002",
    criterion_id: "AC-002-2",
    test_graph: {
        initial_state: {
            type: "IpldGraph",
            constraints: { node_type: "Cid" },
            nodes: [],
            edges: []
        },
        actions: [
            Action::AddNode {
                data: 123  // Invalid type - should be Cid
            }
        ],
        expected_state: "unchanged",
        invariants: [
            "error.type == TypeMismatch",
            "node_count == 0"
        ]
    },
    assertions: [
        {
            path: "$.error",
            condition: "exists",
            expected: true
        },
        {
            path: "$.error.type",
            condition: "equals",
            expected: "TypeMismatch"
        },
        {
            path: "$.error.message",
            condition: "contains",
            expected: "Expected Cid, got Number"
        }
    ]
}
```

### AT-005: Edge Creation

```graph
AcceptanceTest {
    id: "AT-003-1",
    story_id: "US-003",
    criterion_id: "AC-003-1",
    test_graph: {
        initial_state: {
            nodes: [
                { id: "node_a", data: "A" },
                { id: "node_b", data: "B" }
            ],
            edges: []
        },
        actions: [
            Action::AddEdge {
                from: "node_a",
                to: "node_b",
                data: { weight: 1.0, label: "connects_to" }
            }
        ],
        expected_state: {
            nodes: [
                { id: "node_a", data: "A" },
                { id: "node_b", data: "B" }
            ],
            edges: [
                {
                    id: "${generated_id}",
                    from: "node_a",
                    to: "node_b",
                    data: { weight: 1.0, label: "connects_to" }
                }
            ]
        },
        invariants: [
            "edge_count == 1",
            "edges[0].from == 'node_a'",
            "edges[0].to == 'node_b'"
        ]
    },
    assertions: [
        {
            path: "$.edges.length",
            condition: "equals",
            expected: 1
        },
        {
            path: "$.edges[0].from",
            condition: "equals",
            expected: "node_a"
        },
        {
            path: "$.edges[0].to",
            condition: "equals",
            expected: "node_b"
        },
        {
            path: "$.edges[0].data.weight",
            condition: "equals",
            expected: 1.0
        }
    ]
}
```

### AT-006: Directed Edge Queries

```graph
AcceptanceTest {
    id: "AT-003-2",
    story_id: "US-003",
    criterion_id: "AC-003-2",
    test_graph: {
        initial_state: {
            nodes: ["A", "B", "C"],
            edges: [
                { from: "A", to: "B" },
                { from: "B", to: "C" }
            ]
        },
        actions: [
            Action::Query { type: "edges_from", node: "A" },
            Action::Query { type: "edges_from", node: "B" },
            Action::Query { type: "edges_from", node: "C" },
            Action::Query { type: "edges_to", node: "A" },
            Action::Query { type: "edges_to", node: "B" },
            Action::Query { type: "edges_to", node: "C" }
        ],
        expected_state: {
            query_results: [
                { query: "edges_from(A)", result: [{ to: "B" }] },
                { query: "edges_from(B)", result: [{ to: "C" }] },
                { query: "edges_from(C)", result: [] },
                { query: "edges_to(A)", result: [] },
                { query: "edges_to(B)", result: [{ from: "A" }] },
                { query: "edges_to(C)", result: [{ from: "B" }] }
            ]
        },
        invariants: [
            "directed_edges_preserve_direction",
            "no_implicit_reverse_edges"
        ]
    },
    assertions: [
        {
            path: "$.query_results[0].result.length",
            condition: "equals",
            expected: 1
        },
        {
            path: "$.query_results[2].result.length",
            condition: "equals",
            expected: 0
        }
    ]
}
```

### AT-007: Neighbor Query

```graph
AcceptanceTest {
    id: "AT-004-1",
    story_id: "US-004",
    criterion_id: "AC-004-1",
    test_graph: {
        initial_state: {
            nodes: ["A", "B", "C", "D"],
            edges: [
                { from: "A", to: "B" },
                { from: "A", to: "C" },
                { from: "B", to: "D" }
            ]
        },
        actions: [
            Action::Query { 
                type: "neighbors",
                node: "A",
                direction: "outgoing"
            }
        ],
        expected_state: {
            query_result: {
                node: "A",
                neighbors: ["B", "C"]
            }
        },
        invariants: [
            "neighbors_are_directly_connected",
            "no_duplicate_neighbors"
        ]
    },
    assertions: [
        {
            path: "$.query_result.neighbors",
            condition: "contains_exactly",
            expected: ["B", "C"]
        }
    ]
}
```

### AT-008: Cycle Detection

```graph
AcceptanceTest {
    id: "AT-004-2",
    story_id: "US-004",
    criterion_id: "AC-004-2",
    test_graph: {
        initial_state: {
            nodes: ["A", "B", "C"],
            edges: [
                { from: "A", to: "B" },
                { from: "B", to: "C" },
                { from: "C", to: "A" }
            ]
        },
        actions: [
            Action::Query { type: "has_cycles" },
            Action::Query { type: "find_cycles" }
        ],
        expected_state: {
            query_results: [
                { query: "has_cycles", result: true },
                { query: "find_cycles", result: [["A", "B", "C", "A"]] }
            ]
        },
        invariants: [
            "cycle_path_starts_and_ends_with_same_node",
            "cycle_path_length >= 3"
        ]
    },
    assertions: [
        {
            path: "$.query_results[0].result",
            condition: "equals",
            expected: true
        },
        {
            path: "$.query_results[1].result[0]",
            condition: "equals",
            expected: ["A", "B", "C", "A"]
        }
    ]
}
```

### AT-009: Graph Composition

```graph
AcceptanceTest {
    id: "AT-005-1",
    story_id: "US-005",
    criterion_id: "AC-005-1",
    test_graph: {
        initial_state: {
            graph1: {
                type: "IpldGraph",
                nodes: [{ id: "cid_123", type: "Cid", data: "QmHash..." }],
                edges: []
            },
            graph2: {
                type: "ContextGraph",
                nodes: [{ id: "addr_456", type: "Address", data: "123 Main St" }],
                edges: []
            }
        },
        actions: [
            Action::Compose {
                graphs: ["graph1", "graph2"],
                mappings: [
                    {
                        from_graph: "graph1",
                        from_node: "cid_123",
                        to_graph: "graph2",
                        to_node: "addr_456",
                        mapping_type: "references"
                    }
                ]
            }
        ],
        expected_state: {
            type: "ComposedGraph",
            subgraphs: {
                ipld: { type: "IpldGraph", node_count: 1 },
                context: { type: "ContextGraph", node_count: 1 }
            },
            cross_graph_edges: [
                {
                    from: { graph: "ipld", node: "cid_123" },
                    to: { graph: "context", node: "addr_456" },
                    type: "references"
                }
            ]
        },
        invariants: [
            "composed_graph_preserves_subgraph_types",
            "mappings_create_cross_graph_edges",
            "no_type_mixing_within_subgraphs"
        ]
    },
    assertions: [
        {
            path: "$.type",
            condition: "equals",
            expected: "ComposedGraph"
        },
        {
            path: "$.cross_graph_edges.length",
            condition: "equals",
            expected: 1
        }
    ]
}
```

### AT-010: JSON Serialization

```graph
AcceptanceTest {
    id: "AT-006-1",
    story_id: "US-006",
    criterion_id: "AC-006-1",
    test_graph: {
        initial_state: {
            nodes: [
                { id: "n1", data: { value: "test" } }
            ],
            edges: [
                { from: "n1", to: "n2", data: { weight: 1.5 } }
            ]
        },
        actions: [
            Action::Serialize { format: "json" }
        ],
        expected_state: {
            serialized: {
                format: "json",
                valid_json: true,
                schema: {
                    type: "object",
                    properties: {
                        nodes: { type: "array" },
                        edges: { type: "array" },
                        metadata: { type: "object" }
                    }
                }
            }
        },
        invariants: [
            "json_is_parseable",
            "json_roundtrip_preserves_data",
            "json_schema_valid"
        ]
    },
    assertions: [
        {
            path: "$.serialized.valid_json",
            condition: "equals",
            expected: true
        },
        {
            path: "$.serialized.format",
            condition: "equals",
            expected: "json"
        }
    ]
}
```

### AT-011: Graph Transformation

```graph
AcceptanceTest {
    id: "AT-007-1",
    story_id: "US-007",
    criterion_id: "AC-007-1",
    test_graph: {
        initial_state: {
            type: "WorkflowGraph",
            nodes: [
                { id: "start", type: "WorkflowState", data: { name: "Started" } },
                { id: "end", type: "WorkflowState", data: { name: "Completed" } }
            ],
            edges: [
                { 
                    from: "start", 
                    to: "end", 
                    type: "Event", 
                    data: { event: "finish", duration: 100 } 
                }
            ]
        },
        actions: [
            Action::Transform {
                to_type: "ConceptGraph",
                mapping_rules: {
                    node_transform: "WorkflowState -> Concept",
                    edge_transform: "Event -> SemanticRelation"
                }
            }
        ],
        expected_state: {
            type: "ConceptGraph",
            nodes: [
                { 
                    id: "concept_start", 
                    type: "Concept", 
                    data: { 
                        label: "Started",
                        derived_from: { type: "WorkflowState", id: "start" }
                    } 
                },
                { 
                    id: "concept_end", 
                    type: "Concept", 
                    data: { 
                        label: "Completed",
                        derived_from: { type: "WorkflowState", id: "end" }
                    } 
                }
            ],
            edges: [
                { 
                    from: "concept_start", 
                    to: "concept_end", 
                    type: "SemanticRelation", 
                    data: { 
                        relation: "causes",
                        strength: 1.0,
                        derived_from: { type: "Event", event: "finish" }
                    } 
                }
            ]
        },
        invariants: [
            "transformation_preserves_topology",
            "all_nodes_transformed",
            "all_edges_transformed",
            "provenance_tracked"
        ]
    },
    assertions: [
        {
            path: "$.type",
            condition: "equals",
            expected: "ConceptGraph"
        },
        {
            path: "$.nodes.length",
            condition: "equals",
            expected: 2
        },
        {
            path: "$.edges[0].type",
            condition: "equals",
            expected: "SemanticRelation"
        }
    ]
}
```

### AT-012: Constraint Validation

```graph
AcceptanceTest {
    id: "AT-008-1",
    story_id: "US-008",
    criterion_id: "AC-008-1",
    test_graph: {
        initial_state: {
            nodes: ["A", "B", "C"],
            edges: [
                { from: "A", to: "B" },
                { from: "B", to: "C" },
                { from: "C", to: "A" }
            ],
            constraints: {
                max_degree: 3,
                acyclic: true,
                node_validator: "fn(node) -> node.id.len() == 1"
            }
        },
        actions: [
            Action::Validate { constraints: "all" }
        ],
        expected_state: {
            validation_result: {
                valid: false,
                violations: [
                    {
                        type: "CycleViolation",
                        message: "Graph contains cycle: A -> B -> C -> A",
                        path: ["A", "B", "C", "A"]
                    }
                ],
                passed_constraints: ["max_degree", "node_validator"]
            }
        },
        invariants: [
            "all_constraints_checked",
            "violations_have_details",
            "validation_is_deterministic"
        ]
    },
    assertions: [
        {
            path: "$.validation_result.valid",
            condition: "equals",
            expected: false
        },
        {
            path: "$.validation_result.violations.length",
            condition: "equals",
            expected: 1
        },
        {
            path: "$.validation_result.violations[0].type",
            condition: "equals",
            expected: "CycleViolation"
        }
    ]
}
```

## Test Execution Framework

### Test Runner Structure

```rust
struct TestRunner {
    test_suite: Vec<AcceptanceTest>,
    graph_engine: GraphEngine,
    assertion_engine: AssertionEngine,
}

impl TestRunner {
    fn run_test(&mut self, test: &AcceptanceTest) -> TestResult {
        // 1. Initialize graph state
        let mut graph = self.initialize_state(&test.test_graph.initial_state);
        
        // 2. Execute actions
        for action in &test.test_graph.actions {
            graph = self.execute_action(graph, action)?;
        }
        
        // 3. Validate invariants
        for invariant in &test.test_graph.invariants {
            self.check_invariant(&graph, invariant)?;
        }
        
        // 4. Run assertions
        for assertion in &test.assertions {
            self.assert(&graph, assertion)?;
        }
        
        TestResult::Pass
    }
}
```

### Test Categories

1. **Unit Tests**: Single graph operations
2. **Integration Tests**: Multi-step workflows
3. **Property Tests**: Invariant validation
4. **Performance Tests**: Operation benchmarks
5. **Fuzz Tests**: Random operation sequences

## Test Coverage Matrix

| User Story | Acceptance Criteria | Test Coverage |
|------------|-------------------|---------------|
| US-001 | AC-001-1 | AT-001-1 ✓ |
| US-001 | AC-001-2 | AT-001-2 ✓ |
| US-002 | AC-002-1 | AT-002-1 ✓ |
| US-002 | AC-002-2 | AT-002-2 ✓ |
| US-003 | AC-003-1 | AT-003-1 ✓ |
| US-003 | AC-003-2 | AT-003-2 ✓ |
| US-004 | AC-004-1 | AT-004-1 ✓ |
| US-004 | AC-004-2 | AT-004-2 ✓ |
| US-005 | AC-005-1 | AT-005-1 ✓ |
| US-006 | AC-006-1 | AT-006-1 ✓ |
| US-007 | AC-007-1 | AT-007-1 ✓ |
| US-008 | AC-008-1 | AT-008-1 ✓ |

## Next Steps

1. Implement test runner framework
2. Create property-based test generators
3. Set up continuous integration pipeline
4. Generate test reports and coverage metrics