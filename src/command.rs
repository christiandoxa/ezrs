use std::{any::type_name, future::Future, pin::Pin, sync::Arc};

use crate::{CommandSpec, Context, Result};

pub(crate) type CommandFuture = Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>;
pub(crate) type CommandHandler = Arc<dyn Fn(Context) -> CommandFuture + Send + Sync + 'static>;

#[derive(Clone)]
pub(crate) struct Command {
    pub(crate) name: String,
    pub(crate) handler: CommandHandler,
    pub(crate) aliases: Vec<String>,
    pub(crate) hidden: bool,
    pub(crate) about: Option<String>,
    pub(crate) spec: CommandSpec,
}

impl Command {
    pub(crate) fn from_handler<F, Fut>(handler: F) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        Self::named(handler_name::<F>(), handler)
    }

    pub(crate) fn named<F, Fut>(name: impl Into<String>, handler: F) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        Self {
            name: name.into(),
            handler: Arc::new(move |ctx| Box::pin(handler(ctx))),
            aliases: Vec::new(),
            hidden: false,
            about: None,
            spec: CommandSpec::new(),
        }
    }

    pub(crate) fn alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }

    pub(crate) fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    pub(crate) fn about(mut self, about: impl Into<String>) -> Self {
        self.about = Some(about.into());
        self
    }

    pub(crate) fn spec(mut self, spec: CommandSpec) -> Self {
        self.spec = spec;
        self
    }
}

pub(crate) fn handler_name<T>() -> String {
    command_name_from_type_name(type_name::<T>())
}

pub(crate) fn command_name_from_type_name(raw: &str) -> String {
    if raw
        .rsplit("::")
        .next()
        .is_some_and(|part| part.contains("{{closure}}"))
    {
        return String::from("command");
    }

    let parts = raw
        .split("::")
        .filter_map(clean_type_path_part)
        .collect::<Vec<_>>();

    match parts.as_slice() {
        [] => String::from("command"),
        [.., module, name] if name == "run" && is_command_module(&parts) => module.clone(),
        [.., name] => name.clone(),
    }
}

fn is_command_module(parts: &[String]) -> bool {
    parts.len() >= 3
        && parts.last().is_some_and(|part| part == "run")
        && parts
            .get(parts.len() - 2)
            .is_some_and(|part| part != "commands")
        && parts[..parts.len() - 2]
            .iter()
            .any(|part| part == "commands")
}

fn clean_type_path_part(part: &str) -> Option<String> {
    if part.is_empty() || part.contains("{{closure}}") {
        return None;
    }

    let without_generics = part.split('<').next().unwrap_or(part);
    let cleaned = without_generics
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '-')
        .collect::<String>();

    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn hello(_ctx: Context) -> Result<()> {
        Ok(())
    }

    async fn run(_ctx: Context) -> Result<()> {
        Ok(())
    }

    mod commands {
        pub mod hello {
            use crate::{Context, Result};

            pub async fn run(_ctx: Context) -> Result<()> {
                Ok(())
            }
        }
    }

    #[test]
    fn derives_command_name_from_function_item() {
        let command = Command::from_handler(hello);
        assert_eq!(command.name, "hello");
    }

    #[test]
    fn derives_command_name_from_run_module() {
        let command = Command::from_handler(commands::hello::run);
        assert_eq!(command.name, "hello");
    }

    #[test]
    fn keeps_top_level_run_name() {
        let command = Command::from_handler(run);
        assert_eq!(command.name, "run");
    }

    #[test]
    fn parses_type_path_into_command_name() {
        assert_eq!(
            command_name_from_type_name("myapp::commands::hello::run"),
            "hello"
        );
        assert_eq!(command_name_from_type_name("myapp::config::run"), "run");
        assert_eq!(command_name_from_type_name("myapp::commands::scan"), "scan");
        assert_eq!(
            command_name_from_type_name("myapp::main::{{closure}}"),
            "command"
        );
    }
}
