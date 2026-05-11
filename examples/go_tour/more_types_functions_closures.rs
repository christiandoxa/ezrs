//! Go Tour mapping: function values and closures.
//!
//! Rust closures capture environment explicitly by borrow or `move`.

use ezrs::{App, Context, Result};

fn apply(value: i32, f: impl Fn(i32) -> i32) -> i32 {
    f(value)
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command("closure", closure).run().await
}

async fn closure(ctx: Context) -> Result<()> {
    let offset = 10;
    let result = apply(32, |value| value + offset);
    ctx.println(result);
    Ok(())
}
