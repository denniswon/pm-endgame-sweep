//! Metrics endpoint handler

use axum::{extract::State, http::StatusCode};
use std::sync::Arc;

use crate::state::AppState;

/// Metrics endpoint
///
/// Returns Prometheus-formatted metrics
pub async fn metrics_handler(State(state): State<Arc<AppState>>) -> Result<String, (StatusCode, String)> {
    state.metrics.render().map_err(|e| {
        tracing::error!(error = %e, "Failed to render metrics");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to render metrics".to_string(),
        )
    })
}
