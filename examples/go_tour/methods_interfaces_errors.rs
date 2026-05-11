//! Go Tour mapping: errors.
//!
//! Rust uses `Result<T, E>` and `?`; ezrs exposes `ezrs::Error`.

use ezrs::{App, Context, Error, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command("error", error).run().await
}

async fn error(ctx: Context) -> Result<()> {
    let name = ctx.arg_or("name", "");
    if name.is_empty() {
        return Err(Error::invalid_input("missing --name"));
    }
    ctx.println(name);
    Ok(())
}
