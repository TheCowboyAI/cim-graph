/*
 * Copyright (c) 2025 - Cowboy AI, LLC.
 */

//! Core types for Conceptual Spaces
//!
//! Provides serializable mathematical types and domain structures for
//! representing concepts in topological spaces.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Custom serializable 3D point for conceptual spaces
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Point3<T> {
    /// X coordinate
    pub x: T,
    /// Y coordinate
    pub y: T,
    /// Z coordinate
    pub z: T,
}

impl<T> Point3<T> {
    /// Creates a new Point3 with the given coordinates
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }
}

impl<T: Copy> Point3<T> {
    /// Converts to nalgebra Vector3 for computations
    pub fn coords(&self) -> nalgebra::Vector3<T> {
        nalgebra::Vector3::new(self.x, self.y, self.z)
    }
}

impl<T: Copy + std::ops::Sub<Output = T>> std::ops::Sub for Point3<T> {
    type Output = Vector3<T>;
    
    fn sub(self, rhs: Self) -> Self::Output {
        Vector3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl From<nalgebra::Point3<f64>> for Point3<f64> {
    fn from(point: nalgebra::Point3<f64>) -> Self {
        Point3::new(point.x, point.y, point.z)
    }
}

impl From<Point3<f64>> for nalgebra::Point3<f64> {
    fn from(point: Point3<f64>) -> Self {
        nalgebra::Point3::new(point.x, point.y, point.z)
    }
}

/// Custom serializable 3D vector for conceptual spaces
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Vector3<T> {
    /// X component of the vector
    pub x: T,
    /// Y component of the vector
    pub y: T,
    /// Z component of the vector
    pub z: T,
}

impl<T> Vector3<T> {
    /// Creates a new Vector3 with the given components
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }
}

impl<T: Copy + std::ops::Mul<Output = T> + std::ops::Add<Output = T> + num_traits::Float> Vector3<T> {
    /// Computes the magnitude (length) of the vector
    pub fn magnitude(&self) -> T {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
}

impl From<nalgebra::Vector3<f64>> for Vector3<f64> {
    fn from(vec: nalgebra::Vector3<f64>) -> Self {
        Vector3::new(vec.x, vec.y, vec.z)
    }
}

impl From<Vector3<f64>> for nalgebra::Vector3<f64> {
    fn from(vec: Vector3<f64>) -> Self {
        nalgebra::Vector3::new(vec.x, vec.y, vec.z)
    }
}

/// Custom serializable unit vector for conceptual spaces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitVector3<T> {
    /// Normalized vector components
    pub vector: Vector3<T>,
}

impl<T: Copy + num_traits::Float> UnitVector3<T> {
    /// Creates a new unit vector by normalizing the input vector
    pub fn new_normalize(vec: Vector3<T>) -> Self {
        let magnitude = (vec.x * vec.x + vec.y * vec.y + vec.z * vec.z).sqrt();
        Self {
            vector: Vector3::new(vec.x / magnitude, vec.y / magnitude, vec.z / magnitude),
        }
    }
}

/// Concept node represents semantic knowledge in the space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptNode {
    /// Unique identifier for the concept
    pub id: String,
    /// Properties defining the concept's characteristics
    pub properties: HashMap<String, serde_json::Value>,
    /// References to connected edges
    pub edges: Vec<String>,
}

/// Concept edge represents semantic relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptEdge {
    /// Unique identifier for the edge
    pub id: String,
    /// Source concept node ID
    pub from_node: String,
    /// Target concept node ID
    pub to_node: String,
    /// Type of relationship this edge represents
    pub edge_type: String,
    /// Additional properties of the relationship
    pub properties: HashMap<String, serde_json::Value>,
}

/// Quality dimensions emerging from concept relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityDimension {
    /// Unique identifier for this dimension
    pub dimension_id: String,
    /// Type of quality dimension
    pub dimension_type: QualityType,
    /// Concept from which this dimension originates
    pub origin_concept_id: String,
    /// Target concept for this dimension
    pub target_concept_id: String,
    /// Direction vector in conceptual space
    pub direction: UnitVector3<f64>,
    /// Magnitude or strength of this dimension
    pub magnitude: f64,
    /// Whether this dimension emerged from interactions
    pub is_emergent: bool,
    /// Stability measure (0.0 to 1.0)
    pub stability: f64,
}

/// Types of quality dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityType {
    /// Scalar dimension (temperature, brightness)
    Scalar { 
        /// Valid range for scalar values
        range: (f64, f64) 
    },
    /// Vector dimension (force, velocity)
    Vector { 
        /// Basis vectors defining the vector space
        basis: Vec<Vector3<f64>> 
    },
    /// Categorical dimension (color, shape)
    Categorical { 
        /// Available categories
        categories: Vec<String> 
    },
    /// Ordinal dimension with ordering
    Ordinal { 
        /// Ordered list of values
        ordering: Vec<String> 
    },
    /// Derived from other dimensions
    Derived {
        /// Mathematical formula for derivation
        formula: String,
        /// IDs of dimensions this depends on
        dependencies: Vec<String>,
    },
}

/// Emergent patterns from concept interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergentPattern {
    /// Unique identifier for this pattern
    pub pattern_id: String,
    /// Type classification of the pattern
    pub pattern_type: PatternType,
    /// Set of concepts involved in this pattern
    pub involved_concepts: HashSet<String>,
    /// Stability measure (0.0 to 1.0)
    pub stability: f64,
    /// Energy level of the pattern
    pub energy: f64,
    /// Unix timestamp when pattern formed
    pub formation_timestamp: u64,
    /// Events that triggered pattern formation
    pub triggering_events: Vec<String>,
}

/// Types of emergent patterns in conceptual space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    /// Stable cluster of related concepts
    ConceptCluster { 
        /// Center point of the cluster
        centroid: Point3<f64>, 
        /// Radius of the cluster sphere
        radius: f64 
    },
    /// Oscillating relationship strength
    RelationshipOscillation { 
        /// Period of oscillation in time units
        period: f64, 
        /// Amplitude of the oscillation
        amplitude: f64 
    },
    /// Hierarchical structure
    ConceptHierarchy {
        /// Root concept at the top of hierarchy
        root_concept: String,
        /// Nested levels of the hierarchy
        levels: Vec<Vec<String>>,
    },
    /// Traveling activation wave
    ActivationWave {
        /// Wavelength of the activation wave
        wavelength: f64,
        /// Propagation speed
        speed: f64,
        /// Direction of wave propagation
        direction: Vector3<f64>,
    },
    /// Spiral arrangement of concepts
    ConceptSpiral { 
        /// Center point of the spiral
        center: Point3<f64>, 
        /// Pitch of the spiral (vertical spacing)
        pitch: f64 
    },
}

/// Validation result for consistency checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the validation passed
    pub is_valid: bool,
    /// List of validation violations found
    pub violations: Vec<String>,
    /// List of validation warnings
    pub warnings: Vec<String>,
}