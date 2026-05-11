//! Go pattern: error-returning app startup plus command registration.

use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new()
        .name("demo")
        .version("0.1.0")
        .about("Small ezrs app")
        .command("hello", hello)
        .run()
        .await
}

async fn hello(ctx: Context) -> Result<()> {
    ctx.println("hello from ezrs");
    Ok(())
}
