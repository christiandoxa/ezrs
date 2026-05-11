//! Go Tour mapping: channels and buffered channels.
//!
//! Rust async apps commonly use `tokio::sync::mpsc`.

use ezrs::{App, Context, Result};
use tokio::sync::mpsc;

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(channel).run().await
}

async fn channel(ctx: Context) -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<String>(4);
    tx.send(String::from("hello"))
        .await
        .map_err(|err| ezrs::Error::msg(err.to_string()))?;
    drop(tx);

    while let Some(value) = rx.recv().await {
        ctx.println(value);
    }

    Ok(())
}
