//! Tests for projection behaviors and edge cases

use cim_graph::{
    core::{GraphProjection, ProjectionEngine, GenericGraphProjection, Node},
    events::{GraphEvent, EventPayload, WorkflowPayload, ContextPayload, ComposedPayload},
    graphs::{
        WorkflowNode, WorkflowEdge, WorkflowProjection,
        ConceptNode, ConceptEdge, ConceptProjection,
        ComposedNode, ComposedEdge, ComposedProjection,
    },
    Result,
};
use uuid::Uuid;

#[test]
fn test_workflow_projection_methods() {
    // Build a workflow projection from events
    let workflow_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    
    let events = vec![
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            correlation_id,
            causation_id: None,
            payload: EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
                workflow_id,
                name: "Order Processing".to_string(),
                version: "1.0.0".to_string(),
            }),
        },
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            correlation_id,
            causation_id: None,
            payload: EventPayload::Workflow(WorkflowPayload::StateAdded {
                workflow_id,
                state_id: "pending".to_string(),
                state_type: "state".to_string(),
            }),
        },
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            correlation_id,
            causation_id: None,
            payload: EventPayload::Workflow(WorkflowPayload::StateAdded {
                workflow_id,
                state_id: "processing".to_string(),
                state_type: "state".to_string(),
            }),
        },
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            correlation_id,
            causation_id: None,
            payload: EventPayload::Workflow(WorkflowPayload::StateAdded {
                workflow_id,
                state_id: "complete".to_string(),
                state_type: "state".to_string(),
            }),
        },
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            correlation_id,
            causation_id: None,
            payload: EventPayload::Workflow(WorkflowPayload::TransitionAdded {
                workflow_id,
                from_state: "pending".to_string(),
                to_state: "processing".to_string(),
                trigger: "start".to_string(),
            }),
        },
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            correlation_id,
            causation_id: None,
            payload: EventPayload::Workflow(WorkflowPayload::TransitionAdded {
                workflow_id,
                from_state: "processing".to_string(),
                to_state: "complete".to_string(),
                trigger: "finish".to_string(),
            }),
        },
    ];
    
    // Create projection
    let engine = ProjectionEngine::<WorkflowNode, WorkflowEdge>::new();
    let generic_projection = engine.project(events);
    let projection = WorkflowProjection::new(generic_projection);
    
    // Test workflow-specific methods
    let states = projection.get_states();
    assert_eq!(states.len(), 3);
    assert!(states.iter().any(|s| s.id() == "pending"));
    assert!(states.iter().any(|s| s.id() == "processing"));
    assert!(states.iter().any(|s| s.id() == "complete"));
    
    // Test path finding
    let path = projection.find_path("pending", "complete");
    assert!(path.is_some());
    let path = path.unwrap();
    assert_eq!(path.len(), 3); // pending -> processing -> complete
    assert_eq!(path[0], "pending");
    assert_eq!(path[1], "processing");
    assert_eq!(path[2], "complete");
    
    // Test validation
    assert!(projection.validate().is_ok());
}

#[test]
fn test_concept_projection_reasoning() {
    let aggregate_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    
    let events = vec![
        // Define concepts
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id,
            causation_id: None,
            payload: EventPayload::Concept(ConceptPayload::ConceptDefined {
                concept_id: "animal".to_string(),
                name: "Animal".to_string(),
                definition: "Living organism".to_string(),
            }),
        },
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id,
            causation_id: None,
            payload: EventPayload::Concept(ConceptPayload::ConceptDefined {
                concept_id: "mammal".to_string(),
                name: "Mammal".to_string(),
                definition: "Warm-blooded animal".to_string(),
            }),
        },
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id,
            causation_id: None,
            payload: EventPayload::Concept(ConceptPayload::ConceptDefined {
                concept_id: "dog".to_string(),
                name: "Dog".to_string(),
                definition: "Domestic canine".to_string(),
            }),
        },
        // Add relationships
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id,
            causation_id: None,
            payload: EventPayload::Concept(ConceptPayload::RelationAdded {
                source_concept: "mammal".to_string(),
                target_concept: "animal".to_string(),
                relation_type: "is_a".to_string(),
                strength: 1.0,
            }),
        },
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id,
            causation_id: None,
            payload: EventPayload::Concept(ConceptPayload::RelationAdded {
                source_concept: "dog".to_string(),
                target_concept: "mammal".to_string(),
                relation_type: "is_a".to_string(),
                strength: 1.0,
            }),
        },
    ];
    
    // Create projection
    let engine = ProjectionEngine::<ConceptNode, ConceptEdge>::new();
    let generic_projection = engine.project(events);
    let projection = ConceptProjection::new(generic_projection);
    
    // Test concept methods
    let concepts = projection.get_concepts();
    assert_eq!(concepts.len(), 3);
    
    // Test semantic distance
    let distance = projection.semantic_distance("dog", "animal");
    assert!(distance.is_some());
    assert!(distance.unwrap() > 0.0); // There is some distance
    
    // Test reasoning paths
    let paths = projection.find_reasoning_paths("dog", "animal");
    assert!(!paths.is_empty());
    assert_eq!(paths[0].len(), 3); // dog -> mammal -> animal
    
    // Test relationship inference
    let inferred = projection.infer_relationships("dog");
    assert!(!inferred.is_empty());
}

#[test]
fn test_composed_projection_cross_graph() {
    let aggregate_id = Uuid::new_v4();
    let workflow_id = Uuid::new_v4();
    let concept_id = Uuid::new_v4();
    
    let events = vec![
        // Add subgraphs
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Composed(ComposedPayload::SubGraphAdded {
                subgraph_id: workflow_id,
                graph_type: "workflow".to_string(),
                namespace: "processes".to_string(),
            }),
        },
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Composed(ComposedPayload::SubGraphAdded {
                subgraph_id: concept_id,
                graph_type: "concept".to_string(),
                namespace: "domain".to_string(),
            }),
        },
        // Create cross-graph link
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Composed(ComposedPayload::CrossGraphLinkCreated {
                source_graph: workflow_id,
                source_node: "validate".to_string(),
                target_graph: concept_id,
                target_node: "validation_rules".to_string(),
            }),
        },
    ];
    
    // Create projection
    let engine = ProjectionEngine::<ComposedNode, ComposedEdge>::new();
    let generic_projection = engine.project(events);
    let projection = ComposedProjection::new(generic_projection);
    
    // Test composed methods
    let graphs = projection.get_sub_graphs();
    assert_eq!(graphs.len(), 2);
    
    // Test cross-graph links
    let links = projection.get_cross_graph_links(workflow_id, concept_id);
    assert_eq!(links.len(), 1);
    
    // Test validation
    let validation = projection.validate();
    assert!(validation.is_ok());
}

#[test]
fn test_projection_immutability() {
    // Projections should be read-only
    let engine = ProjectionEngine::<WorkflowNode, WorkflowEdge>::new();
    let projection = engine.project(vec![]);
    
    // Test that projection only has read methods
    assert_eq!(projection.node_count(), 0);
    assert_eq!(projection.edge_count(), 0);
    assert_eq!(projection.version(), 0);
    
    // No mutation methods should exist
    // This is enforced at compile time by the GraphProjection trait
}

#[test]
fn test_projection_replay_determinism() {
    // Same events should always produce same projection
    let events = create_test_events();
    
    let engine = ProjectionEngine::<WorkflowNode, WorkflowEdge>::new();
    let projection1 = engine.project(events.clone());
    let projection2 = engine.project(events.clone());
    
    // Both projections should be identical
    assert_eq!(projection1.version(), projection2.version());
    assert_eq!(projection1.node_count(), projection2.node_count());
    assert_eq!(projection1.edge_count(), projection2.edge_count());
}

#[test]
fn test_empty_projection() {
    // Test projection with no events
    let engine = ProjectionEngine::<ConceptNode, ConceptEdge>::new();
    let projection = engine.project(vec![]);
    
    assert_eq!(projection.version(), 0);
    assert_eq!(projection.node_count(), 0);
    assert_eq!(projection.edge_count(), 0);
    assert!(projection.nodes().is_empty());
    assert!(projection.edges().is_empty());
}

// Helper function to create test events
fn create_test_events() -> Vec<GraphEvent> {
    let aggregate_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    
    vec![
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id,
            causation_id: None,
            payload: EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
                workflow_id: aggregate_id,
                name: "Test".to_string(),
                version: "1.0.0".to_string(),
            }),
        },
    ]
}