//! Go Tour mapping: maps, map literals, insert, lookup, delete, and ok checks.
//!
//! Rust uses `HashMap<K, V>` and `Option<&V>` for lookup.

use std::collections::HashMap;

use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command("scores", scores).run().await
}

async fn scores(ctx: Context) -> Result<()> {
    let mut scores = HashMap::from([(String::from("Ada"), 10), (String::from("Grace"), 20)]);

    scores.insert(String::from("Ayu"), 30);
    scores.remove("Grace");

    let name = ctx.arg_or("name", "Ayu");
    match scores.get(&name) {
        Some(score) => ctx.println(format!("{name}={score}")),
        None => ctx.println(format!("{name}=missing")),
    }

    Ok(())
}
