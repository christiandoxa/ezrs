//! Go Tour mapping: arrays and slices.
//!
//! Rust arrays have fixed length. Slices borrow contiguous ranges.

use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command("slice", slice).run().await
}

async fn slice(ctx: Context) -> Result<()> {
    let numbers = [1, 2, 3, 4];
    let window = &numbers[1..3];
    ctx.println(format!("len={} first={}", window.len(), window[0]));
    Ok(())
}
