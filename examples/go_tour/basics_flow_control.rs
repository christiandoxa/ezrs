//! Go Tour mapping: for, if, switch, defer.
//!
//! Rust uses `loop`, `while`, `for`, `if`, `match`, and RAII cleanup.

use ezrs::{App, Context, Result};

struct Cleanup;

impl Drop for Cleanup {
    fn drop(&mut self) {
        eprintln!("cleanup after command");
    }
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(flow).run().await
}

async fn flow(ctx: Context) -> Result<()> {
    let _cleanup = Cleanup;
    let mut sum = 0;
    for n in 1..=3 {
        sum += n;
    }

    let size = match sum {
        0 => "empty",
        1..=5 => "small",
        _ => "large",
    };

    if sum > 0 {
        ctx.println(format!("{sum} is {size}"));
    }

    Ok(())
}
