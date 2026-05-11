//! Go pattern: main calls run and returns errors through startup.

use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command("hello", hello).run().await
}

async fn hello(ctx: Context) -> Result<()> {
    ctx.println("no manual Tokio setup");
    Ok(())
}
