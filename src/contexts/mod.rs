//! Bounded contexts module - defines domain boundaries and relationships
//!
//! This module documents the bounded contexts in the CIM Graph system:
//! - IPLD Context: Content-addressed storage
//! - Context Context: Data schemas and transformations  
//! - Workflow Context: State machines and processes
//! - Concept Context: Domain knowledge and reasoning
//! - Composed Context: Multi-graph orchestration


/// Represents a bounded context in the system
#[derive(Debug, Clone, PartialEq)]
pub enum BoundedContext {
    /// IPLD storage and content addressing
    Ipld,
    /// Data schemas and transformations
    Context,
    /// State machines and workflows
    Workflow,
    /// Domain concepts and reasoning
    Concept,
    /// Multi-graph composition
    Composed,
}

impl BoundedContext {
    /// Get the context name as a string
    pub fn name(&self) -> &'static str {
        match self {
            Self::Ipld => "ipld",
            Self::Context => "context",
            Self::Workflow => "workflow",
            Self::Concept => "concept",
            Self::Composed => "composed",
        }
    }
    
    /// Get the aggregate root type for this context
    pub fn aggregate_type(&self) -> &'static str {
        match self {
            Self::Ipld => "IpldChainAggregate",
            Self::Context => "ContextGraph",
            Self::Workflow => "WorkflowGraph",
            Self::Concept => "ConceptGraph",
            Self::Composed => "ComposedGraph",
        }
    }
    
    /// Check if this context depends on another
    pub fn depends_on(&self, other: &BoundedContext) -> bool {
        match (self, other) {
            // All contexts depend on IPLD for storage
            (_, BoundedContext::Ipld) if self != other => true,
            // Workflow depends on Context for data validation
            (BoundedContext::Workflow, BoundedContext::Context) => true,
            // Workflow depends on Concept for reasoning
            (BoundedContext::Workflow, BoundedContext::Concept) => true,
            // Composed depends on all other contexts
            (BoundedContext::Composed, _) if self != other => true,
            _ => false,
        }
    }
    
    /// Get upstream contexts (contexts this one depends on)
    pub fn upstream_contexts(&self) -> Vec<BoundedContext> {
        let all_contexts = vec![
            BoundedContext::Ipld,
            BoundedContext::Context,
            BoundedContext::Workflow,
            BoundedContext::Concept,
            BoundedContext::Composed,
        ];
        
        all_contexts.into_iter()
            .filter(|ctx| self.depends_on(ctx))
            .collect()
    }
    
    /// Get downstream contexts (contexts that depend on this one)
    pub fn downstream_contexts(&self) -> Vec<BoundedContext> {
        let all_contexts = vec![
            BoundedContext::Ipld,
            BoundedContext::Context,
            BoundedContext::Workflow,
            BoundedContext::Concept,
            BoundedContext::Composed,
        ];
        
        all_contexts.into_iter()
            .filter(|ctx| ctx.depends_on(self))
            .collect()
    }
}

/// Represents a relationship between contexts
#[derive(Debug, Clone)]
pub struct ContextRelationship {
    /// Source context in the relationship
    pub from_context: BoundedContext,
    /// Target context in the relationship
    pub to_context: BoundedContext,
    /// Type of relationship between contexts
    pub relationship_type: RelationshipType,
    /// Human-readable description of the relationship
    pub description: String,
}

/// Types of relationships between contexts
#[derive(Debug, Clone, PartialEq)]
pub enum RelationshipType {
    /// Upstream provides services to downstream
    UpstreamDownstream,
    /// Customer context uses supplier context's services
    CustomerSupplier,
    /// Contexts share common abstractions
    SharedKernel,
    /// Context translates between different models
    AntiCorruptionLayer,
    /// Context publishes events consumed by another
    PublishedLanguage,
}

/// Context map showing all relationships
#[derive(Debug)]
pub struct ContextMap {
    relationships: Vec<ContextRelationship>,
}

impl ContextMap {
    /// Create the context map for CIM Graph
    pub fn new() -> Self {
        let relationships = vec![
            // IPLD is upstream to all
            ContextRelationship {
                from_context: BoundedContext::Ipld,
                to_context: BoundedContext::Context,
                relationship_type: RelationshipType::UpstreamDownstream,
                description: "IPLD provides content-addressed storage".into(),
            },
            ContextRelationship {
                from_context: BoundedContext::Ipld,
                to_context: BoundedContext::Workflow,
                relationship_type: RelationshipType::UpstreamDownstream,
                description: "IPLD stores workflow event payloads".into(),
            },
            ContextRelationship {
                from_context: BoundedContext::Ipld,
                to_context: BoundedContext::Concept,
                relationship_type: RelationshipType::UpstreamDownstream,
                description: "IPLD stores concept definitions".into(),
            },
            ContextRelationship {
                from_context: BoundedContext::Ipld,
                to_context: BoundedContext::Composed,
                relationship_type: RelationshipType::UpstreamDownstream,
                description: "IPLD stores composition metadata".into(),
            },
            // Context supplies data services
            ContextRelationship {
                from_context: BoundedContext::Context,
                to_context: BoundedContext::Workflow,
                relationship_type: RelationshipType::CustomerSupplier,
                description: "Context provides data validation for workflows".into(),
            },
            // Concept supplies reasoning
            ContextRelationship {
                from_context: BoundedContext::Concept,
                to_context: BoundedContext::Workflow,
                relationship_type: RelationshipType::CustomerSupplier,
                description: "Concept provides reasoning for workflow decisions".into(),
            },
            ContextRelationship {
                from_context: BoundedContext::Concept,
                to_context: BoundedContext::Composed,
                relationship_type: RelationshipType::CustomerSupplier,
                description: "Concept provides cross-domain reasoning".into(),
            },
            // Composed is downstream from all
            ContextRelationship {
                from_context: BoundedContext::Workflow,
                to_context: BoundedContext::Composed,
                relationship_type: RelationshipType::UpstreamDownstream,
                description: "Composed orchestrates workflows".into(),
            },
            ContextRelationship {
                from_context: BoundedContext::Context,
                to_context: BoundedContext::Composed,
                relationship_type: RelationshipType::UpstreamDownstream,
                description: "Composed uses data transformations".into(),
            },
            // Shared kernel relationships
            ContextRelationship {
                from_context: BoundedContext::Workflow,
                to_context: BoundedContext::Concept,
                relationship_type: RelationshipType::SharedKernel,
                description: "Share GraphProjection abstraction".into(),
            },
            // Anti-corruption layers
            ContextRelationship {
                from_context: BoundedContext::Composed,
                to_context: BoundedContext::Workflow,
                relationship_type: RelationshipType::AntiCorruptionLayer,
                description: "Projection validates workflow events".into(),
            },
        ];
        
        Self { relationships }
    }
    
    /// Get all relationships for a context
    pub fn relationships_for(&self, context: &BoundedContext) -> Vec<&ContextRelationship> {
        self.relationships.iter()
            .filter(|r| &r.from_context == context || &r.to_context == context)
            .collect()
    }
    
    /// Get relationships between two contexts
    pub fn relationships_between(&self, from: &BoundedContext, to: &BoundedContext) -> Vec<&ContextRelationship> {
        self.relationships.iter()
            .filter(|r| &r.from_context == from && &r.to_context == to)
            .collect()
    }
}

/// Integration patterns between contexts
pub mod integration {
    use super::*;
    
    /// Cross-context event flow
    #[derive(Debug)]
    pub struct EventFlow {
        /// Context where events originate
        pub source_context: BoundedContext,
        /// Context that receives events
        pub target_context: BoundedContext,
        /// Pattern describing the event flow
        pub event_pattern: String,
    }
    
    /// Common integration patterns
    /// Creates a workflow validation event flow pattern
    pub fn workflow_validation_pattern() -> EventFlow {
        EventFlow {
            source_context: BoundedContext::Workflow,
            target_context: BoundedContext::Context,
            event_pattern: "StateTransitioned -> ValidateData -> ValidationCompleted".into(),
        }
    }
    
    /// Creates a concept reasoning event flow pattern
    pub fn concept_reasoning_pattern() -> EventFlow {
        EventFlow {
            source_context: BoundedContext::Concept,
            target_context: BoundedContext::Workflow,
            event_pattern: "InferenceCompleted -> DecisionMade -> StateTransitioned".into(),
        }
    }
    
    /// Creates a composed orchestration event flow pattern
    pub fn composed_orchestration_pattern() -> EventFlow {
        EventFlow {
            source_context: BoundedContext::Composed,
            target_context: BoundedContext::Workflow,
            event_pattern: "IntegrationTriggered -> Execute -> ExecutionCompleted".into(),
        }
    }
    
    /// Creates an IPLD storage event flow pattern
    pub fn ipld_storage_pattern() -> EventFlow {
        EventFlow {
            source_context: BoundedContext::Workflow,
            target_context: BoundedContext::Ipld,
            event_pattern: "EventEmitted -> GenerateCID -> CidAdded".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_context_dependencies() {
        // IPLD has no dependencies
        let ipld = BoundedContext::Ipld;
        assert!(ipld.upstream_contexts().is_empty());
        assert!(!ipld.downstream_contexts().is_empty());
        
        // Workflow depends on IPLD, Context, and Concept
        let workflow = BoundedContext::Workflow;
        let upstream = workflow.upstream_contexts();
        assert!(upstream.contains(&BoundedContext::Ipld));
        assert!(upstream.contains(&BoundedContext::Context));
        assert!(upstream.contains(&BoundedContext::Concept));
        
        // Composed depends on all other contexts
        let composed = BoundedContext::Composed;
        let upstream = composed.upstream_contexts();
        assert_eq!(upstream.len(), 4); // All except itself
    }
    
    #[test]
    fn test_context_map() {
        let map = ContextMap::new();
        
        // Test IPLD relationships
        let ipld_rels = map.relationships_for(&BoundedContext::Ipld);
        assert!(!ipld_rels.is_empty());
        
        // Test workflow-context relationship
        let wf_ctx_rels = map.relationships_between(
            &BoundedContext::Context,
            &BoundedContext::Workflow
        );
        assert_eq!(wf_ctx_rels.len(), 1);
        assert_eq!(wf_ctx_rels[0].relationship_type, RelationshipType::CustomerSupplier);
    }
}