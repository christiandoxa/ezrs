//! Go pattern: flag package or cobra flags, accessed dynamically.

use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command("scan", scan).run().await
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
// cargo run -- scan --recursive --path src --name=demo input.txt
