//! Go Tour mapping: maps.
//!
//! Rust uses `HashMap<K, V>`.

use std::collections::HashMap;

use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(map).run().await
}

async fn map(ctx: Context) -> Result<()> {
    let mut counts = HashMap::new();
    counts.insert("go", 1);
    counts.insert("rust", 2);

    if let Some(value) = counts.get("rust") {
        ctx.println(format!("rust={value}"));
    }

    Ok(())
}
