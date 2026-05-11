//! Go pattern: time.NewTicker loop.

use std::time::Duration;

use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command("tick", tick).run().await
}

async fn tick(ctx: Context) -> Result<()> {
    let mut ticker = tokio::time::interval(Duration::from_millis(100));
    for _ in 0..3 {
        ticker.tick().await;
        ctx.println("tick");
    }
    Ok(())
}
