//! Traits for system metrics collection.

use crate::error::Result;
use crate::metrics::data::SystemSnapshot;
use futures_util::stream::BoxStream;

/// Trait for collecting system metrics.
///
/// This trait defines the interface for collecting various system metrics
/// from different sources. Implementations should provide safe, error-handled
/// access to system information.
pub trait MetricsProvider {
    /// Collect a single snapshot of system metrics.
    fn collect_snapshot(
        &mut self,
    ) -> impl std::future::Future<Output = Result<SystemSnapshot>> + Send;

    /// Start continuous collection of system metrics.
    ///
    /// Returns a stream of system snapshots that are collected at the
    /// specified interval.
    fn start_stream(
        &mut self,
        interval_ms: u64,
    ) -> impl std::future::Future<Output = Result<BoxStream<'static, SystemSnapshot>>> + Send;
}

/// High-level trait for system monitoring.
///
/// This trait provides a simplified interface for starting and managing
/// system monitoring operations.
pub trait SystemMonitor {
    /// Create a new system monitor instance.
    fn new() -> Result<Self>
    where
        Self: Sized;

    /// Start collecting system metrics with default interval.
    fn start_collecting(
        &mut self,
    ) -> impl std::future::Future<Output = Result<BoxStream<'static, SystemSnapshot>>> + Send;

    /// Start collecting system metrics with custom interval.
    fn start_collecting_with_interval(
        &mut self,
        interval_ms: u64,
    ) -> impl std::future::Future<Output = Result<BoxStream<'static, SystemSnapshot>>> + Send;

    /// Get a single system snapshot.
    fn get_snapshot(&mut self) -> impl std::future::Future<Output = Result<SystemSnapshot>> + Send;
}
