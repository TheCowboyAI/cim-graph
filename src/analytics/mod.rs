//! Event analytics and metrics for monitoring graph operations

pub mod metrics;
pub mod dashboard;

pub use metrics::{EventMetrics, MetricsCollector};
pub use dashboard::{MetricsDashboard, DashboardConfig};