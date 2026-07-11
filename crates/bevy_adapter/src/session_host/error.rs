use std::fmt;
use std::error::Error;

/// Errors that can occur during relay lifecycle management.
#[derive(Debug)]
pub enum RelayError {
    /// Relay failed to start (port conflict, system error, etc.).
    StartFailed(String),
    /// Relay failed to shut down cleanly.
    ShutdownFailed(String),
}

impl fmt::Display for RelayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RelayError::StartFailed(msg) => write!(f, "Relay start failed: {}", msg),
            RelayError::ShutdownFailed(msg) => write!(f, "Relay shutdown failed: {}", msg),
        }
    }
}

impl Error for RelayError {}
