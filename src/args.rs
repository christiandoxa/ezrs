use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use crate::{Error, Result};

/// Command argument shape used for validation and help rendering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgKind {
    /// Boolean flag such as `--recursive` or `-r`.
    Flag,
    /// Named option with a value such as `--path src`.
    Option,
    /// Positional value such as `input.txt`.
    Positional,
}

/// One CLI argument or flag definition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArgSpec {
    name: String,
    kind: ArgKind,
    short: Option<char>,
    value_name: Option<String>,
    help: Option<String>,
    required: bool,
    default: Option<String>,
    env: Option<String>,
}

impl ArgSpec {
    /// Defines a boolean flag.
    pub fn flag(name: impl Into<String>) -> Self {
        Self::new(name, ArgKind::Flag)
    }

    /// Defines a named option that takes a value.
    pub fn option(name: impl Into<String>) -> Self {
        Self::new(name, ArgKind::Option)
    }

    /// Defines a positional argument.
    pub fn positional(name: impl Into<String>) -> Self {
        Self::new(name, ArgKind::Positional)
    }

    fn new(name: impl Into<String>, kind: ArgKind) -> Self {
        Self {
            name: name.into(),
            kind,
            short: None,
            value_name: None,
            help: None,
            required: false,
            default: None,
            env: None,
        }
    }

    /// Adds a short flag alias such as `-r`.
    pub fn short(mut self, short: char) -> Self {
        self.short = Some(short);
        self
    }

    /// Adds help text rendered in command help.
    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Marks this argument as required when no default or env value exists.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Adds a default value used when the argument is absent.
    pub fn default(mut self, value: impl Into<String>) -> Self {
        self.default = Some(value.into());
        self
    }

    /// Binds an environment variable used when the argument is absent.
    pub fn env(mut self, key: impl Into<String>) -> Self {
        self.env = Some(key.into());
        self
    }

    /// Sets the displayed value name for options and positionals.
    pub fn value_name(mut self, name: impl Into<String>) -> Self {
        self.value_name = Some(name.into());
        self
    }

    /// Returns the long argument name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the argument kind.
    pub const fn kind(&self) -> ArgKind {
        self.kind
    }

    /// Returns the short alias.
    pub const fn short_name(&self) -> Option<char> {
        self.short
    }
}

/// Per-command CLI schema.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommandSpec {
    args: Vec<ArgSpec>,
    allow_unknown_args: bool,
}

impl CommandSpec {
    /// Creates an empty command schema.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an argument or flag definition.
    pub fn arg(mut self, arg: ArgSpec) -> Self {
        self.args.push(arg);
        self
    }

    /// Adds a boolean flag.
    pub fn flag(self, name: impl Into<String>) -> Self {
        self.arg(ArgSpec::flag(name))
    }

    /// Adds a named option.
    pub fn option(self, name: impl Into<String>) -> Self {
        self.arg(ArgSpec::option(name))
    }

    /// Adds a positional argument.
    pub fn positional(self, name: impl Into<String>) -> Self {
        self.arg(ArgSpec::positional(name))
    }

    /// Allows unknown arguments for passthrough command wrappers.
    pub fn allow_unknown_args(mut self) -> Self {
        self.allow_unknown_args = true;
        self
    }

    pub(crate) fn args(&self) -> &[ArgSpec] {
        &self.args
    }

    pub(crate) const fn allows_unknown_args(&self) -> bool {
        self.allow_unknown_args
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.args.is_empty()
    }

    pub(crate) fn usage_suffix(&self) -> String {
        if self.args.is_empty() {
            return String::from("[ARGS]");
        }

        let mut parts = Vec::new();
        if self
            .args
            .iter()
            .any(|arg| matches!(arg.kind, ArgKind::Flag | ArgKind::Option))
        {
            parts.push(String::from("[OPTIONS]"));
        }
        for arg in self
            .args
            .iter()
            .filter(|arg| arg.kind == ArgKind::Positional)
        {
            let value = arg
                .value_name
                .as_deref()
                .unwrap_or(arg.name.as_str())
                .to_ascii_uppercase();
            if arg.required {
                parts.push(format!("<{value}>"));
            } else {
                parts.push(format!("[{value}]"));
            }
        }

        if self.allow_unknown_args {
            parts.push(String::from("[-- <ARGS>...]"));
        }

        parts.join(" ")
    }

    pub(crate) fn help_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();

        let positionals = self
            .args
            .iter()
            .filter(|arg| arg.kind == ArgKind::Positional)
            .collect::<Vec<_>>();
        if !positionals.is_empty() {
            lines.push(String::from("Arguments:"));
            for arg in positionals {
                lines.push(format_arg_help(arg));
            }
        }

        let options = self
            .args
            .iter()
            .filter(|arg| matches!(arg.kind, ArgKind::Flag | ArgKind::Option))
            .collect::<Vec<_>>();
        if !options.is_empty() {
            if !lines.is_empty() {
                lines.push(String::new());
            }
            lines.push(String::from("Options:"));
            for arg in options {
                lines.push(format_arg_help(arg));
            }
        }

        lines
    }

    pub(crate) fn validation_error(&self) -> Option<String> {
        let mut names = HashSet::new();
        let mut shorts = HashSet::new();

        for arg in &self.args {
            if arg.name.is_empty() {
                return Some(String::from("argument name cannot be empty"));
            }
            if !names.insert(arg.name.clone()) {
                return Some(format!("duplicate argument '{}'", arg.name));
            }
            if let Some(short) = arg.short
                && !shorts.insert(short)
            {
                return Some(format!("duplicate short flag '-{short}'"));
            }
        }

        None
    }
}

/// Dynamic command arguments exposed through Context.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Args {
    values: HashMap<String, String>,
    flags: HashSet<String>,
    positionals: Vec<String>,
    seen: HashSet<String>,
}

impl Args {
    /// Parses dynamic arguments without a command schema.
    pub fn parse(tokens: &[String]) -> Self {
        Self::parse_with_spec(tokens, &CommandSpec::new(), |_| None).unwrap_or_default()
    }

    pub(crate) fn parse_with_spec(
        tokens: &[String],
        spec: &CommandSpec,
        env: impl Fn(&str) -> Option<String>,
    ) -> Result<Self> {
        let mut args = Self::default();
        let mut positional_index = 0_usize;
        let mut index = 0_usize;
        let short = short_map(spec.args());
        let long = long_map(spec.args());
        let positionals = spec
            .args()
            .iter()
            .filter(|arg| arg.kind == ArgKind::Positional)
            .collect::<Vec<_>>();

        while index < tokens.len() {
            let token = &tokens[index];

            if token == "--" {
                index += 1;
                while index < tokens.len() {
                    args.insert_positional(positional_index, tokens[index].clone());
                    if let Some(arg) = positionals.get(positional_index) {
                        args.insert_value(arg.name.clone(), tokens[index].clone());
                    }
                    positional_index += 1;
                    index += 1;
                }
                break;
            }

            if let Some(raw) = token.strip_prefix("--") {
                if raw.is_empty() {
                    index += 1;
                    continue;
                }

                if let Some((key, value)) = raw.split_once('=') {
                    let key = canonical_long(key, &long, spec)?;
                    args.insert_value(key, value.to_string());
                    index += 1;
                    continue;
                }

                if let Some(name) = raw.strip_prefix("no-")
                    && let Some(arg) = long.get(name)
                    && arg.kind == ArgKind::Flag
                {
                    args.insert_value(arg.name.clone(), String::from("false"));
                    index += 1;
                    continue;
                }

                let Some(arg) = long.get(raw) else {
                    if spec.allows_unknown_args() || spec.is_empty() {
                        if index + 1 < tokens.len() && !tokens[index + 1].starts_with('-') {
                            args.insert_value(raw.to_string(), tokens[index + 1].clone());
                            index += 2;
                        } else {
                            args.insert_value(raw.to_string(), String::from("true"));
                            index += 1;
                        }
                        continue;
                    }

                    return Err(Error::invalid_input(format!("unknown flag '--{raw}'")));
                };

                match arg.kind {
                    ArgKind::Flag => {
                        args.insert_value(arg.name.clone(), String::from("true"));
                        index += 1;
                    }
                    ArgKind::Option => {
                        let value = tokens.get(index + 1).ok_or_else(|| {
                            Error::invalid_input(format!("flag '--{}' requires a value", arg.name))
                        })?;
                        args.insert_value(arg.name.clone(), value.clone());
                        index += 2;
                    }
                    ArgKind::Positional => {
                        return Err(Error::invalid_input(format!(
                            "positional argument '{}' cannot be used as a flag",
                            arg.name
                        )));
                    }
                }
                continue;
            }

            if token.starts_with('-') && token.len() > 1 {
                let consumed = parse_short_token(token, tokens, index, &short, spec, &mut args)?;
                index += consumed;
                continue;
            }

            args.insert_positional(positional_index, token.clone());
            if let Some(arg) = positionals.get(positional_index) {
                args.insert_value(arg.name.clone(), token.clone());
            }
            positional_index += 1;
            index += 1;
        }

        for (index, arg) in positionals.iter().enumerate() {
            if args.positional(index).is_some() {
                continue;
            }
            apply_default_env(&mut args, arg, &env)?;
        }

        for arg in spec
            .args()
            .iter()
            .filter(|arg| arg.kind != ArgKind::Positional)
        {
            apply_default_env(&mut args, arg, &env)?;
        }

        Ok(args)
    }

    fn insert_positional(&mut self, index: usize, value: String) {
        let key = index.to_string();
        self.insert_value(key, value.clone());
        self.positionals.push(value);
    }

    fn insert_value(&mut self, key: String, value: String) {
        if is_truthy(&value) {
            self.flags.insert(key.clone());
        } else if is_falsey(&value) {
            self.flags.remove(&key);
        }
        self.seen.insert(key.clone());
        self.values.insert(key, value);
    }

    /// Returns a named or positional argument by key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    /// Parses a named or positional argument into a typed value.
    pub fn parse_value<T>(&self, key: &str) -> Result<T>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        self.get(key)
            .ok_or_else(|| Error::not_found(format!("argument '{key}'")))?
            .parse::<T>()
            .map_err(|err| Error::invalid_input(format!("argument '{key}': {err}")))
    }

    /// Parses a named or positional argument, returning a default when missing.
    pub fn parse_or<T>(&self, key: &str, default: T) -> Result<T>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        match self.get(key) {
            Some(value) => value
                .parse::<T>()
                .map_err(|err| Error::invalid_input(format!("argument '{key}': {err}"))),
            None => Ok(default),
        }
    }

    /// Returns a named argument or default value.
    pub fn get_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.get(key).unwrap_or(default)
    }

    /// Returns true if the flag was present.
    pub fn flag(&self, key: &str) -> bool {
        self.flags.contains(key) || self.values.get(key).is_some_and(|value| is_truthy(value))
    }

    /// Returns true if a named or positional argument was explicitly set or defaulted.
    pub fn contains(&self, key: &str) -> bool {
        self.seen.contains(key) || self.values.contains_key(key)
    }

    /// Returns positional arguments in encounter order.
    pub fn positionals(&self) -> &[String] {
        &self.positionals
    }

    /// Returns a positional argument by numeric index.
    pub fn positional(&self, index: usize) -> Option<&str> {
        self.positionals.get(index).map(String::as_str)
    }

    /// Parses a positional argument by numeric index.
    pub fn parse_positional<T>(&self, index: usize) -> Result<T>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        self.positional(index)
            .ok_or_else(|| Error::not_found(format!("positional argument {index}")))?
            .parse::<T>()
            .map_err(|err| Error::invalid_input(format!("positional argument {index}: {err}")))
    }
}

fn short_map(args: &[ArgSpec]) -> HashMap<char, &ArgSpec> {
    args.iter()
        .filter_map(|arg| arg.short.map(|short| (short, arg)))
        .collect()
}

fn long_map(args: &[ArgSpec]) -> HashMap<&str, &ArgSpec> {
    args.iter().map(|arg| (arg.name.as_str(), arg)).collect()
}

fn canonical_long(raw: &str, long: &HashMap<&str, &ArgSpec>, spec: &CommandSpec) -> Result<String> {
    match long.get(raw) {
        Some(arg) => Ok(arg.name.clone()),
        None if spec.allows_unknown_args() || spec.is_empty() => Ok(raw.to_string()),
        None => Err(Error::invalid_input(format!("unknown flag '--{raw}'"))),
    }
}

fn parse_short_token(
    token: &str,
    tokens: &[String],
    index: usize,
    short: &HashMap<char, &ArgSpec>,
    spec: &CommandSpec,
    args: &mut Args,
) -> Result<usize> {
    let raw = token.trim_start_matches('-');
    let mut chars = raw.chars();
    let Some(first) = chars.next() else {
        return Ok(1);
    };

    if raw.len() == first.len_utf8() {
        let Some(arg) = short.get(&first) else {
            if spec.allows_unknown_args() || spec.is_empty() {
                args.insert_value(first.to_string(), String::from("true"));
                return Ok(1);
            }
            return Err(Error::invalid_input(format!("unknown flag '-{first}'")));
        };

        match arg.kind {
            ArgKind::Flag => {
                args.insert_value(arg.name.clone(), String::from("true"));
                Ok(1)
            }
            ArgKind::Option => {
                let value = tokens.get(index + 1).ok_or_else(|| {
                    Error::invalid_input(format!("flag '-{first}' requires a value"))
                })?;
                args.insert_value(arg.name.clone(), value.clone());
                Ok(2)
            }
            ArgKind::Positional => Err(Error::invalid_input(format!(
                "positional argument '{}' cannot be used as a flag",
                arg.name
            ))),
        }
    } else if let Some(value) = raw
        .strip_prefix(first)
        .and_then(|tail| tail.strip_prefix('='))
    {
        let Some(arg) = short.get(&first) else {
            return Err(Error::invalid_input(format!("unknown flag '-{first}'")));
        };
        if arg.kind != ArgKind::Option {
            return Err(Error::invalid_input(format!(
                "flag '-{first}' does not take a value"
            )));
        }
        args.insert_value(arg.name.clone(), value.to_string());
        Ok(1)
    } else if let Some(arg) = short.get(&first)
        && arg.kind == ArgKind::Option
    {
        let value = raw[first.len_utf8()..].to_string();
        args.insert_value(arg.name.clone(), value);
        Ok(1)
    } else {
        for ch in raw.chars() {
            let Some(arg) = short.get(&ch) else {
                if spec.allows_unknown_args() || spec.is_empty() {
                    args.insert_value(ch.to_string(), String::from("true"));
                    continue;
                }
                return Err(Error::invalid_input(format!("unknown flag '-{ch}'")));
            };
            if arg.kind != ArgKind::Flag {
                return Err(Error::invalid_input(format!(
                    "flag '-{ch}' requires a value and cannot be grouped"
                )));
            }
            args.insert_value(arg.name.clone(), String::from("true"));
        }
        Ok(1)
    }
}

fn apply_default_env(
    args: &mut Args,
    arg: &ArgSpec,
    env: &impl Fn(&str) -> Option<String>,
) -> Result<()> {
    if args.contains(&arg.name) {
        return Ok(());
    }

    if let Some(key) = &arg.env
        && let Some(value) = env(key)
    {
        args.insert_value(arg.name.clone(), value);
        return Ok(());
    }

    if let Some(default) = &arg.default {
        args.insert_value(arg.name.clone(), default.clone());
        return Ok(());
    }

    if arg.required {
        return Err(Error::invalid_input(format!(
            "missing required argument '{}'",
            arg.name
        )));
    }

    Ok(())
}

fn format_arg_help(arg: &ArgSpec) -> String {
    let mut name = String::from("  ");
    match arg.kind {
        ArgKind::Flag | ArgKind::Option => {
            if let Some(short) = arg.short {
                name.push('-');
                name.push(short);
                name.push_str(", ");
            } else {
                name.push_str("    ");
            }
            name.push_str("--");
            name.push_str(&arg.name);
            if arg.kind == ArgKind::Option {
                let value = arg
                    .value_name
                    .as_deref()
                    .unwrap_or(arg.name.as_str())
                    .to_ascii_uppercase();
                name.push(' ');
                name.push('<');
                name.push_str(&value);
                name.push('>');
            }
        }
        ArgKind::Positional => {
            let value = arg
                .value_name
                .as_deref()
                .unwrap_or(arg.name.as_str())
                .to_ascii_uppercase();
            name.push('<');
            name.push_str(&value);
            name.push('>');
        }
    }

    let mut meta = Vec::new();
    if arg.required {
        meta.push("required".to_string());
    }
    if let Some(default) = &arg.default {
        meta.push(format!("default: {default}"));
    }
    if let Some(env) = &arg.env {
        meta.push(format!("env: {env}"));
    }

    if let Some(help) = &arg.help {
        name.push_str(" - ");
        name.push_str(help);
    }
    if !meta.is_empty() {
        name.push_str(" [");
        name.push_str(&meta.join(", "));
        name.push(']');
    }
    name
}

fn is_truthy(value: &str) -> bool {
    matches!(value, "1" | "true" | "TRUE" | "True" | "yes" | "on")
}

fn is_falsey(value: &str) -> bool {
    matches!(value, "0" | "false" | "FALSE" | "False" | "no" | "off")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Args {
        Args::parse(
            &args
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>(),
        )
    }

    #[test]
    fn parses_boolean_flags() {
        let args = parse(&["--recursive"]);
        assert!(args.flag("recursive"));
        assert_eq!(args.get("recursive"), Some("true"));
    }

    #[test]
    fn parses_key_value_args() {
        let args = parse(&["--path", "src", "--name=Ayu"]);
        assert_eq!(args.get("path"), Some("src"));
        assert_eq!(args.get("name"), Some("Ayu"));
    }

    #[test]
    fn parses_positionals() {
        let args = parse(&["input.txt", "out.txt"]);
        assert_eq!(args.get("0"), Some("input.txt"));
        assert_eq!(args.get("1"), Some("out.txt"));
        assert_eq!(args.positionals(), &["input.txt", "out.txt"]);
        assert_eq!(args.positional(0), Some("input.txt"));
    }

    #[test]
    fn parses_typed_values() {
        let args = parse(&["--limit", "5", "42"]);

        assert_eq!(args.parse_value::<usize>("limit").expect("limit"), 5);
        assert_eq!(args.parse_or::<usize>("missing", 7).expect("default"), 7);
        assert_eq!(args.parse_positional::<u64>(0).expect("positional"), 42);
    }

    #[test]
    fn parses_schema_short_flags_defaults_env_and_positionals() {
        let spec = CommandSpec::new()
            .arg(
                ArgSpec::option("path")
                    .short('p')
                    .required()
                    .value_name("PATH"),
            )
            .arg(ArgSpec::flag("recursive").short('r'))
            .arg(ArgSpec::option("limit").default("10").env("SCAN_LIMIT"))
            .arg(ArgSpec::positional("input").required());

        let args = Args::parse_with_spec(
            &["-r", "-p", "src", "input.txt"]
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>(),
            &spec,
            |key| (key == "SCAN_LIMIT").then(|| "20".to_string()),
        )
        .expect("parse schema");

        assert!(args.flag("recursive"));
        assert_eq!(args.get("path"), Some("src"));
        assert_eq!(args.get("limit"), Some("20"));
        assert_eq!(args.get("input"), Some("input.txt"));
        assert_eq!(args.get("0"), Some("input.txt"));
    }

    #[test]
    fn schema_rejects_unknown_and_missing_required_args() {
        let spec = CommandSpec::new().arg(ArgSpec::option("path").required());

        let unknown = Args::parse_with_spec(
            &["--bad"]
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>(),
            &spec,
            |_| None,
        )
        .expect_err("unknown flag");
        assert!(unknown.to_string().contains("unknown flag '--bad'"));

        let missing = Args::parse_with_spec(&[], &spec, |_| None).expect_err("missing path");
        assert!(
            missing
                .to_string()
                .contains("missing required argument 'path'")
        );
    }

    #[test]
    fn schema_names_positionals_after_double_dash() {
        let spec = CommandSpec::new().arg(ArgSpec::positional("input").required());

        let args = Args::parse_with_spec(
            &["--", "--literal"]
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>(),
            &spec,
            |_| None,
        )
        .expect("parse positional");

        assert_eq!(args.get("input"), Some("--literal"));
        assert_eq!(args.get("0"), Some("--literal"));
    }
}
