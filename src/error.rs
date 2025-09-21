//! Error handling for the Life of Pi diagnostics crate.

/// A specialized `Result` type for Life of Pi operations.
pub type Result<T> = std::result::Result<T, SystemError>;

/// The main error type for Life of Pi system operations.
#[derive(Debug, thiserror::Error)]
pub enum SystemError {
    /// I/O operation failed
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// System information parsing failed
    #[error("Failed to parse system information: {0}")]
    ParseError(String),

    /// Network operation failed
    #[error("Network error: {0}")]
    Network(String),

    /// Web server error
    #[error("Web server error: {0}")]
    WebServer(String),

    /// GPIO operation failed (only available with gpio feature)
    #[cfg(feature = "gpio")]
    #[error("GPIO error: {0}")]
    Gpio(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Generic system error
    #[error("System error: {0}")]
    System(String),
}

impl SystemError {
    /// Create a new parse error
    pub fn parse_error(msg: impl Into<String>) -> Self {
        Self::ParseError(msg.into())
    }

    /// Create a new network error
    pub fn network_error(msg: impl Into<String>) -> Self {
        Self::Network(msg.into())
    }

    /// Create a new web server error
    pub fn web_server_error(msg: impl Into<String>) -> Self {
        Self::WebServer(msg.into())
    }

    /// Create a new GPIO error
    #[cfg(feature = "gpio")]
    pub fn gpio_error(msg: impl Into<String>) -> Self {
        Self::Gpio(msg.into())
    }

    /// Create a new configuration error
    pub fn config_error(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create a new generic system error
    #[allow(clippy::self_named_constructors)]
    pub fn system_error(msg: impl Into<String>) -> Self {
        Self::System(msg.into())
    }
}
