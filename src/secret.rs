//! Explicit secret wrapper with redacted formatting.

use std::fmt;

const REDACTED: &str = "[REDACTED]";

/// String wrapper that redacts accidental Display and Debug output.
#[derive(Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SecretString(String);

impl SecretString {
    /// Wraps a secret string.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Explicitly exposes the secret value.
    pub fn expose(&self) -> &str {
        &self.0
    }

    /// Returns true when the wrapped secret is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<String> for SecretString {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for SecretString {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(REDACTED)
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("SecretString")
            .field(&REDACTED)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_and_debug_are_redacted() {
        let secret = SecretString::new("token-123");

        assert_eq!(secret.to_string(), REDACTED);
        assert_eq!(format!("{secret:?}"), "SecretString(\"[REDACTED]\")");
    }

    #[test]
    fn expose_returns_inner_value_explicitly() {
        let secret = SecretString::from("token-123");

        assert_eq!(secret.expose(), "token-123");
        assert!(!secret.is_empty());
    }
}
