//! Typed config loading from TOML files plus env overlays.

use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use toml::{Table, Value};

use crate::{Error, Result};

/// Config source settings for layered application config.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigSource {
    files: Vec<PathBuf>,
    env_prefix: Option<String>,
    required: bool,
}

impl ConfigSource {
    /// Creates an empty config source.
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            env_prefix: None,
            required: false,
        }
    }

    /// Creates the default `ezrs.toml` config source.
    pub fn ezrs() -> Self {
        Self::new().file("ezrs.toml")
    }

    /// Adds a TOML file layer. Later files override earlier files.
    pub fn file(mut self, path: impl Into<PathBuf>) -> Self {
        self.files.push(path.into());
        self
    }

    /// Adds an env prefix. `APP_WORKERS=4` maps to `workers`.
    ///
    /// Use double underscore for nesting: `APP_DATABASE__URL=...`.
    pub fn env_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.env_prefix = Some(prefix.into());
        self
    }

    /// Requires at least one file or env value to be present.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

impl Default for ConfigSource {
    fn default() -> Self {
        Self::ezrs()
    }
}

/// Loads .env into the process environment when present.
pub fn load_env() {
    let _ = dotenvy::dotenv();
}

/// Loads typed config from ezrs.toml in the current directory if it exists.
pub fn load_optional<T>() -> Result<Option<T>>
where
    T: DeserializeOwned,
{
    load_from_source(ConfigSource::ezrs())
}

/// Loads typed config from a specific TOML path if it exists.
pub fn load_optional_from_path<T>(path: impl AsRef<Path>) -> Result<Option<T>>
where
    T: DeserializeOwned,
{
    load_from_source(ConfigSource::new().file(path.as_ref()))
}

/// Loads typed config from ordered TOML files and an optional env overlay.
pub fn load_from_source<T>(source: ConfigSource) -> Result<Option<T>>
where
    T: DeserializeOwned,
{
    load_from_source_with_env(source, |key| std::env::var(key).ok())
}

/// Loads typed config and validates the decoded value.
pub fn load_validated<T>(
    source: ConfigSource,
    validate: impl Fn(&T) -> Result<()>,
) -> Result<Option<T>>
where
    T: DeserializeOwned,
{
    let Some(value) = load_from_source(source)? else {
        return Ok(None);
    };
    validate(&value)?;
    Ok(Some(value))
}

pub(crate) fn load_from_source_with_env<T>(
    source: ConfigSource,
    env: impl Fn(&str) -> Option<String>,
) -> Result<Option<T>>
where
    T: DeserializeOwned,
{
    let mut merged = Table::new();
    let mut found = false;

    for path in &source.files {
        if !path.exists() {
            continue;
        }
        found = true;
        let text = std::fs::read_to_string(path)?;
        let table = text.parse::<Table>().map_err(Error::from)?;
        merge_table(&mut merged, table);
    }

    if let Some(prefix) = &source.env_prefix {
        let normalized = prefix.trim_end_matches('_').to_ascii_uppercase();
        let prefix_with_sep = format!("{normalized}_");
        for (key, value) in std::env::vars() {
            if !key.starts_with(&prefix_with_sep) {
                continue;
            }
            found = true;
            let config_key = key[prefix_with_sep.len()..].to_ascii_lowercase();
            insert_env_value(&mut merged, &config_key, env_value(&value));
        }
        if let Some(value) = env(&normalized) {
            found = true;
            insert_env_value(&mut merged, "value", env_value(&value));
        }
    }

    if !found {
        if source.required {
            return Err(Error::not_found("required config source"));
        }
        return Ok(None);
    }

    let value = Value::Table(merged);
    Ok(Some(value.try_into()?))
}

fn merge_table(target: &mut Table, source: Table) {
    for (key, value) in source {
        match (target.get_mut(&key), value) {
            (Some(Value::Table(target)), Value::Table(source)) => merge_table(target, source),
            (_, value) => {
                target.insert(key, value);
            }
        }
    }
}

fn insert_env_value(table: &mut Table, key: &str, value: Value) {
    let mut parts = key.split("__").filter(|part| !part.is_empty()).peekable();
    let mut current = table;

    while let Some(part) = parts.next() {
        if parts.peek().is_none() {
            current.insert(part.to_string(), value);
            return;
        }

        let entry = current
            .entry(part.to_string())
            .or_insert_with(|| Value::Table(Table::new()));
        if !entry.is_table() {
            *entry = Value::Table(Table::new());
        }
        current = entry.as_table_mut().expect("env table");
    }
}

fn env_value(value: &str) -> Value {
    if let Ok(value) = value.parse::<bool>() {
        return Value::Boolean(value);
    }
    if let Ok(value) = value.parse::<i64>() {
        return Value::Integer(value);
    }
    if let Ok(value) = value.parse::<f64>() {
        return Value::Float(value);
    }
    Value::String(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, serde::Deserialize, PartialEq)]
    struct Config {
        name: String,
        workers: usize,
    }

    #[test]
    fn loads_optional_toml_config() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("ezrs.toml");
        std::fs::write(&path, "name = 'demo'\nworkers = 2\n").expect("write config");

        let cfg: Config = load_optional_from_path(path)
            .expect("load")
            .expect("config present");

        assert_eq!(
            cfg,
            Config {
                name: String::from("demo"),
                workers: 2
            }
        );
    }

    #[test]
    fn layered_files_override_earlier_values_and_validate() {
        let dir = tempfile::tempdir().expect("temp dir");
        let base = dir.path().join("base.toml");
        let local = dir.path().join("local.toml");
        std::fs::write(&base, "name = 'demo'\nworkers = 2\n").expect("write base config");
        std::fs::write(&local, "workers = 4\n").expect("write local config");

        let cfg: Config = load_validated(
            ConfigSource::new().file(base).file(local).required(),
            |config: &Config| {
                if config.workers == 0 {
                    Err(crate::Error::invalid_input("workers must be > 0"))
                } else {
                    Ok(())
                }
            },
        )
        .expect("load")
        .expect("config present");

        assert_eq!(
            cfg,
            Config {
                name: String::from("demo"),
                workers: 4
            }
        );
    }
}
