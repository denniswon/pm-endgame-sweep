//! Quote-related domain types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Top-of-book quote snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub market_id: String,
    pub as_of: DateTime<Utc>,
    pub yes_bid: Option<f64>,
    pub yes_ask: Option<f64>,
    pub no_bid: Option<f64>,
    pub no_ask: Option<f64>,
    pub spread_yes: Option<f64>,
    pub spread_no: Option<f64>,
    pub mid_yes: Option<f64>,
    pub mid_no: Option<f64>,
    pub quote_source: String,
}
