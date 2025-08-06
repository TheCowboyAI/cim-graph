//! Event stream optimization utilities

pub mod compression;
pub mod batching;
pub mod replay;

pub use compression::{CompressedEventStream, EventCompressor};
pub use batching::{EventBatch, EventBatcher};
pub use replay::{ReplayOptimizer, ReplayStrategy};