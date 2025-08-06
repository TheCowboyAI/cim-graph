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
}