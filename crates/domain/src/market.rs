//! Market-related domain types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Market status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarketStatus {
    Active,
    Closed,
    Resolved,
    Halted,
}

/// A prediction market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub market_id: String,
    pub venue: String,
    pub title: String,
    pub slug: Option<String>,
    pub category: Option<String>,
    pub status: MarketStatus,
    pub open_time: Option<DateTime<Utc>>,
    pub close_time: Option<DateTime<Utc>>,
    pub resolved_time: Option<DateTime<Utc>>,
    pub url: Option<String>,
}

/// Binary outcome (YES/NO)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    pub market_id: String,
    pub outcome: String,
    pub token_id: Option<String>,
}
