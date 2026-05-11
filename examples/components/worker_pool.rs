//! Component example: syntax-first worker pool.

use ezrs::{Result, WorkerPool};

#[ezrs::main]
async fn main() -> Result<()> {
    WorkerPool::new(process_job)
        .workers(4)
        .buffer(16)
        .run(1..=8)
        .await
}

async fn process_job(job: u64) -> Result<()> {
    println!("processed job {job}");
    Ok(())
}
