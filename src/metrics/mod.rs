//! System metrics collection and data structures.
//!
//! This module provides the core functionality for collecting system metrics
//! from a Raspberry Pi, including CPU usage, temperature, memory statistics,
//! storage information, and network status.

pub mod collector;
pub mod data;
pub mod traits;

#[cfg(feature = "gpio")]
pub mod gpio;

// Re-export commonly used items
pub use collector::SystemCollector;
pub use data::SystemSnapshot;
pub use traits::{MetricsProvider, SystemMonitor};