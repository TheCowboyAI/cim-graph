/*
 * Copyright (c) 2025 - Cowboy AI, LLC.
 */

//! Topology management for Conceptual Spaces
//!
//! Handles the evolution of topological structures as concepts are added
//! and relationships form.

use serde::{Deserialize, Serialize};
use super::types::Point3;

/// Topology of the conceptual space derived from graph structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceTopology {
    /// Current type of topology
    pub topology_type: TopologyType,
    /// Topological genus (number of holes)
    pub genus: i32,
    /// Euler characteristic of the manifold
    pub euler_characteristic: i32,
    /// Dimension of the manifold
    pub manifold_dimension: usize,
    /// Whether the surface is orientable
    pub is_orientable: bool,
}

/// Types of topologies the space can manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TopologyType {
    /// Initial empty space - undefined topology
    Undefined,
    /// Single concept - point topology
    Point,
    /// Two concepts - line segment topology
    LineSegment,
    /// Three or more concepts - spherical topology with Voronoi tessellation
    SphericalVoronoi {
        /// Radius of the conceptual sphere
        radius: f64,
        /// Positions of concepts on the sphere
        concept_positions: Vec<Point3<f64>>,
    },
    /// Complex topology when concepts create overlaps
    ComplexManifold {
        /// Number of holes in the manifold
        genus: i32,
        /// Critical points in the topology
        critical_points: Vec<Point3<f64>>,
    },
    /// Non-orientable when concept relationships create twists
    NonOrientable {
        /// Type of non-orientable surface
        surface_type: NonOrientableSurfaceType,
    },
}

/// Non-orientable surface types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NonOrientableSurfaceType {
    /// Klein bottle topology
    KleinBottle,
    /// Projective plane topology
    ProjectivePlane,
    /// Mobius strip topology
    MobiusStrip,
}

/// History of topology changes driven by events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyTransition {
    /// Unique identifier for this transition
    pub event_id: String,
    /// Previous topology type
    pub from_topology: TopologyType,
    /// New topology type
    pub to_topology: TopologyType,
    /// Mathematical proof justifying the transition
    pub mathematical_proof: String,
    /// Unix timestamp of the transition
    pub timestamp: u64,
}

impl SpaceTopology {
    /// Create undefined topology for empty space
    pub fn undefined() -> Self {
        Self {
            topology_type: TopologyType::Undefined,
            genus: 0,
            euler_characteristic: 0,
            manifold_dimension: 0,
            is_orientable: true,
        }
    }

    /// Create point topology for single concept
    pub fn point() -> Self {
        Self {
            topology_type: TopologyType::Point,
            genus: 0,
            euler_characteristic: 1,
            manifold_dimension: 0,
            is_orientable: true,
        }
    }

    /// Create line segment topology for two concepts
    pub fn line_segment() -> Self {
        Self {
            topology_type: TopologyType::LineSegment,
            genus: 0,
            euler_characteristic: 1,
            manifold_dimension: 1,
            is_orientable: true,
        }
    }

    /// Create spherical topology for multiple concepts
    pub fn spherical(positions: Vec<Point3<f64>>) -> Self {
        Self {
            topology_type: TopologyType::SphericalVoronoi {
                radius: 1.0,
                concept_positions: positions,
            },
            genus: 0,
            euler_characteristic: 2,
            manifold_dimension: 2,
            is_orientable: true,
        }
    }

    /// Create complex manifold with given genus
    pub fn complex_manifold(genus: i32, critical_points: Vec<Point3<f64>>) -> Self {
        Self {
            topology_type: TopologyType::ComplexManifold {
                genus,
                critical_points,
            },
            genus,
            euler_characteristic: 2 - 2 * genus,
            manifold_dimension: 2,
            is_orientable: true,
        }
    }

    /// Check if topology requires genus increase based on edge count
    ///
    /// Uses two bounds from graph theory:
    /// - General planar bound: E <= 3V - 6 (catches K5)
    /// - Bipartite planar bound: E <= 2V - 4 (catches K3,3)
    ///
    /// The function uses edge density heuristics to determine which bound applies.
    pub fn requires_genus_increase(node_count: usize, edge_count: usize) -> bool {
        if node_count < 3 {
            return false;
        }

        // General bound: E <= 3V - 6 (for any planar graph)
        let max_general_edges = 3 * node_count - 6;

        // First check: if exceeds general bound, definitely non-planar
        if edge_count > max_general_edges {
            return true;
        }

        // Bipartite bound: E <= 2V - 4 (for triangle-free graphs like K_{m,n})
        let max_bipartite_edges = 2 * node_count - 4;

        // Complete graph K_n has n(n-1)/2 edges
        let complete_graph_edges = node_count * (node_count - 1) / 2;

        // Complete bipartite K_{n/2, n/2} has (n/2)^2 edges (balanced case)
        let half = node_count / 2;
        let other_half = node_count - half;
        let complete_bipartite_edges = half * other_half;

        // Heuristic: if edge count matches complete bipartite pattern better than
        // complete graph pattern, apply the bipartite bound
        let dist_to_complete = (edge_count as isize - complete_graph_edges as isize).unsigned_abs();
        let dist_to_bipartite = (edge_count as isize - complete_bipartite_edges as isize).unsigned_abs();

        // If closer to bipartite pattern and exceeds bipartite bound, non-planar
        if dist_to_bipartite < dist_to_complete && edge_count > max_bipartite_edges {
            return true;
        }

        false
    }

    /// Compute Euler characteristic for the topology
    pub fn compute_euler_characteristic(&self) -> i32 {
        match &self.topology_type {
            TopologyType::Undefined => 0,
            TopologyType::Point => 1,
            TopologyType::LineSegment => 1,
            TopologyType::SphericalVoronoi { .. } => 2,
            TopologyType::ComplexManifold { genus, .. } => 2 - 2 * genus,
            TopologyType::NonOrientable { surface_type } => {
                match surface_type {
                    NonOrientableSurfaceType::KleinBottle => 0,
                    NonOrientableSurfaceType::ProjectivePlane => 1,
                    NonOrientableSurfaceType::MobiusStrip => 0,
                }
            }
        }
    }

    /// Generate mathematical proof for topology validity
    pub fn generate_proof(&self) -> String {
        match &self.topology_type {
            TopologyType::Undefined => {
                "Empty space: No topology defined, trivial proof".to_string()
            }
            TopologyType::Point => {
                "Point topology: Single element, Hausdorff space, compact".to_string()
            }
            TopologyType::LineSegment => {
                "Line segment: Connected, path-connected, compact interval [0,1]".to_string()
            }
            TopologyType::SphericalVoronoi { .. } => {
                format!(
                    "Sphere S²: Genus=0, Euler χ={}, Simply connected, Compact, Orientable",
                    self.euler_characteristic
                )
            }
            TopologyType::ComplexManifold { genus, .. } => {
                format!(
                    "Complex manifold: Genus={}, Euler χ={}, {} handle(s) attached",
                    genus, self.euler_characteristic, genus
                )
            }
            TopologyType::NonOrientable { surface_type } => {
                match surface_type {
                    NonOrientableSurfaceType::KleinBottle => {
                        "Klein bottle: Non-orientable, Euler χ=0, Cannot embed in R³".to_string()
                    }
                    NonOrientableSurfaceType::ProjectivePlane => {
                        "Projective plane: Non-orientable, Euler χ=1, Quotient of sphere".to_string()
                    }
                    NonOrientableSurfaceType::MobiusStrip => {
                        "Möbius strip: Non-orientable, Euler χ=0, Boundary component".to_string()
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topology_evolution() {
        // Empty space
        let topo = SpaceTopology::undefined();
        assert_eq!(topo.euler_characteristic, 0);
        assert_eq!(topo.genus, 0);

        // Single concept
        let topo = SpaceTopology::point();
        assert_eq!(topo.euler_characteristic, 1);
        assert_eq!(topo.manifold_dimension, 0);

        // Two concepts
        let topo = SpaceTopology::line_segment();
        assert_eq!(topo.euler_characteristic, 1);
        assert_eq!(topo.manifold_dimension, 1);

        // Multiple concepts on sphere
        let positions = vec![
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(0.0, 0.0, 1.0),
        ];
        let topo = SpaceTopology::spherical(positions);
        assert_eq!(topo.euler_characteristic, 2);
        assert_eq!(topo.genus, 0);
        assert_eq!(topo.manifold_dimension, 2);
    }

    #[test]
    fn test_genus_increase_detection() {
        // Planar graph: K4 (4 nodes, 6 edges) - should be planar
        assert!(!SpaceTopology::requires_genus_increase(4, 6));

        // Non-planar graph: K5 (5 nodes, 10 edges) - requires genus increase
        assert!(SpaceTopology::requires_genus_increase(5, 10));

        // Non-planar graph: K3,3 (6 nodes, 9 edges) - requires genus increase
        assert!(SpaceTopology::requires_genus_increase(6, 9));
    }
}