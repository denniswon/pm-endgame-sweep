//! HTTP server with route configuration

use axum::{
    routing::get,
    Router,
};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

use crate::{
    config::ApiConfig,
    handlers::{health_handler, market_handler, opportunities_handler},
    state::AppState,
};

/// API server
pub struct ApiServer {
    app: Router,
    config: ApiConfig,
}

impl ApiServer {
    /// Create new API server
    pub fn new(pool: PgPool, config: ApiConfig) -> Self {
        let state = AppState::new(pool, config.clone());

        let app = Router::new()
            // Health check
            .route("/health", get(health_handler))
            // API v1 routes
            .route("/v1/opportunities", get(opportunities_handler))
            .route("/v1/market/:market_id", get(market_handler))
            // Add trace layer for request logging
            .layer(TraceLayer::new_for_http())
            .with_state(state);

        Self { app, config }
    }

    /// Run the server
    pub async fn run(self) -> anyhow::Result<()> {
        let addr = format!("{}:{}", self.config.bind_addr, self.config.port);
        tracing::info!("Starting API server on {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;

        axum::serve(listener, self.app)
            .await
            .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

        Ok(())
    }
}
