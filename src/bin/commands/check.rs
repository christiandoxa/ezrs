use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub fn run() -> Result<(), String> {
    for command in commands() {
        run_cargo(&command)?;
    }
    run_example_check()?;
    Ok(())
}

pub fn commands() -> Vec<Vec<&'static str>> {
    vec![
        vec!["fmt", "--all", "--check"],
        vec!["check", "--workspace", "--all-targets"],
        vec!["test", "--workspace", "--all-targets"],
        vec![
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
    ]
}

fn run_cargo(args: &[&str]) -> Result<(), String> {
    let output = Command::new("cargo")
        .args(args)
        .output()
        .map_err(|err| err.to_string())?;

    if output.status.success() {
        return Ok(());
    }

    let message = command_failure("cargo", args, &output);
    save_last_error(&message)?;
    Err(message)
}

fn run_example_check() -> Result<(), String> {
    let manifest = Path::new("target/example-check/Cargo.toml");
    write_example_manifest(manifest)?;

    let manifest_arg = manifest
        .to_str()
        .ok_or_else(|| String::from("example manifest path is not UTF-8"))?;
    run_cargo(&[
        "check",
        "--manifest-path",
        manifest_arg,
        "--bins",
        "--tests",
    ])
}

fn write_example_manifest(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    let mut text = String::from(
        r#"[package]
name = "ezrs-example-check"
version = "0.0.0"
edition = "2024"
publish = false

[workspace]

[dependencies]
ezrs = { path = "../.." }
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
"#,
    );

    for example in example_files()? {
        let name = example
            .with_extension("")
            .to_string_lossy()
            .replace(['/', '\\'], "_");
        text.push_str(&format!(
            r#"

[[bin]]
name = "{name}"
path = "../../{}"
"#,
            example.display()
        ));
    }

    fs::write(path, text).map_err(|err| err.to_string())
}

fn example_files() -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for dir in [
        "examples/components",
        "examples/golang_patterns",
        "examples/go_tour",
    ] {
        for entry in fs::read_dir(dir).map_err(|err| err.to_string())? {
            let entry = entry.map_err(|err| err.to_string())?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "rs") {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn command_failure(program: &str, args: &[&str], output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    format!(
        "{program} {} failed with status {}\n\nstdout:\n{stdout}\n\nstderr:\n{stderr}",
        args.join(" "),
        output.status
    )
}

fn save_last_error(message: &str) -> Result<(), String> {
    fs::create_dir_all(".ezrs").map_err(|err| err.to_string())?;
    fs::write(".ezrs/last-error.txt", message).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_command_construction() {
        assert_eq!(
            commands(),
            vec![
                vec!["fmt", "--all", "--check"],
                vec!["check", "--workspace", "--all-targets"],
                vec!["test", "--workspace", "--all-targets"],
                vec![
                    "clippy",
                    "--workspace",
                    "--all-targets",
                    "--",
                    "-D",
                    "warnings"
                ],
            ]
        );
    }
}
