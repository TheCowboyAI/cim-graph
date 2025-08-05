//! Tests for event-driven projections

use cim_graph::{
    core::{GraphStateMachine, PolicyEngine, PolicyContext, ProjectionEngine, GenericGraphProjection},
    events::{GraphEvent, EventPayload, GraphCommand, WorkflowPayload, IpldPayload, ConceptPayload},
    graphs::{WorkflowNode, WorkflowEdge},
    Result,
};
use std::collections::HashMap;
use uuid::Uuid;

#[test]
fn test_workflow_projection_from_events() {
    // Create a workflow through events
    let workflow_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    
    let events = vec![
        // Event 1: Define workflow
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            correlation_id,
            causation_id: None,
            payload: EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
                workflow_id,
                name: "Test Workflow".to_string(),
                version: "1.0.0".to_string(),
            }),
        },
        // Event 2: Add states
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            correlation_id,
            causation_id: None,
            payload: EventPayload::Workflow(WorkflowPayload::StateAdded {
                workflow_id,
                state_id: "start".to_string(),
                state_type: "initial".to_string(),
            }),
        },
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            correlation_id,
            causation_id: None,
            payload: EventPayload::Workflow(WorkflowPayload::StateAdded {
                workflow_id,
                state_id: "end".to_string(),
                state_type: "final".to_string(),
            }),
        },
        // Event 3: Add transition
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: workflow_id,
            correlation_id,
            causation_id: None,
            payload: EventPayload::Workflow(WorkflowPayload::TransitionAdded {
                workflow_id,
                from_state: "start".to_string(),
                to_state: "end".to_string(),
                trigger: "complete".to_string(),
            }),
        },
    ];
    
    // Build projection from events
    let projection_engine = ProjectionEngine::<WorkflowNode, WorkflowEdge>::new();
    let projection = projection_engine.project(events);
    
    // Verify projection state
    assert_eq!(projection.aggregate_id(), workflow_id);
    assert_eq!(projection.version(), 4); // 4 events
    assert_eq!(projection.node_count(), 2); // start and end states
    assert_eq!(projection.edge_count(), 1); // one transition
}

#[test]
fn test_state_machine_command_validation() {
    let mut state_machine = GraphStateMachine::new();
    let aggregate_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    
    // Test invalid command on uninitialized graph
    let invalid_command = GraphCommand::Workflow {
        aggregate_id,
        correlation_id,
        command: crate::events::WorkflowCommand::AddState {
            workflow_id: aggregate_id,
            state_id: "test".to_string(),
            state_type: "normal".to_string(),
        },
    };
    
    let projection = crate::core::aggregate_projection::GraphAggregateProjection::new(aggregate_id);
    let result = state_machine.handle_command(invalid_command, &projection);
    
    // Should fail because graph is not initialized
    assert!(result.is_err());
    
    // Initialize the graph first
    let init_command = GraphCommand::InitializeGraph {
        aggregate_id,
        graph_type: "workflow".to_string(),
        correlation_id,
    };
    
    let events = state_machine.handle_command(init_command, &projection).unwrap();
    assert!(!events.is_empty());
    
    // Apply the initialization event to state machine
    for event in &events {
        state_machine.apply_event(event);
    }
    
    // Now the workflow command should succeed
    let valid_command = GraphCommand::Workflow {
        aggregate_id,
        correlation_id,
        command: crate::events::WorkflowCommand::DefineWorkflow {
            workflow_id: aggregate_id,
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
        },
    };
    
    let result = state_machine.handle_command(valid_command, &projection);
    assert!(result.is_ok());
}

#[test]
fn test_policy_engine_integration() {
    let mut policy_engine = PolicyEngine::new();
    let mut state_machine = GraphStateMachine::new();
    let mut ipld_chains = HashMap::new();
    
    let aggregate_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    
    // Create an IPLD event
    let event = GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        correlation_id,
        causation_id: None,
        payload: EventPayload::Ipld(IpldPayload::CidAdded {
            cid: "QmTest123".to_string(),
            codec: "dag-cbor".to_string(),
            size: 256,
            data: serde_json::json!({"test": "data"}),
        }),
    };
    
    // Execute policies
    let mut context = PolicyContext {
        state_machine: &mut state_machine,
        ipld_chains: &mut ipld_chains,
        metrics: Default::default(),
    };
    
    let actions = policy_engine.execute_policies(&event, &mut context).unwrap();
    
    // Verify policy actions were generated
    assert!(!actions.is_empty());
    
    // Check metrics
    let metrics = policy_engine.get_metrics();
    assert!(metrics.events_processed > 0);
}

#[test]
fn test_event_correlation_and_causation() {
    let aggregate_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    
    // Create a chain of events with causation
    let event1 = GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        correlation_id,
        causation_id: None,
        payload: EventPayload::Concept(ConceptPayload::ConceptDefined {
            concept_id: "c1".to_string(),
            name: "Root Concept".to_string(),
            definition: "The root of our ontology".to_string(),
        }),
    };
    
    let event2 = GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        correlation_id,
        causation_id: Some(event1.event_id), // Caused by event1
        payload: EventPayload::Concept(ConceptPayload::PropertiesAdded {
            concept_id: "c1".to_string(),
            properties: vec![
                ("weight".to_string(), 1.0),
                ("priority".to_string(), 0.9),
            ],
        }),
    };
    
    let event3 = GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        correlation_id: Uuid::new_v4(), // Different correlation
        causation_id: Some(event2.event_id), // Caused by event2
        payload: EventPayload::Concept(ConceptPayload::RelationAdded {
            source_concept: "c1".to_string(),
            target_concept: "c2".to_string(),
            relation_type: "specializes".to_string(),
            strength: 0.8,
        }),
    };
    
    // Verify causation chain
    assert!(event1.causation_id.is_none());
    assert_eq!(event2.causation_id, Some(event1.event_id));
    assert_eq!(event3.causation_id, Some(event2.event_id));
    
    // Verify correlation
    assert_eq!(event1.correlation_id, event2.correlation_id);
    assert_ne!(event1.correlation_id, event3.correlation_id);
}

#[test]
fn test_projection_rebuild_from_events() {
    // Simulate a complex event stream
    let aggregate_id = Uuid::new_v4();
    let mut events = Vec::new();
    
    // Generate multiple events
    for i in 0..10 {
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id: Uuid::new_v4(),
            causation_id: if i > 0 { Some(events[i-1].event_id) } else { None },
            payload: EventPayload::Generic(crate::events::GenericPayload {
                event_type: format!("TestEvent{}", i),
                data: serde_json::json!({ "index": i }),
            }),
        };
        events.push(event);
    }
    
    // Build projection
    let projection_engine = ProjectionEngine::<crate::core::GenericNode<String>, crate::core::GenericEdge<()>>::new();
    let projection = projection_engine.project(events.clone());
    
    // Verify version matches event count
    assert_eq!(projection.version(), events.len() as u64);
    
    // Test partial rebuild (first 5 events)
    let partial_projection = projection_engine.project(events[..5].to_vec());
    assert_eq!(partial_projection.version(), 5);
}

#[test]
fn test_ipld_chain_construction() {
    let aggregate_id = Uuid::new_v4();
    let mut ipld_chains = HashMap::new();
    
    // Create IPLD chain through events
    let events = vec![
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Ipld(IpldPayload::CidAdded {
                cid: "QmRoot".to_string(),
                codec: "dag-cbor".to_string(),
                size: 100,
                data: serde_json::json!({"root": true}),
            }),
        },
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Ipld(IpldPayload::CidLinkAdded {
                cid: "QmRoot".to_string(),
                link_name: "child".to_string(),
                target_cid: "QmChild".to_string(),
            }),
        },
    ];
    
    // Process events through policy that builds IPLD chains
    let mut policy_engine = PolicyEngine::new();
    let mut state_machine = GraphStateMachine::new();
    
    for event in &events {
        let mut context = PolicyContext {
            state_machine: &mut state_machine,
            ipld_chains: &mut ipld_chains,
            metrics: Default::default(),
        };
        
        let _ = policy_engine.execute_policies(event, &mut context);
    }
    
    // Verify chain was built
    assert!(ipld_chains.contains_key(&aggregate_id));
}

#[test]
fn test_graph_lifecycle_states() {
    use crate::core::state_machine::GraphState;
    
    let mut state_machine = GraphStateMachine::new();
    let aggregate_id = Uuid::new_v4();
    
    // Initial state should be Uninitialized
    let state = state_machine.get_state(&aggregate_id);
    assert!(matches!(state, GraphState::Uninitialized));
    
    // After initialization
    let init_event = GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        correlation_id: Uuid::new_v4(),
        causation_id: None,
        payload: EventPayload::Generic(crate::events::GenericPayload {
            event_type: "GraphInitialized".to_string(),
            data: serde_json::json!({"graph_type": "workflow"}),
        }),
    };
    
    state_machine.apply_event(&init_event);
    let state = state_machine.get_state(&aggregate_id);
    assert!(matches!(state, GraphState::Initialized { .. }));
}

#[test] 
fn test_event_subject_patterns() {
    use cim_graph::events::{build_event_subject, GraphType, EventType};
    
    let aggregate_id = Uuid::new_v4();
    
    // Test subject construction
    let subject = build_event_subject(
        GraphType::Workflow,
        aggregate_id,
        EventType::Created
    );
    
    assert!(subject.contains("cim.graph.workflow"));
    assert!(subject.contains(&aggregate_id.to_string()));
    assert!(subject.contains("created"));
}