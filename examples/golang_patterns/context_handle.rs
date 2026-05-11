//! Go pattern: pass context.Context through handlers.

use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(handle).run().await
}

async fn handle(ctx: Context) -> Result<()> {
    do_work(ctx.clone()).await?;
    ctx.println("done");
    Ok(())
}

async fn do_work(ctx: Context) -> Result<()> {
    ctx.check_cancelled()?;
    ctx.log().info("working");
    Ok(())
}
