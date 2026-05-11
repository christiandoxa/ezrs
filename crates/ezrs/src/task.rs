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

/// WaitGroup-like collection of spawned tasks with optional fail-fast cancellation.
#[derive(Clone, Debug)]
pub struct TaskGroup {
    tasks: Arc<Mutex<Vec<TaskJoin>>>,
    cancellation: Cancellation,
    cancel_on_error: bool,
}

#[allow(dead_code)]
impl TaskGroup {
    /// Creates an empty task group.
    pub fn new() -> Self {
        Self {
            tasks: Arc::default(),
            cancellation: Cancellation::new(),
            cancel_on_error: false,
        }
    }

    /// Creates an empty task group using an existing cancellation handle.
    pub fn with_cancellation(cancellation: Cancellation) -> Self {
        Self {
            tasks: Arc::default(),
            cancellation,
            cancel_on_error: false,
        }
    }

    /// Enables or disables cancellation when any task returns an error.
    pub fn cancel_on_error(mut self, enabled: bool) -> Self {
        self.cancel_on_error = enabled;
        self
    }

    /// Returns the group's cooperative cancellation handle.
    pub fn cancellation(&self) -> Cancellation {
        self.cancellation.clone()
    }

    /// Spawns a task and derives a diagnostic name from the future type.
    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = Result<()>> + Send + 'static,
    {
        self.spawn_named(task_name::<F>(), future);
    }

    /// Spawns a task with an explicit diagnostic name.
    pub fn spawn_named<F>(&self, name: impl Into<String>, future: F)
    where
        F: Future<Output = Result<()>> + Send + 'static,
    {
        let name = name.into();
        let task_name = name.clone();
        let cancellation = self.cancellation.clone();
        let cancel_on_error = self.cancel_on_error;
        let handle = tokio::spawn(async move {
            tracing::debug!(task = %task_name, "task started");
            let result = future.await;
            if let Err(error) = &result {
                tracing::warn!(task = %task_name, error = %error, "task failed");
                if cancel_on_error {
                    cancellation.cancel();
                }
            }
            (task_name, result)
        });

        self.tasks.lock().expect("task list poisoned").push(handle);
    }

    /// Joins all currently tracked tasks and returns the first error.
    pub async fn join(&self) -> Result<()> {
        join_tasks(&self.tasks).await
    }
}

impl Default for TaskGroup {
    fn default() -> Self {
        Self::new()
    }
}

/// Tracks spawned tasks and joins them later.
#[derive(Clone, Debug, Default)]
pub struct TaskManager {
    group: TaskGroup,
}

impl TaskManager {
    /// Creates an empty task manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Spawns a task and derives a diagnostic name from the future type.
    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = Result<()>> + Send + 'static,
    {
        self.spawn_named(task_name::<F>(), future);
    }

    /// Spawns a task with an explicit diagnostic name.
    pub fn spawn_named<F>(&self, name: impl Into<String>, future: F)
    where
        F: Future<Output = Result<()>> + Send + 'static,
    {
        self.group.spawn_named(name, future);
    }

    /// Joins all currently tracked tasks and returns the first error.
    pub async fn join_all(&self) -> Result<()> {
        self.group.join().await
    }
}

async fn join_tasks(tasks: &Mutex<Vec<TaskJoin>>) -> Result<()> {
    let tasks = {
        let mut guard = tasks.lock().expect("task list poisoned");
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

fn task_name<T>() -> String {
    let raw = std::any::type_name::<T>();
    if raw
        .rsplit("::")
        .next()
        .is_some_and(|part| part.contains("{{closure}}"))
    {
        String::from("task")
    } else {
        crate::command::command_name_from_type_name(raw)
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
        tasks.spawn(async { Ok(()) });
        tasks.join_all().await.expect("join");
    }

    #[tokio::test]
    async fn task_group_returns_task_error() {
        let group = TaskGroup::new();
        group.spawn_named("bad", async { Err(Error::msg("failed")) });

        let error = group.join().await.expect_err("error");
        assert_eq!(error.to_string(), "failed");
    }

    #[tokio::test]
    async fn task_group_cancel_on_error_cancels_token() {
        let group = TaskGroup::new().cancel_on_error(true);
        let cancellation = group.cancellation();

        group.spawn_named("bad", async { Err(Error::msg("failed")) });
        assert!(group.join().await.is_err());
        assert!(cancellation.is_cancelled());
    }
}
