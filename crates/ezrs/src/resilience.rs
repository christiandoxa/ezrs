//! Small retry, backoff, and timeout helpers.

use std::{future::Future, time::Duration};

use crate::{Error, Result};

/// Retry settings for fallible async operations.
#[derive(Clone, Debug)]
pub struct RetryPolicy {
    /// Total attempts, including the first call.
    pub max_attempts: usize,
    /// Delay before the second attempt.
    pub initial_delay: Duration,
    /// Maximum delay between attempts.
    pub max_delay: Duration,
    /// Multiplier applied after each failed attempt.
    pub multiplier: f64,
}

impl RetryPolicy {
    /// Creates a policy with exponential backoff.
    pub fn exponential(max_attempts: usize, initial_delay: Duration, max_delay: Duration) -> Self {
        Self {
            max_attempts: max_attempts.max(1),
            initial_delay,
            max_delay,
            multiplier: 2.0,
        }
    }

    /// Creates a policy with the same delay between attempts.
    pub fn fixed(max_attempts: usize, delay: Duration) -> Self {
        Self {
            max_attempts: max_attempts.max(1),
            initial_delay: delay,
            max_delay: delay,
            multiplier: 1.0,
        }
    }

    /// Returns the delay before the next attempt after `failed_attempt`.
    pub fn delay_after(&self, failed_attempt: usize) -> Duration {
        backoff_delay(
            self.initial_delay,
            self.multiplier,
            self.max_delay,
            failed_attempt,
        )
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::exponential(3, Duration::from_millis(100), Duration::from_secs(2))
    }
}

/// Computes a capped exponential backoff delay.
pub fn backoff_delay(
    initial_delay: Duration,
    multiplier: f64,
    max_delay: Duration,
    failed_attempt: usize,
) -> Duration {
    if failed_attempt <= 1 {
        return initial_delay.min(max_delay);
    }

    let multiplier = multiplier.max(1.0);
    let factor = multiplier.powi((failed_attempt - 1) as i32);
    let delay = initial_delay.mul_f64(factor);
    delay.min(max_delay)
}

/// Retries an async operation until it succeeds or attempts are exhausted.
///
/// The closure receives a one-based attempt number.
pub async fn retry<T, F, Fut>(policy: RetryPolicy, mut operation: F) -> Result<T>
where
    F: FnMut(usize) -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let max_attempts = policy.max_attempts.max(1);
    let mut attempt = 1;

    loop {
        match operation(attempt).await {
            Ok(value) => return Ok(value),
            Err(error) if attempt >= max_attempts => return Err(error),
            Err(_) => {
                tokio::time::sleep(policy.delay_after(attempt)).await;
                attempt += 1;
            }
        }
    }
}

/// Fails an async operation when it does not complete before `duration`.
pub async fn timeout<T, F>(duration: Duration, future: F) -> Result<T>
where
    F: Future<Output = Result<T>>,
{
    match tokio::time::timeout(duration, future).await {
        Ok(result) => result,
        Err(_) => Err(Error::timeout(format!(
            "operation exceeded {}ms",
            duration.as_millis()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use super::*;

    #[test]
    fn backoff_is_capped() {
        let delay = backoff_delay(
            Duration::from_millis(100),
            2.0,
            Duration::from_millis(250),
            4,
        );

        assert_eq!(delay, Duration::from_millis(250));
    }

    #[tokio::test]
    async fn retry_stops_after_success() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let seen = attempts.clone();

        let value = retry(RetryPolicy::fixed(3, Duration::ZERO), move |_| {
            let seen = seen.clone();
            async move {
                let attempt = seen.fetch_add(1, Ordering::SeqCst) + 1;
                if attempt < 2 {
                    Err(Error::msg("not yet"))
                } else {
                    Ok("ready")
                }
            }
        })
        .await
        .expect("retry");

        assert_eq!(value, "ready");
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn timeout_reports_timeout() {
        let result = timeout(Duration::from_millis(1), async {
            tokio::time::sleep(Duration::from_millis(20)).await;
            Ok(())
        })
        .await;

        assert!(matches!(result, Err(Error::Timeout(_))));
    }
}
