use std::{
    fmt::Display,
    future::Future,
    io::Write,
    sync::{Arc, Mutex},
    time::Duration,
};

use ezrs_error::{Error, Result};
use ezrs_fs::Fs;
use ezrs_log::Logger;
use ezrs_task::{Cancellation, TaskManager};
use tokio::io::AsyncReadExt;

use crate::{Args, state::TypeStore};

#[derive(Clone)]
enum Output {
    Process,
    Memory {
        stdout: Arc<Mutex<String>>,
        stderr: Arc<Mutex<String>>,
    },
}

impl Output {
    fn process() -> Self {
        Self::Process
    }

    fn memory(stdout: Arc<Mutex<String>>, stderr: Arc<Mutex<String>>) -> Self {
        Self::Memory { stdout, stderr }
    }

    fn stdout(&self, message: String) {
        match self {
            Self::Process => {
                let mut out = std::io::stdout().lock();
                let _ = writeln!(out, "{message}");
            }
            Self::Memory { stdout, .. } => {
                let mut out = stdout.lock().expect("stdout buffer poisoned");
                out.push_str(&message);
                out.push('\n');
            }
        }
    }

    fn stderr(&self, message: String) {
        match self {
            Self::Process => {
                let mut err = std::io::stderr().lock();
                let _ = writeln!(err, "{message}");
            }
            Self::Memory { stderr, .. } => {
                let mut err = stderr.lock().expect("stderr buffer poisoned");
                err.push_str(&message);
                err.push('\n');
            }
        }
    }
}

#[derive(Clone)]
struct ContextInner {
    args: Args,
    state: TypeStore,
    config: TypeStore,
    logger: Logger,
    fs: Fs,
    tasks: TaskManager,
    cancellation: Cancellation,
    output: Output,
}

/// App capability handle passed to every command.
#[derive(Clone)]
pub struct Context {
    inner: Arc<ContextInner>,
}

impl Context {
    pub(crate) fn process(args: Args, state: TypeStore, config: TypeStore) -> Self {
        Self::new(args, state, config, Output::process())
    }

    pub(crate) fn memory(
        args: Args,
        state: TypeStore,
        config: TypeStore,
        stdout: Arc<Mutex<String>>,
        stderr: Arc<Mutex<String>>,
    ) -> Self {
        Self::new(args, state, config, Output::memory(stdout, stderr))
    }

    fn new(args: Args, state: TypeStore, config: TypeStore, output: Output) -> Self {
        Self {
            inner: Arc::new(ContextInner {
                args,
                state,
                config,
                logger: Logger,
                fs: Fs,
                tasks: TaskManager::new(),
                cancellation: Cancellation::new(),
                output,
            }),
        }
    }

    pub(crate) fn install_ctrl_c(&self) {
        ezrs_task::install_ctrl_c(self.inner.cancellation.clone());
    }

    /// Reads a required dynamic argument by name or positional index.
    pub fn arg(&self, key: &str) -> Result<String> {
        self.inner
            .args
            .get(key)
            .map(ToOwned::to_owned)
            .ok_or_else(|| Error::not_found(format!("argument '{key}'")))
    }

    /// Reads an optional dynamic argument by name or positional index.
    pub fn arg_or(&self, key: &str, default: impl Into<String>) -> String {
        self.inner
            .args
            .get(key)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| default.into())
    }

    /// Returns true when a dynamic flag was present.
    pub fn flag(&self, key: &str) -> bool {
        self.inner.args.flag(key)
    }

    /// Reads an environment variable.
    pub fn env(&self, key: &str) -> Result<String> {
        Ok(std::env::var(key)?)
    }

    /// Retrieves cloned app state by type.
    pub fn state<T>(&self) -> Result<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.inner.state.get("state")
    }

    /// Retrieves cloned typed config by type.
    pub fn config<T>(&self) -> Result<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.inner.config.get("config")
    }

    /// Returns the default logger handle.
    pub fn log(&self) -> Logger {
        self.inner.logger
    }

    /// Writes one line to stdout or the test output sink.
    pub fn println(&self, message: impl Display) {
        self.inner.output.stdout(message.to_string());
    }

    /// Writes one line to stderr or the test output sink.
    pub fn eprintln(&self, message: impl Display) {
        self.inner.output.stderr(message.to_string());
    }

    /// Reads all stdin as UTF-8.
    pub async fn read_stdin(&self) -> Result<String> {
        let mut input = String::new();
        tokio::io::stdin().read_to_string(&mut input).await?;
        Ok(input)
    }

    /// Waits for cooperative cancellation.
    pub async fn cancelled(&self) {
        self.inner.cancellation.cancelled().await;
    }

    /// Returns true when cooperative cancellation was requested.
    pub fn is_cancelled(&self) -> bool {
        self.inner.cancellation.is_cancelled()
    }

    /// Returns a cancellation error when cancellation was requested.
    pub fn check_cancelled(&self) -> Result<()> {
        self.inner.cancellation.check_cancelled()
    }

    /// Requests cancellation. Useful in tests and orchestrators.
    pub fn cancel(&self) {
        self.inner.cancellation.cancel();
    }

    /// Returns file helper handle.
    pub fn fs(&self) -> Fs {
        self.inner.fs
    }

    /// Spawns a named background task.
    pub fn spawn<F>(&self, name: impl Into<String>, future: F)
    where
        F: Future<Output = Result<()>> + Send + 'static,
    {
        self.inner.tasks.spawn(name, future);
    }

    /// Joins all tasks spawned through this Context.
    pub async fn join_all(&self) -> Result<()> {
        self.inner.tasks.join_all().await
    }

    /// Sleeps for whole seconds.
    pub async fn sleep_secs(&self, seconds: u64) {
        tokio::time::sleep(Duration::from_secs(seconds)).await;
    }
}
