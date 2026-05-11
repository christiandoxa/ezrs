//! Go pattern: worker pool with jobs channel.

use std::sync::Arc;

use ezrs::{App, Context, Result};
use tokio::sync::{Mutex, mpsc};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(pool).run().await
}

async fn pool(ctx: Context) -> Result<()> {
    let (tx, rx) = mpsc::channel::<u64>(8);
    let rx = Arc::new(Mutex::new(rx));

    for id in 0..2 {
        let rx = Arc::clone(&rx);
        let worker_ctx = ctx.clone();
        ctx.spawn(async move {
            loop {
                let job = rx.lock().await.recv().await;
                let Some(job) = job else { break };
                worker_ctx.println(format!("worker {id}: {job}"));
            }
            Ok(())
        });
    }

    for job in 0..4 {
        tx.send(job).await.map_err(|err| ezrs::Error::msg(err.to_string()))?;
    }
    drop(tx);
    ctx.join_all().await
}
