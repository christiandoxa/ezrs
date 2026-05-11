//! Go pattern: channels use tokio::sync::mpsc.

use ezrs::{App, Context, Result};
use tokio::sync::mpsc;

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command("channel", channel).run().await
}

async fn channel(ctx: Context) -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<String>(8);
    tx.send(String::from("hello")).await.map_err(|err| ezrs::Error::msg(err.to_string()))?;
    drop(tx);

    while let Some(message) = rx.recv().await {
        ctx.println(message);
    }
    Ok(())
}
