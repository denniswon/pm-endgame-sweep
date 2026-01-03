//! Health check handler

use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::state::AppState;

/// Health check endpoint
///
/// Returns service status and database connectivity
pub async fn health_handler(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    // Check database connectivity
    let db_healthy = sqlx::query("SELECT 1")
        .fetch_one(&state.pool)
        .await
        .is_ok();

    let status = if db_healthy { "healthy" } else { "unhealthy" };
    let status_code = if db_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(json!({
            "status": status,
            "database": db_healthy,
        })),
    )
}
