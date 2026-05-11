//! Go pattern: main calls run and exits on error.

use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(run).run().await
}

async fn run(ctx: Context) -> Result<()> {
    ctx.println("application started");
    Ok(())
}
