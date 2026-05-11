//! Go pattern: go worker(ctx) becomes ctx.spawn.

use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(work).run().await
}

async fn work(ctx: Context) -> Result<()> {
    let worker_ctx = ctx.clone();
    ctx.spawn(async move {
        worker(worker_ctx).await?;
        Ok(())
    });
    ctx.join_all().await
}

async fn worker(ctx: Context) -> Result<()> {
    ctx.println("worker finished");
    Ok(())
}
