//! Shared application state

use std::sync::Arc;

use sqlx::PgPool;

use crate::{config::ApiConfig, metrics::Metrics};

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
