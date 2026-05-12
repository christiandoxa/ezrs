//! Go pattern: retry with backoff plus operation timeout.

use std::time::Duration;

use ezrs::{Cancellation, Result, RetryPolicy, retry_with_cancellation, timeout};

#[ezrs::main]
async fn main() -> Result<()> {
    let policy = RetryPolicy::exponential(3, Duration::from_millis(50), Duration::from_secs(1));
    let cancellation = Cancellation::new();

    let value = timeout(Duration::from_secs(2), async {
        retry_with_cancellation(&cancellation, policy, |attempt| async move {
            println!("attempt {attempt}");
            Ok("ready")
        })
        .await
    })
    .await?;

    println!("{value}");
    Ok(())
}
