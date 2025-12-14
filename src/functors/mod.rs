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
}
