use cim_graph::graphs::IpldGraph;
use cim_graph::serde_support::GraphSerialize;
use serde_json::json;

#[test]
fn test_ipld_serialization() {
    let mut graph = IpldGraph::new();
    
    // Add some content
    let data1 = json!({"type": "file", "name": "test.txt"});
    let data2 = json!({"type": "folder", "name": "documents"});
    
    let cid1 = graph.add_content(data1).unwrap();
    let cid2 = graph.add_content(data2).unwrap();
    graph.add_link(&cid2, &cid1, "contains").unwrap();
    
    // Serialize to JSON
    let json = graph.to_json().unwrap();
    assert!(!json.is_empty());
    
    // Deserialize back
    let restored = IpldGraph::from_json(&json).unwrap();
    
    // Verify content
    assert_eq!(restored.graph().node_count(), 2);
    assert_eq!(restored.graph().edge_count(), 1);
}