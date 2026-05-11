//! Default tracing setup and a tiny logger handle for ezrs contexts.

use std::{fmt::Display, sync::Once};

use ezrs_error::Result;
use tracing_subscriber::{EnvFilter, fmt};

static INIT: Once = Once::new();

/// Initializes tracing once.
pub fn init_default() -> Result<()> {
    INIT.call_once(|| {
        let filter = std::env::var("EZRS_LOG")
            .or_else(|_| std::env::var("RUST_LOG"))
            .unwrap_or_else(|_| String::from("info"));

        let subscriber = fmt()
            .with_env_filter(EnvFilter::new(filter))
            .with_target(false)
            .compact()
            .finish();

        let _ = tracing::subscriber::set_global_default(subscriber);
    });

    Ok(())
}

/// Small logger handle exposed through Context.
#[derive(Clone, Copy, Debug, Default)]
pub struct Logger;

impl Logger {
    /// Logs an info message.
    pub fn info(&self, message: impl Display) {
        tracing::info!("{message}");
    }

    /// Logs a warning message.
    pub fn warn(&self, message: impl Display) {
        tracing::warn!("{message}");
    }

    /// Logs an error message.
    pub fn error(&self, message: impl Display) {
        tracing::error!("{message}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_default_does_not_panic() {
        init_default().expect("logger should initialize");
        init_default().expect("logger should be idempotent");
    }
}
