//! IPLD (InterPlanetary Linked Data) graph projection
//! 
//! Content-addressed graph where nodes are identified by their content hash (CID)
//! This is a read-only projection computed from events.

use crate::core::{Node, Edge};
use crate::core::cim_graph::{GraphEvent, GraphCommand};
use crate::core::projection_engine::{ProjectionEngine, GenericGraphProjection};
// Projections are ephemeral - no serialization
// Commands still need serialization for events
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Content Identifier (CID) - simplified version for this implementation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cid(String);

impl Cid {
    /// Create a new CID from a string
    pub fn new(s: impl Into<String>) -> Self {
        Cid(s.into())
    }
    
    /// Get the CID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// IPLD node containing content-addressed data
#[derive(Debug, Clone)]
pub struct IpldNode {
    /// Content identifier (hash of the data)
    cid: Cid,
    /// The actual data stored in the node
    data: serde_json::Value,
    /// Links to other IPLD nodes
    links: HashMap<String, Cid>,
}

impl IpldNode {
    /// Create a new IPLD node
    pub fn new(cid: Cid, data: serde_json::Value) -> Self {
        Self {
            cid,
            data,
            links: HashMap::new(),
        }
    }
    
    /// Get the CID
    pub fn cid(&self) -> &Cid {
        &self.cid
    }
    
    /// Get the data
    pub fn data(&self) -> &serde_json::Value {
        &self.data
    }
    
    /// Get links
    pub fn links(&self) -> &HashMap<String, Cid> {
        &self.links
    }
}

impl Node for IpldNode {
    fn id(&self) -> String {
        self.cid.0.clone()
    }
}


/// IPLD edge (link between content-addressed nodes)
#[derive(Debug, Clone)]
pub struct IpldEdge {
    /// Edge identifier
    id: String,
    /// Source CID
    source: Cid,
    /// Target CID
    target: Cid,
    /// Link name/label
    label: String,
}

impl IpldEdge {
    /// Create a new IPLD edge
    pub fn new(id: String, source: Cid, target: Cid, label: String) -> Self {
        Self { id, source, target, label }
    }
    
    /// Get the edge label
    pub fn label(&self) -> &str {
        &self.label
    }
}

impl Edge for IpldEdge {
    fn id(&self) -> String {
        self.id.clone()
    }
    
    fn source(&self) -> String {
        self.source.0.clone()
    }
    
    fn target(&self) -> String {
        self.target.0.clone()
    }
}

/// IPLD graph projection
pub type IpldProjection = GenericGraphProjection<IpldNode, IpldEdge>;

/// Commands specific to IPLD graphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpldCommand {
    /// Add a content-addressed node
    AddCid {
        /// Content identifier (hash)
        cid: String,
        /// Data stored at this CID
        data: serde_json::Value,
        /// Named links to other CIDs (name -> target CID)
        links: HashMap<String, String>,
    },
    /// Add a link between CIDs
    AddLink {
        /// CID of the source node
        source_cid: String,
        /// CID of the target node
        target_cid: String,
        /// Label for the link
        label: String,
    },
    /// Remove a CID and all its links
    RemoveCid {
        /// CID to remove
        cid: String,
    },
}

/// Convert IPLD-specific commands to generic graph commands
pub fn ipld_command_to_graph_command(aggregate_id: Uuid, cmd: IpldCommand) -> GraphCommand {
    match cmd {
        IpldCommand::AddCid { cid, data, links } => {
            GraphCommand::AddNode {
                aggregate_id,
                node_id: cid.clone(),
                node_type: "ipld_node".to_string(),
                data: serde_json::json!({
                    "cid": cid,
                    "data": data,
                    "links": links,
                }),
            }
        }
        IpldCommand::AddLink { source_cid, target_cid, label } => {
            let edge_id = format!("{}->{}:{}", source_cid, target_cid, label);
            GraphCommand::AddEdge {
                aggregate_id,
                edge_id,
                source_id: source_cid,
                target_id: target_cid,
                edge_type: "ipld_link".to_string(),
                data: serde_json::json!({
                    "label": label,
                }),
            }
        }
        IpldCommand::RemoveCid { cid } => {
            GraphCommand::RemoveNode {
                aggregate_id,
                node_id: cid,
            }
        }
    }
}

/// IPLD projection builder
#[derive(Debug)]
pub struct IpldProjectionBuilder {
    engine: ProjectionEngine<IpldProjection>,
}

impl IpldProjectionBuilder {
    /// Create a new IPLD projection builder
    pub fn new() -> Self {
        Self {
            engine: ProjectionEngine::new(),
        }
    }
    
    /// Build projection from events
    pub fn from_events(&self, events: Vec<GraphEvent>) -> IpldProjection {
        self.engine.project(events)
    }
    
    /// Update projection with new event
    pub fn apply_event(&self, projection: &mut IpldProjection, event: &GraphEvent) {
        self.engine.apply(projection, event);
    }
}

/// Helper functions for working with IPLD projections
impl IpldProjection {
    /// Get a node by CID
    pub fn get_by_cid(&self, cid: &str) -> Option<&IpldNode> {
        self.get_node(cid)
    }
    
    /// Get all CIDs in the graph
    pub fn all_cids(&self) -> Vec<Cid> {
        self.nodes()
            .into_iter()
            .map(|node| node.cid().clone())
            .collect()
    }
    
    /// Get all links from a CID
    pub fn get_links(&self, cid: &str) -> Vec<(&str, &Cid)> {
        if let Some(node) = self.get_node(cid) {
            node.links()
                .iter()
                .map(|(name, target)| (name.as_str(), target))
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Check if a CID exists
    pub fn has_cid(&self, cid: &str) -> bool {
        self.get_node(cid).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::core::cim_graph::{EventData, GraphProjection};

    // ========================================================================
    // Cid tests
    // ========================================================================

    #[test]
    fn test_cid_new_from_string() {
        let cid = Cid::new("QmTestHash123");
        assert_eq!(cid.as_str(), "QmTestHash123");
    }

    #[test]
    fn test_cid_new_from_owned_string() {
        let cid = Cid::new(String::from("QmOwnedHash456"));
        assert_eq!(cid.as_str(), "QmOwnedHash456");
    }

    #[test]
    fn test_cid_clone() {
        let cid1 = Cid::new("QmCloneTest");
        let cid2 = cid1.clone();
        assert_eq!(cid1, cid2);
        assert_eq!(cid1.as_str(), cid2.as_str());
    }

    #[test]
    fn test_cid_equality() {
        let cid1 = Cid::new("QmSame");
        let cid2 = Cid::new("QmSame");
        let cid3 = Cid::new("QmDifferent");

        assert_eq!(cid1, cid2);
        assert_ne!(cid1, cid3);
    }

    #[test]
    fn test_cid_hash_for_hashmap() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(Cid::new("QmHash1"));
        set.insert(Cid::new("QmHash2"));
        set.insert(Cid::new("QmHash1")); // Duplicate

        assert_eq!(set.len(), 2);
        assert!(set.contains(&Cid::new("QmHash1")));
        assert!(set.contains(&Cid::new("QmHash2")));
    }

    // ========================================================================
    // IpldNode tests
    // ========================================================================

    #[test]
    fn test_ipld_node_new() {
        let cid = Cid::new("QmTestNode");
        let data = serde_json::json!({"key": "value"});
        let node = IpldNode::new(cid.clone(), data.clone());

        assert_eq!(node.cid(), &cid);
        assert_eq!(node.data(), &data);
        assert!(node.links().is_empty());
    }

    #[test]
    fn test_ipld_node_implements_node_trait() {
        let cid = Cid::new("QmNodeTrait");
        let node = IpldNode::new(cid.clone(), serde_json::json!({}));

        assert_eq!(node.id(), "QmNodeTrait");
    }

    #[test]
    fn test_ipld_node_clone() {
        let cid = Cid::new("QmCloneNode");
        let data = serde_json::json!({"nested": {"value": 42}});
        let node1 = IpldNode::new(cid.clone(), data.clone());
        let node2 = node1.clone();

        assert_eq!(node1.cid(), node2.cid());
        assert_eq!(node1.data(), node2.data());
    }

    // ========================================================================
    // IpldEdge tests
    // ========================================================================

    #[test]
    fn test_ipld_edge_new() {
        let source = Cid::new("QmSource");
        let target = Cid::new("QmTarget");
        let edge = IpldEdge::new("edge1".to_string(), source.clone(), target.clone(), "link".to_string());

        assert_eq!(edge.id(), "edge1");
        assert_eq!(edge.source(), "QmSource");
        assert_eq!(edge.target(), "QmTarget");
        assert_eq!(edge.label(), "link");
    }

    #[test]
    fn test_ipld_edge_implements_edge_trait() {
        let edge = IpldEdge::new(
            "e1".to_string(),
            Cid::new("QmA"),
            Cid::new("QmB"),
            "contains".to_string(),
        );

        assert_eq!(edge.id(), "e1");
        assert_eq!(edge.source(), "QmA");
        assert_eq!(edge.target(), "QmB");
    }

    // ========================================================================
    // IpldCommand conversion tests
    // ========================================================================

    #[test]
    fn test_ipld_command_add_cid_to_graph_command() {
        let aggregate_id = Uuid::new_v4();
        let cmd = IpldCommand::AddCid {
            cid: "QmAddCid".to_string(),
            data: serde_json::json!({"content": "test"}),
            links: HashMap::new(),
        };

        let graph_cmd = ipld_command_to_graph_command(aggregate_id, cmd);

        match graph_cmd {
            GraphCommand::AddNode { node_id, node_type, data, .. } => {
                assert_eq!(node_id, "QmAddCid");
                assert_eq!(node_type, "ipld_node");
                assert!(data.get("cid").is_some());
            }
            _ => panic!("Expected AddNode command"),
        }
    }

    #[test]
    fn test_ipld_command_add_link_to_graph_command() {
        let aggregate_id = Uuid::new_v4();
        let cmd = IpldCommand::AddLink {
            source_cid: "QmSource".to_string(),
            target_cid: "QmTarget".to_string(),
            label: "contains".to_string(),
        };

        let graph_cmd = ipld_command_to_graph_command(aggregate_id, cmd);

        match graph_cmd {
            GraphCommand::AddEdge { source_id, target_id, edge_type, edge_id, .. } => {
                assert_eq!(source_id, "QmSource");
                assert_eq!(target_id, "QmTarget");
                assert_eq!(edge_type, "ipld_link");
                assert!(edge_id.contains("QmSource"));
                assert!(edge_id.contains("QmTarget"));
            }
            _ => panic!("Expected AddEdge command"),
        }
    }

    #[test]
    fn test_ipld_command_remove_cid_to_graph_command() {
        let aggregate_id = Uuid::new_v4();
        let cmd = IpldCommand::RemoveCid {
            cid: "QmToRemove".to_string(),
        };

        let graph_cmd = ipld_command_to_graph_command(aggregate_id, cmd);

        match graph_cmd {
            GraphCommand::RemoveNode { node_id, .. } => {
                assert_eq!(node_id, "QmToRemove");
            }
            _ => panic!("Expected RemoveNode command"),
        }
    }

    #[test]
    fn test_ipld_command_add_cid_with_links() {
        let aggregate_id = Uuid::new_v4();
        let mut links = HashMap::new();
        links.insert("parent".to_string(), "QmParent".to_string());
        links.insert("child".to_string(), "QmChild".to_string());

        let cmd = IpldCommand::AddCid {
            cid: "QmWithLinks".to_string(),
            data: serde_json::json!({}),
            links,
        };

        let graph_cmd = ipld_command_to_graph_command(aggregate_id, cmd);

        match graph_cmd {
            GraphCommand::AddNode { data, .. } => {
                let links_data = data.get("links").unwrap();
                assert!(links_data.get("parent").is_some());
                assert!(links_data.get("child").is_some());
            }
            _ => panic!("Expected AddNode command"),
        }
    }

    // ========================================================================
    // IpldProjectionBuilder tests
    // ========================================================================

    #[test]
    fn test_ipld_projection_builder_new() {
        let builder = IpldProjectionBuilder::new();
        // Should create successfully
        let _ = format!("{:?}", builder);
    }

    #[test]
    fn test_ipld_projection_from_events() {
        let aggregate_id = Uuid::new_v4();
        let builder = IpldProjectionBuilder::new();

        // Create events
        let events = vec![
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id,
                sequence: 1,
                subject: "ipld.graph.events".to_string(),
                timestamp: Utc::now(),
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                data: EventData::GraphInitialized {
                    graph_type: "ipld".to_string(),
                    metadata: HashMap::new(),
                },
            },
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id,
                sequence: 2,
                subject: "ipld.graph.events".to_string(),
                timestamp: Utc::now(),
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                data: EventData::NodeAdded {
                    node_id: "QmHash123".to_string(),
                    node_type: "ipld_node".to_string(),
                    data: serde_json::json!({"content": "Hello IPLD"}),
                },
            },
        ];

        // Build projection
        let projection = builder.from_events(events);

        // The generic projection doesn't create nodes from the events
        // It just tracks adjacency and metadata
        assert_eq!(projection.version(), 2);
    }

    #[test]
    #[should_panic(expected = "Cannot build projection from empty event stream")]
    fn test_ipld_projection_from_empty_events_panics() {
        let builder = IpldProjectionBuilder::new();
        let events: Vec<GraphEvent> = vec![];
        // This should panic because the projection engine requires at least one event
        let _projection = builder.from_events(events);
    }

    // ========================================================================
    // IpldProjection helper method tests
    // ========================================================================

    #[test]
    fn test_ipld_projection_has_cid() {
        use crate::core::GraphType;
        use crate::core::projection_engine::GenericGraphProjection;

        let mut projection: IpldProjection = GenericGraphProjection::new(Uuid::new_v4(), GraphType::Generic);

        // Initially should not have any CIDs
        assert!(!projection.has_cid("QmTest"));

        // Add a node
        let node = IpldNode::new(Cid::new("QmTest"), serde_json::json!({}));
        projection.nodes.insert("QmTest".to_string(), node);

        // Now should have the CID
        assert!(projection.has_cid("QmTest"));
        assert!(!projection.has_cid("QmOther"));
    }

    #[test]
    fn test_ipld_projection_get_by_cid() {
        use crate::core::GraphType;
        use crate::core::projection_engine::GenericGraphProjection;

        let mut projection: IpldProjection = GenericGraphProjection::new(Uuid::new_v4(), GraphType::Generic);

        let node = IpldNode::new(Cid::new("QmGetTest"), serde_json::json!({"data": "value"}));
        projection.nodes.insert("QmGetTest".to_string(), node);

        let retrieved = projection.get_by_cid("QmGetTest");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().cid().as_str(), "QmGetTest");

        assert!(projection.get_by_cid("QmNotExist").is_none());
    }

    #[test]
    fn test_ipld_projection_all_cids() {
        use crate::core::GraphType;
        use crate::core::projection_engine::GenericGraphProjection;

        let mut projection: IpldProjection = GenericGraphProjection::new(Uuid::new_v4(), GraphType::Generic);

        let node1 = IpldNode::new(Cid::new("QmCid1"), serde_json::json!({}));
        let node2 = IpldNode::new(Cid::new("QmCid2"), serde_json::json!({}));
        let node3 = IpldNode::new(Cid::new("QmCid3"), serde_json::json!({}));

        projection.nodes.insert("QmCid1".to_string(), node1);
        projection.nodes.insert("QmCid2".to_string(), node2);
        projection.nodes.insert("QmCid3".to_string(), node3);

        let all_cids = projection.all_cids();
        assert_eq!(all_cids.len(), 3);

        let cid_strings: Vec<&str> = all_cids.iter().map(|c| c.as_str()).collect();
        assert!(cid_strings.contains(&"QmCid1"));
        assert!(cid_strings.contains(&"QmCid2"));
        assert!(cid_strings.contains(&"QmCid3"));
    }

    #[test]
    fn test_ipld_projection_get_links_no_links() {
        use crate::core::GraphType;
        use crate::core::projection_engine::GenericGraphProjection;

        let mut projection: IpldProjection = GenericGraphProjection::new(Uuid::new_v4(), GraphType::Generic);

        let node = IpldNode::new(Cid::new("QmNoLinks"), serde_json::json!({}));
        projection.nodes.insert("QmNoLinks".to_string(), node);

        let links = projection.get_links("QmNoLinks");
        assert!(links.is_empty());
    }

    #[test]
    fn test_ipld_projection_get_links_nonexistent_node() {
        use crate::core::GraphType;
        use crate::core::projection_engine::GenericGraphProjection;

        let projection: IpldProjection = GenericGraphProjection::new(Uuid::new_v4(), GraphType::Generic);
        let links = projection.get_links("QmNotExist");
        assert!(links.is_empty());
    }
}