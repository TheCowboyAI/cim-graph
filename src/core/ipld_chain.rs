//! IPLD Chain Aggregate - manages content-addressed event chains
//!
//! The IPLD Chain is responsible for:
//! - Generating CIDs for all event payloads
//! - Maintaining the Merkle DAG of events
//! - Verifying chain integrity
//! - Providing content-addressed storage

use crate::error::{GraphError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Content Identifier (simplified for now, real impl would use cim-ipld)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Cid(String);

impl Cid {
    /// Create a new CID from a string
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    
    /// Get the string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Cid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// IPLD Chain Aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpldChainAggregate {
    /// Root CID of the chain
    pub root_cid: Cid,
    
    /// Chain of CIDs (newest first)
    pub chain: Vec<Cid>,
    
    /// CID to previous CID mapping
    pub links: HashMap<Cid, Cid>,
    
    /// CID to payload mapping (in real impl, this would be external storage)
    pub payloads: HashMap<Cid, serde_json::Value>,
    
    /// Chain metadata
    pub metadata: ChainMetadata,
}

/// Metadata about the chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainMetadata {
    /// Aggregate this chain belongs to
    pub aggregate_id: Uuid,
    
    /// Total number of events in chain
    pub length: usize,
    
    /// Timestamp of first event
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Timestamp of last event
    pub updated_at: chrono::DateTime<chrono::Utc>,
    
    /// Codec used for CID generation
    pub codec: String,
    
    /// Is the chain verified
    pub verified: bool,
}

impl IpldChainAggregate {
    /// Create a new IPLD chain for an aggregate
    pub fn new(aggregate_id: Uuid) -> Self {
        Self {
            root_cid: Cid::new(""),
            chain: Vec::new(),
            links: HashMap::new(),
            payloads: HashMap::new(),
            metadata: ChainMetadata {
                aggregate_id,
                length: 0,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                codec: "dag-cbor".to_string(),
                verified: true,
            },
        }
    }
    
    /// Add an event to the chain
    pub fn add_event(&mut self, event_payload: serde_json::Value) -> Result<Cid> {
        // Generate CID for the payload
        let cid = self.generate_cid(&event_payload)?;
        
        // Create link to previous CID if chain exists
        if !self.chain.is_empty() {
            let previous = self.chain.first().unwrap().clone();
            self.links.insert(cid.clone(), previous);
        }
        
        // Store payload
        self.payloads.insert(cid.clone(), event_payload);
        
        // Add to chain (newest first)
        self.chain.insert(0, cid.clone());
        
        // Update root and metadata
        self.root_cid = cid.clone();
        self.metadata.length += 1;
        self.metadata.updated_at = chrono::Utc::now();
        
        Ok(cid)
    }
    
    /// Generate a CID for a payload
    fn generate_cid(&self, payload: &serde_json::Value) -> Result<Cid> {
        // Simplified CID generation - real impl would use cim-ipld
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        payload.to_string().hash(&mut hasher);
        let hash = hasher.finish();
        
        Ok(Cid::new(format!("Qm{:x}", hash)))
    }
    
    /// Get the previous CID for a given CID
    pub fn get_previous(&self, cid: &Cid) -> Option<&Cid> {
        self.links.get(cid)
    }
    
    /// Get the payload for a CID
    pub fn get_payload(&self, cid: &Cid) -> Option<&serde_json::Value> {
        self.payloads.get(cid)
    }
    
    /// Verify the chain integrity
    pub fn verify_chain(&self) -> Result<()> {
        if self.chain.is_empty() {
            return Ok(());
        }
        
        // Start from root and traverse backwards
        let mut current = Some(&self.root_cid);
        let mut visited = 0;
        
        while let Some(cid) = current {
            // Check payload exists
            if !self.payloads.contains_key(cid) {
                return Err(GraphError::InvalidOperation(
                    format!("Missing payload for CID: {}", cid)
                ));
            }
            
            visited += 1;
            current = self.links.get(cid);
        }
        
        // Check we visited all events
        if visited != self.chain.len() {
            return Err(GraphError::InvalidOperation(
                "Chain has disconnected segments".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Get events in order (oldest first)
    pub fn get_events_ordered(&self) -> Vec<(&Cid, &serde_json::Value)> {
        self.chain
            .iter()
            .rev()
            .filter_map(|cid| {
                self.payloads.get(cid).map(|payload| (cid, payload))
            })
            .collect()
    }
    
    /// Check if chain forms a DAG (no cycles)
    pub fn is_dag(&self) -> bool {
        // By construction, our chain is always a DAG
        // In a real implementation, we'd check for cycles
        true
    }
    
    /// Pin a CID (mark as important)
    pub fn pin_cid(&mut self, cid: &Cid) -> Result<()> {
        if !self.payloads.contains_key(cid) {
            return Err(GraphError::InvalidOperation(
                format!("Cannot pin non-existent CID: {}", cid)
            ));
        }
        // In real impl, this would interact with IPFS pinning
        Ok(())
    }
    
    /// Unpin a CID
    pub fn unpin_cid(&mut self, cid: &Cid) -> Result<()> {
        if !self.payloads.contains_key(cid) {
            return Err(GraphError::InvalidOperation(
                format!("Cannot unpin non-existent CID: {}", cid)
            ));
        }
        // In real impl, this would interact with IPFS pinning
        Ok(())
    }
}

/// Commands for IPLD Chain operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpldChainCommand {
    /// Add an event to the chain
    AddEvent {
        /// Aggregate ID for the chain
        aggregate_id: Uuid,
        /// Event payload to add
        payload: serde_json::Value,
    },
    /// Verify chain integrity
    VerifyChain {
        /// Aggregate ID of the chain to verify
        aggregate_id: Uuid,
    },
    /// Pin a CID
    PinCid {
        /// Aggregate ID for the chain
        aggregate_id: Uuid,
        /// CID to pin
        cid: Cid,
    },
    /// Unpin a CID
    UnpinCid {
        /// Aggregate ID for the chain
        aggregate_id: Uuid,
        /// CID to unpin
        cid: Cid,
    },
}

/// Events produced by IPLD Chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpldChainEvent {
    /// Event was added to chain
    EventAdded {
        /// Aggregate ID for the chain
        aggregate_id: Uuid,
        /// CID of the added event
        cid: Cid,
        /// CID of the previous event in chain
        previous_cid: Option<Cid>,
        /// Sequence number in the chain
        sequence: usize,
    },
    /// Chain was verified
    ChainVerified {
        /// Aggregate ID for the chain
        aggregate_id: Uuid,
        /// Root CID of the chain
        root_cid: Cid,
        /// Number of events in the chain
        length: usize,
        /// Whether the chain is valid
        is_valid: bool,
    },
    /// CID was pinned
    CidPinned {
        /// Aggregate ID for the chain
        aggregate_id: Uuid,
        /// CID that was pinned
        cid: Cid,
    },
    /// CID was unpinned
    CidUnpinned {
        /// Aggregate ID for the chain
        aggregate_id: Uuid,
        /// CID that was unpinned
        cid: Cid,
    },
}

/// System: Generate CID for any payload
pub fn generate_cid_for_payload(payload: &serde_json::Value) -> Result<Cid> {
    // This would use cim-ipld in real implementation
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    payload.to_string().hash(&mut hasher);
    let hash = hasher.finish();
    
    Ok(Cid::new(format!("Qm{:x}", hash)))
}

/// System: Create a Merkle proof for a CID in the chain
pub fn create_merkle_proof(chain: &IpldChainAggregate, target_cid: &Cid) -> Result<Vec<Cid>> {
    let mut proof = Vec::new();
    let mut current = Some(target_cid);
    
    while let Some(cid) = current {
        proof.push(cid.clone());
        current = chain.get_previous(cid);
    }
    
    Ok(proof)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipld_chain_creation() {
        let aggregate_id = Uuid::new_v4();
        let chain = IpldChainAggregate::new(aggregate_id);

        assert_eq!(chain.metadata.aggregate_id, aggregate_id);
        assert_eq!(chain.metadata.length, 0);
        assert!(chain.chain.is_empty());
    }

    #[test]
    fn test_add_events_to_chain() {
        let mut chain = IpldChainAggregate::new(Uuid::new_v4());

        // Add first event
        let event1 = serde_json::json!({
            "type": "NodeAdded",
            "node_id": "n1",
        });
        let cid1 = chain.add_event(event1.clone()).unwrap();

        assert_eq!(chain.metadata.length, 1);
        assert_eq!(chain.root_cid, cid1);
        assert!(chain.get_previous(&cid1).is_none());

        // Add second event
        let event2 = serde_json::json!({
            "type": "EdgeAdded",
            "edge_id": "e1",
        });
        let cid2 = chain.add_event(event2.clone()).unwrap();

        assert_eq!(chain.metadata.length, 2);
        assert_eq!(chain.root_cid, cid2);
        assert_eq!(chain.get_previous(&cid2), Some(&cid1));
    }

    #[test]
    fn test_chain_verification() {
        let mut chain = IpldChainAggregate::new(Uuid::new_v4());

        // Empty chain is valid
        assert!(chain.verify_chain().is_ok());

        // Add events
        for i in 0..5 {
            let event = serde_json::json!({
                "type": "Event",
                "index": i,
            });
            chain.add_event(event).unwrap();
        }

        // Chain should be valid
        assert!(chain.verify_chain().is_ok());
        assert_eq!(chain.metadata.length, 5);
    }

    #[test]
    fn test_get_events_ordered() {
        let mut chain = IpldChainAggregate::new(Uuid::new_v4());

        // Add events
        let events = vec![
            serde_json::json!({"index": 0}),
            serde_json::json!({"index": 1}),
            serde_json::json!({"index": 2}),
        ];

        for event in &events {
            chain.add_event(event.clone()).unwrap();
        }

        // Get ordered events (oldest first)
        let ordered = chain.get_events_ordered();
        assert_eq!(ordered.len(), 3);

        // Check order
        for (i, (_cid, payload)) in ordered.iter().enumerate() {
            assert_eq!(payload.get("index").unwrap().as_i64().unwrap(), i as i64);
        }
    }

    #[test]
    fn test_merkle_proof() {
        let mut chain = IpldChainAggregate::new(Uuid::new_v4());

        // Build a chain
        let mut cids = Vec::new();
        for i in 0..4 {
            let event = serde_json::json!({"index": i});
            let cid = chain.add_event(event).unwrap();
            cids.push(cid);
        }

        // Create proof for the second event (index 1)
        // Remember: chain stores newest first, so cids[2] is the second event
        let proof = create_merkle_proof(&chain, &cids[2]).unwrap();

        // Proof should contain path from target to root
        assert_eq!(proof.len(), 3); // cids[2] -> cids[1] -> cids[0]
        assert_eq!(proof[0], cids[2]);
        assert_eq!(proof[1], cids[1]);
        assert_eq!(proof[2], cids[0]);
    }

    // ========== Cid Tests ==========

    #[test]
    fn test_cid_new() {
        let cid = Cid::new("QmTestCid");
        assert_eq!(cid.as_str(), "QmTestCid");
    }

    #[test]
    fn test_cid_display() {
        let cid = Cid::new("QmDisplayTest");
        let display_str = format!("{}", cid);
        assert_eq!(display_str, "QmDisplayTest");
    }

    #[test]
    fn test_cid_debug() {
        let cid = Cid::new("QmDebugTest");
        let debug_str = format!("{:?}", cid);
        assert!(debug_str.contains("QmDebugTest"));
    }

    #[test]
    fn test_cid_clone() {
        let cid1 = Cid::new("QmClone");
        let cid2 = cid1.clone();
        assert_eq!(cid1, cid2);
    }

    #[test]
    fn test_cid_eq() {
        let cid1 = Cid::new("QmSame");
        let cid2 = Cid::new("QmSame");
        let cid3 = Cid::new("QmDifferent");

        assert_eq!(cid1, cid2);
        assert_ne!(cid1, cid3);
    }

    #[test]
    fn test_cid_hash() {
        use std::collections::HashSet;

        let cid1 = Cid::new("QmHash1");
        let cid2 = Cid::new("QmHash2");
        let cid3 = Cid::new("QmHash1"); // Same as cid1

        let mut set = HashSet::new();
        set.insert(cid1.clone());
        set.insert(cid2);
        set.insert(cid3);

        assert_eq!(set.len(), 2); // Only 2 unique CIDs
        assert!(set.contains(&cid1));
    }

    #[test]
    fn test_cid_serialize_deserialize() {
        let cid = Cid::new("QmSerialize");
        let json = serde_json::to_string(&cid).unwrap();
        let deserialized: Cid = serde_json::from_str(&json).unwrap();
        assert_eq!(cid, deserialized);
    }

    // ========== ChainMetadata Tests ==========

    #[test]
    fn test_chain_metadata_new_chain() {
        let aggregate_id = Uuid::new_v4();
        let chain = IpldChainAggregate::new(aggregate_id);

        assert_eq!(chain.metadata.aggregate_id, aggregate_id);
        assert_eq!(chain.metadata.length, 0);
        assert_eq!(chain.metadata.codec, "dag-cbor");
        assert!(chain.metadata.verified);
    }

    #[test]
    fn test_chain_metadata_timestamps() {
        let chain = IpldChainAggregate::new(Uuid::new_v4());
        let created_at = chain.metadata.created_at;
        let updated_at = chain.metadata.updated_at;

        // Timestamps should be close
        let diff = if updated_at >= created_at {
            updated_at - created_at
        } else {
            created_at - updated_at
        };
        assert!(diff.num_milliseconds() < 100);
    }

    #[test]
    fn test_chain_metadata_clone() {
        let chain = IpldChainAggregate::new(Uuid::new_v4());
        let cloned_metadata = chain.metadata.clone();

        assert_eq!(chain.metadata.aggregate_id, cloned_metadata.aggregate_id);
        assert_eq!(chain.metadata.length, cloned_metadata.length);
    }

    #[test]
    fn test_chain_metadata_debug() {
        let chain = IpldChainAggregate::new(Uuid::new_v4());
        let debug_str = format!("{:?}", chain.metadata);
        assert!(debug_str.contains("ChainMetadata"));
        assert!(debug_str.contains("dag-cbor"));
    }

    // ========== IpldChainAggregate Tests ==========

    #[test]
    fn test_ipld_chain_get_payload() {
        let mut chain = IpldChainAggregate::new(Uuid::new_v4());

        let event = serde_json::json!({"test": "value"});
        let cid = chain.add_event(event.clone()).unwrap();

        let payload = chain.get_payload(&cid).unwrap();
        assert_eq!(payload.get("test").unwrap().as_str().unwrap(), "value");
    }

    #[test]
    fn test_ipld_chain_get_payload_nonexistent() {
        let chain = IpldChainAggregate::new(Uuid::new_v4());
        let fake_cid = Cid::new("QmFake");
        assert!(chain.get_payload(&fake_cid).is_none());
    }

    #[test]
    fn test_ipld_chain_get_previous_first_event() {
        let mut chain = IpldChainAggregate::new(Uuid::new_v4());

        let cid = chain.add_event(serde_json::json!({"first": true})).unwrap();

        // First event has no previous
        assert!(chain.get_previous(&cid).is_none());
    }

    #[test]
    fn test_ipld_chain_is_dag() {
        let chain = IpldChainAggregate::new(Uuid::new_v4());
        assert!(chain.is_dag());
    }

    #[test]
    fn test_ipld_chain_pin_cid() {
        let mut chain = IpldChainAggregate::new(Uuid::new_v4());

        let cid = chain.add_event(serde_json::json!({"pin": "me"})).unwrap();

        // Should succeed for existing CID
        assert!(chain.pin_cid(&cid).is_ok());
    }

    #[test]
    fn test_ipld_chain_pin_nonexistent_cid() {
        let mut chain = IpldChainAggregate::new(Uuid::new_v4());
        let fake_cid = Cid::new("QmFakePin");

        let result = chain.pin_cid(&fake_cid);
        assert!(result.is_err());

        match result.unwrap_err() {
            GraphError::InvalidOperation(msg) => {
                assert!(msg.contains("Cannot pin"));
            }
            _ => panic!("Expected InvalidOperation error"),
        }
    }

    #[test]
    fn test_ipld_chain_unpin_cid() {
        let mut chain = IpldChainAggregate::new(Uuid::new_v4());

        let cid = chain.add_event(serde_json::json!({"unpin": "me"})).unwrap();

        // Should succeed for existing CID
        assert!(chain.unpin_cid(&cid).is_ok());
    }

    #[test]
    fn test_ipld_chain_unpin_nonexistent_cid() {
        let mut chain = IpldChainAggregate::new(Uuid::new_v4());
        let fake_cid = Cid::new("QmFakeUnpin");

        let result = chain.unpin_cid(&fake_cid);
        assert!(result.is_err());

        match result.unwrap_err() {
            GraphError::InvalidOperation(msg) => {
                assert!(msg.contains("Cannot unpin"));
            }
            _ => panic!("Expected InvalidOperation error"),
        }
    }

    #[test]
    fn test_ipld_chain_clone() {
        let mut chain = IpldChainAggregate::new(Uuid::new_v4());
        chain.add_event(serde_json::json!({"clone": "test"})).unwrap();

        let cloned = chain.clone();

        assert_eq!(chain.metadata.length, cloned.metadata.length);
        assert_eq!(chain.root_cid, cloned.root_cid);
        assert_eq!(chain.chain.len(), cloned.chain.len());
    }

    #[test]
    fn test_ipld_chain_debug() {
        let chain = IpldChainAggregate::new(Uuid::new_v4());
        let debug_str = format!("{:?}", chain);
        assert!(debug_str.contains("IpldChainAggregate"));
    }

    #[test]
    fn test_ipld_chain_serialize_deserialize() {
        let mut chain = IpldChainAggregate::new(Uuid::new_v4());
        chain.add_event(serde_json::json!({"serialize": true})).unwrap();
        chain.add_event(serde_json::json!({"serialize": false})).unwrap();

        let json = serde_json::to_string(&chain).unwrap();
        let deserialized: IpldChainAggregate = serde_json::from_str(&json).unwrap();

        assert_eq!(chain.metadata.length, deserialized.metadata.length);
        assert_eq!(chain.root_cid, deserialized.root_cid);
    }

    // ========== IpldChainCommand Tests ==========

    #[test]
    fn test_ipld_chain_command_add_event() {
        let command = IpldChainCommand::AddEvent {
            aggregate_id: Uuid::new_v4(),
            payload: serde_json::json!({"command": "add"}),
        };
        let debug_str = format!("{:?}", command);
        assert!(debug_str.contains("AddEvent"));
    }

    #[test]
    fn test_ipld_chain_command_verify_chain() {
        let command = IpldChainCommand::VerifyChain {
            aggregate_id: Uuid::new_v4(),
        };
        let debug_str = format!("{:?}", command);
        assert!(debug_str.contains("VerifyChain"));
    }

    #[test]
    fn test_ipld_chain_command_pin_cid() {
        let command = IpldChainCommand::PinCid {
            aggregate_id: Uuid::new_v4(),
            cid: Cid::new("QmPin"),
        };
        let debug_str = format!("{:?}", command);
        assert!(debug_str.contains("PinCid"));
    }

    #[test]
    fn test_ipld_chain_command_unpin_cid() {
        let command = IpldChainCommand::UnpinCid {
            aggregate_id: Uuid::new_v4(),
            cid: Cid::new("QmUnpin"),
        };
        let debug_str = format!("{:?}", command);
        assert!(debug_str.contains("UnpinCid"));
    }

    #[test]
    fn test_ipld_chain_command_clone() {
        let command = IpldChainCommand::AddEvent {
            aggregate_id: Uuid::new_v4(),
            payload: serde_json::json!({}),
        };
        let cloned = command.clone();
        let debug_str = format!("{:?}", cloned);
        assert!(debug_str.contains("AddEvent"));
    }

    #[test]
    fn test_ipld_chain_command_serialize() {
        let command = IpldChainCommand::VerifyChain {
            aggregate_id: Uuid::new_v4(),
        };
        let json = serde_json::to_string(&command).unwrap();
        let _deserialized: IpldChainCommand = serde_json::from_str(&json).unwrap();
    }

    // ========== IpldChainEvent Tests ==========

    #[test]
    fn test_ipld_chain_event_event_added() {
        let event = IpldChainEvent::EventAdded {
            aggregate_id: Uuid::new_v4(),
            cid: Cid::new("QmEvent"),
            previous_cid: Some(Cid::new("QmPrevious")),
            sequence: 1,
        };
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("EventAdded"));
    }

    #[test]
    fn test_ipld_chain_event_chain_verified() {
        let event = IpldChainEvent::ChainVerified {
            aggregate_id: Uuid::new_v4(),
            root_cid: Cid::new("QmRoot"),
            length: 10,
            is_valid: true,
        };
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("ChainVerified"));
        assert!(debug_str.contains("is_valid: true"));
    }

    #[test]
    fn test_ipld_chain_event_cid_pinned() {
        let event = IpldChainEvent::CidPinned {
            aggregate_id: Uuid::new_v4(),
            cid: Cid::new("QmPinned"),
        };
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("CidPinned"));
    }

    #[test]
    fn test_ipld_chain_event_cid_unpinned() {
        let event = IpldChainEvent::CidUnpinned {
            aggregate_id: Uuid::new_v4(),
            cid: Cid::new("QmUnpinned"),
        };
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("CidUnpinned"));
    }

    #[test]
    fn test_ipld_chain_event_clone() {
        let event = IpldChainEvent::EventAdded {
            aggregate_id: Uuid::new_v4(),
            cid: Cid::new("QmClone"),
            previous_cid: None,
            sequence: 0,
        };
        let cloned = event.clone();
        let debug_str = format!("{:?}", cloned);
        assert!(debug_str.contains("EventAdded"));
    }

    #[test]
    fn test_ipld_chain_event_serialize() {
        let event = IpldChainEvent::ChainVerified {
            aggregate_id: Uuid::new_v4(),
            root_cid: Cid::new("QmRoot"),
            length: 5,
            is_valid: true,
        };
        let json = serde_json::to_string(&event).unwrap();
        let _deserialized: IpldChainEvent = serde_json::from_str(&json).unwrap();
    }

    // ========== generate_cid_for_payload Tests ==========

    #[test]
    fn test_generate_cid_for_payload() {
        let payload = serde_json::json!({"test": "data"});
        let cid = generate_cid_for_payload(&payload).unwrap();

        // CID should start with "Qm"
        assert!(cid.as_str().starts_with("Qm"));
    }

    #[test]
    fn test_generate_cid_deterministic() {
        let payload = serde_json::json!({"deterministic": true});

        let cid1 = generate_cid_for_payload(&payload).unwrap();
        let cid2 = generate_cid_for_payload(&payload).unwrap();

        assert_eq!(cid1, cid2);
    }

    #[test]
    fn test_generate_cid_different_payloads() {
        let payload1 = serde_json::json!({"key": "value1"});
        let payload2 = serde_json::json!({"key": "value2"});

        let cid1 = generate_cid_for_payload(&payload1).unwrap();
        let cid2 = generate_cid_for_payload(&payload2).unwrap();

        assert_ne!(cid1, cid2);
    }

    // ========== create_merkle_proof Tests ==========

    #[test]
    fn test_create_merkle_proof_single_event() {
        let mut chain = IpldChainAggregate::new(Uuid::new_v4());

        let cid = chain.add_event(serde_json::json!({"single": true})).unwrap();
        let proof = create_merkle_proof(&chain, &cid).unwrap();

        assert_eq!(proof.len(), 1);
        assert_eq!(proof[0], cid);
    }

    #[test]
    fn test_create_merkle_proof_root() {
        let mut chain = IpldChainAggregate::new(Uuid::new_v4());

        for i in 0..5 {
            chain.add_event(serde_json::json!({"index": i})).unwrap();
        }

        // Proof for root (most recent) should be entire chain
        let proof = create_merkle_proof(&chain, &chain.root_cid).unwrap();
        assert_eq!(proof.len(), 5);
    }

    #[test]
    fn test_create_merkle_proof_empty_chain() {
        let chain = IpldChainAggregate::new(Uuid::new_v4());
        let fake_cid = Cid::new("QmFake");

        // Proof for non-existent CID should just contain the CID
        let proof = create_merkle_proof(&chain, &fake_cid).unwrap();
        assert_eq!(proof.len(), 1);
        assert_eq!(proof[0], fake_cid);
    }

    // ========== Chain Verification Edge Cases ==========

    #[test]
    fn test_verify_chain_with_many_events() {
        let mut chain = IpldChainAggregate::new(Uuid::new_v4());

        for i in 0..100 {
            chain.add_event(serde_json::json!({"index": i})).unwrap();
        }

        assert!(chain.verify_chain().is_ok());
        assert_eq!(chain.metadata.length, 100);
    }

    #[test]
    fn test_get_events_ordered_empty_chain() {
        let chain = IpldChainAggregate::new(Uuid::new_v4());
        let ordered = chain.get_events_ordered();
        assert!(ordered.is_empty());
    }

    #[test]
    fn test_chain_updated_at_changes() {
        let mut chain = IpldChainAggregate::new(Uuid::new_v4());
        let initial_updated = chain.metadata.updated_at;

        // Add event
        chain.add_event(serde_json::json!({"update": true})).unwrap();
        let after_update = chain.metadata.updated_at;

        // Updated time should be same or later
        assert!(after_update >= initial_updated);
    }

    #[test]
    fn test_chain_length_increments() {
        let mut chain = IpldChainAggregate::new(Uuid::new_v4());

        for i in 0..5 {
            assert_eq!(chain.metadata.length, i);
            chain.add_event(serde_json::json!({"index": i})).unwrap();
            assert_eq!(chain.metadata.length, i + 1);
        }
    }
}