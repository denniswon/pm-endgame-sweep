//! Score-related domain types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::risk::RiskFlag;

/// Score snapshot with computed features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    pub market_id: String,
    pub as_of: DateTime<Utc>,
    pub t_remaining_sec: i64,
    pub gross_yield: f64,
    pub fee_bps: f64,
    pub net_yield: f64,
    pub yield_velocity: f64,
    pub liquidity_score: f64,
    pub staleness_sec: i64,
    pub staleness_penalty: f64,
    pub definition_risk_score: f64,
    pub overall_score: f64,
    pub score_breakdown: serde_json::Value,
}

/// Recommendation for a market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub market_id: String,
    pub as_of: DateTime<Utc>,
    pub recommended_side: String,
    pub entry_price: f64,
    pub expected_payout: f64,
    pub max_position_pct: f64,
    pub risk_score: f64,
    pub risk_flags: Vec<RiskFlag>,
    pub notes: Option<String>,
}
