//! Tests for graph implementations

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::core::Node;
    use serde_json::json;
    
    #[test]
    fn test_ipld_graph_comprehensive() {
        use ipld::{IpldGraph, Cid};
        
        let mut graph = IpldGraph::new();
        
        // Test content operations
        let cid1 = graph.add_content(json!({
            "type": "document",
            "title": "Test Document"
        })).unwrap();
        
        let cid2 = graph.add_content(json!({
            "type": "attachment",
            "name": "test.txt"
        })).unwrap();
        
        // Test node access
        let node1 = graph.get_node(&cid1).unwrap();
        assert_eq!(node1.id(), cid1.as_str());
        assert!(node1.links().is_empty());
        
        // Test link operations
        graph.add_link(&cid1, &cid2, "attachment").unwrap();
        
        // Test content retrieval
        let content = graph.get_content(&cid1).unwrap();
        assert_eq!(content["type"], "document");
        
        // Test error cases
        let fake_cid = Cid::new("fake");
        assert!(graph.add_link(&fake_cid, &cid2, "bad").is_err());
        assert!(graph.get_content(&fake_cid).is_none());
    }
    
    #[test]
    fn test_context_graph_comprehensive() {
        use context::{ContextGraph, ContextNode, RelationshipType};
        
        let mut graph = ContextGraph::new();
        
        // Test bounded context
        let _bc_id = graph.add_bounded_context("sales", "Sales Domain").unwrap();
        // Bounded context itself doesn't have a bounded_context field set, so it won't be in get_context_objects
        let context_objects = graph.get_context_objects("sales");
        assert_eq!(context_objects.len(), 0);
        
        // Test aggregate
        let agg_id = graph.add_aggregate("order-123", "Order", "sales").unwrap();
        let aggregates = graph.get_aggregates("sales");
        assert_eq!(aggregates.len(), 1);
        
        // Test entity
        let _entity_id = graph.add_entity("item-456", "OrderItem", &agg_id).unwrap();
        
        // Value objects would be added similarly if needed
        
        // Test relationships - add nodes first
        let event_id = "evt-001";
        let cmd_id = "cmd-001";
        graph.graph_mut().add_node(ContextNode::new(event_id, "OrderPlaced", crate::graphs::context::DomainObjectType::DomainEvent).with_context("sales")).unwrap();
        graph.graph_mut().add_node(ContextNode::new(cmd_id, "PlaceOrder", crate::graphs::context::DomainObjectType::DomainService).with_context("sales")).unwrap();
        graph.add_relationship(&agg_id, event_id, RelationshipType::EmittedBy).unwrap();
        graph.add_relationship(cmd_id, &agg_id, RelationshipType::HandledBy).unwrap();
        
        // Test node access
        if let Some(node) = graph.graph().get_node(&agg_id) {
            assert_eq!(node.object_type(), crate::graphs::context::DomainObjectType::AggregateRoot);
            assert_eq!(node.name(), "Order");
        }
        
        // Verify graph structure
        assert_eq!(graph.graph().node_count(), 5); // context + aggregate + entity + event + command
        assert_eq!(graph.graph().edge_count(), 3); // entity->aggregate + aggregate->event + command->aggregate
        
        // Test error cases - try to add entity to non-existent aggregate
        assert!(graph.add_entity("test", "Test", "nonexistent").is_err());
    }
    
    #[test]
    fn test_workflow_graph_comprehensive() {
        use workflow::{WorkflowGraph, WorkflowNode, StateType, WorkflowEdge};
        
        let mut graph = WorkflowGraph::new();
        
        // Add states
        let start = graph.add_state(WorkflowNode::new("start", "Start", StateType::Initial)).unwrap();
        let processing = graph.add_state(WorkflowNode::new("processing", "Processing", StateType::Normal)).unwrap();
        let error = graph.add_state(WorkflowNode::new("error", "Error", StateType::Normal)).unwrap();
        let end = graph.add_state(WorkflowNode::new("end", "End", StateType::Final)).unwrap();
        
        // Add transitions
        graph.add_transition(&start, &processing, "begin").unwrap();
        graph.add_transition(&processing, &end, "complete").unwrap();
        graph.add_transition(&processing, &error, "fail").unwrap();
        graph.add_transition(&error, &processing, "retry").unwrap();
        
        // Test transition with metadata
        let mut edge = WorkflowEdge::new(&processing, &end)
            .with_trigger("approve")
            .with_guard("is_valid");
        edge.add_action("send_notification");
        graph.graph_mut().add_edge(edge).unwrap();
        
        // Test state queries
        let start_node = graph.graph().get_node(&start).unwrap();
        assert_eq!(start_node.state_type(), StateType::Initial);
        assert!(!graph.is_final_state()); // Not in final state yet
        
        // Test current states
        assert_eq!(graph.current_states().len(), 1);
        assert!(graph.current_states().contains(&start));
        
        // Test transitions
        let transitions = graph.process_event("begin").unwrap();
        assert!(!transitions.is_empty());
        assert!(graph.current_states().contains(&processing));
        
        // Test transition history
        let history = graph.transition_history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0], (start.clone(), processing.clone(), "begin".to_string()));
        
        // Test parallel states - manually add error to active states for testing
        // In a real scenario, this would happen through a transition
        assert_eq!(graph.current_states().len(), 1);
        
        // Test error cases - can't transition from final state
        graph.process_event("complete").unwrap(); // Move to end state
        assert!(graph.is_final_state()); // Now in final state
        let result = graph.process_event("invalid").unwrap();
        assert!(result.is_empty()); // No transitions from final state
    }
    
    #[test]
    fn test_concept_graph_comprehensive() {
        use concept::{ConceptGraph, ConceptType, SemanticRelation};
        
        let mut graph = ConceptGraph::new();
        
        // Add concepts
        let animal = graph.add_concept("animal", "Animal", json!({
            "kingdom": "Animalia"
        })).unwrap();
        
        let mammal = graph.add_concept("mammal", "Mammal", json!({
            "class": "Mammalia"
        })).unwrap();
        
        let dog = graph.add_concept("dog", "Dog", json!({
            "species": "Canis familiaris"
        })).unwrap();
        
        let fido = graph.add_concept("fido", "Fido", json!({
            "breed": "Labrador"
        })).unwrap();
        
        // Add relations
        graph.add_relation(&mammal, &animal, SemanticRelation::SubClassOf).unwrap();
        graph.add_relation(&dog, &mammal, SemanticRelation::SubClassOf).unwrap();
        graph.add_relation(&fido, &dog, SemanticRelation::InstanceOf).unwrap();
        
        // Test concept properties
        let node = graph.graph().get_node(&dog).unwrap();
        assert_eq!(node.label(), "Dog");
        assert_eq!(node.concept_type(), ConceptType::Class);
        
        // Test get_relations
        let relations = graph.get_relations(&dog);
        assert_eq!(relations.len(), 1);
        assert_eq!(relations[0].1, SemanticRelation::SubClassOf);
        
        // Test hierarchy queries
        let superclasses = graph.get_superclasses(&dog);
        assert!(superclasses.contains(&mammal));
        
        let instances = graph.get_instances(&dog);
        assert!(instances.contains(&fido));
        
        // Test inference
        let inferred_count = graph.apply_inference();
        assert_eq!(inferred_count, 0); // No new inferences from this simple hierarchy
        
        // Test custom relation - would need to create a custom edge with label
        // For now, just use a standard relation
        graph.add_relation(&fido, &dog, SemanticRelation::HasProperty).unwrap();
        
        // Test all concepts
        let all_concepts = graph.get_all_concepts();
        assert_eq!(all_concepts.len(), 4);
    }
    
    #[test]
    fn test_composed_graph_comprehensive() {
        use composed::{ComposedGraph, ComposedNode, ComposedEdge};
        use workflow::{WorkflowNode, StateType};
        use context::{ContextNode, DomainObjectType};
        
        let mut graph = ComposedGraph::new();
        
        // Add constraint for context-workflow relationships
        graph.add_constraint("context-workflow", "context", "workflow", vec!["manages".to_string(), "controls".to_string()]);
        
        // Add nodes using actual node types
        let context_node = ContextNode::new("ctx1", "OrderContext", DomainObjectType::BoundedContext);
        let workflow_node = WorkflowNode::new("wf1", "OrderWorkflow", StateType::Normal);
        
        let ctx_id = graph.add_node(ComposedNode::Context(context_node)).unwrap();
        let wf_id = graph.add_node(ComposedNode::Workflow(workflow_node)).unwrap();
        
        // Add cross-type edge
        let edge = ComposedEdge::cross_type(&ctx_id, &wf_id, "context", "workflow", "manages");
        graph.add_edge(edge).unwrap();
        
        // Test get_nodes_by_type
        let context_nodes = graph.get_nodes_by_type("context");
        assert_eq!(context_nodes.len(), 1);
        assert_eq!(context_nodes[0].id(), "ctx1");
        
        // Test layers (they exist in the struct but no add_layer method)
        let layers = graph.get_layers();
        assert_eq!(layers.len(), 0); // No layers added by default
        
        // Test cross-type edges
        let cross_edges = graph.get_cross_type_edges();
        assert_eq!(cross_edges.len(), 1);
        
        // Test error cases - invalid cross-type edge
        let bad_edge = ComposedEdge::cross_type(&ctx_id, &wf_id, "context", "workflow", "invalid");
        assert!(graph.add_edge(bad_edge).is_err());
    }
}