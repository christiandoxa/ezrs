//! Go Tour mapping: variables, zero values, basic types, conversions, inference, constants.

use ezrs::{App, Context, Result};

const DEFAULT_LIMIT: usize = 3;
const MAX_SCORE: f64 = 100.0;

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(types).run().await
}

async fn types(ctx: Context) -> Result<()> {
    let name = ctx.arg_or("name", "gopher");
    let mut count: usize = 0;
    let enabled = ctx.flag("enabled");
    let score = 42_i32;
    let normalized = f64::from(score) / MAX_SCORE;

    while count < DEFAULT_LIMIT {
        count += 1;
    }

    ctx.println(format!(
        "name={name} enabled={enabled} count={count} normalized={normalized}"
    ));
    Ok(())
}
