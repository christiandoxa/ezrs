//! Go pattern: select over channel, cancellation, and timer.

use std::time::Duration;

use ezrs::{App, Context, Result};
use tokio::sync::mpsc;

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command("select", select_example).run().await
}

async fn select_example(ctx: Context) -> Result<()> {
    let (_tx, mut rx) = mpsc::channel::<String>(8);

    tokio::select! {
        Some(value) = rx.recv() => ctx.println(value),
        _ = ctx.cancelled() => ctx.println("cancelled"),
        _ = tokio::time::sleep(Duration::from_millis(10)) => ctx.println("timeout"),
    }

    Ok(())
}
