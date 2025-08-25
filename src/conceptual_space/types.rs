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
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Point3<T> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }
}

impl<T: Copy> Point3<T> {
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
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Vector3<T> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }
}

impl<T: Copy + std::ops::Mul<Output = T> + std::ops::Add<Output = T> + num_traits::Float> Vector3<T> {
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
    pub vector: Vector3<T>,
}

impl<T: Copy + num_traits::Float> UnitVector3<T> {
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
    pub id: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub edges: Vec<String>,
}

/// Concept edge represents semantic relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptEdge {
    pub id: String,
    pub from_node: String,
    pub to_node: String,
    pub edge_type: String,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Quality dimensions emerging from concept relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityDimension {
    pub dimension_id: String,
    pub dimension_type: QualityType,
    pub origin_concept_id: String,
    pub target_concept_id: String,
    pub direction: UnitVector3<f64>,
    pub magnitude: f64,
    pub is_emergent: bool,
    pub stability: f64,
}

/// Types of quality dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityType {
    /// Scalar dimension (temperature, brightness)
    Scalar { range: (f64, f64) },
    /// Vector dimension (force, velocity)
    Vector { basis: Vec<Vector3<f64>> },
    /// Categorical dimension (color, shape)
    Categorical { categories: Vec<String> },
    /// Ordinal dimension with ordering
    Ordinal { ordering: Vec<String> },
    /// Derived from other dimensions
    Derived {
        formula: String,
        dependencies: Vec<String>,
    },
}

/// Emergent patterns from concept interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergentPattern {
    pub pattern_id: String,
    pub pattern_type: PatternType,
    pub involved_concepts: HashSet<String>,
    pub stability: f64,
    pub energy: f64,
    pub formation_timestamp: u64,
    pub triggering_events: Vec<String>,
}

/// Types of emergent patterns in conceptual space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    /// Stable cluster of related concepts
    ConceptCluster { 
        centroid: Point3<f64>, 
        radius: f64 
    },
    /// Oscillating relationship strength
    RelationshipOscillation { 
        period: f64, 
        amplitude: f64 
    },
    /// Hierarchical structure
    ConceptHierarchy {
        root_concept: String,
        levels: Vec<Vec<String>>,
    },
    /// Traveling activation wave
    ActivationWave {
        wavelength: f64,
        speed: f64,
        direction: Vector3<f64>,
    },
    /// Spiral arrangement of concepts
    ConceptSpiral { 
        center: Point3<f64>, 
        pitch: f64 
    },
}

/// Validation result for consistency checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub violations: Vec<String>,
    pub warnings: Vec<String>,
}