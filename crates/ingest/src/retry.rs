//! Retry logic with exponential backoff and jitter

use std::time::Duration;

use tokio::time::sleep;

use crate::config::RetryConfig;

/// Retry a fallible async operation with exponential backoff
pub async fn retry_with_backoff<F, Fut, T, E>(
    config: &RetryConfig,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;
    let mut delay_ms = config.initial_delay_ms;

    loop {
        attempt += 1;

        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    tracing::info!(
                        attempt,
                        "Operation succeeded after retry"
                    );
                }
                return Ok(result);
            }
            Err(err) => {
                if attempt >= config.max_attempts {
                    tracing::error!(
                        attempt,
                        error = %err,
                        "Operation failed after max retries"
                    );
                    return Err(err);
                }

                let actual_delay = if config.jitter {
                    let jitter = (rand::random::<f64>() * 0.3) + 0.85; // ±15% jitter
                    (delay_ms as f64 * jitter) as u64
                } else {
                    delay_ms
                };

                tracing::warn!(
                    attempt,
                    delay_ms = actual_delay,
                    error = %err,
                    "Operation failed, retrying"
                );

                sleep(Duration::from_millis(actual_delay)).await;

                // Exponential backoff with cap
                delay_ms = (delay_ms * 2).min(config.max_delay_ms);
            }
        }
    }
}
