use std::fmt;

/// Error types for Amiya
#[derive(Debug)]
pub enum AmiyaError {
    /// Configuration error
    Config(String),

    /// IPC communication error
    Ipc(String),

    /// Backend unavailable or failed
    Backend(String),

    /// UI initialization error
    Ui(String),

    /// Generic I/O error
    Io(std::io::Error),

    /// Other errors
    Other(String),
}

impl fmt::Display for AmiyaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AmiyaError::Config(msg) => write!(f, "Configuration error: {}", msg),
            AmiyaError::Ipc(msg) => write!(f, "IPC error: {}", msg),
            AmiyaError::Backend(msg) => write!(f, "Backend error: {}", msg),
            AmiyaError::Ui(msg) => write!(f, "UI error: {}", msg),
            AmiyaError::Io(err) => write!(f, "I/O error: {}", err),
            AmiyaError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for AmiyaError {}

impl From<std::io::Error> for AmiyaError {
    fn from(err: std::io::Error) -> Self {
        AmiyaError::Io(err)
    }
}

impl From<anyhow::Error> for AmiyaError {
    fn from(err: anyhow::Error) -> Self {
        AmiyaError::Other(err.to_string())
    }
}

/// Backend availability status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendStatus {
    Available,
    Unavailable,
    Error,
}

/// Result type for Amiya operations
pub type Result<T> = std::result::Result<T, AmiyaError>;
