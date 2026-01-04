//! Opportunities listing handler

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use pm_storage::recs;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::state::AppState;

/// Query parameters for opportunities endpoint
#[derive(Debug, Deserialize)]
pub struct OpportunitiesQuery {
    /// Minimum overall score filter
    pub min_score: Option<f64>,

    /// Maximum time remaining in seconds
    pub max_t_remaining_sec: Option<i64>,

    /// Maximum risk score
    pub max_risk_score: Option<f64>,

    /// Filter for markets with risk flags
    pub has_flags: Option<bool>,

    /// Page size (limited by config)
    pub limit: Option<usize>,

    /// Offset for pagination
    pub offset: Option<usize>,
}

/// Response for opportunities endpoint
#[derive(Debug, Serialize)]
pub struct OpportunitiesResponse {
    pub opportunities: Vec<OpportunityItem>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}

/// Single opportunity item
#[derive(Debug, Serialize)]
pub struct OpportunityItem {
    pub market_id: String,
    pub as_of: String,
    pub recommended_side: String,
    pub entry_price: f64,
    pub expected_payout: f64,
    pub max_position_pct: f64,
    pub risk_score: f64,
    pub risk_flags: Vec<Value>,
    pub notes: Option<String>,
}

/// List opportunities endpoint
///
/// Returns filtered and paginated list of recommendations
pub async fn opportunities_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<OpportunitiesQuery>,
) -> Result<Json<OpportunitiesResponse>, (StatusCode, String)> {
    // Apply config limits to pagination
    let limit = params
        .limit
        .unwrap_or(state.config.default_page_size)
        .min(state.config.max_page_size);
    let offset = params.offset.unwrap_or(0);

    // Fetch recommendations with filters
    let recs = recs::list_recs(
        &state.pool,
        params.min_score,
        params.max_t_remaining_sec,
        params.max_risk_score,
        params.has_flags,
        limit,
        offset,
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to fetch recommendations");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to fetch opportunities".to_string(),
        )
    })?;

    // Count total matching recommendations
    let total = recs::count_recs(
        &state.pool,
        params.min_score,
        params.max_t_remaining_sec,
        params.max_risk_score,
        params.has_flags,
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to count recommendations");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to count opportunities".to_string(),
        )
    })?;

    // Convert to response format
    let opportunities = recs
        .into_iter()
        .map(|rec| OpportunityItem {
            market_id: rec.market_id,
            as_of: rec.as_of.to_rfc3339(),
            recommended_side: rec.recommended_side,
            entry_price: rec.entry_price,
            expected_payout: rec.expected_payout,
            max_position_pct: rec.max_position_pct,
            risk_score: rec.risk_score,
            risk_flags: rec
                .risk_flags
                .into_iter()
                .map(|f| serde_json::to_value(f).unwrap_or(serde_json::Value::Null))
                .collect(),
            notes: rec.notes,
        })
        .collect();

    Ok(Json(OpportunitiesResponse {
        opportunities,
        total,
        limit,
        offset,
    }))
}
