//! Go Tour mapping: functions, multiple results, named returns.
//!
//! Rust returns tuples for multiple values and uses `Result` for fallible code.

use ezrs::{App, Context, Result};

fn split_pair(value: &str) -> (&str, &str) {
    value.split_once('=').unwrap_or((value, ""))
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command("split", split).run().await
}

async fn split(ctx: Context) -> Result<()> {
    let input = ctx.arg_or("input", "key=value");
    let (key, value) = split_pair(&input);
    ctx.println(format!("{key}:{value}"));
    Ok(())
}
