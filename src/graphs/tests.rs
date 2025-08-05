//! Tests for graph implementations - Event-driven tests only

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::core::Node;
    use serde_json::json;
    use uuid::Uuid;
    
    #[test]
    fn test_workflow_node_types() {
        use workflow::{WorkflowNode, WorkflowNodeType, WorkflowState};
        
        // Test state node
        let state_node = WorkflowNode::new(
            "start",
            WorkflowNodeType::State {
                name: "Start".to_string(),
                description: Some("Initial state".to_string()),
                entry_actions: vec![],
                exit_actions: vec![],
            }
        );
        assert_eq!(state_node.id(), "start");
        assert!(matches!(state_node.node_type(), WorkflowNodeType::State { .. }));
        
        // Test decision node
        let decision_node = WorkflowNode::new(
            "check",
            WorkflowNodeType::Decision {
                condition: "status == 'active'".to_string(),
            }
        );
        assert_eq!(decision_node.id(), "check");
        assert!(matches!(decision_node.node_type(), WorkflowNodeType::Decision { .. }));
    }
    
    #[test] 
    fn test_concept_node_types() {
        use concept::{ConceptNode, ConceptNodeType};
        
        // Test concept node
        let concept = ConceptNode::new(
            "c1",
            ConceptNodeType::Concept {
                definition: "A fundamental concept".to_string(),
            }
        );
        assert_eq!(concept.id(), "c1");
        
        // Test property node
        let property = ConceptNode::new(
            "p1",
            ConceptNodeType::Property {
                name: "weight".to_string(),
                value: 0.8,
            }
        );
        assert_eq!(property.id(), "p1");
        
        // Test category node
        let category = ConceptNode::new(
            "cat1",
            ConceptNodeType::Category {
                name: "Abstract".to_string(),
                parent: None,
            }
        );
        assert_eq!(category.id(), "cat1");
    }
    
    #[test]
    fn test_composed_node_types() {
        use composed::{ComposedNode, ComposedNodeType, GraphDomain};
        
        // Test graph reference node
        let graph_ref = ComposedNode::new(
            "ref1",
            ComposedNodeType::GraphReference {
                domain: GraphDomain::Workflow { graph_id: Uuid::new_v4() },
            }
        );
        assert_eq!(graph_ref.id(), "ref1");
        
        // Test junction node
        let junction = ComposedNode::new(
            "j1",
            ComposedNodeType::Junction {
                junction_type: "merge".to_string(),
            }
        );
        assert_eq!(junction.id(), "j1");
        
        // Test adapter node
        let adapter = ComposedNode::new(
            "a1",
            ComposedNodeType::Adapter {
                adapter_type: "transform".to_string(),
                config: json!({"format": "json"}),
            }
        );
        assert_eq!(adapter.id(), "a1");
    }
    
    #[test]
    fn test_ipld_projection_node() {
        use ipld_projection::{IpldNode, IpldNodeData};
        
        let node = IpldNode::new(
            "Qm123",
            IpldNodeData {
                cid: "Qm123".to_string(),
                codec: "dag-cbor".to_string(),
                size: 1024,
                links: vec![],
                metadata: json!({"type": "file"}),
            }
        );
        
        assert_eq!(node.id(), "Qm123");
        assert_eq!(node.data().codec, "dag-cbor");
        assert_eq!(node.data().size, 1024);
    }
    
    // Note: Direct mutation tests have been removed
    // All graph operations must go through the event-driven system
    // using GraphCommand -> GraphEvent -> Projection pattern
}