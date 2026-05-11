//! Shared ownership helpers for app state without user-facing Arc or locks.

use std::sync::Arc;

use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Cloneable read-only shared value.
#[derive(Debug)]
pub struct Shared<T> {
    inner: Arc<T>,
}

impl<T> Shared<T> {
    /// Wraps a value in shared ownership.
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(value),
        }
    }

    /// Returns a shared reference to the wrapped value.
    pub fn get(&self) -> &T {
        &self.inner
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// Cloneable async mutable shared value.
#[derive(Debug)]
pub struct SharedMut<T> {
    inner: Arc<RwLock<T>>,
}

impl<T> SharedMut<T> {
    /// Wraps a value in async mutable shared ownership.
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(value)),
        }
    }

    /// Acquires a read guard.
    pub async fn read(&self) -> RwLockReadGuard<'_, T> {
        self.inner.read().await
    }

    /// Acquires a write guard.
    pub async fn write(&self) -> RwLockWriteGuard<'_, T> {
        self.inner.write().await
    }

    /// Updates the wrapped value while holding the write lock.
    pub async fn update<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut value = self.inner.write().await;
        f(&mut value)
    }
}

impl<T> Clone for SharedMut<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_get_returns_value() {
        let value = Shared::new(String::from("demo"));
        let cloned = value.clone();
        assert_eq!(cloned.get(), "demo");
    }

    #[tokio::test]
    async fn shared_mut_update_changes_value() {
        let value = SharedMut::new(1_u64);
        value.update(|n| *n += 1).await;
        assert_eq!(*value.read().await, 2);
    }
}
