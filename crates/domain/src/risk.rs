//! Risk-related domain types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Risk flag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFlag {
    pub code: String,
    pub severity: String,
    pub evidence_spans: Vec<EvidenceSpan>,
}

/// Evidence span for highlighting rule text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSpan {
    pub start: usize,
    pub end: usize,
}

/// Rule snapshot with extracted risk features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSnapshot {
    pub market_id: String,
    pub as_of: DateTime<Utc>,
    pub rule_text: String,
    pub rule_hash: String,
    pub settlement_source: Option<String>,
    pub settlement_window: Option<String>,
    pub definition_risk_score: f64,
    pub risk_flags: Vec<RiskFlag>,
}
