use std::{fs, path::Path};

pub fn run(last_error: bool) -> Result<(), String> {
    let input = if last_error {
        read_last_error().unwrap_or_else(|| String::from("No saved cargo check error found."))
    } else {
        String::new()
    };

    println!("{}", explain(&input));
    Ok(())
}

fn read_last_error() -> Option<String> {
    for path in [
        ".ezrs/last-error.txt",
        "target/ezrs-last-error.txt",
        "target/last-error.txt",
    ] {
        let path = Path::new(path);
        if path.exists()
            && let Ok(text) = fs::read_to_string(path)
        {
            return Some(text);
        }
    }
    None
}

pub fn explain(error: &str) -> String {
    let lower = error.to_ascii_lowercase();
    if lower.contains("does not live long enough") {
        return advice(
            "Your command captures a borrowed value that does not live long enough.",
            "Own the data before moving it into an async command or store it in State.",
            "let name = name.to_string();",
        );
    }
    if lower.contains("future cannot be sent") || lower.contains("send is not implemented") {
        return advice(
            "Your async command creates a future that is not Send.",
            "Use Send-safe types in spawned tasks and command futures. Avoid Rc and RefCell there.",
            "use std::sync::Arc;",
        );
    }
    if lower.contains("cannot move out") {
        return advice(
            "Code tries to move a value out of borrowed content.",
            "Clone the app state value or redesign the function to take ownership.",
            "let value = state.value.clone();",
        );
    }
    if lower.contains("use of moved value") {
        return advice(
            "A value was moved and then used again.",
            "Clone cheap handles like Context, Shared, or SharedMut before moving them into tasks.",
            "let ctx2 = ctx.clone();",
        );
    }
    if lower.contains("mismatched types") && lower.contains("result") {
        return advice(
            "The function returns a different Result error type than ezrs expects.",
            "Return ezrs::Result<()> from commands and use ? for supported errors.",
            "async fn run(ctx: ezrs::Context) -> ezrs::Result<()> { Ok(()) }",
        );
    }
    if lower.contains("unknown flag") {
        return advice(
            "A command received a flag that is not declared in its CommandSpec.",
            "Declare the flag with ArgSpec or allow passthrough args for wrapper commands.",
            "CommandSpec::new().arg(ArgSpec::flag(\"verbose\").short('v'))",
        );
    }
    if lower.contains("missing required argument") {
        return advice(
            "A command schema requires an argument that was not provided.",
            "Pass the argument, add a default, or bind an environment variable.",
            "ArgSpec::option(\"path\").required().env(\"APP_PATH\")",
        );
    }
    if lower.contains("required config source") || lower.contains("toml") {
        return advice(
            "Typed config could not be loaded from the configured source.",
            "Check the file path, TOML shape, env prefix, and validation function.",
            "App::new().config_from::<Config>(ConfigSource::ezrs().required())",
        );
    }
    if lower.contains("process") && lower.contains("cancelled") {
        return advice(
            "A child process was cancelled through the ezrs Context.",
            "Handle cancellation as a normal shutdown path, or use Process::new for low-level detached behavior.",
            "ctx.process(\"cargo\").arg(\"check\").run().await?",
        );
    }

    advice(
        "No specific ezrs explanation matched this error.",
        "Read the first Rust compiler error, then check lifetimes, Send bounds, moved values, and Result types.",
        "ezrs explain --last-error",
    )
}

fn advice(problem: &str, fix: &str, change: &str) -> String {
    format!("Problem:\n{problem}\n\nFix:\n{fix}\n\nSuggested change:\n{change}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explain_known_send_error() {
        let text = explain("future cannot be sent between threads safely");
        assert!(text.contains("not Send"));
    }
}
