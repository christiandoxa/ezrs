mod commands;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "ezrs",
    version,
    about = "Go-style application patterns, Rust-grade safety."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Create a new ezrs application.
    New { name: String },
    /// Add a component to an ezrs application.
    Add {
        #[command(subcommand)]
        command: AddCommand,
    },
    /// Run the current application with cargo run.
    Run {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Run fmt, check, and test.
    Check,
    /// Explain common Rust errors for Go developers.
    Explain {
        /// Explain the last saved cargo check error when available.
        #[arg(long)]
        last_error: bool,
    },
}

#[derive(Debug, Subcommand)]
enum AddCommand {
    /// Add a command module.
    Command { name: String },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::New { name } => commands::new::run(&name),
        Command::Add {
            command: AddCommand::Command { name },
        } => commands::add::run_command(&name),
        Command::Run { args } => commands::run::run(&args),
        Command::Check => commands::check::run(),
        Command::Explain { last_error } => commands::explain::run(last_error),
    };

    if let Err(error) = result {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
