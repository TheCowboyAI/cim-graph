//! NATS JetStream integration for event persistence

#[cfg(feature = "nats")]
pub mod jetstream;

#[cfg(feature = "nats")]
pub use jetstream::{JetStreamEventStore, JetStreamConfig, NatsError};