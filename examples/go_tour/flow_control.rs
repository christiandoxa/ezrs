//! Go Tour mapping: for, while-style loops, if, switch, and loop values.
//!
//! Rust uses `for`, `while`, `loop`, `if`, and `match`.

use ezrs::{App, Context, Result};

fn classify(value: i32) -> &'static str {
    match value {
        i32::MIN..=-1 => "negative",
        0 => "zero",
        1..=9 => "small",
        _ => "large",
    }
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command("flow", flow).run().await
}

async fn flow(ctx: Context) -> Result<()> {
    let mut sum = 0;
    for n in 1..=3 {
        sum += n;
    }

    let mut doubled = 1;
    while doubled < 8 {
        doubled *= 2;
    }

    let first_large = loop {
        sum += 1;
        if sum > 10 {
            break sum;
        }
    };

    ctx.println(format!(
        "sum={sum} doubled={doubled} first_large={first_large} class={}",
        classify(first_large)
    ));
    Ok(())
}
