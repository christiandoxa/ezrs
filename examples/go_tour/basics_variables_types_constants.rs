//! Go Tour mapping: variables, zero values, basic types, constants.
//!
//! Rust variables are immutable by default and constants require explicit types.

use ezrs::{App, Context, Result};

const DEFAULT_WORKERS: usize = 4;

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command("types", types).run().await
}

async fn types(ctx: Context) -> Result<()> {
    let name = String::from("ezrs");
    let mut workers = DEFAULT_WORKERS;
    workers += 1;
    let enabled: bool = true;
    let ratio: f64 = 0.75;

    ctx.println(format!("{name} workers={workers} enabled={enabled} ratio={ratio}"));
    Ok(())
}
