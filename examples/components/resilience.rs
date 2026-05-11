//! Go pattern: retry with backoff plus operation timeout.

use std::time::Duration;

use ezrs::{Result, RetryPolicy, retry, timeout};

#[ezrs::main]
async fn main() -> Result<()> {
    let policy = RetryPolicy::exponential(3, Duration::from_millis(50), Duration::from_secs(1));

    let value = timeout(Duration::from_secs(2), async {
        retry(policy, |attempt| async move {
            println!("attempt {attempt}");
            Ok("ready")
        })
        .await
    })
    .await?;

    println!("{value}");
    Ok(())
}
