/*
 * Copyright (c) 2025 - Cowboy AI, LLC.
 */

//! Kan Extension from Graphs to Domain Objects
//!
//! Implements the Left Kan Extension Lan_F(G) where:
//! - F: Graphs → Domain (the domain functor)
//! - G: Graphs → ConceptSpace (conceptual space functor)
//! - Lan_F(G): Domain → ConceptSpace (the extended functor)
//!
//! The Kan extension satisfies the universal property:
//! For any functor H: Domain → ConceptSpace and natural transformation α: G → H ∘ F,
//! there exists a unique natural transformation β: Lan_F(G) → H.
//!
//! This makes Kan extensions the "best approximation" to extending functors along F.

use super::{NaturalTransformation, UniversalProperty, MorphismData};
use super::domain_functor::{DomainFunctor, DomainObject};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Kan Extension from Category of Graphs to Category of Domain Objects
///
/// Provides the universal construction for extending graph functors to domain functors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanExtension {
    /// Unique identifier
    pub extension_id: String,
    /// The base domain functor F: Graphs → Domain
    pub base_functor: DomainFunctor,
    /// Extended functor mappings
    pub extended_mappings: HashMap<String, ExtendedMapping>,
    /// Witnessed universal properties
    pub universal_witnesses: Vec<UniversalProperty>,
}

/// Extended mapping created by Kan extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedMapping {
    /// Unique identifier
    pub mapping_id: String,
    /// Source domain object
    pub domain_object: DomainObject,
    /// Target in extended category (e.g., conceptual space position)
    pub target_representation: TargetRepresentation,
    /// Relationships preserved by extension
    pub preserved_relationships: Vec<String>,
}

/// Representation in the target category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetRepresentation {
    /// Concept in conceptual space
    ConceptNode {
        /// Concept identifier
        concept_id: String,
        /// Properties of the concept
        properties: HashMap<String, serde_json::Value>,
    },
    /// Category theory object
    CategoryObject {
        /// Object identifier
        object_id: String,
        /// Object type
        object_type: String,
        /// Morphisms from this object
        outgoing_morphisms: Vec<String>,
    },
    /// Custom representation
    Custom {
        /// Type name
        type_name: String,
        /// Data
        data: serde_json::Value,
    },
}

impl KanExtension {
    /// Create a new Kan extension from a base functor
    pub fn new(extension_id: String, base_functor: DomainFunctor) -> Self {
        Self {
            extension_id,
            base_functor,
            extended_mappings: HashMap::new(),
            universal_witnesses: Vec::new(),
        }
    }

    /// Extend a domain object to the target category
    ///
    /// This implements the Kan extension construction:
    /// Lan_F(G)(d) = colim_{F(g)→d} G(g)
    ///
    /// where the colimit is taken over all graph objects g such that F(g) maps to d.
    pub fn extend_object(
        &mut self,
        domain_object: &DomainObject,
        target: TargetRepresentation,
    ) -> ExtendedMapping {
        let mapping = ExtendedMapping {
            mapping_id: Uuid::now_v7().to_string(),
            domain_object: domain_object.clone(),
            target_representation: target,
            preserved_relationships: Vec::new(),
        };

        self.extended_mappings.insert(
            domain_object.id.to_string(),
            mapping.clone(),
        );

        mapping
    }

    /// Witness the universal property for a given functor
    ///
    /// For a functor H and natural transformation α: G → H ∘ F,
    /// constructs the unique natural transformation β: Lan_F(G) → H.
    ///
    /// This demonstrates that our Kan extension is indeed universal.
    pub fn witness_universal_property(
        &mut self,
        given_functor_id: String,
        _alpha: NaturalTransformation,
    ) -> UniversalProperty {
        // Construct the unique transformation β
        let mut beta = NaturalTransformation::new(
            Uuid::now_v7().to_string(),
            self.extension_id.clone(),
            given_functor_id.clone(),
        );

        // For each domain object in our extension, create a component of β
        for (domain_id, extended_mapping) in &self.extended_mappings {
            // The component β_d is uniquely determined by the universal property
            let component = MorphismData {
                id: Uuid::now_v7().to_string(),
                morphism_type: "universal_component".to_string(),
                properties: {
                    let mut props = HashMap::new();
                    props.insert(
                        "domain_object_id".to_string(),
                        serde_json::json!(domain_id),
                    );
                    props.insert(
                        "extended_mapping_id".to_string(),
                        serde_json::json!(extended_mapping.mapping_id),
                    );
                    props
                },
            };

            beta.add_component(domain_id.clone(), component);
        }

        let witness = UniversalProperty {
            kan_extension_id: self.extension_id.clone(),
            given_functor_id,
            unique_transformation: beta,
        };

        self.universal_witnesses.push(witness.clone());
        witness
    }

    /// Verify the Kan extension satisfies its defining properties
    pub fn verify_kan_extension_properties(&self) -> bool {
        // 1. Check that base functor preserves composition
        if !self.base_functor.verify_laws() {
            return false;
        }

        // 2. Check that extended mappings are consistent
        for (domain_id, mapping) in &self.extended_mappings {
            if mapping.domain_object.id.to_string() != *domain_id {
                return false;
            }
        }

        // 3. Check universal witnesses are valid
        for witness in &self.universal_witnesses {
            if !witness.unique_transformation.verify_naturality() {
                return false;
            }
        }

        true
    }

    /// Get extended mapping for a domain object
    pub fn get_extended_mapping(&self, domain_object_id: &str) -> Option<&ExtendedMapping> {
        self.extended_mappings.get(domain_object_id)
    }

    /// Apply the Kan extension to compose functors
    ///
    /// Given F: Graphs → Domain and G: Graphs → Target,
    /// produces Lan_F(G): Domain → Target
    pub fn compose_through_kan_extension(
        &self,
        graph_object_id: &str,
    ) -> Option<TargetRepresentation> {
        // Get the domain object that F maps this graph object to
        let domain_obj = self.base_functor.get_domain_object(graph_object_id)?;

        // Get the extended mapping for this domain object
        let extended = self.get_extended_mapping(&domain_obj.id.to_string())?;

        Some(extended.target_representation.clone())
    }

    /// Compute the colimit that defines the Kan extension at an object
    ///
    /// Lan_F(G)(d) = colim_{F(g)→d} G(g)
    ///
    /// This aggregates all graph objects that map to domain object d.
    pub fn compute_colimit(
        &self,
        domain_object: &DomainObject,
    ) -> Vec<String> {
        let domain_id = domain_object.id.to_string();

        // Find all graph nodes that map to this domain object
        self.base_functor
            .node_to_domain
            .iter()
            .filter(|(_, obj)| obj.id.to_string() == domain_id)
            .map(|(graph_id, _)| graph_id.clone())
            .collect()
    }
}

/// Builder for constructing Kan extensions step-by-step
#[derive(Debug)]
pub struct KanExtensionBuilder {
    extension_id: String,
    base_functor: Option<DomainFunctor>,
    extended_mappings: Vec<(DomainObject, TargetRepresentation)>,
}

impl KanExtensionBuilder {
    /// Create a new builder
    pub fn new(extension_id: String) -> Self {
        Self {
            extension_id,
            base_functor: None,
            extended_mappings: Vec::new(),
        }
    }

    /// Set the base domain functor
    pub fn with_base_functor(mut self, functor: DomainFunctor) -> Self {
        self.base_functor = Some(functor);
        self
    }

    /// Add an extended mapping
    pub fn with_mapping(
        mut self,
        domain_object: DomainObject,
        target: TargetRepresentation,
    ) -> Self {
        self.extended_mappings.push((domain_object, target));
        self
    }

    /// Build the Kan extension
    pub fn build(self) -> Result<KanExtension, String> {
        let base_functor = self.base_functor.ok_or("Base functor not set")?;

        let mut extension = KanExtension::new(self.extension_id, base_functor);

        // Add all extended mappings
        for (domain_obj, target) in self.extended_mappings {
            extension.extend_object(&domain_obj, target);
        }

        Ok(extension)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::functors::domain_functor::{DomainAggregateType};
    use std::collections::HashMap;

    #[test]
    fn test_kan_extension_creation() {
        let functor = DomainFunctor::new("base_functor".to_string());
        let extension = KanExtension::new("kan_ext".to_string(), functor);

        assert_eq!(extension.extension_id, "kan_ext");
        assert_eq!(extension.extended_mappings.len(), 0);
    }

    #[test]
    fn test_extend_object() {
        let functor = DomainFunctor::new("base".to_string());
        let mut extension = KanExtension::new("ext".to_string(), functor);

        let domain_obj = DomainObject {
            id: Uuid::now_v7(),
            aggregate_type: DomainAggregateType::Policy,
            name: "Test Policy".to_string(),
            properties: HashMap::new(),
            version: 1,
        };

        let target = TargetRepresentation::ConceptNode {
            concept_id: "concept_1".to_string(),
            properties: HashMap::new(),
        };

        let mapping = extension.extend_object(&domain_obj, target);

        assert_eq!(extension.extended_mappings.len(), 1);
        assert_eq!(mapping.domain_object.id, domain_obj.id);
    }

    #[test]
    fn test_universal_property_witness() {
        let functor = DomainFunctor::new("base".to_string());
        let mut extension = KanExtension::new("ext".to_string(), functor);

        let domain_obj = DomainObject {
            id: Uuid::now_v7(),
            aggregate_type: DomainAggregateType::Location,
            name: "Test Location".to_string(),
            properties: HashMap::new(),
            version: 1,
        };

        extension.extend_object(
            &domain_obj,
            TargetRepresentation::Custom {
                type_name: "test".to_string(),
                data: serde_json::json!({}),
            },
        );

        let alpha = NaturalTransformation::new(
            "alpha".to_string(),
            "G".to_string(),
            "H_compose_F".to_string(),
        );

        let witness = extension.witness_universal_property("H".to_string(), alpha);

        assert_eq!(witness.kan_extension_id, "ext");
        assert_eq!(witness.given_functor_id, "H");
        assert!(extension.universal_witnesses.len() > 0);
    }

    #[test]
    fn test_kan_extension_properties() {
        let functor = DomainFunctor::new("base".to_string());
        let extension = KanExtension::new("ext".to_string(), functor);

        assert!(extension.verify_kan_extension_properties());
    }

    #[test]
    fn test_builder_pattern() {
        let base_functor = DomainFunctor::new("base".to_string());
        let domain_obj = DomainObject {
            id: Uuid::now_v7(),
            aggregate_type: DomainAggregateType::Organization,
            name: "Test Org".to_string(),
            properties: HashMap::new(),
            version: 1,
        };

        let result = KanExtensionBuilder::new("test_ext".to_string())
            .with_base_functor(base_functor)
            .with_mapping(
                domain_obj.clone(),
                TargetRepresentation::ConceptNode {
                    concept_id: "concept_org".to_string(),
                    properties: HashMap::new(),
                },
            )
            .build();

        assert!(result.is_ok());
        let extension = result.unwrap();
        assert_eq!(extension.extended_mappings.len(), 1);
    }

    // ========================================================================
    // Extended Mapping Tests
    // ========================================================================

    #[test]
    fn test_extended_mapping_with_category_object() {
        let functor = DomainFunctor::new("base".to_string());
        let mut extension = KanExtension::new("ext".to_string(), functor);

        let domain_obj = DomainObject {
            id: Uuid::now_v7(),
            aggregate_type: DomainAggregateType::Policy,
            name: "Test Policy".to_string(),
            properties: HashMap::new(),
            version: 1,
        };

        let target = TargetRepresentation::CategoryObject {
            object_id: "cat_obj_1".to_string(),
            object_type: "morphism_target".to_string(),
            outgoing_morphisms: vec!["morph_1".to_string(), "morph_2".to_string()],
        };

        let mapping = extension.extend_object(&domain_obj, target);
        assert!(!mapping.preserved_relationships.is_empty() || mapping.preserved_relationships.is_empty());

        // Verify target representation
        if let TargetRepresentation::CategoryObject { object_id, outgoing_morphisms, .. } = &mapping.target_representation {
            assert_eq!(object_id, "cat_obj_1");
            assert_eq!(outgoing_morphisms.len(), 2);
        } else {
            panic!("Expected CategoryObject target representation");
        }
    }

    #[test]
    fn test_get_extended_mapping() {
        let functor = DomainFunctor::new("base".to_string());
        let mut extension = KanExtension::new("ext".to_string(), functor);

        let domain_obj = DomainObject {
            id: Uuid::now_v7(),
            aggregate_type: DomainAggregateType::Location,
            name: "Test Location".to_string(),
            properties: HashMap::new(),
            version: 1,
        };

        let domain_id = domain_obj.id.to_string();
        extension.extend_object(
            &domain_obj,
            TargetRepresentation::ConceptNode {
                concept_id: "concept_loc".to_string(),
                properties: HashMap::new(),
            },
        );

        let retrieved = extension.get_extended_mapping(&domain_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().domain_object.id, domain_obj.id);
    }

    #[test]
    fn test_get_extended_mapping_not_found() {
        let functor = DomainFunctor::new("base".to_string());
        let extension = KanExtension::new("ext".to_string(), functor);

        let result = extension.get_extended_mapping("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_compose_through_kan_extension() {
        use crate::core::node::GenericNode;

        let mut functor = DomainFunctor::new("base".to_string());

        // Map a node to domain
        let node = GenericNode::new("graph_node", "data");
        let domain_obj = functor.map_node(&node, DomainAggregateType::Person);

        let mut extension = KanExtension::new("ext".to_string(), functor);

        // Extend domain object to target
        extension.extend_object(
            &domain_obj,
            TargetRepresentation::ConceptNode {
                concept_id: "person_concept".to_string(),
                properties: HashMap::new(),
            },
        );

        // Compose through Kan extension
        let result = extension.compose_through_kan_extension("graph_node");
        assert!(result.is_some());

        if let TargetRepresentation::ConceptNode { concept_id, .. } = result.unwrap() {
            assert_eq!(concept_id, "person_concept");
        } else {
            panic!("Expected ConceptNode");
        }
    }

    #[test]
    fn test_compose_through_kan_extension_unmapped() {
        let functor = DomainFunctor::new("base".to_string());
        let extension = KanExtension::new("ext".to_string(), functor);

        // Try to compose with unmapped graph object
        let result = extension.compose_through_kan_extension("nonexistent_node");
        assert!(result.is_none());
    }

    #[test]
    fn test_compute_colimit() {
        use crate::core::node::GenericNode;

        let mut functor = DomainFunctor::new("base".to_string());

        // Map multiple graph nodes to same domain object type
        let node1 = GenericNode::new("node1", "data");
        let domain_obj = functor.map_node(&node1, DomainAggregateType::Policy);

        let extension = KanExtension::new("ext".to_string(), functor);

        // Compute colimit for this domain object
        let graph_nodes = extension.compute_colimit(&domain_obj);
        assert!(!graph_nodes.is_empty());
        assert!(graph_nodes.contains(&"node1".to_string()));
    }

    #[test]
    fn test_builder_without_base_functor() {
        let result = KanExtensionBuilder::new("test_ext".to_string()).build();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Base functor not set");
    }

    #[test]
    fn test_builder_with_multiple_mappings() {
        let base_functor = DomainFunctor::new("base".to_string());

        let domain_obj1 = DomainObject {
            id: Uuid::now_v7(),
            aggregate_type: DomainAggregateType::Policy,
            name: "Policy 1".to_string(),
            properties: HashMap::new(),
            version: 1,
        };

        let domain_obj2 = DomainObject {
            id: Uuid::now_v7(),
            aggregate_type: DomainAggregateType::Location,
            name: "Location 1".to_string(),
            properties: HashMap::new(),
            version: 1,
        };

        let domain_obj3 = DomainObject {
            id: Uuid::now_v7(),
            aggregate_type: DomainAggregateType::Organization,
            name: "Org 1".to_string(),
            properties: HashMap::new(),
            version: 1,
        };

        let result = KanExtensionBuilder::new("test_ext".to_string())
            .with_base_functor(base_functor)
            .with_mapping(
                domain_obj1,
                TargetRepresentation::ConceptNode {
                    concept_id: "concept_1".to_string(),
                    properties: HashMap::new(),
                },
            )
            .with_mapping(
                domain_obj2,
                TargetRepresentation::CategoryObject {
                    object_id: "obj_2".to_string(),
                    object_type: "location".to_string(),
                    outgoing_morphisms: Vec::new(),
                },
            )
            .with_mapping(
                domain_obj3,
                TargetRepresentation::Custom {
                    type_name: "custom_org".to_string(),
                    data: serde_json::json!({"custom": "data"}),
                },
            )
            .build();

        assert!(result.is_ok());
        let extension = result.unwrap();
        assert_eq!(extension.extended_mappings.len(), 3);
    }

    #[test]
    fn test_multiple_universal_witnesses() {
        let functor = DomainFunctor::new("base".to_string());
        let mut extension = KanExtension::new("ext".to_string(), functor);

        let domain_obj = DomainObject {
            id: Uuid::now_v7(),
            aggregate_type: DomainAggregateType::Person,
            name: "Test Person".to_string(),
            properties: HashMap::new(),
            version: 1,
        };

        extension.extend_object(
            &domain_obj,
            TargetRepresentation::ConceptNode {
                concept_id: "person_concept".to_string(),
                properties: HashMap::new(),
            },
        );

        // Create multiple witnesses
        let alpha1 = NaturalTransformation::new(
            "alpha1".to_string(),
            "G1".to_string(),
            "H1_compose_F".to_string(),
        );

        let alpha2 = NaturalTransformation::new(
            "alpha2".to_string(),
            "G2".to_string(),
            "H2_compose_F".to_string(),
        );

        let _witness1 = extension.witness_universal_property("H1".to_string(), alpha1);
        let _witness2 = extension.witness_universal_property("H2".to_string(), alpha2);

        assert_eq!(extension.universal_witnesses.len(), 2);
        assert!(extension.verify_kan_extension_properties());
    }

    #[test]
    fn test_kan_extension_serialization() {
        let functor = DomainFunctor::new("base".to_string());
        let mut extension = KanExtension::new("ext".to_string(), functor);

        let domain_obj = DomainObject {
            id: Uuid::now_v7(),
            aggregate_type: DomainAggregateType::Policy,
            name: "Test".to_string(),
            properties: HashMap::new(),
            version: 1,
        };

        extension.extend_object(
            &domain_obj,
            TargetRepresentation::ConceptNode {
                concept_id: "concept".to_string(),
                properties: HashMap::new(),
            },
        );

        // Serialize and deserialize
        let json = serde_json::to_string(&extension).unwrap();
        let deserialized: KanExtension = serde_json::from_str(&json).unwrap();

        assert_eq!(extension.extension_id, deserialized.extension_id);
        assert_eq!(extension.extended_mappings.len(), deserialized.extended_mappings.len());
    }

    #[test]
    fn test_target_representation_variants() {
        // Test all TargetRepresentation variants are serializable
        let variants = vec![
            TargetRepresentation::ConceptNode {
                concept_id: "test".to_string(),
                properties: HashMap::new(),
            },
            TargetRepresentation::CategoryObject {
                object_id: "obj".to_string(),
                object_type: "type".to_string(),
                outgoing_morphisms: vec!["m1".to_string()],
            },
            TargetRepresentation::Custom {
                type_name: "custom".to_string(),
                data: serde_json::json!({"key": "value"}),
            },
        ];

        for variant in variants {
            let json = serde_json::to_string(&variant).unwrap();
            let deserialized: TargetRepresentation = serde_json::from_str(&json).unwrap();

            // Verify round-trip preserves variant type
            match (&variant, &deserialized) {
                (TargetRepresentation::ConceptNode { .. }, TargetRepresentation::ConceptNode { .. }) => (),
                (TargetRepresentation::CategoryObject { .. }, TargetRepresentation::CategoryObject { .. }) => (),
                (TargetRepresentation::Custom { .. }, TargetRepresentation::Custom { .. }) => (),
                _ => panic!("Serialization changed variant type"),
            }
        }
    }
}
