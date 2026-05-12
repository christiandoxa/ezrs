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
    /// Process or command should terminate with an explicit exit code.
    #[error("{message}")]
    Exit {
        /// Exit code intended for CLI callers.
        code: i32,
        /// Human-readable error message.
        message: String,
    },
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

    /// Creates an error with an explicit process exit code.
    pub fn exit(code: i32, message: impl Into<String>) -> Self {
        Self::Exit {
            code,
            message: message.into(),
        }
    }

    /// Wraps an error with app-level context.
    pub fn wrap(context: impl Into<String>, error: impl std::fmt::Display) -> Self {
        Self::Message(format!("{}: {error}", context.into()))
    }

    /// Returns the best CLI exit code for this error.
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::InvalidInput(_) => 2,
            Self::Cancelled(_) => 130,
            Self::Exit { code, .. } => *code,
            _ => 1,
        }
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
        assert_eq!(Error::exit(7, "bad").exit_code(), 7);
        assert_eq!(Error::invalid_input("bad").exit_code(), 2);
        assert_eq!(Error::cancelled("stop").exit_code(), 130);
        assert_eq!(
            Error::wrap("load config", Error::not_found("ezrs.toml")).to_string(),
            "load config: not found: ezrs.toml"
        );
    }
}
