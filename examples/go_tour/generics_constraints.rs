//! Go Tour mapping: generic constraints.
//!
//! Rust uses trait bounds as constraints.

use std::fmt::Display;

use ezrs::{App, Context, Result};

fn join_display<T: Display>(left: T, right: T) -> String {
    format!("{left},{right}")
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command("constraint", constraint).run().await
}

async fn constraint(ctx: Context) -> Result<()> {
    ctx.println(join_display(1, 2));
    Ok(())
}
