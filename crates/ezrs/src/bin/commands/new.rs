use std::{fs, path::Path};

pub fn run(name: &str) -> Result<(), String> {
    run_in(Path::new("."), name)
}

fn run_in(base: &Path, name: &str) -> Result<(), String> {
    validate_name(name)?;

    let root = base.join(name);
    if root.exists() {
        return Err(format!("directory '{name}' already exists"));
    }

    fs::create_dir_all(root.join("src/commands")).map_err(|err| err.to_string())?;
    fs::write(root.join("Cargo.toml"), cargo_toml(name)).map_err(|err| err.to_string())?;
    fs::write(root.join("ezrs.toml"), "greeting = \"hello\"\n").map_err(|err| err.to_string())?;
    fs::write(root.join("src/main.rs"), main_rs(name)).map_err(|err| err.to_string())?;
    fs::write(root.join("src/config.rs"), config_rs()).map_err(|err| err.to_string())?;
    fs::write(root.join("src/state.rs"), state_rs()).map_err(|err| err.to_string())?;
    fs::write(root.join("src/commands/mod.rs"), "pub mod hello;\n")
        .map_err(|err| err.to_string())?;
    fs::write(root.join("src/commands/hello.rs"), hello_rs()).map_err(|err| err.to_string())?;

    println!("created ezrs app '{name}'");
    Ok(())
}

fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || !name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return Err("project name must contain only ASCII letters, numbers, '_' or '-'".into());
    }
    Ok(())
}

fn cargo_toml(name: &str) -> String {
    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"

[dependencies]
ezrs = "0.1.0"
serde = {{ version = "1", features = ["derive"] }}
"#
    )
}

fn main_rs(name: &str) -> String {
    r#"mod commands;
mod config;
mod state;

use ezrs::{App, Result};

use crate::{config::Config, state::State};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new()
        .name("__EZRS_NAME__")
        .version("0.1.0")
        .about("Generated ezrs application")
        .config::<Config>()
        .state(State::new("__EZRS_NAME__"))
        .command("hello", commands::hello::run)
        .run()
        .await
}
"#
    .replace("__EZRS_NAME__", name)
}

fn config_rs() -> &'static str {
    r#"#[derive(Clone, serde::Deserialize)]
pub struct Config {
    pub greeting: String,
}
"#
}

fn state_rs() -> &'static str {
    r#"#[derive(Clone)]
pub struct State {
    pub app_name: String,
}

impl State {
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
        }
    }
}
"#
}

fn hello_rs() -> &'static str {
    r#"use ezrs::{Context, Result};

use crate::{config::Config, state::State};

pub async fn run(ctx: Context) -> Result<()> {
    let name = ctx.arg_or("name", "world");
    let state = ctx.state::<State>()?;
    let config = ctx.config::<Config>()?;

    ctx.println(format!("{} {name} from {}", config.greeting, state.app_name));
    Ok(())
}
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_generates_project_files() {
        let dir = tempfile::tempdir().expect("temp dir");
        run_in(dir.path(), "demo").expect("new");

        assert!(dir.path().join("demo/Cargo.toml").exists());
        assert!(dir.path().join("demo/src/commands/hello.rs").exists());
    }
}
