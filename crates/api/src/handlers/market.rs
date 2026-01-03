//! Market details handler

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;

use pm_storage::{markets, quotes, recs, rules, scores};

use crate::state::AppState;

/// Market details response
#[derive(Debug, Serialize)]
pub struct MarketDetailsResponse {
    pub market: MarketInfo,
    pub quote: Option<QuoteInfo>,
    pub rule: Option<RuleInfo>,
    pub score: Option<ScoreInfo>,
    pub recommendation: Option<RecommendationInfo>,
}

#[derive(Debug, Serialize)]
pub struct MarketInfo {
    pub market_id: String,
    pub venue: String,
    pub title: String,
    pub slug: Option<String>,
    pub category: Option<String>,
    pub status: String,
    pub open_time: Option<String>,
    pub close_time: Option<String>,
    pub resolved_time: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct QuoteInfo {
    pub as_of: String,
    pub yes_bid: Option<f64>,
    pub yes_ask: Option<f64>,
    pub no_bid: Option<f64>,
    pub no_ask: Option<f64>,
    pub spread_yes: Option<f64>,
    pub spread_no: Option<f64>,
    pub mid_yes: Option<f64>,
    pub mid_no: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct RuleInfo {
    pub as_of: String,
    pub rule_text: String,
    pub rule_hash: String,
    pub settlement_source: Option<String>,
    pub settlement_window: Option<String>,
    pub definition_risk_score: f64,
    pub risk_flags: Vec<Value>,
}

#[derive(Debug, Serialize)]
pub struct ScoreInfo {
    pub as_of: String,
    pub t_remaining_sec: i64,
    pub gross_yield: f64,
    pub net_yield: f64,
    pub yield_velocity: f64,
    pub liquidity_score: f64,
    pub staleness_sec: i64,
    pub staleness_penalty: f64,
    pub definition_risk_score: f64,
    pub overall_score: f64,
    pub score_breakdown: Value,
}

#[derive(Debug, Serialize)]
pub struct RecommendationInfo {
    pub as_of: String,
    pub recommended_side: String,
    pub entry_price: f64,
    pub expected_payout: f64,
    pub max_position_pct: f64,
    pub risk_score: f64,
    pub risk_flags: Vec<Value>,
    pub notes: Option<String>,
}

/// Get market details endpoint
///
/// Returns comprehensive information about a specific market
pub async fn market_handler(
    State(state): State<Arc<AppState>>,
    Path(market_id): Path<String>,
) -> Result<Json<MarketDetailsResponse>, (StatusCode, String)> {
    // Fetch market
    let market = markets::get_market(&state.pool, &market_id)
        .await
        .map_err(|e| match e {
            pm_storage::markets::MarketError::NotFound(_) => {
                (StatusCode::NOT_FOUND, "Market not found".to_string())
            }
            _ => {
                tracing::error!(error = %e, market_id, "Failed to fetch market");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to fetch market".to_string(),
                )
            }
        })?;

    // Fetch quote (optional)
    let quote = quotes::get_quote_latest(&state.pool, &market_id)
        .await
        .ok()
        .map(|q| QuoteInfo {
            as_of: q.as_of.to_rfc3339(),
            yes_bid: q.yes_bid,
            yes_ask: q.yes_ask,
            no_bid: q.no_bid,
            no_ask: q.no_ask,
            spread_yes: q.spread_yes,
            spread_no: q.spread_no,
            mid_yes: q.mid_yes,
            mid_no: q.mid_no,
        });

    // Fetch rule (optional)
    let rule = rules::get_rule(&state.pool, &market_id)
        .await
        .ok()
        .map(|r| RuleInfo {
            as_of: r.as_of.to_rfc3339(),
            rule_text: r.rule_text,
            rule_hash: r.rule_hash,
            settlement_source: r.settlement_source,
            settlement_window: r.settlement_window,
            definition_risk_score: r.definition_risk_score,
            risk_flags: r
                .risk_flags
                .into_iter()
                .map(|f| serde_json::to_value(f).unwrap_or(serde_json::Value::Null))
                .collect(),
        });

    // Fetch score (optional)
    let score = scores::get_score(&state.pool, &market_id)
        .await
        .ok()
        .map(|s| ScoreInfo {
            as_of: s.as_of.to_rfc3339(),
            t_remaining_sec: s.t_remaining_sec,
            gross_yield: s.gross_yield,
            net_yield: s.net_yield,
            yield_velocity: s.yield_velocity,
            liquidity_score: s.liquidity_score,
            staleness_sec: s.staleness_sec,
            staleness_penalty: s.staleness_penalty,
            definition_risk_score: s.definition_risk_score,
            overall_score: s.overall_score,
            score_breakdown: s.score_breakdown,
        });

    // Fetch recommendation (optional)
    let recommendation = recs::get_rec(&state.pool, &market_id)
        .await
        .ok()
        .map(|r| RecommendationInfo {
            as_of: r.as_of.to_rfc3339(),
            recommended_side: r.recommended_side,
            entry_price: r.entry_price,
            expected_payout: r.expected_payout,
            max_position_pct: r.max_position_pct,
            risk_score: r.risk_score,
            risk_flags: r
                .risk_flags
                .into_iter()
                .map(|f| serde_json::to_value(f).unwrap_or(serde_json::Value::Null))
                .collect(),
            notes: r.notes,
        });

    let market_info = MarketInfo {
        market_id: market.market_id,
        venue: market.venue,
        title: market.title,
        slug: market.slug,
        category: market.category,
        status: format!("{:?}", market.status),
        open_time: market.open_time.map(|t| t.to_rfc3339()),
        close_time: market.close_time.map(|t| t.to_rfc3339()),
        resolved_time: market.resolved_time.map(|t| t.to_rfc3339()),
        url: market.url,
    };

    Ok(Json(MarketDetailsResponse {
        market: market_info,
        quote,
        rule,
        score,
        recommendation,
    }))
}
