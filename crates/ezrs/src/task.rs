//! Goroutine-like task spawning and cooperative cancellation.

use std::{
    future::Future,
    sync::{Arc, Mutex},
};

use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::{Error, Result};

type TaskJoin = JoinHandle<(String, Result<()>)>;

/// Cooperative cancellation handle.
#[derive(Clone, Debug)]
pub struct Cancellation {
    token: CancellationToken,
}

impl Cancellation {
    /// Creates a fresh cancellation token.
    pub fn new() -> Self {
        Self {
            token: CancellationToken::new(),
        }
    }

    /// Requests cancellation.
    pub fn cancel(&self) {
        self.token.cancel();
    }

    /// Returns true when cancellation was requested.
    pub fn is_cancelled(&self) -> bool {
        self.token.is_cancelled()
    }

    /// Waits until cancellation is requested.
    pub async fn cancelled(&self) {
        self.token.cancelled().await;
    }

    /// Returns a cancellation error when cancellation was requested.
    pub fn check_cancelled(&self) -> Result<()> {
        if self.is_cancelled() {
            Err(Error::cancelled("operation cancelled"))
        } else {
            Ok(())
        }
    }
}

impl Default for Cancellation {
    fn default() -> Self {
        Self::new()
    }
}

/// Installs Ctrl+C handling that cancels the token.
pub fn install_ctrl_c(cancellation: Cancellation) {
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            cancellation.cancel();
        }
    });
}

/// Tracks spawned tasks and joins them later.
#[derive(Clone, Debug, Default)]
pub struct TaskManager {
    tasks: Arc<Mutex<Vec<TaskJoin>>>,
}

impl TaskManager {
    /// Creates an empty task manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Spawns a named task.
    pub fn spawn<F>(&self, name: impl Into<String>, future: F)
    where
        F: Future<Output = Result<()>> + Send + 'static,
    {
        let name = name.into();
        let task_name = name.clone();
        let handle = tokio::spawn(async move {
            tracing::debug!(task = %task_name, "task started");
            let result = future.await;
            if let Err(error) = &result {
                tracing::warn!(task = %task_name, error = %error, "task failed");
            }
            (task_name, result)
        });

        self.tasks.lock().expect("task list poisoned").push(handle);
    }

    /// Joins all currently tracked tasks and returns the first error.
    pub async fn join_all(&self) -> Result<()> {
        let tasks = {
            let mut guard = self.tasks.lock().expect("task list poisoned");
            std::mem::take(&mut *guard)
        };

        for task in tasks {
            match task.await {
                Ok((_, Ok(()))) => {}
                Ok((_, Err(error))) => return Err(error),
                Err(error) if error.is_panic() => {
                    return Err(Error::msg("task panicked"));
                }
                Err(error) if error.is_cancelled() => {
                    return Err(Error::cancelled("task was cancelled"));
                }
                Err(error) => return Err(Error::msg(format!("task join failed: {error}"))),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cancellation_check_reports_cancelled() {
        let cancellation = Cancellation::new();
        assert!(cancellation.check_cancelled().is_ok());
        cancellation.cancel();
        assert!(cancellation.check_cancelled().is_err());
    }

    #[tokio::test]
    async fn task_manager_joins_spawned_tasks() {
        let tasks = TaskManager::new();
        tasks.spawn("worker", async { Ok(()) });
        tasks.join_all().await.expect("join");
    }
}
