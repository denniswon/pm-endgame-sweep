//! Shared application state

use sqlx::PgPool;
use std::sync::Arc;

use crate::config::ApiConfig;
use crate::metrics::Metrics;

/// Shared application state accessible to all handlers
#[derive(Clone)]
pub struct AppState {
    /// Database connection pool
    pub pool: PgPool,

    /// API configuration
    pub config: ApiConfig,

    /// Prometheus metrics
    pub metrics: Metrics,
}

impl AppState {
    /// Create new application state
    pub fn new(pool: PgPool, config: ApiConfig, metrics: Metrics) -> Arc<Self> {
        Arc::new(Self {
            pool,
            config,
            metrics,
        })
    }
}
