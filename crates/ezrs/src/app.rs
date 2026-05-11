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
    groups: HashMap<String, CommandGroup>,
    default_command: Option<CommandHandler>,
    registration_errors: Vec<String>,
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

    /// Registers an async command and derives its CLI name from the handler function.
    ///
    /// Function `scan` becomes command `scan`. Module handlers named `run`, such as
    /// `commands::scan::run`, become command `scan`.
    pub fn command<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let command = Command::from_handler(handler);
        self.insert_command(command);
        self
    }

    /// Registers an async command with an explicit name.
    ///
    /// Prefer [`App::command`] with a function item when Rust syntax can express the
    /// command identity directly. Use this escape hatch for closures, aliases, and
    /// compatibility with dynamic command tables.
    pub fn command_named<F, Fut>(mut self, name: impl Into<String>, handler: F) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let command = Command::named(name, handler);
        self.insert_command(command);
        self
    }

    /// Registers a nested command group.
    pub fn group(mut self, group: CommandGroup) -> Self {
        if self.commands.contains_key(&group.name) || self.groups.contains_key(&group.name) {
            self.registration_errors
                .push(format!("duplicate command group '{}'", group.name));
            return self;
        }

        self.groups.insert(group.name.clone(), group);
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
            let output = self.help_text(selection.group.as_deref(), selection.command.as_deref());
            println_to_process(&output);
            return Ok(());
        }

        if selection.version {
            let output = self.version_text();
            println_to_process(&output);
            return Ok(());
        }

        self.registration_result()?;

        let config = self.load_config()?;
        let ctx = Context::runtime(selection.args, self.state.clone(), config);
        ctx.install_ctrl_c();

        let handler = self.handler_for(selection.group.as_deref(), selection.command.as_deref())?;
        let result = match handler(ctx.clone()).await {
            Ok(()) => ctx.join_all().await,
            Err(error) => Err(error),
        };

        match result {
            Ok(()) => Ok(()),
            Err(error) => Err(error),
        }
    }

    fn insert_command(&mut self, command: Command) {
        if self.commands.contains_key(&command.name) {
            self.registration_errors
                .push(format!("duplicate command '{}'", command.name));
            return;
        }

        self.commands.insert(command.name.clone(), command);
    }

    fn registration_result(&self) -> Result<()> {
        if let Some(error) = self.registration_errors.first() {
            Err(Error::invalid_input(error.clone()))
        } else {
            for group in self.groups.values() {
                if let Err(error) = group.registration_result() {
                    return Err(Error::invalid_input(format!(
                        "command group '{}': {error}",
                        group.name
                    )));
                }
            }
            Ok(())
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

    fn handler_for(&self, group: Option<&str>, command: Option<&str>) -> Result<CommandHandler> {
        if let Some(group) = group {
            let group = self
                .groups
                .get(group)
                .ok_or_else(|| Error::invalid_input(format!("unknown command group '{group}'")))?;
            let command = command.ok_or_else(|| {
                Error::invalid_input(format!("no command provided for group '{}'", group.name))
            })?;
            return group.handler_for(command);
        }

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
        let mut group = None;
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
            } else if !first.starts_with('-') && self.groups.contains_key(first) {
                group = Some(first.clone());
                start = 1;
                if let Some(second) = tokens.get(1)
                    && !second.starts_with('-')
                {
                    command = Some(second.clone());
                    start = 2;
                }
            } else if !first.starts_with('-')
                && (self.commands.contains_key(first) || self.default_command.is_none())
            {
                command = Some(first.clone());
                start = 1;
            }
        } else if self.default_command.is_none()
            && (!self.commands.is_empty() || !self.groups.is_empty())
        {
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
            group,
            command,
            args: Args::parse(&remaining),
            help,
            version,
        }
    }

    fn help_text(&self, group: Option<&str>, command: Option<&str>) -> String {
        let name = self.name.as_deref().unwrap_or("ezrs-app");
        let mut lines = Vec::new();
        lines.push(format!("Usage: {name} [COMMAND] [ARGS]"));

        if let Some(about) = &self.about {
            lines.push(String::new());
            lines.push(about.clone());
        }

        if let Some(group) = group {
            lines.push(String::new());
            lines.push(format!("Command group: {group}"));

            if let Some(group) = self.groups.get(group)
                && !group.commands.is_empty()
            {
                lines.push(String::new());
                lines.push(String::from("Group commands:"));
                let mut names = group.commands.keys().cloned().collect::<Vec<_>>();
                names.sort();
                for name in names {
                    lines.push(format!("  {name}"));
                }
            }
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

        if !self.groups.is_empty() {
            lines.push(String::new());
            lines.push(String::from("Command groups:"));
            let mut names = self.groups.keys().cloned().collect::<Vec<_>>();
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
    group: Option<String>,
    command: Option<String>,
    args: Args,
    help: bool,
    version: bool,
}

/// Nested command group for command trees.
#[derive(Clone, Default)]
pub struct CommandGroup {
    name: String,
    commands: HashMap<String, Command>,
    registration_errors: Vec<String>,
}

impl CommandGroup {
    /// Creates a command group from macro-provided Rust syntax.
    #[doc(hidden)]
    pub fn __from_static(name: &'static str) -> Self {
        Self {
            name: name.trim_start_matches("r#").to_string(),
            commands: HashMap::new(),
            registration_errors: Vec::new(),
        }
    }

    /// Registers a command inside this group and derives its name from the handler.
    pub fn command<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.insert_command(Command::from_handler(handler));
        self
    }

    /// Registers a command inside this group with an explicit name.
    pub fn command_named<F, Fut>(mut self, name: impl Into<String>, handler: F) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.insert_command(Command::named(name, handler));
        self
    }

    fn insert_command(&mut self, command: Command) {
        if self.commands.contains_key(&command.name) {
            self.registration_errors
                .push(format!("duplicate command '{}'", command.name));
            return;
        }

        self.commands.insert(command.name.clone(), command);
    }

    fn registration_result(&self) -> Result<()> {
        if let Some(error) = self.registration_errors.first() {
            Err(Error::invalid_input(error.clone()))
        } else {
            Ok(())
        }
    }

    fn handler_for(&self, command: &str) -> Result<CommandHandler> {
        self.commands
            .get(command)
            .map(|entry| Arc::clone(&entry.handler))
            .ok_or_else(|| {
                Error::invalid_input(format!(
                    "unknown command '{command}' in group '{}'",
                    self.name
                ))
            })
    }
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
                    self.help_text(selection.group.as_deref(), selection.command.as_deref())
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

        if let Err(error) = self.registration_result() {
            return TestResult::failure(stdout, stderr, error);
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
        let handler =
            match self.handler_for(selection.group.as_deref(), selection.command.as_deref()) {
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

    async fn status(ctx: Context) -> Result<()> {
        ctx.println("status ok");
        Ok(())
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
            .command(hello)
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
            .command(hello)
            .test()
            .args(["missing"])
            .run()
            .await;

        res.assert_failure();
        res.assert_stderr_contains("unknown command");
    }

    #[tokio::test]
    async fn captures_stderr_assertion() {
        let res = App::new().command(fail).test().args(["fail"]).run().await;

        res.assert_failure();
        res.assert_stderr_contains("about to fail");
        res.assert_stderr_contains("failed");
    }

    #[tokio::test]
    async fn help_and_version_output() {
        let help = App::new()
            .name("demo")
            .version("1.2.3")
            .command(hello)
            .test()
            .args(["--help"])
            .run()
            .await;
        help.assert_success();
        help.assert_stdout_contains("Commands:");

        let version = App::new()
            .name("demo")
            .version("1.2.3")
            .command(hello)
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

        async fn state(ctx: Context) -> Result<()> {
            let state = ctx.state::<State>()?;
            ctx.println(state.name);
            Ok(())
        }

        let res = App::new()
            .state(State {
                name: String::from("demo"),
            })
            .command(state)
            .test()
            .args(["state"])
            .run()
            .await;

        res.assert_success();
        res.assert_stdout_contains("demo");
    }

    #[tokio::test]
    async fn dynamic_args_include_flags_values_and_positionals() {
        async fn args(ctx: Context) -> Result<()> {
            ctx.println(ctx.arg("path")?);
            ctx.println(ctx.arg("0")?);
            ctx.println(ctx.flag("recursive"));
            Ok(())
        }

        let res = App::new()
            .command(args)
            .test()
            .args(["args", "--path=src", "input.txt", "--recursive"])
            .run()
            .await;

        res.assert_success();
        res.assert_stdout_contains("src");
        res.assert_stdout_contains("input.txt");
        res.assert_stdout_contains("true");
    }

    #[tokio::test]
    async fn duplicate_command_registration_fails_clearly() {
        let res = App::new()
            .command(hello)
            .command(hello)
            .test()
            .args(["hello"])
            .run()
            .await;

        res.assert_failure();
        res.assert_stderr_contains("duplicate command 'hello'");
    }

    #[tokio::test]
    async fn nested_command_group_routes_command() {
        let res = App::new()
            .group(crate::command_group!(admin { status }))
            .test()
            .args(["admin", "status"])
            .run()
            .await;

        res.assert_success();
        res.assert_stdout_contains("status ok");
    }

    #[tokio::test]
    async fn nested_command_group_help_lists_group_commands() {
        let res = App::new()
            .group(crate::command_group!(admin { status }))
            .test()
            .args(["admin", "--help"])
            .run()
            .await;

        res.assert_success();
        res.assert_stdout_contains("Command group: admin");
        res.assert_stdout_contains("status");
    }

    #[tokio::test]
    async fn nested_command_group_unknown_command_fails() {
        let res = App::new()
            .group(crate::command_group!(admin { status }))
            .test()
            .args(["admin", "missing"])
            .run()
            .await;

        res.assert_failure();
        res.assert_stderr_contains("unknown command 'missing' in group 'admin'");
    }
}
