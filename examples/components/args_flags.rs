//! Go pattern: flag package or cobra flags with schema validation.

use ezrs::{App, ArgSpec, CommandSpec, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    let spec = CommandSpec::new()
        .arg(
            ArgSpec::option("path")
                .short('p')
                .default(".")
                .help("Path to scan"),
        )
        .arg(ArgSpec::flag("recursive").short('r').help("Scan recursively"))
        .arg(ArgSpec::option("name").default("default"))
        .arg(ArgSpec::positional("input").value_name("INPUT"));

    App::new().command_with(scan, spec).run().await
}

async fn scan(ctx: Context) -> Result<()> {
    let recursive = ctx.flag("recursive");
    let path = ctx.arg_or("path", ".");
    let name = ctx.arg_or("name", "default");
    let first_positional = ctx.arg_or("0", "none");

    ctx.println(format!("recursive={recursive}"));
    ctx.println(format!("path={path}"));
    ctx.println(format!("name={name}"));
    ctx.println(format!("first positional={first_positional}"));
    Ok(())
}

// Try:
// cargo run -- scan -r --path src --name=demo input.txt
