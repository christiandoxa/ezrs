use std::{
    any::{Any, TypeId},
    collections::HashMap,
    future::Future,
    sync::{Arc, Mutex},
};

use serde::de::DeserializeOwned;

use crate::{
    Args, CommandSpec, ConfigSource, Context, Error, Result, command::Command,
    lifecycle::Lifecycle, state::TypeStore, test_support::EnvMap,
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
    default_command: Option<Command>,
    registration_errors: Vec<String>,
    state: TypeStore,
    config_loaders: Vec<ConfigLoader>,
    lifecycle: Lifecycle,
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

    /// Registers typed config loaded from explicit file/env layers.
    pub fn config_from<T>(mut self, source: ConfigSource) -> Self
    where
        T: DeserializeOwned + Clone + Send + Sync + 'static,
    {
        self.config_loaders.push(Arc::new(move || {
            let value = crate::config::load_from_source::<T>(source.clone())?;
            Ok(value.map(|config| {
                (
                    TypeId::of::<T>(),
                    Arc::new(config) as Arc<dyn Any + Send + Sync>,
                )
            }))
        }));
        self
    }

    /// Registers typed config and validates it after loading.
    pub fn config_validated<T, F>(mut self, source: ConfigSource, validate: F) -> Self
    where
        T: DeserializeOwned + Clone + Send + Sync + 'static,
        F: Fn(&T) -> Result<()> + Send + Sync + 'static,
    {
        let validate = Arc::new(validate);
        self.config_loaders.push(Arc::new(move || {
            let validate = Arc::clone(&validate);
            let value =
                crate::config::load_validated(source.clone(), move |config: &T| validate(config))?;
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

    /// Registers an async command with a typed argument schema.
    pub fn command_with<F, Fut>(mut self, handler: F, spec: CommandSpec) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let command = Command::from_handler(handler).spec(spec);
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

    /// Registers an async command with an additional CLI alias.
    pub fn command_alias<F, Fut>(mut self, alias: impl Into<String>, handler: F) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let command = Command::from_handler(handler).alias(alias);
        self.insert_command(command);
        self
    }

    /// Registers a command that is routable but omitted from help output.
    pub fn hidden_command<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let command = Command::from_handler(handler).hidden();
        self.insert_command(command);
        self
    }

    /// Registers a command with short help text.
    pub fn command_about<F, Fut>(mut self, about: impl Into<String>, handler: F) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let command = Command::from_handler(handler).about(about);
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
        self.default_command = Some(Command::from_handler(handler));
        self
    }

    /// Registers a default command with a typed argument schema.
    pub fn default_command_with<F, Fut>(mut self, handler: F, spec: CommandSpec) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.default_command = Some(Command::from_handler(handler).spec(spec));
        self
    }

    /// Registers lifecycle hooks for startup, readiness, and shutdown.
    pub fn lifecycle(mut self, lifecycle: Lifecycle) -> Self {
        self.lifecycle = lifecycle;
        self
    }

    /// Registers a startup lifecycle hook.
    pub fn on_start<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.lifecycle = self.lifecycle.on_start(handler);
        self
    }

    /// Registers a readiness lifecycle hook.
    pub fn on_ready<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.lifecycle = self.lifecycle.on_ready(handler);
        self
    }

    /// Registers a shutdown lifecycle hook.
    pub fn on_shutdown<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.lifecycle = self.lifecycle.on_shutdown(handler);
        self
    }

    /// Sets the shutdown timeout for lifecycle hooks.
    pub fn shutdown_timeout_secs(mut self, seconds: u64) -> Self {
        self.lifecycle = self.lifecycle.shutdown_timeout_secs(seconds);
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

    /// Runs the app and returns the shell exit code implied by the result.
    pub async fn run_exit_code(self) -> i32 {
        match self.run().await {
            Ok(()) => 0,
            Err(error) => error.exit_code(),
        }
    }

    /// Runs the app and terminates the process with the implied shell exit code.
    pub async fn run_and_exit(self) -> ! {
        let code = self.run_exit_code().await;
        std::process::exit(code);
    }

    /// Builds an in-memory test runner for this app.
    pub fn test(self) -> AppTest {
        AppTest {
            app: self,
            args: Vec::new(),
            env: None,
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

        let command = self.command_for(selection.group.as_deref(), selection.command.as_deref())?;
        let args = Args::parse_with_spec(&selection.args, &command.spec, |key| {
            std::env::var(key).ok()
        })?;
        let config = self.load_config()?;
        let ctx = Context::runtime(args, self.state.clone(), config);
        ctx.install_ctrl_c();

        self.lifecycle.run_start(ctx.clone()).await?;
        self.lifecycle.run_ready(ctx.clone()).await?;

        let handler = Arc::clone(&command.handler);
        let command_result = match handler(ctx.clone()).await {
            Ok(()) => ctx.join_all().await,
            Err(error) => Err(error),
        };

        let shutdown_result = self.lifecycle.run_shutdown(ctx.clone()).await;

        match (command_result, shutdown_result) {
            (Err(command_error), _) => Err(command_error),
            (Ok(()), Err(shutdown_error)) => Err(shutdown_error),
            (Ok(()), Ok(())) => Ok(()),
        }
    }

    fn insert_command(&mut self, command: Command) {
        if let Some(error) = command.spec.validation_error() {
            self.registration_errors.push(format!(
                "command '{}': invalid argument schema: {error}",
                command.name
            ));
            return;
        }

        if self.command_name_taken(&command.name)
            || command
                .aliases
                .iter()
                .any(|alias| self.command_name_taken(alias))
        {
            self.registration_errors
                .push(format!("duplicate command '{}'", command.name));
            return;
        }

        self.commands.insert(command.name.clone(), command);
    }

    fn command_name_taken(&self, name: &str) -> bool {
        self.commands.contains_key(name)
            || self
                .commands
                .values()
                .any(|command| command.aliases.iter().any(|alias| alias == name))
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

    fn command_for(&self, group: Option<&str>, command: Option<&str>) -> Result<&Command> {
        if let Some(group) = group {
            let group = self
                .groups
                .get(group)
                .ok_or_else(|| Error::invalid_input(format!("unknown command group '{group}'")))?;
            let command = command.ok_or_else(|| {
                Error::invalid_input(format!("no command provided for group '{}'", group.name))
            })?;
            return group.command_for(command);
        }

        if let Some(command) = command {
            return self
                .find_command(command)
                .ok_or_else(|| Error::invalid_input(format!("unknown command '{command}'")));
        }

        self.default_command
            .as_ref()
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
                && (self.find_command(first).is_some() || self.default_command.is_none())
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
            args: remaining,
            help,
            version,
        }
    }

    fn help_text(&self, group: Option<&str>, command: Option<&str>) -> String {
        let name = self.name.as_deref().unwrap_or("ezrs-app");
        let mut lines = Vec::new();

        if let Some(command_name) = command {
            let route = match group {
                Some(group) => format!("{group} {command_name}"),
                None => command_name.to_string(),
            };
            if let Ok(command) = self.command_for(group, Some(command_name)) {
                lines.push(format!(
                    "Usage: {name} {route} {}",
                    command.spec.usage_suffix()
                ));
                lines.push(String::new());
                lines.push(format!("Command: {route}"));
                if let Some(about) = &command.about {
                    lines.push(about.clone());
                }
                let arg_help = command.spec.help_lines();
                if !arg_help.is_empty() {
                    lines.push(String::new());
                    lines.extend(arg_help);
                }
                return lines.join("\n");
            }
        }

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
                    if let Some(command) = group.commands.get(&name)
                        && !command.hidden
                    {
                        lines.push(format_command_help(command));
                    }
                }
            }
        }

        if !self.commands.is_empty() {
            lines.push(String::new());
            lines.push(String::from("Commands:"));
            let mut names = self.commands.keys().cloned().collect::<Vec<_>>();
            names.sort();
            for name in names {
                if let Some(command) = self.commands.get(&name)
                    && !command.hidden
                {
                    lines.push(format_command_help(command));
                }
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

    fn find_command(&self, name: &str) -> Option<&Command> {
        self.commands.get(name).or_else(|| {
            self.commands
                .values()
                .find(|command| command.aliases.iter().any(|alias| alias == name))
        })
    }
}

fn format_command_help(command: &Command) -> String {
    let mut line = format!("  {}", command.name);
    if !command.aliases.is_empty() {
        line.push_str(&format!(" (aliases: {})", command.aliases.join(", ")));
    }
    if let Some(about) = &command.about {
        line.push_str(&format!(" - {about}"));
    }
    line
}

fn println_to_process(message: &str) {
    use std::io::Write as _;
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "{message}");
}

struct Selection {
    group: Option<String>,
    command: Option<String>,
    args: Vec<String>,
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

    /// Registers a command inside this group with a typed argument schema.
    pub fn command_with<F, Fut>(mut self, handler: F, spec: CommandSpec) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.insert_command(Command::from_handler(handler).spec(spec));
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

    /// Registers a command inside this group with an additional CLI alias.
    pub fn command_alias<F, Fut>(mut self, alias: impl Into<String>, handler: F) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.insert_command(Command::from_handler(handler).alias(alias));
        self
    }

    /// Registers a hidden command inside this group.
    pub fn hidden_command<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.insert_command(Command::from_handler(handler).hidden());
        self
    }

    /// Registers a command inside this group with short help text.
    pub fn command_about<F, Fut>(mut self, about: impl Into<String>, handler: F) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.insert_command(Command::from_handler(handler).about(about));
        self
    }

    fn insert_command(&mut self, command: Command) {
        if let Some(error) = command.spec.validation_error() {
            self.registration_errors.push(format!(
                "command '{}': invalid argument schema: {error}",
                command.name
            ));
            return;
        }

        if self.command_name_taken(&command.name)
            || command
                .aliases
                .iter()
                .any(|alias| self.command_name_taken(alias))
        {
            self.registration_errors
                .push(format!("duplicate command '{}'", command.name));
            return;
        }

        self.commands.insert(command.name.clone(), command);
    }

    fn command_name_taken(&self, name: &str) -> bool {
        self.commands.contains_key(name)
            || self
                .commands
                .values()
                .any(|command| command.aliases.iter().any(|alias| alias == name))
    }

    fn registration_result(&self) -> Result<()> {
        if let Some(error) = self.registration_errors.first() {
            Err(Error::invalid_input(error.clone()))
        } else {
            Ok(())
        }
    }

    fn command_for(&self, command: &str) -> Result<&Command> {
        self.find_command(command).ok_or_else(|| {
            Error::invalid_input(format!(
                "unknown command '{command}' in group '{}'",
                self.name
            ))
        })
    }

    fn find_command(&self, name: &str) -> Option<&Command> {
        self.commands.get(name).or_else(|| {
            self.commands
                .values()
                .find(|command| command.aliases.iter().any(|alias| alias == name))
        })
    }
}

/// In-memory command test runner.
pub struct AppTest {
    app: App,
    args: Vec<String>,
    env: Option<EnvMap>,
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

    /// Sets owned environment values visible to `ctx.env` and schema env bindings.
    pub fn env(mut self, env: EnvMap) -> Self {
        self.env = Some(env);
        self
    }

    /// Runs the selected command in memory.
    pub async fn run(self) -> TestResult {
        crate::config::load_env();
        self.app.run_test_tokens(self.args, self.env).await
    }
}

impl App {
    async fn run_test_tokens(&self, tokens: Vec<String>, env: Option<EnvMap>) -> TestResult {
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

        let command =
            match self.command_for(selection.group.as_deref(), selection.command.as_deref()) {
                Ok(command) => command,
                Err(error) => return TestResult::failure(stdout, stderr, error),
            };

        let env_for_args = env.clone();
        let args = match Args::parse_with_spec(&selection.args, &command.spec, |key| {
            env_for_args
                .as_ref()
                .and_then(|env| env.get_string(key))
                .or_else(|| std::env::var(key).ok())
        }) {
            Ok(args) => args,
            Err(error) => return TestResult::failure(stdout, stderr, error),
        };

        let config = match self.load_config() {
            Ok(config) => config,
            Err(error) => return TestResult::failure(stdout, stderr, error),
        };

        let ctx = Context::memory(
            args,
            self.state.clone(),
            config,
            stdout.clone(),
            stderr.clone(),
            env,
        );

        if let Err(error) = self.lifecycle.run_start(ctx.clone()).await {
            return TestResult::failure(stdout, stderr, error);
        }
        if let Err(error) = self.lifecycle.run_ready(ctx.clone()).await {
            return TestResult::failure(stdout, stderr, error);
        }

        let handler = Arc::clone(&command.handler);
        let command_result = match handler(ctx.clone()).await {
            Ok(()) => ctx.join_all().await,
            Err(error) => Err(error),
        };

        let shutdown_result = self.lifecycle.run_shutdown(ctx.clone()).await;

        match (command_result, shutdown_result) {
            (Err(command_error), _) => TestResult::failure(stdout, stderr, command_error),
            (Ok(()), Err(shutdown_error)) => TestResult::failure(stdout, stderr, shutdown_error),
            (Ok(()), Ok(())) => TestResult::success(stdout, stderr),
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
    /// CLI-oriented exit code. Success is 0.
    pub exit_code: i32,
}

impl TestResult {
    fn success(stdout: Arc<Mutex<String>>, stderr: Arc<Mutex<String>>) -> Self {
        Self {
            success: true,
            stdout: stdout.lock().expect("stdout buffer poisoned").clone(),
            stderr: stderr.lock().expect("stderr buffer poisoned").clone(),
            error: None,
            exit_code: 0,
        }
    }

    fn failure(stdout: Arc<Mutex<String>>, stderr: Arc<Mutex<String>>, error: Error) -> Self {
        let exit_code = error.exit_code();
        {
            let mut err = stderr.lock().expect("stderr buffer poisoned");
            err.push_str(&format!("error: {error}\n"));
        }

        Self {
            success: false,
            stdout: stdout.lock().expect("stdout buffer poisoned").clone(),
            stderr: stderr.lock().expect("stderr buffer poisoned").clone(),
            error: Some(error.to_string()),
            exit_code,
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

    /// Asserts the captured exit code.
    pub fn assert_exit_code(&self, expected: i32) {
        assert_eq!(
            self.exit_code, expected,
            "expected exit code {expected}, got {}",
            self.exit_code
        );
    }

    /// Asserts stdout contains text.
    pub fn assert_stdout_contains(&self, expected: &str) {
        assert!(
            self.stdout.contains(expected),
            "expected stdout to contain {expected:?}, got {:?}",
            self.stdout
        );
    }

    /// Asserts stdout equals text exactly.
    pub fn assert_stdout_eq(&self, expected: &str) {
        assert_eq!(self.stdout, expected, "stdout mismatch");
    }

    /// Asserts stderr contains text.
    pub fn assert_stderr_contains(&self, expected: &str) {
        assert!(
            self.stderr.contains(expected),
            "expected stderr to contain {expected:?}, got {:?}",
            self.stderr
        );
    }

    /// Asserts stderr equals text exactly.
    pub fn assert_stderr_eq(&self, expected: &str) {
        assert_eq!(self.stderr, expected, "stderr mismatch");
    }

    /// Asserts the captured error contains text.
    pub fn assert_error_contains(&self, expected: &str) {
        let error = self.error.as_deref().unwrap_or("");
        assert!(
            error.contains(expected),
            "expected error to contain {expected:?}, got {error:?}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ArgSpec;

    async fn hello(ctx: Context) -> Result<()> {
        let name = ctx.arg_or("name", "world");
        ctx.println(format!("hello {name}"));
        Ok(())
    }

    async fn fail(ctx: Context) -> Result<()> {
        ctx.eprintln("about to fail");
        Err(Error::msg("failed"))
    }

    async fn exit(ctx: Context) -> Result<()> {
        ctx.eprintln("exiting");
        Err(Error::exit(7, "explicit exit"))
    }

    async fn status(ctx: Context) -> Result<()> {
        ctx.println("status ok");
        Ok(())
    }

    async fn scan(ctx: Context) -> Result<()> {
        ctx.println(format!(
            "scan path={} recursive={} limit={}",
            ctx.arg("path")?,
            ctx.flag("recursive"),
            ctx.arg("limit")?
        ));
        Ok(())
    }

    async fn start(ctx: Context) -> Result<()> {
        ctx.println("start");
        Ok(())
    }

    async fn ready(ctx: Context) -> Result<()> {
        ctx.println("ready");
        Ok(())
    }

    async fn shutdown(ctx: Context) -> Result<()> {
        ctx.println("shutdown");
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
        res.assert_exit_code(1);
    }

    #[tokio::test]
    async fn captures_explicit_exit_code() {
        let res = App::new().command(exit).test().args(["exit"]).run().await;

        res.assert_failure();
        res.assert_stderr_contains("explicit exit");
        res.assert_exit_code(7);
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
    async fn command_schema_validates_defaults_env_and_short_flags() {
        let spec = CommandSpec::new()
            .arg(ArgSpec::option("path").short('p').required())
            .arg(ArgSpec::flag("recursive").short('r'))
            .arg(ArgSpec::option("limit").env("SCAN_LIMIT").default("10"));

        let res = App::new()
            .command_with(scan, spec)
            .test()
            .env(EnvMap::new().set("SCAN_LIMIT", "25"))
            .args(["scan", "-r", "-p", "src"])
            .run()
            .await;

        res.assert_success();
        res.assert_stdout_contains("scan path=src recursive=true limit=25");
    }

    #[tokio::test]
    async fn command_schema_help_lists_flags() {
        let spec = CommandSpec::new().arg(
            ArgSpec::option("path")
                .short('p')
                .required()
                .help("Path to scan"),
        );

        let res = App::new()
            .name("demo")
            .command_with(scan, spec)
            .test()
            .args(["scan", "--help"])
            .run()
            .await;

        res.assert_success();
        res.assert_stdout_contains("Usage: demo scan [OPTIONS]");
        res.assert_stdout_contains("-p, --path <PATH> - Path to scan [required]");
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
        res.assert_exit_code(2);
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

    #[tokio::test]
    async fn lifecycle_hooks_run_around_command() {
        let res = App::new()
            .on_start(start)
            .on_ready(ready)
            .on_shutdown(shutdown)
            .command(status)
            .test()
            .args(["status"])
            .run()
            .await;

        res.assert_success();
        assert_eq!(res.stdout, "start\nready\nstatus ok\nshutdown\n");
    }

    #[tokio::test]
    async fn lifecycle_shutdown_runs_after_command_failure() {
        let res = App::new()
            .on_shutdown(shutdown)
            .command(fail)
            .test()
            .args(["fail"])
            .run()
            .await;

        res.assert_failure();
        res.assert_stdout_contains("shutdown");
        res.assert_stderr_contains("failed");
    }

    #[tokio::test]
    async fn command_alias_routes_to_handler() {
        let res = App::new()
            .command_alias("hi", hello)
            .test()
            .args(["hi", "--name", "Ayu"])
            .run()
            .await;

        res.assert_success();
        res.assert_stdout_contains("hello Ayu");
    }

    #[tokio::test]
    async fn nested_command_group_alias_routes_to_handler() {
        let res = App::new()
            .group(crate::CommandGroup::__from_static("admin").command_alias("ok", status))
            .test()
            .args(["admin", "ok"])
            .run()
            .await;

        res.assert_success();
        res.assert_stdout_contains("status ok");
    }

    #[tokio::test]
    async fn hidden_command_routes_but_is_not_listed_in_help() {
        let help = App::new()
            .hidden_command(status)
            .test()
            .args(["--help"])
            .run()
            .await;

        help.assert_success();
        assert!(!help.stdout.contains("status"));

        let run = App::new()
            .hidden_command(status)
            .test()
            .args(["status"])
            .run()
            .await;

        run.assert_success();
        run.assert_stdout_contains("status ok");
    }
}
