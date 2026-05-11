//! Error helpers for Go-style error-returning Rust application code.

/// Result alias used by all ezrs public APIs.
pub type Result<T> = std::result::Result<T, Error>;

/// Small framework error type for application-level failures.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Generic message error.
    #[error("{0}")]
    Message(String),
    /// Invalid user input or CLI input.
    #[error("invalid input: {0}")]
    InvalidInput(String),
    /// Missing file, config, state, command, or value.
    #[error("not found: {0}")]
    NotFound(String),
    /// Operation timed out.
    #[error("timeout: {0}")]
    Timeout(String),
    /// Cooperative cancellation was observed.
    #[error("cancelled: {0}")]
    Cancelled(String),
    /// IO error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Environment variable error.
    #[error(transparent)]
    Env(#[from] std::env::VarError),
    /// JSON error.
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    /// TOML decode error.
    #[error(transparent)]
    TomlDe(#[from] toml::de::Error),
    /// Dynamic error from anyhow-based application code.
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl Error {
    /// Creates a generic message error.
    pub fn msg(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }

    /// Creates an invalid-input error.
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }

    /// Creates a not-found error.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    /// Creates a timeout error.
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::Timeout(message.into())
    }

    /// Creates a cancellation error.
    pub fn cancelled(message: impl Into<String>) -> Self {
        Self::Cancelled(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helpers_render_clear_messages() {
        assert_eq!(
            Error::invalid_input("bad").to_string(),
            "invalid input: bad"
        );
        assert_eq!(Error::not_found("file").to_string(), "not found: file");
        assert_eq!(Error::timeout("slow").to_string(), "timeout: slow");
        assert_eq!(Error::cancelled("stop").to_string(), "cancelled: stop");
    }
}
