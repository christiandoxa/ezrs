//! Go pattern: small functions return errors explicitly.

use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(load).run().await
}

async fn load(ctx: Context) -> Result<()> {
    let path = ctx.arg("path")?;
    let text = std::fs::read_to_string(path)?;
    ctx.println(format!("bytes: {}", text.len()));
    Ok(())
}
