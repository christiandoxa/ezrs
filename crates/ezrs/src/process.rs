//! Child process helpers for application command execution.

use std::{
    path::{Path, PathBuf},
    process::{ExitStatus, Stdio},
    time::Duration,
};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{Error, Result};

/// Builder for running a child process.
#[derive(Debug, Clone)]
pub struct Process {
    program: String,
    args: Vec<String>,
    envs: Vec<(String, String)>,
    current_dir: Option<PathBuf>,
    stdin: Option<Vec<u8>>,
    timeout: Option<Duration>,
    capture: bool,
}

impl Process {
    /// Creates a process builder for an external program path or name.
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            envs: Vec::new(),
            current_dir: None,
            stdin: None,
            timeout: None,
            capture: false,
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

    /// Sets one environment variable for the child.
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.envs.push((key.into(), value.into()));
        self
    }

    /// Sets the working directory for the child.
    pub fn current_dir(mut self, dir: impl AsRef<Path>) -> Self {
        self.current_dir = Some(dir.as_ref().to_path_buf());
        self
    }

    /// Sends bytes to the child's standard input.
    pub fn stdin(mut self, input: impl Into<Vec<u8>>) -> Self {
        self.stdin = Some(input.into());
        self
    }

    /// Kills the child if it has not exited after the given number of seconds.
    pub fn timeout_secs(mut self, seconds: u64) -> Self {
        self.timeout = Some(Duration::from_secs(seconds));
        self
    }

    /// Captures stdout and stderr instead of inheriting them from the parent.
    pub fn capture(mut self) -> Self {
        self.capture = true;
        self
    }

    /// Runs the child and returns status plus any captured output.
    pub async fn run(self) -> Result<ProcessOutput> {
        let mut command = tokio::process::Command::new(&self.program);
        command.args(&self.args).envs(self.envs.iter().cloned());

        if let Some(current_dir) = &self.current_dir {
            command.current_dir(current_dir);
        }

        if self.stdin.is_some() {
            command.stdin(Stdio::piped());
        }

        if self.capture {
            command.stdout(Stdio::piped()).stderr(Stdio::piped());
        }

        let mut child = command.spawn()?;

        let stdin_task = match (self.stdin, child.stdin.take()) {
            (Some(input), Some(mut stdin)) => Some(tokio::spawn(async move {
                stdin.write_all(&input).await?;
                stdin.shutdown().await
            })),
            _ => None,
        };

        let stdout_task = child.stdout.take().map(|mut stdout| {
            tokio::spawn(async move {
                let mut bytes = Vec::new();
                stdout.read_to_end(&mut bytes).await?;
                Ok::<_, std::io::Error>(bytes)
            })
        });

        let stderr_task = child.stderr.take().map(|mut stderr| {
            tokio::spawn(async move {
                let mut bytes = Vec::new();
                stderr.read_to_end(&mut bytes).await?;
                Ok::<_, std::io::Error>(bytes)
            })
        });

        let status = match self.timeout {
            Some(timeout) => match tokio::time::timeout(timeout, child.wait()).await {
                Ok(status) => status?,
                Err(_) => {
                    let _ = child.kill().await;
                    let _ = child.wait().await;
                    return Err(Error::timeout(format!(
                        "process `{}` exceeded {}s",
                        self.program,
                        timeout.as_secs()
                    )));
                }
            },
            None => child.wait().await?,
        };

        if let Some(task) = stdin_task {
            task.await
                .map_err(|error| Error::msg(format!("stdin task failed: {error}")))??;
        }

        let stdout = join_bytes(stdout_task, "stdout").await?;
        let stderr = join_bytes(stderr_task, "stderr").await?;

        Ok(ProcessOutput {
            status: ProcessStatus::from(status),
            stdout,
            stderr,
        })
    }

    /// Runs the child and returns only its exit status.
    pub async fn status(self) -> Result<ProcessStatus> {
        Ok(self.run().await?.status)
    }

    /// Runs the child and returns an error when it exits unsuccessfully.
    pub async fn success(self) -> Result<()> {
        let program = self.program.clone();
        let output = self.run().await?;
        if output.status.success {
            Ok(())
        } else {
            Err(Error::msg(format!(
                "process `{}` exited with status {}",
                program, output.status
            )))
        }
    }
}

/// Exit status for a completed child process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessStatus {
    /// Platform exit code when available.
    pub code: Option<i32>,
    /// True when the process reported success.
    pub success: bool,
}

impl From<ExitStatus> for ProcessStatus {
    fn from(status: ExitStatus) -> Self {
        Self {
            code: status.code(),
            success: status.success(),
        }
    }
}

impl std::fmt::Display for ProcessStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.code {
            Some(code) => write!(formatter, "code {code}"),
            None => formatter.write_str("terminated by signal"),
        }
    }
}

/// Completed process output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessOutput {
    /// Process exit status.
    pub status: ProcessStatus,
    /// Captured stdout bytes. Empty unless [`Process::capture`] was used.
    pub stdout: Vec<u8>,
    /// Captured stderr bytes. Empty unless [`Process::capture`] was used.
    pub stderr: Vec<u8>,
}

impl ProcessOutput {
    /// Captured stdout as UTF-8 lossily decoded text.
    pub fn stdout_lossy(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }

    /// Captured stderr as UTF-8 lossily decoded text.
    pub fn stderr_lossy(&self) -> String {
        String::from_utf8_lossy(&self.stderr).into_owned()
    }
}

async fn join_bytes(
    task: Option<tokio::task::JoinHandle<std::io::Result<Vec<u8>>>>,
    stream: &str,
) -> Result<Vec<u8>> {
    match task {
        Some(task) => task
            .await
            .map_err(|error| Error::msg(format!("{stream} task failed: {error}")))?
            .map_err(Error::from),
        None => Ok(Vec::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[tokio::test]
    async fn captures_stdout_stderr_and_status() {
        let output = Process::new("sh")
            .args(["-c", "printf out; printf err >&2"])
            .capture()
            .run()
            .await
            .expect("process output");

        assert!(output.status.success);
        assert_eq!(output.status.code, Some(0));
        assert_eq!(output.stdout_lossy(), "out");
        assert_eq!(output.stderr_lossy(), "err");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn sends_stdin_to_child() {
        let output = Process::new("sh")
            .args(["-c", "cat"])
            .stdin("input")
            .capture()
            .run()
            .await
            .expect("process output");

        assert_eq!(output.stdout_lossy(), "input");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn success_reports_non_zero_exit() {
        let error = Process::new("sh")
            .args(["-c", "exit 7"])
            .success()
            .await
            .expect_err("non-zero exit");

        assert!(error.to_string().contains("code 7"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn timeout_kills_child() {
        let error = Process::new("sh")
            .args(["-c", "sleep 2"])
            .timeout_secs(1)
            .run()
            .await
            .expect_err("timeout");

        assert!(
            error
                .to_string()
                .contains("timeout: process `sh` exceeded 1s")
        );
    }
}
