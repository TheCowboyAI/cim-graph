//! IPLD graph - deprecated, use ipld_projection instead
//!
//! This module is kept for backward compatibility but all functionality
//! has been moved to the event-driven ipld_projection module.

pub use super::ipld_projection::*;

// Re-export for compatibility
pub type IpldGraph = IpldProjection;