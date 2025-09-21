//! Web server and API endpoints for the Life of Pi diagnostics dashboard.
//!
//! This module provides a complete web interface for viewing real-time system metrics
//! including REST API endpoints and WebSocket streaming for live data updates.

pub mod config;
pub mod handlers;
pub mod router;
pub mod websocket;

// Re-export commonly used items
pub use config::WebConfig;
pub use router::create_app;

use crate::error::{Result, SystemError};
use crate::metrics::SystemSnapshot;
// Note: axum 0.7+ doesn't have a Server struct, we'll use tokio directly
use futures_util::stream::BoxStream;
use std::net::SocketAddr;
use tokio_stream::StreamExt;
use tracing::{error, info};

/// Start the web server with the provided configuration and metrics stream.
pub async fn start_web_server(
    config: WebConfig,
    mut metrics_stream: BoxStream<'static, SystemSnapshot>,
) -> Result<()> {
    // Create the axum application
    let app = create_app(config.clone()).await?;

    // Parse the bind address
    let addr = format!("{}:{}", config.host, config.port)
        .parse::<SocketAddr>()
        .map_err(|e| SystemError::config_error(format!("Invalid bind address: {}", e)))?;

    info!("Starting Life of Pi web server on http://{}", addr);
    info!("Dashboard available at http://{}/", addr);
    info!("API endpoint: http://{}/api/snapshot", addr);
    info!("WebSocket endpoint: ws://{}/ws", addr);

    // Start the server using tokio's TcpListener
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| SystemError::web_server_error(format!("Failed to bind to address: {}", e)))?;

    // Start the metrics collection task
    let _metrics_task = tokio::spawn(async move {
        while let Some(snapshot) = metrics_stream.next().await {
            // Broadcast the snapshot to all connected WebSocket clients
            // This will be handled by the WebSocket handler
            if let Err(e) = websocket::broadcast_snapshot(snapshot).await {
                error!("Failed to broadcast snapshot: {}", e);
            }
        }
    });

    // Run the server
    axum::serve(listener, app)
        .await
        .map_err(|e| SystemError::web_server_error(format!("Server error: {}", e)))?;

    Ok(())
}

/// Start a web server with simple port-only configuration.
///
/// This is a convenience function for the common use case of just specifying a port.
/// It creates a SystemCollector, starts metrics collection, and serves the web dashboard.
pub async fn start_web_server_simple(
    port: u16,
    stream: BoxStream<'static, SystemSnapshot>,
) -> Result<()> {
    let config = WebConfig::default().with_port(port);
    start_web_server(config, stream).await
}
