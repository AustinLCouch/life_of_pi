//! # Life of Pi - Raspberry Pi System Diagnostics
//!
//! A clean, minimalist Rust crate for real-time Raspberry Pi system monitoring
//! with a web interface. Designed for plug-and-play operation on Raspberry Pi 5
//! running RaspberryOS x64.
//!
//! ## Features
//!
//! - **Real-time system monitoring**: CPU usage, temperature, memory, storage, network
//! - **GPIO status monitoring**: Pin states and availability (feature-gated)
//! - **Web dashboard**: Live charts and metrics via WebSocket
//! - **Cross-compilation**: Build on macOS for Raspberry Pi deployment
//! - **Library + Binary**: Use as a crate or standalone application
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use life_of_pi::{SystemMonitor, start_web_server};
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let monitor = SystemMonitor::new()?;
//!     let mut stream = monitor.start_collecting().await?;
//!     
//!     // Start web server on port 8080
//!     start_web_server(8080, stream).await?;
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod metrics;
pub mod web;

// Re-export public API
pub use error::{Result, SystemError};
pub use metrics::{
    collector::SystemCollector,
    data::{SystemSnapshot, CpuInfo, MemoryInfo, StorageInfo, NetworkInfo},
    traits::{MetricsProvider, SystemMonitor},
};

#[cfg(feature = "gpio")]
pub use metrics::gpio::{GpioProvider, GpioStatus};

pub use web::{start_web_server, start_web_server_simple, WebConfig};

/// The default monitoring interval in milliseconds
pub const DEFAULT_INTERVAL_MS: u64 = 500;

/// The default web server port
pub const DEFAULT_WEB_PORT: u16 = 8080;