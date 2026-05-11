use std::{
    any::{Any, TypeId},
    collections::HashMap,
    future::Future,
    sync::{Arc, Mutex},
};

use serde::de::DeserializeOwned;

use crate::{
    Args, Context, Error, Result,
    command::{Command, CommandHandler},
    state::TypeStore,
};

type ConfigLoader =
    Arc<dyn Fn() -> Result<Option<(TypeId, Arc<dyn Any + Send + Sync>)>> + Send + Sync>;

/// Builder-style application model.
#[derive(Clone, Default)]
pub struct App {
    name: Option<String>,
    version: Option<String>,
    about: Option<String>,
    commands: HashMap<String, Command>,
    default_command: Option<CommandHandler>,
    state: TypeStore,
    config_loaders: Vec<ConfigLoader>,
}

impl App {
    /// Creates an empty app.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets app name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets app version.
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Sets app about text.
    pub fn about(mut self, about: impl Into<String>) -> Self {
        self.about = Some(about.into());
        self
    }

    /// Adds cloned app state by type.
    pub fn state<T>(mut self, value: T) -> Self
    where
        T: Clone + Send + Sync + 'static,
    {
        self.state.insert(value);
        self
    }

    /// Registers typed config loaded from ezrs.toml when present.
    pub fn config<T>(mut self) -> Self
    where
        T: DeserializeOwned + Clone + Send + Sync + 'static,
    {
        self.config_loaders.push(Arc::new(|| {
            let value = crate::config::load_optional::<T>()?;
            Ok(value.map(|config| {
                (
                    TypeId::of::<T>(),
                    Arc::new(config) as Arc<dyn Any + Send + Sync>,
                )
            }))
        }));
        self
    }

    /// Registers a named async command.
    pub fn command<F, Fut>(mut self, name: impl Into<String>, handler: F) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let command = Command::new(name, handler);
        self.commands.insert(command.name.clone(), command);
        self
    }

    /// Registers a default command used when no known subcommand is given.
    pub fn default_command<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.default_command = Some(Arc::new(move |ctx| Box::pin(handler(ctx))));
        self
    }

    /// Parses process args and runs the selected command.
    pub async fn run(self) -> Result<()> {
        crate::log::init_default()?;
        crate::config::load_env();

        let tokens = std::env::args().skip(1).collect::<Vec<_>>();
        let outcome = self.run_tokens(tokens).await;
        if let Err(error) = &outcome {
            let mut err = std::io::stderr().lock();
            use std::io::Write as _;
            let _ = writeln!(err, "error: {error}");
        }
        outcome.map(|_| ())
    }

    /// Builds an in-memory test runner for this app.
    pub fn test(self) -> AppTest {
        AppTest {
            app: self,
            args: Vec::new(),
        }
    }

    async fn run_tokens(&self, tokens: Vec<String>) -> Result<()> {
        let selection = self.select(&tokens);

        if selection.help {
            let output = self.help_text(selection.command.as_deref());
            println_to_process(&output);
            return Ok(());
        }

        if selection.version {
            let output = self.version_text();
            println_to_process(&output);
            return Ok(());
        }

        let config = self.load_config()?;
        let ctx = Context::process(selection.args, self.state.clone(), config);
        ctx.install_ctrl_c();

        let handler = self.handler_for(selection.command.as_deref())?;
        let result = match handler(ctx.clone()).await {
            Ok(()) => ctx.join_all().await,
            Err(error) => Err(error),
        };

        match result {
            Ok(()) => Ok(()),
            Err(error) => Err(error),
        }
    }

    fn load_config(&self) -> Result<TypeStore> {
        let mut store = TypeStore::default();
        for loader in &self.config_loaders {
            if let Some((type_id, value)) = loader()? {
                store.insert_arc(type_id, value);
            }
        }
        Ok(store)
    }

    fn handler_for(&self, command: Option<&str>) -> Result<CommandHandler> {
        if let Some(command) = command {
            return self
                .commands
                .get(command)
                .map(|entry| Arc::clone(&entry.handler))
                .ok_or_else(|| Error::invalid_input(format!("unknown command '{command}'")));
        }

        self.default_command
            .as_ref()
            .map(Arc::clone)
            .ok_or_else(|| Error::invalid_input("no command provided"))
    }

    fn select(&self, tokens: &[String]) -> Selection {
        let mut command = None;
        let mut start = 0_usize;
        let mut help = false;
        let mut version = false;

        if let Some(first) = tokens.first() {
            if first == "--help" || first == "-h" {
                help = true;
                start = 1;
            } else if first == "--version" || first == "-V" {
                version = true;
                start = 1;
            } else if !first.starts_with('-')
                && (self.commands.contains_key(first) || self.default_command.is_none())
            {
                command = Some(first.clone());
                start = 1;
            }
        } else if self.default_command.is_none() && !self.commands.is_empty() {
            help = true;
        }

        let remaining = tokens[start..].to_vec();
        if remaining.iter().any(|arg| arg == "--help" || arg == "-h") {
            help = true;
        }
        if remaining
            .iter()
            .any(|arg| arg == "--version" || arg == "-V")
        {
            version = true;
        }

        Selection {
            command,
            args: Args::parse(&remaining),
            help,
            version,
        }
    }

    fn help_text(&self, command: Option<&str>) -> String {
        let name = self.name.as_deref().unwrap_or("ezrs-app");
        let mut lines = Vec::new();
        lines.push(format!("Usage: {name} [COMMAND] [ARGS]"));

        if let Some(about) = &self.about {
            lines.push(String::new());
            lines.push(about.clone());
        }

        if let Some(command) = command {
            lines.push(String::new());
            lines.push(format!("Command: {command}"));
        }

        if !self.commands.is_empty() {
            lines.push(String::new());
            lines.push(String::from("Commands:"));
            let mut names = self.commands.keys().cloned().collect::<Vec<_>>();
            names.sort();
            for name in names {
                lines.push(format!("  {name}"));
            }
        }

        lines.join("\n")
    }

    fn version_text(&self) -> String {
        let name = self.name.as_deref().unwrap_or("ezrs-app");
        let version = self.version.as_deref().unwrap_or("0.1.0");
        format!("{name} {version}")
    }
}

fn println_to_process(message: &str) {
    use std::io::Write as _;
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "{message}");
}

struct Selection {
    command: Option<String>,
    args: Args,
    help: bool,
    version: bool,
}

/// In-memory command test runner.
pub struct AppTest {
    app: App,
    args: Vec<String>,
}

impl AppTest {
    /// Sets test args without binary name.
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args = args.into_iter().map(Into::into).collect();
        self
    }

    /// Runs the selected command in memory.
    pub async fn run(self) -> TestResult {
        crate::config::load_env();
        self.app.run_test_tokens(self.args).await
    }
}

impl App {
    async fn run_test_tokens(&self, tokens: Vec<String>) -> TestResult {
        let selection = self.select(&tokens);
        let stdout = Arc::new(Mutex::new(String::new()));
        let stderr = Arc::new(Mutex::new(String::new()));

        if selection.help {
            stdout
                .lock()
                .expect("stdout buffer poisoned")
                .push_str(&format!(
                    "{}\n",
                    self.help_text(selection.command.as_deref())
                ));
            return TestResult::success(stdout, stderr);
        }

        if selection.version {
            stdout
                .lock()
                .expect("stdout buffer poisoned")
                .push_str(&format!("{}\n", self.version_text()));
            return TestResult::success(stdout, stderr);
        }

        let config = match self.load_config() {
            Ok(config) => config,
            Err(error) => return TestResult::failure(stdout, stderr, error),
        };

        let ctx = Context::memory(
            selection.args,
            self.state.clone(),
            config,
            stdout.clone(),
            stderr.clone(),
        );
        let handler = match self.handler_for(selection.command.as_deref()) {
            Ok(handler) => handler,
            Err(error) => return TestResult::failure(stdout, stderr, error),
        };

        let result = match handler(ctx.clone()).await {
            Ok(()) => ctx.join_all().await,
            Err(error) => Err(error),
        };

        match result {
            Ok(()) => TestResult::success(stdout, stderr),
            Err(error) => TestResult::failure(stdout, stderr, error),
        }
    }
}

/// Captured command test result.
#[derive(Debug, Clone)]
pub struct TestResult {
    /// True when the command returned Ok.
    pub success: bool,
    /// Captured stdout.
    pub stdout: String,
    /// Captured stderr.
    pub stderr: String,
    /// Error message, when the command failed.
    pub error: Option<String>,
}

impl TestResult {
    fn success(stdout: Arc<Mutex<String>>, stderr: Arc<Mutex<String>>) -> Self {
        Self {
            success: true,
            stdout: stdout.lock().expect("stdout buffer poisoned").clone(),
            stderr: stderr.lock().expect("stderr buffer poisoned").clone(),
            error: None,
        }
    }

    fn failure(stdout: Arc<Mutex<String>>, stderr: Arc<Mutex<String>>, error: Error) -> Self {
        {
            let mut err = stderr.lock().expect("stderr buffer poisoned");
            err.push_str(&format!("error: {error}\n"));
        }

        Self {
            success: false,
            stdout: stdout.lock().expect("stdout buffer poisoned").clone(),
            stderr: stderr.lock().expect("stderr buffer poisoned").clone(),
            error: Some(error.to_string()),
        }
    }

    /// Asserts the command succeeded.
    pub fn assert_success(&self) {
        assert!(
            self.success,
            "expected success, got failure: {}",
            self.error.as_deref().unwrap_or("unknown error")
        );
    }

    /// Asserts the command failed.
    pub fn assert_failure(&self) {
        assert!(self.error.is_some(), "expected failure, got success");
    }

    /// Asserts stdout contains text.
    pub fn assert_stdout_contains(&self, expected: &str) {
        assert!(
            self.stdout.contains(expected),
            "expected stdout to contain {expected:?}, got {:?}",
            self.stdout
        );
    }

    /// Asserts stderr contains text.
    pub fn assert_stderr_contains(&self, expected: &str) {
        assert!(
            self.stderr.contains(expected),
            "expected stderr to contain {expected:?}, got {:?}",
            self.stderr
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn hello(ctx: Context) -> Result<()> {
        let name = ctx.arg_or("name", "world");
        ctx.println(format!("hello {name}"));
        Ok(())
    }

    async fn fail(ctx: Context) -> Result<()> {
        ctx.eprintln("about to fail");
        Err(Error::msg("failed"))
    }

    #[test]
    fn builder_sets_metadata() {
        let app = App::new().name("demo").version("0.1.0").about("Demo");
        assert_eq!(app.name.as_deref(), Some("demo"));
        assert_eq!(app.version.as_deref(), Some("0.1.0"));
        assert_eq!(app.about.as_deref(), Some("Demo"));
    }

    #[tokio::test]
    async fn command_routing_runs_registered_command() {
        let res = App::new()
            .command("hello", hello)
            .test()
            .args(["hello", "--name", "Ayu"])
            .run()
            .await;

        res.assert_success();
        res.assert_stdout_contains("hello Ayu");
    }

    #[tokio::test]
    async fn default_command_runs_without_subcommand() {
        let res = App::new()
            .default_command(hello)
            .test()
            .args(["--name", "Ayu"])
            .run()
            .await;

        res.assert_success();
        res.assert_stdout_contains("hello Ayu");
    }

    #[tokio::test]
    async fn unknown_command_fails() {
        let res = App::new()
            .command("hello", hello)
            .test()
            .args(["missing"])
            .run()
            .await;

        res.assert_failure();
        res.assert_stderr_contains("unknown command");
    }

    #[tokio::test]
    async fn captures_stderr_assertion() {
        let res = App::new()
            .command("fail", fail)
            .test()
            .args(["fail"])
            .run()
            .await;

        res.assert_failure();
        res.assert_stderr_contains("about to fail");
        res.assert_stderr_contains("failed");
    }

    #[tokio::test]
    async fn help_and_version_output() {
        let help = App::new()
            .name("demo")
            .version("1.2.3")
            .command("hello", hello)
            .test()
            .args(["--help"])
            .run()
            .await;
        help.assert_success();
        help.assert_stdout_contains("Commands:");

        let version = App::new()
            .name("demo")
            .version("1.2.3")
            .command("hello", hello)
            .test()
            .args(["--version"])
            .run()
            .await;
        version.assert_success();
        version.assert_stdout_contains("demo 1.2.3");
    }

    #[tokio::test]
    async fn state_lookup_returns_cloned_state() {
        #[derive(Clone)]
        struct State {
            name: String,
        }

        async fn state_cmd(ctx: Context) -> Result<()> {
            let state = ctx.state::<State>()?;
            ctx.println(state.name);
            Ok(())
        }

        let res = App::new()
            .state(State {
                name: String::from("demo"),
            })
            .command("state", state_cmd)
            .test()
            .args(["state"])
            .run()
            .await;

        res.assert_success();
        res.assert_stdout_contains("demo");
    }

    #[tokio::test]
    async fn dynamic_args_include_flags_values_and_positionals() {
        async fn args_cmd(ctx: Context) -> Result<()> {
            ctx.println(ctx.arg("path")?);
            ctx.println(ctx.arg("0")?);
            ctx.println(ctx.flag("recursive"));
            Ok(())
        }

        let res = App::new()
            .command("args", args_cmd)
            .test()
            .args(["args", "--path=src", "input.txt", "--recursive"])
            .run()
            .await;

        res.assert_success();
        res.assert_stdout_contains("src");
        res.assert_stdout_contains("input.txt");
        res.assert_stdout_contains("true");
    }
}
