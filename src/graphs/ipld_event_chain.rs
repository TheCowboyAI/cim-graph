//! IPLD Event Chain - The heart of CIM's storage system
//!
//! When events are created, their payloads (data without metadata) are given CIDs.
//! These CIDs form Merkle DAGs that create CID chains for entire aggregate transactions.
//! This enables referring to and retrieving entire event streams with a single CID.
//!
//! Integration with cim-ipld library handles the actual CID generation and DAG construction.

use crate::core::{Node, Edge};
use crate::core::cim_graph::{GraphEvent, EventData};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// CID (Content Identifier) - the hash of event payload data
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Cid(pub String);

impl Cid {
    /// Create a new CID
    pub fn new(hash: impl Into<String>) -> Self {
        Cid(hash.into())
    }
    
    /// Get the CID string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Cid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}


/// Event payload with its CID - this is what gets stored in IPLD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPayload {
    /// The CID of this payload (hash of the data)
    pub cid: Cid,
    /// The actual event data (without metadata)
    pub data: EventData,
    /// CID of the previous event in the chain (forms the Merkle DAG)
    pub previous: Option<Cid>,
    /// Aggregate this event belongs to
    pub aggregate_id: Uuid,
    /// Sequence number in the aggregate's event stream
    pub sequence: u64,
}

/// CID Chain - represents an entire aggregate transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CidChain {
    /// The root CID (latest event in the chain)
    pub root: Cid,
    /// Aggregate ID this chain represents
    pub aggregate_id: Uuid,
    /// Total number of events in the chain
    pub length: u64,
    /// Timestamp of the latest event
    pub latest_timestamp: DateTime<Utc>,
    /// All CIDs in the chain (sequence -> CID mapping)
    pub cids: HashMap<u64, Cid>,
}

impl CidChain {
    /// Create a new CID chain starting from a root CID
    pub fn new(root: Cid, aggregate_id: Uuid) -> Self {
        Self {
            root,
            aggregate_id,
            length: 0,
            latest_timestamp: Utc::now(),
            cids: HashMap::new(),
        }
    }
    
    /// Add an event to the chain
    pub fn add_event(&mut self, sequence: u64, cid: Cid, timestamp: DateTime<Utc>) {
        self.cids.insert(sequence, cid.clone());
        if sequence >= self.length {
            self.length = sequence + 1;
            self.root = cid;
            self.latest_timestamp = timestamp;
        }
    }
    
    /// Get the CID for a specific sequence number
    pub fn get_cid(&self, sequence: u64) -> Option<&Cid> {
        self.cids.get(&sequence)
    }
    
    /// Get all CIDs in sequence order
    pub fn get_ordered_cids(&self) -> Vec<&Cid> {
        let mut sequences: Vec<_> = self.cids.keys().cloned().collect();
        sequences.sort();
        sequences.iter()
            .filter_map(|seq| self.cids.get(seq))
            .collect()
    }
}

/// IPLD Event Chain Node - represents an event in the Merkle DAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpldEventNode {
    /// CID of this event's payload
    pub cid: Cid,
    /// The event payload
    pub payload: EventPayload,
    /// Links to other events (forms the DAG)
    pub links: HashMap<String, Cid>,
}

impl IpldEventNode {
    /// Create a new event node
    pub fn new(cid: Cid, payload: EventPayload) -> Self {
        let mut links = HashMap::new();
        
        // Add link to previous event if it exists
        if let Some(prev) = &payload.previous {
            links.insert("previous".to_string(), prev.clone());
        }
        
        Self { cid, payload, links }
    }
    
    /// Get the previous event's CID
    pub fn previous(&self) -> Option<&Cid> {
        self.links.get("previous")
    }
}

impl Node for IpldEventNode {
    fn id(&self) -> String {
        self.cid.0.clone()
    }
}


/// IPLD Chain Edge - represents the link between events in the chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpldChainEdge {
    /// Edge ID
    pub id: String,
    /// Source event CID
    pub source: Cid,
    /// Target event CID
    pub target: Cid,
    /// Link type (e.g., "previous", "branch", "merge")
    pub link_type: String,
}

impl Edge for IpldChainEdge {
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

/// Functions to integrate with cim-ipld for CID generation
pub trait CidGenerator {
    /// Generate a CID for an event payload
    fn generate_cid(&self, data: &EventData) -> Cid;
    
    /// Verify a CID matches its data
    fn verify_cid(&self, cid: &Cid, data: &EventData) -> bool;
}

/// Mock CID generator for testing (real implementation would use cim-ipld)
#[derive(Debug)]
pub struct MockCidGenerator;

impl CidGenerator for MockCidGenerator {
    fn generate_cid(&self, data: &EventData) -> Cid {
        // In real implementation, this would use cim-ipld to generate proper CIDs
        let json = serde_json::to_string(data).unwrap_or_default();
        let hash = format!("Qm{}", &json.as_bytes().iter().take(8).map(|b| format!("{:02x}", b)).collect::<String>());
        Cid::new(hash)
    }
    
    fn verify_cid(&self, cid: &Cid, data: &EventData) -> bool {
        self.generate_cid(data) == *cid
    }
}

/// Event chain builder - constructs CID chains from event streams
#[derive(Debug)]
pub struct EventChainBuilder<G: CidGenerator> {
    generator: G,
    metadata: HashMap<String, serde_json::Value>,
}

impl<G: CidGenerator> EventChainBuilder<G> {
    /// Create a new event chain builder
    pub fn new(generator: G) -> Self {
        Self { 
            generator,
            metadata: HashMap::new(),
        }
    }
    
    /// Build a CID chain from a stream of graph events
    pub fn build_chain(&mut self, events: &[GraphEvent]) -> CidChain {
        if events.is_empty() {
            panic!("Cannot build chain from empty event stream");
        }
        
        let aggregate_id = events[0].aggregate_id;
        let mut chain = CidChain::new(Cid::new(""), aggregate_id);
        let mut previous_cid: Option<Cid> = None;
        
        for event in events {
            // Generate CID for the event data payload
            let cid = self.generator.generate_cid(&event.data);
            
            // Event payload would be stored in IPLD with this structure
            // For now just store in metadata for debugging
            self.metadata.insert(
                format!("event_{}", event.sequence),
                serde_json::json!({
                    "cid": cid.to_string(),
                    "data": event.data.clone(),
                    "previous": previous_cid.as_ref().map(|c| c.to_string()),
                    "aggregate_id": event.aggregate_id,
                    "sequence": event.sequence,
                })
            );
            
            // Add to chain
            chain.add_event(event.sequence, cid.clone(), event.timestamp);
            previous_cid = Some(cid);
        }
        
        chain
    }
    
    /// Retrieve an entire event stream using a single root CID
    /// (In real implementation, this would fetch from NATS JetStream)
    pub fn retrieve_by_cid(&self, _root_cid: &Cid) -> Result<Vec<GraphEvent>, String> {
        // This is where cim-ipld would retrieve the entire Merkle DAG
        // from NATS JetStream using the root CID
        Err("Not implemented - would use cim-ipld to fetch from JetStream".to_string())
    }
}

/// IPLD-specific commands for working with CID chains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpldChainCommand {
    /// Store an event with its CID
    StoreEvent {
        /// Event to store
        event: GraphEvent,
        /// CID of the previous event in the chain
        previous_cid: Option<Cid>,
    },
    /// Retrieve an event stream by root CID
    RetrieveChain {
        /// Root CID of the chain to retrieve
        root_cid: Cid,
    },
    /// Verify the integrity of a CID chain
    VerifyChain {
        /// Root CID of the chain to verify
        root_cid: Cid,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Cid tests
    // ========================================================================

    #[test]
    fn test_cid_new() {
        let cid = Cid::new("QmTestCid123");
        assert_eq!(cid.as_str(), "QmTestCid123");
    }

    #[test]
    fn test_cid_display() {
        let cid = Cid::new("QmDisplayTest");
        assert_eq!(format!("{}", cid), "QmDisplayTest");
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
    fn test_cid_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(Cid::new("QmA"));
        set.insert(Cid::new("QmB"));
        set.insert(Cid::new("QmA")); // Duplicate

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_cid_serialization() {
        let cid = Cid::new("QmSerialize");
        let json = serde_json::to_string(&cid).unwrap();
        let deserialized: Cid = serde_json::from_str(&json).unwrap();
        assert_eq!(cid, deserialized);
    }

    // ========================================================================
    // CidChain tests
    // ========================================================================

    #[test]
    fn test_cid_chain_new() {
        let root = Cid::new("QmRoot");
        let aggregate_id = Uuid::new_v4();
        let chain = CidChain::new(root.clone(), aggregate_id);

        assert_eq!(chain.root, root);
        assert_eq!(chain.aggregate_id, aggregate_id);
        assert_eq!(chain.length, 0);
        assert!(chain.cids.is_empty());
    }

    #[test]
    fn test_cid_chain_add_event() {
        let root = Cid::new("QmRoot");
        let aggregate_id = Uuid::new_v4();
        let mut chain = CidChain::new(root, aggregate_id);

        let cid1 = Cid::new("QmEvent1");
        let timestamp1 = Utc::now();
        chain.add_event(0, cid1.clone(), timestamp1);

        assert_eq!(chain.length, 1);
        assert_eq!(chain.root, cid1);
        assert_eq!(chain.cids.len(), 1);
    }

    #[test]
    fn test_cid_chain_add_multiple_events() {
        let root = Cid::new("QmRoot");
        let aggregate_id = Uuid::new_v4();
        let mut chain = CidChain::new(root, aggregate_id);

        let cid1 = Cid::new("QmEvent1");
        let cid2 = Cid::new("QmEvent2");
        let cid3 = Cid::new("QmEvent3");

        chain.add_event(0, cid1.clone(), Utc::now());
        chain.add_event(1, cid2.clone(), Utc::now());
        chain.add_event(2, cid3.clone(), Utc::now());

        assert_eq!(chain.length, 3);
        assert_eq!(chain.root, cid3);
        assert_eq!(chain.cids.len(), 3);
    }

    #[test]
    fn test_cid_chain_get_cid() {
        let root = Cid::new("QmRoot");
        let aggregate_id = Uuid::new_v4();
        let mut chain = CidChain::new(root, aggregate_id);

        let cid1 = Cid::new("QmEvent1");
        let cid2 = Cid::new("QmEvent2");

        chain.add_event(0, cid1.clone(), Utc::now());
        chain.add_event(1, cid2.clone(), Utc::now());

        assert_eq!(chain.get_cid(0), Some(&cid1));
        assert_eq!(chain.get_cid(1), Some(&cid2));
        assert_eq!(chain.get_cid(99), None);
    }

    #[test]
    fn test_cid_chain_get_ordered_cids() {
        let root = Cid::new("QmRoot");
        let aggregate_id = Uuid::new_v4();
        let mut chain = CidChain::new(root, aggregate_id);

        // Add out of order
        let cid0 = Cid::new("QmEvent0");
        let cid1 = Cid::new("QmEvent1");
        let cid2 = Cid::new("QmEvent2");

        chain.add_event(2, cid2.clone(), Utc::now());
        chain.add_event(0, cid0.clone(), Utc::now());
        chain.add_event(1, cid1.clone(), Utc::now());

        let ordered = chain.get_ordered_cids();
        assert_eq!(ordered.len(), 3);
        assert_eq!(ordered[0], &cid0);
        assert_eq!(ordered[1], &cid1);
        assert_eq!(ordered[2], &cid2);
    }

    // ========================================================================
    // EventPayload tests
    // ========================================================================

    #[test]
    fn test_event_payload_creation() {
        let cid = Cid::new("QmPayload");
        let payload = EventPayload {
            cid: cid.clone(),
            data: EventData::NodeAdded {
                node_id: "node1".to_string(),
                node_type: "data".to_string(),
                data: serde_json::json!({}),
            },
            previous: None,
            aggregate_id: Uuid::new_v4(),
            sequence: 1,
        };

        assert_eq!(payload.cid, cid);
        assert_eq!(payload.sequence, 1);
        assert!(payload.previous.is_none());
    }

    #[test]
    fn test_event_payload_with_previous() {
        let prev_cid = Cid::new("QmPrevious");
        let cid = Cid::new("QmCurrent");
        let payload = EventPayload {
            cid: cid.clone(),
            data: EventData::NodeAdded {
                node_id: "node1".to_string(),
                node_type: "data".to_string(),
                data: serde_json::json!({}),
            },
            previous: Some(prev_cid.clone()),
            aggregate_id: Uuid::new_v4(),
            sequence: 2,
        };

        assert_eq!(payload.previous, Some(prev_cid));
    }

    // ========================================================================
    // IpldEventNode tests
    // ========================================================================

    #[test]
    fn test_ipld_event_node_new() {
        let cid = Cid::new("QmEventNode");
        let payload = EventPayload {
            cid: cid.clone(),
            data: EventData::NodeAdded {
                node_id: "test".to_string(),
                node_type: "test".to_string(),
                data: serde_json::json!({}),
            },
            previous: None,
            aggregate_id: Uuid::new_v4(),
            sequence: 1,
        };

        let node = IpldEventNode::new(cid.clone(), payload);

        assert_eq!(node.cid, cid);
        assert!(node.links.is_empty()); // No previous
    }

    #[test]
    fn test_event_node_linkage() {
        let cid1 = Cid::new("QmFirst");
        let cid2 = Cid::new("QmSecond");

        let payload = EventPayload {
            cid: cid2.clone(),
            data: EventData::NodeAdded {
                node_id: "test".to_string(),
                node_type: "test".to_string(),
                data: serde_json::json!({}),
            },
            previous: Some(cid1.clone()),
            aggregate_id: Uuid::new_v4(),
            sequence: 2,
        };

        let node = IpldEventNode::new(cid2.clone(), payload);

        assert_eq!(node.previous(), Some(&cid1));
        assert_eq!(node.links.get("previous"), Some(&cid1));
    }

    #[test]
    fn test_ipld_event_node_implements_node_trait() {
        let cid = Cid::new("QmNodeTrait");
        let payload = EventPayload {
            cid: cid.clone(),
            data: EventData::NodeAdded {
                node_id: "test".to_string(),
                node_type: "test".to_string(),
                data: serde_json::json!({}),
            },
            previous: None,
            aggregate_id: Uuid::new_v4(),
            sequence: 1,
        };

        let node = IpldEventNode::new(cid.clone(), payload);
        assert_eq!(node.id(), "QmNodeTrait");
    }

    // ========================================================================
    // IpldChainEdge tests
    // ========================================================================

    #[test]
    fn test_ipld_chain_edge() {
        let edge = IpldChainEdge {
            id: "edge1".to_string(),
            source: Cid::new("QmSource"),
            target: Cid::new("QmTarget"),
            link_type: "previous".to_string(),
        };

        assert_eq!(edge.id(), "edge1");
        assert_eq!(edge.source(), "QmSource");
        assert_eq!(edge.target(), "QmTarget");
    }

    // ========================================================================
    // MockCidGenerator tests
    // ========================================================================

    #[test]
    fn test_mock_cid_generator_generate() {
        let generator = MockCidGenerator;

        let data1 = EventData::NodeAdded {
            node_id: "unique_node_1".to_string(),
            node_type: "test_type_first".to_string(),
            data: serde_json::json!({"value": 111, "key": "first"}),
        };

        // Use a completely different event type to ensure different CIDs
        let data2 = EventData::NodeRemoved {
            node_id: "unique_node_2".to_string(),
        };

        let cid1 = generator.generate_cid(&data1);
        let cid2 = generator.generate_cid(&data2);

        // Different data should produce different CIDs
        assert_ne!(cid1, cid2);
        // CIDs should start with "Qm"
        assert!(cid1.as_str().starts_with("Qm"));
        assert!(cid2.as_str().starts_with("Qm"));
    }

    #[test]
    fn test_mock_cid_generator_verify() {
        let generator = MockCidGenerator;

        let data = EventData::NodeAdded {
            node_id: "node1".to_string(),
            node_type: "test".to_string(),
            data: serde_json::json!({}),
        };

        let cid = generator.generate_cid(&data);

        // Same data should verify
        assert!(generator.verify_cid(&cid, &data));

        // Different data should not verify
        let different_data = EventData::NodeRemoved {
            node_id: "node1".to_string(),
        };
        assert!(!generator.verify_cid(&cid, &different_data));
    }

    // ========================================================================
    // EventChainBuilder tests
    // ========================================================================

    #[test]
    fn test_event_chain_builder_new() {
        let generator = MockCidGenerator;
        let builder = EventChainBuilder::new(generator);

        // Should create successfully
        let _ = format!("{:?}", builder);
    }

    #[test]
    fn test_cid_chain_construction() {
        let generator = MockCidGenerator;
        let mut builder = EventChainBuilder::new(generator);

        let aggregate_id = Uuid::new_v4();
        let events = vec![
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id,
                sequence: 1,
                subject: "graph.events".to_string(),
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
                subject: "graph.events".to_string(),
                timestamp: Utc::now(),
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                data: EventData::NodeAdded {
                    node_id: "node1".to_string(),
                    node_type: "data".to_string(),
                    data: serde_json::json!({"value": 42}),
                },
            },
        ];

        let chain = builder.build_chain(&events);

        assert_eq!(chain.aggregate_id, aggregate_id);
        // Length is the highest sequence + 1
        assert_eq!(chain.length, 3);
        assert_eq!(chain.cids.len(), 2);

        // Verify chain linkage
        let cid1 = chain.get_cid(1).unwrap();
        let cid2 = chain.get_cid(2).unwrap();
        assert_ne!(cid1, cid2); // Different CIDs for different events
    }

    #[test]
    fn test_event_chain_builder_retrieve_by_cid() {
        let generator = MockCidGenerator;
        let builder = EventChainBuilder::new(generator);

        let cid = Cid::new("QmTest");
        let result = builder.retrieve_by_cid(&cid);

        // Should return error (not implemented)
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Not implemented"));
    }

    // ========================================================================
    // IpldChainCommand tests
    // ========================================================================

    #[test]
    fn test_ipld_chain_command_store_event() {
        let cmd = IpldChainCommand::StoreEvent {
            event: GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: Uuid::new_v4(),
                sequence: 1,
                subject: "test".to_string(),
                timestamp: Utc::now(),
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                data: EventData::NodeAdded {
                    node_id: "node1".to_string(),
                    node_type: "test".to_string(),
                    data: serde_json::json!({}),
                },
            },
            previous_cid: Some(Cid::new("QmPrev")),
        };

        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("StoreEvent"));
    }

    #[test]
    fn test_ipld_chain_command_retrieve_chain() {
        let cmd = IpldChainCommand::RetrieveChain {
            root_cid: Cid::new("QmRoot"),
        };

        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("RetrieveChain"));
        assert!(json.contains("QmRoot"));
    }

    #[test]
    fn test_ipld_chain_command_verify_chain() {
        let cmd = IpldChainCommand::VerifyChain {
            root_cid: Cid::new("QmVerify"),
        };

        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("VerifyChain"));
    }
}