//! Go Tour mapping: packages, imports, exported names.
//!
//! Rust uses crates and modules. Public names use `pub`, not capitalization.

use ezrs::{App, Context, Result};

mod math_tools {
    pub fn add(left: i32, right: i32) -> i32 {
        left + right
    }
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(add).run().await
}

async fn add(ctx: Context) -> Result<()> {
    ctx.println(math_tools::add(20, 22));
    Ok(())
}
