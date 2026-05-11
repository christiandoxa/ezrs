//! Go Tour mapping: empty interface, type assertions, and type switches.
//!
//! Rust uses trait objects for behavior and `Any` only when runtime type checks are needed.

use std::any::Any;

use ezrs::{App, Context, Result};

fn describe(value: &dyn Any) -> String {
    if let Some(text) = value.downcast_ref::<String>() {
        format!("string:{text}")
    } else if let Some(number) = value.downcast_ref::<i32>() {
        format!("i32:{number}")
    } else {
        String::from("unknown")
    }
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command("types", types).run().await
}

async fn types(ctx: Context) -> Result<()> {
    let text = String::from("hello");
    let number = 42_i32;

    ctx.println(describe(&text));
    ctx.println(describe(&number));
    Ok(())
}
