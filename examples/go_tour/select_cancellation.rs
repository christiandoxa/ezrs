//! Go Tour mapping: select and cancellation.
//!
//! Rust uses `tokio::select!`; ezrs `Context` supplies cooperative cancellation.

use std::time::Duration;

use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command("select", select).run().await
}

async fn select(ctx: Context) -> Result<()> {
    let cancel_ctx = ctx.clone();
    ctx.spawn("cancel-soon", async move {
        tokio::time::sleep(Duration::from_millis(10)).await;
        cancel_ctx.cancel();
        Ok(())
    });

    tokio::select! {
        _ = ctx.cancelled() => ctx.println("cancelled"),
        _ = tokio::time::sleep(Duration::from_secs(1)) => ctx.println("timeout"),
    }

    if ctx.check_cancelled().is_err() {
        ctx.println("observed cancellation");
    }

    ctx.join_all().await
}
