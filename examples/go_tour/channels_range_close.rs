//! Go Tour mapping: channels, buffered channels, range, and close.
//!
//! Rust uses Tokio `mpsc`; dropping all senders closes the channel.

use ezrs::{App, Context, Result};
use tokio::sync::mpsc;

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(channel).run().await
}

async fn channel(ctx: Context) -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<String>(4);

    ctx.spawn(async move {
        for item in ["alpha", "beta", "gamma"] {
            tx.send(item.to_owned())
                .await
                .map_err(|error| ezrs::Error::msg(format!("send failed: {error}")))?;
        }
        Ok(())
    });

    while let Some(item) = rx.recv().await {
        ctx.println(format!("received {item}"));
    }

    ctx.join_all().await
}
