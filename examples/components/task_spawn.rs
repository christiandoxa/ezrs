//! Go pattern: goroutine plus WaitGroup-style join.

use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(work).run().await
}

async fn work(ctx: Context) -> Result<()> {
    let worker_ctx = ctx.clone();
    ctx.spawn(async move {
        worker_ctx.println("worker ran");
        Ok(())
    });

    ctx.join_all().await?;
    ctx.println("all workers done");
    Ok(())
}
