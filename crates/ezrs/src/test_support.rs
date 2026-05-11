//! Test support building blocks for application and service tests.
//!
//! This module avoids mutating the process environment. Use [`EnvMap`] or
//! [`TestEnv`] to pass owned environment data into code under test.

use std::{
    collections::{HashMap, VecDeque},
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use crate::{Error, Result};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

/// Owned environment map for tests.
///
/// This is the safe Rust 2024 pattern for env-dependent code: inject owned
/// values instead of calling `std::env::set_var` or `std::env::remove_var`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EnvMap {
    values: HashMap<OsString, OsString>,
}

impl EnvMap {
    /// Creates an empty environment map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Captures the current process environment into owned test data.
    pub fn capture_current() -> Self {
        Self {
            values: std::env::vars_os().collect(),
        }
    }

    /// Inserts or replaces one environment value.
    pub fn set(mut self, key: impl Into<OsString>, value: impl Into<OsString>) -> Self {
        self.values.insert(key.into(), value.into());
        self
    }

    /// Removes one environment value from this owned map.
    pub fn without(mut self, key: impl Into<OsString>) -> Self {
        self.values.remove(&key.into());
        self
    }

    /// Reads one environment value.
    pub fn get(&self, key: impl AsRef<std::ffi::OsStr>) -> Option<&OsString> {
        self.values.get(key.as_ref())
    }

    /// Reads one environment value as lossy UTF-8.
    pub fn get_string(&self, key: impl AsRef<std::ffi::OsStr>) -> Option<String> {
        self.get(key)
            .map(|value| value.to_string_lossy().into_owned())
    }

    /// Iterates key/value pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&OsString, &OsString)> {
        self.values.iter()
    }
}

/// Named test environment fixture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestEnv {
    name: String,
    vars: EnvMap,
}

impl TestEnv {
    /// Creates a named empty test environment.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            vars: EnvMap::new(),
        }
    }

    /// Adds one environment value.
    pub fn set(mut self, key: impl Into<OsString>, value: impl Into<OsString>) -> Self {
        self.vars = self.vars.set(key, value);
        self
    }

    /// Returns the fixture name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns owned environment values.
    pub fn vars(&self) -> &EnvMap {
        &self.vars
    }
}

/// RAII temporary workspace backed by `std::env::temp_dir`.
///
/// The directory is removed on drop. Cleanup errors are intentionally ignored
/// on drop; call [`TempWorkspace::close`] when a test must assert cleanup.
#[derive(Debug)]
pub struct TempWorkspace {
    root: PathBuf,
}

impl TempWorkspace {
    /// Creates a unique workspace directory.
    pub fn new(prefix: impl AsRef<str>) -> Result<Self> {
        let mut last_error = None;
        for _ in 0..100 {
            let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "{}-{}-{id}",
                sanitize_path_part(prefix.as_ref()),
                std::process::id()
            ));

            match fs::create_dir(&root) {
                Ok(()) => return Ok(Self { root }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    last_error = Some(error);
                }
                Err(error) => return Err(error.into()),
            }
        }

        Err(last_error
            .unwrap_or_else(|| std::io::Error::other("could not create temp workspace"))
            .into())
    }

    /// Returns the workspace root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Joins a path below the workspace root.
    pub fn path(&self, path: impl AsRef<Path>) -> PathBuf {
        self.root.join(path)
    }

    /// Creates a directory below the workspace root.
    pub fn create_dir_all(&self, path: impl AsRef<Path>) -> Result<PathBuf> {
        let path = self.path(path);
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    /// Writes a UTF-8 text file below the workspace root.
    pub fn write(&self, path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Result<PathBuf> {
        let path = self.path(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, contents)?;
        Ok(path)
    }

    /// Reads a UTF-8 text file below the workspace root.
    pub fn read_to_string(&self, path: impl AsRef<Path>) -> Result<String> {
        Ok(fs::read_to_string(self.path(path))?)
    }

    /// Removes the workspace and consumes this handle.
    pub fn close(self) -> Result<()> {
        let root = self.root.clone();
        std::mem::forget(self);
        fs::remove_dir_all(root)?;
        Ok(())
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Test fixture rooted at a [`TempWorkspace`].
#[derive(Debug)]
pub struct Fixture {
    workspace: TempWorkspace,
}

impl Fixture {
    /// Creates a fixture with a unique workspace.
    pub fn new(name: impl AsRef<str>) -> Result<Self> {
        Ok(Self {
            workspace: TempWorkspace::new(name)?,
        })
    }

    /// Returns the underlying workspace.
    pub fn workspace(&self) -> &TempWorkspace {
        &self.workspace
    }

    /// Writes many files below the fixture root.
    pub fn files<I, P, C>(&self, files: I) -> Result<&Self>
    where
        I: IntoIterator<Item = (P, C)>,
        P: AsRef<Path>,
        C: AsRef<[u8]>,
    {
        for (path, contents) in files {
            self.workspace.write(path, contents)?;
        }
        Ok(self)
    }
}

/// Asserts exact text against a golden file.
///
/// Set `EZRS_ACCEPT_GOLDEN=1` in the shell to update the file. This helper
/// reads the environment only; it never mutates global environment state.
pub fn assert_golden(path: impl AsRef<Path>, actual: impl AsRef<str>) {
    let path = path.as_ref();
    let actual = actual.as_ref();
    let accept = std::env::var_os("EZRS_ACCEPT_GOLDEN").is_some_and(|value| value == "1");

    if accept {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create golden parent directory");
        }
        fs::write(path, actual).expect("write golden file");
        return;
    }

    let expected = fs::read_to_string(path).unwrap_or_else(|error| {
        panic!("read golden file {}: {error}", path.display());
    });
    assert_eq!(actual, expected, "golden mismatch: {}", path.display());
}

/// Captured command output for fake process runners.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeCommandOutput {
    /// Platform-style exit code.
    pub code: i32,
    /// Captured stdout bytes.
    pub stdout: Vec<u8>,
    /// Captured stderr bytes.
    pub stderr: Vec<u8>,
}

impl FakeCommandOutput {
    /// Creates a successful output with stdout text.
    pub fn success(stdout: impl Into<Vec<u8>>) -> Self {
        Self {
            code: 0,
            stdout: stdout.into(),
            stderr: Vec::new(),
        }
    }

    /// Creates a failing output with stderr text.
    pub fn failure(code: i32, stderr: impl Into<Vec<u8>>) -> Self {
        Self {
            code,
            stdout: Vec::new(),
            stderr: stderr.into(),
        }
    }

    /// Returns true when `code == 0`.
    pub fn is_success(&self) -> bool {
        self.code == 0
    }

    /// Captured stdout as UTF-8 lossily decoded text.
    pub fn stdout_lossy(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }

    /// Captured stderr as UTF-8 lossily decoded text.
    pub fn stderr_lossy(&self) -> String {
        String::from_utf8_lossy(&self.stderr).into_owned()
    }
}

/// Captured fake process request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeCommandRequest {
    /// Program name or path.
    pub program: String,
    /// Process arguments.
    pub args: Vec<String>,
    /// Owned environment overlay.
    pub env: EnvMap,
    /// Working directory.
    pub current_dir: Option<PathBuf>,
    /// Standard input bytes.
    pub stdin: Vec<u8>,
}

impl FakeCommandRequest {
    /// Creates a fake request.
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            env: EnvMap::new(),
            current_dir: None,
            stdin: Vec::new(),
        }
    }

    /// Adds one argument.
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Adds many arguments.
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    /// Adds one environment overlay value.
    pub fn env(mut self, key: impl Into<OsString>, value: impl Into<OsString>) -> Self {
        self.env = self.env.set(key, value);
        self
    }

    /// Sets working directory.
    pub fn current_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.current_dir = Some(path.into());
        self
    }

    /// Sets stdin bytes.
    pub fn stdin(mut self, input: impl Into<Vec<u8>>) -> Self {
        self.stdin = input.into();
        self
    }
}

#[derive(Debug, Default)]
struct FakeProcessRunnerState {
    requests: Vec<FakeCommandRequest>,
    outputs: VecDeque<FakeCommandOutput>,
}

/// Simple fake command runner for dependency-injected service tests.
#[derive(Debug, Clone, Default)]
pub struct FakeProcessRunner {
    state: Arc<Mutex<FakeProcessRunnerState>>,
}

impl FakeProcessRunner {
    /// Creates an empty fake process runner.
    pub fn new() -> Self {
        Self::default()
    }

    /// Queues one output for the next run.
    pub fn push_output(&self, output: FakeCommandOutput) {
        self.state
            .lock()
            .expect("fake process mutex")
            .outputs
            .push_back(output);
    }

    /// Queues many outputs for subsequent runs.
    pub fn with_outputs<I>(self, outputs: I) -> Self
    where
        I: IntoIterator<Item = FakeCommandOutput>,
    {
        {
            let mut state = self.state.lock().expect("fake process mutex");
            state.outputs.extend(outputs);
        }
        self
    }

    /// Records a request and returns the next queued output.
    pub fn run(&self, request: FakeCommandRequest) -> Result<FakeCommandOutput> {
        let mut state = self.state.lock().expect("fake process mutex");
        state.requests.push(request);
        state
            .outputs
            .pop_front()
            .ok_or_else(|| Error::not_found("no fake process output queued"))
    }

    /// Returns all captured requests.
    pub fn requests(&self) -> Vec<FakeCommandRequest> {
        self.state
            .lock()
            .expect("fake process mutex")
            .requests
            .clone()
    }

    /// Returns the last captured request.
    pub fn last_request(&self) -> Option<FakeCommandRequest> {
        self.state
            .lock()
            .expect("fake process mutex")
            .requests
            .last()
            .cloned()
    }
}

fn sanitize_path_part(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>();

    if sanitized.is_empty() {
        String::from("ezrs-test")
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_map_is_owned_and_mutable_without_global_env_changes() {
        let env = EnvMap::new().set("PORT", "8080").without("DEBUG");

        assert_eq!(env.get_string("PORT").as_deref(), Some("8080"));
        assert_eq!(env.get_string("DEBUG"), None);
    }

    #[test]
    fn temp_workspace_writes_reads_and_cleans_up() {
        let root;
        {
            let workspace = TempWorkspace::new("ezrs workspace").expect("workspace");
            root = workspace.root().to_path_buf();
            workspace
                .write("nested/file.txt", "hello")
                .expect("write fixture file");

            assert_eq!(
                workspace
                    .read_to_string("nested/file.txt")
                    .expect("read fixture file"),
                "hello"
            );
            assert!(workspace.path("nested/file.txt").exists());
        }

        assert!(!root.exists());
    }

    #[test]
    fn fixture_writes_many_files() {
        let fixture = Fixture::new("fixture").expect("fixture");
        fixture
            .files([("a.txt", "a"), ("nested/b.txt", "b")])
            .expect("fixture files");

        assert_eq!(
            fixture
                .workspace()
                .read_to_string("nested/b.txt")
                .expect("read fixture file"),
            "b"
        );
    }

    #[test]
    fn fake_process_runner_records_requests_and_returns_outputs() {
        let runner = FakeProcessRunner::new().with_outputs([FakeCommandOutput::success("done\n")]);

        let output = runner
            .run(
                FakeCommandRequest::new("cargo")
                    .args(["check", "--workspace"])
                    .env("RUST_LOG", "debug"),
            )
            .expect("fake process output");

        assert!(output.is_success());
        assert_eq!(output.stdout_lossy(), "done\n");

        let request = runner.last_request().expect("recorded request");
        assert_eq!(request.program, "cargo");
        assert_eq!(request.args, ["check", "--workspace"]);
        assert_eq!(request.env.get_string("RUST_LOG").as_deref(), Some("debug"));
    }
}
