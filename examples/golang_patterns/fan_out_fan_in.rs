//! Go pattern: fan out work, fan in results.

use std::sync::Arc;

use ezrs::{App, Context, Result};
use tokio::sync::{Mutex, mpsc};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command("fan", fan).run().await
}

async fn fan(ctx: Context) -> Result<()> {
    let (jobs_tx, jobs_rx) = mpsc::channel::<u64>(8);
    let (results_tx, mut results_rx) = mpsc::channel::<u64>(8);
    let jobs_rx = Arc::new(Mutex::new(jobs_rx));

    for _ in 0..2 {
        let jobs_rx = Arc::clone(&jobs_rx);
        let results_tx = results_tx.clone();
        ctx.spawn("fan-worker", async move {
            loop {
                let job = jobs_rx.lock().await.recv().await;
                let Some(job) = job else { break };
                results_tx.send(job * job).await.map_err(|err| ezrs::Error::msg(err.to_string()))?;
            }
            Ok(())
        });
    }
    drop(results_tx);

    for job in 1..=3 {
        jobs_tx.send(job).await.map_err(|err| ezrs::Error::msg(err.to_string()))?;
    }
    drop(jobs_tx);

    while let Some(result) = results_rx.recv().await {
        ctx.println(result);
    }
    ctx.join_all().await
}
