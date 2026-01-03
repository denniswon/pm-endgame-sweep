//! Ingestion service configuration

use serde::{Deserialize, Serialize};

/// Configuration for ingestion cadences and resource bounds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestConfig {
    /// How often to poll for new quotes (seconds)
    pub quotes_cadence_sec: u64,

    /// How often to discover new markets (seconds)
    pub discovery_cadence_sec: u64,

    /// How often to refresh rule text (seconds)
    pub rules_refresh_cadence_sec: u64,

    /// Maximum markets to discover per batch
    pub max_markets_per_discovery: usize,

    /// Maximum quotes to fetch per batch
    pub max_quotes_per_fetch: usize,

    /// Maximum channel size for bounded queues
    pub max_channel_size: usize,

    /// Retry configuration
    pub retry: RetryConfig,
}

/// Retry configuration for HTTP requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,

    /// Initial delay between retries (milliseconds)
    pub initial_delay_ms: u64,

    /// Maximum delay between retries (milliseconds)
    pub max_delay_ms: u64,

    /// Whether to add jitter to delays
    pub jitter: bool,
}

impl Default for IngestConfig {
    fn default() -> Self {
        Self {
            quotes_cadence_sec: 60,
            discovery_cadence_sec: 1800,
            rules_refresh_cadence_sec: 3600,
            max_markets_per_discovery: 1000,
            max_quotes_per_fetch: 100,
            max_channel_size: 10000,
            retry: RetryConfig::default(),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            jitter: true,
        }
    }
}
