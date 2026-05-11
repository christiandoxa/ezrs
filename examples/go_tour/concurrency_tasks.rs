//! Go Tour mapping: goroutines.
//!
//! ezrs maps app-level goroutine-style work to `ctx.spawn`.

use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(task).run().await
}

async fn task(ctx: Context) -> Result<()> {
    let worker_ctx = ctx.clone();
    ctx.spawn(async move {
        worker_ctx.println("task finished");
        Ok(())
    });
    ctx.join_all().await
}
