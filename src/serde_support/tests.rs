//! Tests for serialization support

#[cfg(test)]
mod tests {
    use crate::serde_support::GraphSerialize;
    use crate::graphs::ipld::IpldGraph;
    use serde_json::json;
    
    // WorkflowGraph serialization test disabled - not implemented yet
    #[test]
    #[ignore]
    fn test_workflow_serialization() {
        // TODO: Implement GraphSerialize for WorkflowGraph
    }
    
    #[test]
    fn test_ipld_serialization() {
        let mut graph = IpldGraph::new();
        
        // Add content
        let cid1 = graph.add_content(json!({
            "test": "data"
        })).unwrap();
        
        let cid2 = graph.add_content(json!({
            "more": "content"
        })).unwrap();
        
        graph.add_link(&cid1, &cid2, "next").unwrap();
        
        // Serialize
        let serialized = graph.to_serialized().unwrap();
        assert_eq!(serialized.graph_type, crate::core::graph::GraphType::IpldGraph);
        assert_eq!(serialized.nodes.len(), 2);
        assert_eq!(serialized.edges.len(), 1);
        
        // Check extra data contains content store
        assert!(serialized.extra_data.is_some());
        let extra = serialized.extra_data.as_ref().unwrap();
        // The content_store is serialized directly as the extra_data
        assert!(extra.is_object());
        
        // Deserialize
        let deserialized = IpldGraph::from_serialized(serialized).unwrap();
        assert_eq!(deserialized.graph().node_count(), 2);
        assert!(deserialized.get_content(&cid1).is_some());
        
        // JSON round trip
        let json = graph.to_json().unwrap();
        let from_json = IpldGraph::from_json(&json).unwrap();
        assert_eq!(from_json.get_content(&cid1).unwrap()["test"], "data");
    }
    
    // ContextGraph serialization test disabled - not implemented yet
    #[test]
    #[ignore]
    fn test_context_serialization() {
        // TODO: Implement GraphSerialize for ContextGraph
    }
    
    // ConceptGraph serialization test disabled - not implemented yet
    #[test]
    #[ignore]
    fn test_concept_serialization() {
        // TODO: Implement GraphSerialize for ConceptGraph
    }
    
    // ComposedGraph serialization test disabled - not implemented yet
    #[test]
    #[ignore]
    fn test_composed_serialization() {
        // TODO: Implement GraphSerialize for ComposedGraph
    }
    
    #[test]
    fn test_serialization_errors() {
        // Test invalid JSON deserialization for IpldGraph
        let bad_json = r#"{"invalid": json}"#;
        assert!(IpldGraph::from_json(bad_json).is_err());
        
        // Test with valid IPLD graph
        let mut ipld_graph = IpldGraph::new();
        ipld_graph.add_content(json!({"test": "data"})).unwrap();
        let _serialized = ipld_graph.to_serialized().unwrap();
    }
}