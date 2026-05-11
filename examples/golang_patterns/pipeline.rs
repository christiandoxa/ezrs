//! Go pattern: generator -> worker -> sink pipeline.

use ezrs::{App, Context, Result};
use tokio::sync::mpsc;

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(pipeline).run().await
}

async fn pipeline(ctx: Context) -> Result<()> {
    let (jobs_tx, mut jobs_rx) = mpsc::channel::<u64>(8);
    let (out_tx, mut out_rx) = mpsc::channel::<u64>(8);

    ctx.spawn(async move {
        while let Some(job) = jobs_rx.recv().await {
            out_tx.send(job * 2).await.map_err(|err| ezrs::Error::msg(err.to_string()))?;
        }
        Ok(())
    });

    for job in 1..=3 {
        jobs_tx.send(job).await.map_err(|err| ezrs::Error::msg(err.to_string()))?;
    }
    drop(jobs_tx);

    while let Some(value) = out_rx.recv().await {
        ctx.println(value);
    }

    ctx.join_all().await
}
