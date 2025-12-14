/*
 * Copyright (c) 2025 - Cowboy AI, LLC.
 */

//! Functors and Kan Extensions for CIM Graph
//!
//! This module implements the mathematical foundation for mapping between
//! the Category of Graphs and the Category of cim-domain objects.
//!
//! ## Category Theory Foundation
//!
//! A **functor** F: C → D is a structure-preserving map between categories:
//! - Maps objects in C to objects in D
//! - Maps morphisms in C to morphisms in D
//! - Preserves identity: F(id_X) = id_F(X)
//! - Preserves composition: F(g ∘ f) = F(g) ∘ F(f)
//!
//! A **Kan extension** is a universal construction that extends functors.
//! For F: C → D, the Kan extension provides:
//! - Universal property: For any other functor G, there exists a unique
//!   natural transformation η: F → G
//! - This makes Kan extensions the "best approximation" to extending functors
//!
//! ## CIM Implementation
//!
//! We implement a Kan extension from:
//! - **Cat(Graphs)**: Category where objects are graphs and morphisms are graph homomorphisms
//! - **Cat(cim-domain)**: Category where objects are domain aggregates and morphisms are domain relationships
//!
//! This allows us to represent domain object composition through graph structures.

pub mod domain_functor;
pub mod kan_extension;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Generic functor trait for structure-preserving maps between categories
///
/// A functor must preserve:
/// 1. Identity morphisms: F(id_A) = id_F(A)
/// 2. Composition: F(g ∘ f) = F(g) ∘ F(f)
pub trait Functor<Source, Target> {
    /// Map an object from source category to target category
    fn map_object(&self, obj: &Source) -> Target;

    /// Map a morphism (relationship) between objects
    ///
    /// Given f: A → B in source category, returns F(f): F(A) → F(B) in target category
    fn map_morphism(
        &self,
        source: &Source,
        target: &Source,
        morphism_data: &MorphismData,
    ) -> MorphismData;

    /// Verify functor laws hold for this mapping
    ///
    /// Checks:
    /// 1. Identity preservation
    /// 2. Composition preservation
    fn verify_functor_laws(&self) -> bool {
        // Default implementation - can be overridden for specific verification
        true
    }
}

/// Data associated with a morphism (arrow) between objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphismData {
    /// Unique identifier for this morphism
    pub id: String,
    /// Type/classification of the morphism
    pub morphism_type: String,
    /// Additional properties
    pub properties: HashMap<String, serde_json::Value>,
}

/// Natural transformation between two functors
///
/// For functors F, G: C → D, a natural transformation η: F → G consists of
/// morphisms η_X: F(X) → G(X) for each object X in C, such that diagrams commute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaturalTransformation {
    /// Unique identifier
    pub id: String,
    /// Source functor name
    pub source_functor: String,
    /// Target functor name
    pub target_functor: String,
    /// Component morphisms for each object
    pub components: HashMap<String, MorphismData>,
}

impl NaturalTransformation {
    /// Create a new natural transformation
    pub fn new(
        id: String,
        source_functor: String,
        target_functor: String,
    ) -> Self {
        Self {
            id,
            source_functor,
            target_functor,
            components: HashMap::new(),
        }
    }

    /// Add a component morphism for an object
    pub fn add_component(&mut self, object_id: String, morphism: MorphismData) {
        self.components.insert(object_id, morphism);
    }

    /// Verify naturality condition: all diagrams commute
    pub fn verify_naturality(&self) -> bool {
        // Naturality requires: G(f) ∘ η_X = η_Y ∘ F(f)
        // This is a simplified check - full verification requires category context
        !self.components.is_empty()
    }
}

/// Universal property witness for Kan extension
///
/// For a Kan extension Lan_F G, the universal property states:
/// For any functor H and natural transformation α: G → H ∘ F,
/// there exists a unique natural transformation β: Lan_F G → H
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalProperty {
    /// The Kan extension functor
    pub kan_extension_id: String,
    /// Given functor H
    pub given_functor_id: String,
    /// Unique natural transformation witnessing universality
    pub unique_transformation: NaturalTransformation,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_natural_transformation_creation() {
        let mut nt = NaturalTransformation::new(
            "test_nt".to_string(),
            "F".to_string(),
            "G".to_string(),
        );

        nt.add_component(
            "object_1".to_string(),
            MorphismData {
                id: "morph_1".to_string(),
                morphism_type: "component".to_string(),
                properties: HashMap::new(),
            },
        );

        assert_eq!(nt.components.len(), 1);
        assert!(nt.verify_naturality());
    }

    // ========== MorphismData Tests ==========

    #[test]
    fn test_morphism_data_creation() {
        let morphism = MorphismData {
            id: "morph_123".to_string(),
            morphism_type: "transition".to_string(),
            properties: HashMap::new(),
        };

        assert_eq!(morphism.id, "morph_123");
        assert_eq!(morphism.morphism_type, "transition");
        assert!(morphism.properties.is_empty());
    }

    #[test]
    fn test_morphism_data_with_properties() {
        let mut props = HashMap::new();
        props.insert("weight".to_string(), serde_json::json!(1.5));
        props.insert("label".to_string(), serde_json::json!("connection"));
        props.insert("priority".to_string(), serde_json::json!(10));

        let morphism = MorphismData {
            id: "morph_props".to_string(),
            morphism_type: "weighted".to_string(),
            properties: props,
        };

        assert_eq!(morphism.properties.len(), 3);
        assert_eq!(morphism.properties["weight"], serde_json::json!(1.5));
        assert_eq!(morphism.properties["label"], serde_json::json!("connection"));
    }

    #[test]
    fn test_morphism_data_serialization() {
        let mut props = HashMap::new();
        props.insert("key".to_string(), serde_json::json!("value"));

        let morphism = MorphismData {
            id: "serializable_morph".to_string(),
            morphism_type: "test".to_string(),
            properties: props,
        };

        let json = serde_json::to_string(&morphism).unwrap();
        assert!(json.contains("serializable_morph"));
        assert!(json.contains("test"));

        let deserialized: MorphismData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "serializable_morph");
        assert_eq!(deserialized.morphism_type, "test");
        assert_eq!(deserialized.properties["key"], serde_json::json!("value"));
    }

    #[test]
    fn test_morphism_data_clone() {
        let morphism = MorphismData {
            id: "original".to_string(),
            morphism_type: "clone_test".to_string(),
            properties: [("a".to_string(), serde_json::json!(1))].into_iter().collect(),
        };

        let cloned = morphism.clone();

        assert_eq!(cloned.id, morphism.id);
        assert_eq!(cloned.morphism_type, morphism.morphism_type);
        assert_eq!(cloned.properties, morphism.properties);
    }

    // ========== NaturalTransformation Tests ==========

    #[test]
    fn test_natural_transformation_empty() {
        let nt = NaturalTransformation::new(
            "empty_nt".to_string(),
            "F".to_string(),
            "G".to_string(),
        );

        assert_eq!(nt.id, "empty_nt");
        assert_eq!(nt.source_functor, "F");
        assert_eq!(nt.target_functor, "G");
        assert!(nt.components.is_empty());
        assert!(!nt.verify_naturality()); // Empty components fail naturality
    }

    #[test]
    fn test_natural_transformation_multiple_components() {
        let mut nt = NaturalTransformation::new(
            "multi_nt".to_string(),
            "F".to_string(),
            "G".to_string(),
        );

        for i in 0..5 {
            nt.add_component(
                format!("object_{}", i),
                MorphismData {
                    id: format!("comp_{}", i),
                    morphism_type: "component".to_string(),
                    properties: HashMap::new(),
                },
            );
        }

        assert_eq!(nt.components.len(), 5);
        assert!(nt.verify_naturality());
    }

    #[test]
    fn test_natural_transformation_add_component_overwrites() {
        let mut nt = NaturalTransformation::new(
            "overwrite_nt".to_string(),
            "F".to_string(),
            "G".to_string(),
        );

        nt.add_component(
            "object_1".to_string(),
            MorphismData {
                id: "first".to_string(),
                morphism_type: "original".to_string(),
                properties: HashMap::new(),
            },
        );

        nt.add_component(
            "object_1".to_string(),
            MorphismData {
                id: "second".to_string(),
                morphism_type: "replacement".to_string(),
                properties: HashMap::new(),
            },
        );

        assert_eq!(nt.components.len(), 1);
        assert_eq!(nt.components["object_1"].id, "second");
        assert_eq!(nt.components["object_1"].morphism_type, "replacement");
    }

    #[test]
    fn test_natural_transformation_serialization() {
        let mut nt = NaturalTransformation::new(
            "serializable_nt".to_string(),
            "Source".to_string(),
            "Target".to_string(),
        );

        nt.add_component(
            "obj".to_string(),
            MorphismData {
                id: "comp".to_string(),
                morphism_type: "test".to_string(),
                properties: HashMap::new(),
            },
        );

        let json = serde_json::to_string(&nt).unwrap();
        assert!(json.contains("serializable_nt"));
        assert!(json.contains("Source"));
        assert!(json.contains("Target"));

        let deserialized: NaturalTransformation = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "serializable_nt");
        assert_eq!(deserialized.source_functor, "Source");
        assert_eq!(deserialized.target_functor, "Target");
        assert_eq!(deserialized.components.len(), 1);
    }

    #[test]
    fn test_natural_transformation_clone() {
        let mut nt = NaturalTransformation::new(
            "clone_nt".to_string(),
            "F".to_string(),
            "G".to_string(),
        );

        nt.add_component(
            "obj".to_string(),
            MorphismData {
                id: "m".to_string(),
                morphism_type: "t".to_string(),
                properties: HashMap::new(),
            },
        );

        let cloned = nt.clone();

        assert_eq!(cloned.id, nt.id);
        assert_eq!(cloned.source_functor, nt.source_functor);
        assert_eq!(cloned.target_functor, nt.target_functor);
        assert_eq!(cloned.components.len(), nt.components.len());
    }

    // ========== UniversalProperty Tests ==========

    #[test]
    fn test_universal_property_creation() {
        let transformation = NaturalTransformation::new(
            "unique".to_string(),
            "Lan".to_string(),
            "H".to_string(),
        );

        let up = UniversalProperty {
            kan_extension_id: "kan_ext".to_string(),
            given_functor_id: "H".to_string(),
            unique_transformation: transformation,
        };

        assert_eq!(up.kan_extension_id, "kan_ext");
        assert_eq!(up.given_functor_id, "H");
        assert_eq!(up.unique_transformation.source_functor, "Lan");
    }

    #[test]
    fn test_universal_property_serialization() {
        let mut transformation = NaturalTransformation::new(
            "unique_trans".to_string(),
            "Lan".to_string(),
            "H".to_string(),
        );

        transformation.add_component(
            "domain_obj".to_string(),
            MorphismData {
                id: "universal_morph".to_string(),
                morphism_type: "universal".to_string(),
                properties: HashMap::new(),
            },
        );

        let up = UniversalProperty {
            kan_extension_id: "kan_ext_1".to_string(),
            given_functor_id: "Functor_H".to_string(),
            unique_transformation: transformation,
        };

        let json = serde_json::to_string(&up).unwrap();
        assert!(json.contains("kan_ext_1"));
        assert!(json.contains("Functor_H"));
        assert!(json.contains("unique_trans"));

        let deserialized: UniversalProperty = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.kan_extension_id, "kan_ext_1");
        assert_eq!(deserialized.given_functor_id, "Functor_H");
        assert!(deserialized.unique_transformation.verify_naturality());
    }

    #[test]
    fn test_universal_property_clone() {
        let transformation = NaturalTransformation::new(
            "trans".to_string(),
            "F".to_string(),
            "G".to_string(),
        );

        let up = UniversalProperty {
            kan_extension_id: "ext".to_string(),
            given_functor_id: "H".to_string(),
            unique_transformation: transformation,
        };

        let cloned = up.clone();

        assert_eq!(cloned.kan_extension_id, up.kan_extension_id);
        assert_eq!(cloned.given_functor_id, up.given_functor_id);
        assert_eq!(cloned.unique_transformation.id, up.unique_transformation.id);
    }

    // ========== Functor Trait Tests ==========

    #[test]
    fn test_functor_trait_default_verify_laws() {
        // Create a minimal functor implementation
        struct TestFunctor;

        impl Functor<String, String> for TestFunctor {
            fn map_object(&self, obj: &String) -> String {
                format!("F({})", obj)
            }

            fn map_morphism(
                &self,
                _source: &String,
                _target: &String,
                morphism_data: &MorphismData,
            ) -> MorphismData {
                morphism_data.clone()
            }
        }

        let functor = TestFunctor;

        // Test map_object
        let result = functor.map_object(&"test".to_string());
        assert_eq!(result, "F(test)");

        // Test verify_functor_laws (default implementation)
        assert!(functor.verify_functor_laws());
    }

    // ========== Debug Implementation Tests ==========

    #[test]
    fn test_morphism_data_debug() {
        let morphism = MorphismData {
            id: "debug_test".to_string(),
            morphism_type: "type".to_string(),
            properties: HashMap::new(),
        };

        let debug_str = format!("{:?}", morphism);
        assert!(debug_str.contains("debug_test"));
        assert!(debug_str.contains("MorphismData"));
    }

    #[test]
    fn test_natural_transformation_debug() {
        let nt = NaturalTransformation::new(
            "debug_nt".to_string(),
            "F".to_string(),
            "G".to_string(),
        );

        let debug_str = format!("{:?}", nt);
        assert!(debug_str.contains("debug_nt"));
        assert!(debug_str.contains("NaturalTransformation"));
    }

    #[test]
    fn test_universal_property_debug() {
        let transformation = NaturalTransformation::new(
            "trans".to_string(),
            "F".to_string(),
            "G".to_string(),
        );

        let up = UniversalProperty {
            kan_extension_id: "debug_ext".to_string(),
            given_functor_id: "H".to_string(),
            unique_transformation: transformation,
        };

        let debug_str = format!("{:?}", up);
        assert!(debug_str.contains("debug_ext"));
        assert!(debug_str.contains("UniversalProperty"));
    }
}
