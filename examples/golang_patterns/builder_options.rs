//! Go pattern: options pattern translated to a Rust builder.

use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new()
        .name("builder-demo")
        .version("0.1.0")
        .about("Builder options example")
        .command(hello)
        .run()
        .await
}

async fn hello(ctx: Context) -> Result<()> {
    ctx.println("builder configured app");
    Ok(())
}
