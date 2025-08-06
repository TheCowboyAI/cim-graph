//! Automated policies for event-driven graph operations
//!
//! Policies are automated reactions to events that maintain system invariants

use crate::core::ipld_chain::{IpldChainAggregate, Cid, generate_cid_for_payload};
use crate::core::state_machine::GraphStateMachine;
use crate::events::{GraphEvent, EventPayload};
use crate::error::Result;
use std::collections::HashMap;
use uuid::Uuid;

/// Policy execution context
pub struct PolicyContext<'a> {
    /// State machine for validation
    pub state_machine: &'a mut GraphStateMachine,
    
    /// IPLD chains for each aggregate
    pub ipld_chains: &'a mut HashMap<Uuid, IpldChainAggregate>,
    
    /// Policy execution metrics
    pub metrics: PolicyMetrics,
}

impl<'a> std::fmt::Debug for PolicyContext<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PolicyContext")
            .field("state_machine", &"<mutable reference>")
            .field("ipld_chains", &self.ipld_chains.len())
            .field("metrics", &self.metrics)
            .finish()
    }
}

/// Metrics for policy execution
#[derive(Debug, Default, Clone)]
pub struct PolicyMetrics {
    /// Number of CIDs generated
    pub cids_generated: usize,
    /// Number of projections updated
    pub projections_updated: usize,
    /// Number of chains validated
    pub chains_validated: usize,
    /// Number of errors caught and handled
    pub errors_caught: usize,
    /// Number of events replayed
    pub events_replayed: usize,
}

/// Trait for automated policies
pub trait Policy: Send + Sync {
    /// Name of the policy
    fn name(&self) -> &str;
    
    /// Check if policy should trigger for this event
    fn should_trigger(&self, event: &GraphEvent) -> bool;
    
    /// Execute the policy
    fn execute(&self, event: &GraphEvent, context: &mut PolicyContext<'_>) -> Result<Vec<PolicyAction>>;
}

/// Actions that policies can take
#[derive(Debug, Clone)]
pub enum PolicyAction {
    /// Generate and store CID for event
    GenerateCid { 
        /// ID of the event to generate CID for
        event_id: Uuid, 
        /// Generated CID
        cid: Cid 
    },
    
    /// Update projection for aggregate
    UpdateProjection { 
        /// ID of the aggregate to update projection for
        aggregate_id: Uuid 
    },
    
    /// Validate state transition
    ValidateTransition { 
        /// ID of the aggregate to validate
        aggregate_id: Uuid 
    },
    
    /// Verify chain integrity
    VerifyChain { 
        /// ID of the aggregate chain to verify
        aggregate_id: Uuid 
    },
    
    /// Invalidate cache entry
    InvalidateCache { 
        /// ID of the aggregate to invalidate cache for
        aggregate_id: Uuid 
    },
    
    /// Replay events from sequence
    ReplayEvents { 
        /// ID of the aggregate to replay events for
        aggregate_id: Uuid, 
        /// Starting sequence number
        from_sequence: u64 
    },
}

/// CID Generation Policy - generates CIDs for all events
#[derive(Debug)]
pub struct CidGenerationPolicy;

impl Policy for CidGenerationPolicy {
    fn name(&self) -> &str {
        "CID Generation Policy"
    }
    
    fn should_trigger(&self, _event: &GraphEvent) -> bool {
        // Triggers for ALL events
        true
    }
    
    fn execute(&self, event: &GraphEvent, context: &mut PolicyContext<'_>) -> Result<Vec<PolicyAction>> {
        // Generate CID from event payload
        let payload_json = serde_json::to_value(&event.payload)?;
        let cid = generate_cid_for_payload(&payload_json)?;
        
        // Get or create IPLD chain for this aggregate
        let chain = context
            .ipld_chains
            .entry(event.aggregate_id)
            .or_insert_with(|| IpldChainAggregate::new(event.aggregate_id));
        
        // Add event to chain
        chain.add_event(payload_json)?;
        
        context.metrics.cids_generated += 1;
        
        Ok(vec![PolicyAction::GenerateCid {
            event_id: event.event_id,
            cid,
        }])
    }
}

/// Projection Update Policy - updates projections when events are published
#[derive(Debug)]
pub struct ProjectionUpdatePolicy;

impl Policy for ProjectionUpdatePolicy {
    fn name(&self) -> &str {
        "Projection Update Policy"
    }
    
    fn should_trigger(&self, _event: &GraphEvent) -> bool {
        // Triggers for all events that modify state
        true
    }
    
    fn execute(&self, event: &GraphEvent, context: &mut PolicyContext<'_>) -> Result<Vec<PolicyAction>> {
        let mut actions = vec![];
        
        // Update projection for this aggregate
        actions.push(PolicyAction::UpdateProjection {
            aggregate_id: event.aggregate_id,
        });
        
        // Invalidate cache
        actions.push(PolicyAction::InvalidateCache {
            aggregate_id: event.aggregate_id,
        });
        
        context.metrics.projections_updated += 1;
        
        Ok(actions)
    }
}

/// State Validation Policy - validates state transitions
#[derive(Debug)]
pub struct StateValidationPolicy;

impl Policy for StateValidationPolicy {
    fn name(&self) -> &str {
        "State Validation Policy"
    }
    
    fn should_trigger(&self, event: &GraphEvent) -> bool {
        // Trigger for events that change state
        matches!(
            &event.payload,
            EventPayload::Generic(p) if p.event_type.contains("State") || 
                                       p.event_type.contains("Transitioned")
        )
    }
    
    fn execute(&self, event: &GraphEvent, context: &mut PolicyContext<'_>) -> Result<Vec<PolicyAction>> {
        // Apply event to state machine
        context.state_machine.apply_event(event);
        
        Ok(vec![PolicyAction::ValidateTransition {
            aggregate_id: event.aggregate_id,
        }])
    }
}

/// Chain Validation Policy - validates IPLD chain integrity
#[derive(Debug)]
pub struct ChainValidationPolicy;

impl Policy for ChainValidationPolicy {
    fn name(&self) -> &str {
        "Chain Validation Policy"
    }
    
    fn should_trigger(&self, event: &GraphEvent) -> bool {
        // Trigger periodically or for specific events
        match &event.payload {
            EventPayload::Ipld(_) => true,
            EventPayload::Generic(p) if p.event_type == "ChainModified" => true,
            _ => false,
        }
    }
    
    fn execute(&self, event: &GraphEvent, context: &mut PolicyContext<'_>) -> Result<Vec<PolicyAction>> {
        if let Some(chain) = context.ipld_chains.get(&event.aggregate_id) {
            // Verify chain integrity
            match chain.verify_chain() {
                Ok(_) => {
                    context.metrics.chains_validated += 1;
                    Ok(vec![PolicyAction::VerifyChain {
                        aggregate_id: event.aggregate_id,
                    }])
                }
                Err(e) => {
                    context.metrics.errors_caught += 1;
                    Err(e)
                }
            }
        } else {
            Ok(vec![])
        }
    }
}

/// Collaboration Policy - handles client subscriptions and event replay
#[derive(Debug)]
pub struct CollaborationPolicy;

impl Policy for CollaborationPolicy {
    fn name(&self) -> &str {
        "Collaboration Policy"
    }
    
    fn should_trigger(&self, event: &GraphEvent) -> bool {
        // Trigger for subscription events
        matches!(
            &event.payload,
            EventPayload::Generic(p) if p.event_type == "ClientSubscribed"
        )
    }
    
    fn execute(&self, event: &GraphEvent, context: &mut PolicyContext<'_>) -> Result<Vec<PolicyAction>> {
        // Extract client's last known sequence from event
        let last_sequence = event.payload
            .as_generic()
            .and_then(|p| p.data.get("last_sequence"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        context.metrics.events_replayed += 1;
        
        Ok(vec![PolicyAction::ReplayEvents {
            aggregate_id: event.aggregate_id,
            from_sequence: last_sequence,
        }])
    }
}

/// Policy engine that executes all policies
pub struct PolicyEngine {
    policies: Vec<Box<dyn Policy>>,
    metrics: PolicyMetrics,
}

impl std::fmt::Debug for PolicyEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PolicyEngine")
            .field("policies", &self.policies.len())
            .field("metrics", &self.metrics)
            .finish()
    }
}

impl PolicyEngine {
    /// Create a new policy engine with default policies
    pub fn new() -> Self {
        Self {
            policies: vec![
                Box::new(CidGenerationPolicy),
                Box::new(ProjectionUpdatePolicy),
                Box::new(StateValidationPolicy),
                Box::new(ChainValidationPolicy),
                Box::new(CollaborationPolicy),
            ],
            metrics: PolicyMetrics::default(),
        }
    }
    
    /// Add a custom policy
    pub fn add_policy(&mut self, policy: Box<dyn Policy>) {
        self.policies.push(policy);
    }
    
    /// Execute all applicable policies for an event
    pub fn execute_policies(
        &mut self,
        event: &GraphEvent,
        context: &mut PolicyContext<'_>,
    ) -> Result<Vec<PolicyAction>> {
        let mut all_actions = Vec::new();
        
        for policy in &self.policies {
            if policy.should_trigger(event) {
                match policy.execute(event, context) {
                    Ok(actions) => all_actions.extend(actions),
                    Err(e) => {
                        // Log error but continue with other policies
                        eprintln!("Policy {} failed: {}", policy.name(), e);
                        context.metrics.errors_caught += 1;
                    }
                }
            }
        }
        
        // Update engine metrics from context
        self.metrics.cids_generated = context.metrics.cids_generated;
        self.metrics.projections_updated = context.metrics.projections_updated;
        self.metrics.chains_validated = context.metrics.chains_validated;
        self.metrics.errors_caught = context.metrics.errors_caught;
        self.metrics.events_replayed = context.metrics.events_replayed;
        
        Ok(all_actions)
    }
    
    /// Get current metrics
    pub fn get_metrics(&self) -> &PolicyMetrics {
        &self.metrics
    }
}

/// Extension for EventPayload to check type
impl EventPayload {
    fn as_generic(&self) -> Option<&crate::events::GenericPayload> {
        match self {
            EventPayload::Generic(p) => Some(p),
            _ => None,
        }
    }
}

/// System: Process event through all policies
pub fn process_event_policies(
    event: &GraphEvent,
    engine: &mut PolicyEngine,
    context: &mut PolicyContext<'_>,
) -> Result<Vec<PolicyAction>> {
    engine.execute_policies(event, context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::GenericPayload;

    #[test]
    fn test_cid_generation_policy() {
        let policy = CidGenerationPolicy;
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Generic(GenericPayload {
                event_type: "TestEvent".to_string(),
                data: serde_json::json!({ "test": true }),
            }),
        };
        
        assert!(policy.should_trigger(&event));
        
        let mut state_machine = GraphStateMachine::new();
        let mut ipld_chains = HashMap::new();
        let mut context = PolicyContext {
            state_machine: &mut state_machine,
            ipld_chains: &mut ipld_chains,
            metrics: PolicyMetrics::default(),
        };
        
        let actions = policy.execute(&event, &mut context).unwrap();
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], PolicyAction::GenerateCid { .. }));
        assert_eq!(context.metrics.cids_generated, 1);
    }

    #[test]
    fn test_projection_update_policy() {
        let policy = ProjectionUpdatePolicy;
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Generic(GenericPayload {
                event_type: "NodeAdded".to_string(),
                data: serde_json::json!({ "node_id": "n1" }),
            }),
        };
        
        assert!(policy.should_trigger(&event));
        
        let mut state_machine = GraphStateMachine::new();
        let mut ipld_chains = HashMap::new();
        let mut context = PolicyContext {
            state_machine: &mut state_machine,
            ipld_chains: &mut ipld_chains,
            metrics: PolicyMetrics::default(),
        };
        
        let actions = policy.execute(&event, &mut context).unwrap();
        assert_eq!(actions.len(), 2);
        assert!(matches!(actions[0], PolicyAction::UpdateProjection { .. }));
        assert!(matches!(actions[1], PolicyAction::InvalidateCache { .. }));
        assert_eq!(context.metrics.projections_updated, 1);
    }

    #[test]
    fn test_policy_engine() {
        let mut engine = PolicyEngine::new();
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Generic(GenericPayload {
                event_type: "TestEvent".to_string(),
                data: serde_json::json!({ "test": true }),
            }),
        };
        
        let mut state_machine = GraphStateMachine::new();
        let mut ipld_chains = HashMap::new();
        let mut context = PolicyContext {
            state_machine: &mut state_machine,
            ipld_chains: &mut ipld_chains,
            metrics: PolicyMetrics::default(),
        };
        
        let actions = engine.execute_policies(&event, &mut context).unwrap();
        
        // Should have actions from CID generation and projection update policies
        assert!(actions.len() >= 2);
        assert!(context.metrics.cids_generated > 0);
        assert!(context.metrics.projections_updated > 0);
    }
}