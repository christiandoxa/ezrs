//! Go Tour mapping: range.
//!
//! Rust uses iterators with `for`, `iter`, `enumerate`, and ranges like `0..n`.

use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command("range", range).run().await
}

async fn range(ctx: Context) -> Result<()> {
    let names = ["go", "rust", "ezrs"];
    for (index, name) in names.iter().enumerate() {
        ctx.println(format!("{index}:{name}"));
    }
    Ok(())
}
