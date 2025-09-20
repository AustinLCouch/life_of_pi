//! Web server configuration.

use serde::{Deserialize, Serialize};

/// Configuration for the web server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    /// Host to bind the server to
    pub host: String,
    /// Port to bind the server to
    pub port: u16,
    /// Whether to enable CORS
    pub enable_cors: bool,
    /// Path to serve static files from
    pub static_path: Option<String>,
    /// Maximum number of WebSocket connections
    pub max_websocket_connections: usize,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: crate::DEFAULT_WEB_PORT,
            enable_cors: true,
            static_path: Some("static".to_string()),
            max_websocket_connections: 100,
        }
    }
}

impl WebConfig {
    /// Create a new web configuration with custom host and port.
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            ..Default::default()
        }
    }
    
    /// Set the host for the web server.
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }
    
    /// Set the port for the web server.
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
    
    /// Enable or disable CORS.
    pub fn with_cors(mut self, enable_cors: bool) -> Self {
        self.enable_cors = enable_cors;
        self
    }
    
    /// Set the static files path.
    pub fn with_static_path(mut self, path: Option<String>) -> Self {
        self.static_path = path;
        self
    }
    
    /// Set the maximum number of WebSocket connections.
    pub fn with_max_websocket_connections(mut self, max: usize) -> Self {
        self.max_websocket_connections = max;
        self
    }
    
    /// Get the full bind address.
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}