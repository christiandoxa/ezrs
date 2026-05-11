//! Go Tour mapping: function values and closures.
//!
//! Rust closures capture by borrow or by move. Functions and closures can be passed around.

use ezrs::{App, Context, Result};

fn apply(value: i32, f: impl Fn(i32) -> i32) -> i32 {
    f(value)
}

fn counter_from(start: i32) -> impl FnMut() -> i32 {
    let mut current = start;
    move || {
        current += 1;
        current
    }
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(closures).run().await
}

async fn closures(ctx: Context) -> Result<()> {
    let factor = 3;
    let scaled = apply(4, |n| n * factor);

    let mut next = counter_from(scaled);
    ctx.println(format!("next={} then={}", next(), next()));
    Ok(())
}
