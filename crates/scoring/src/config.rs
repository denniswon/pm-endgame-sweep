//! Scoring service configuration

use serde::{Deserialize, Serialize};

/// Configuration for scoring service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfig {
    /// How often to run scoring (seconds)
    pub cadence_sec: u64,

    /// Scoring weights
    pub weights: ScoringWeights,

    /// Bounds for eligibility
    pub bounds: ScoringBounds,

    /// Fee configuration (basis points)
    pub fee_bps: f64,

    /// Sizing configuration
    pub sizing: SizingConfig,
}

/// Weights for overall score computation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringWeights {
    /// Weight for yield velocity
    pub w1: f64,
    /// Weight for net yield
    pub w2: f64,
    /// Weight for liquidity score
    pub w3: f64,
    /// Weight for definition risk (penalty)
    pub w4: f64,
    /// Weight for staleness (penalty)
    pub w5: f64,
}

/// Bounds for eligibility filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringBounds {
    /// Minimum time remaining (seconds)
    pub min_t_remaining_sec: i64,
    /// Maximum time remaining (seconds)
    pub max_t_remaining_sec: i64,
    /// Maximum quote staleness (seconds)
    pub quote_stale_max_sec: i64,
    /// Minimum time in days for velocity calculation
    pub min_t_days: f64,
    /// Target spread for liquidity scoring
    pub spread_target: f64,
}

/// Position sizing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizingConfig {
    /// Base position size as percentage of NAV
    pub base_position_pct: f64,
}

impl Default for ScoringConfig {
    fn default() -> Self {
        Self {
            cadence_sec: 120,
            weights: ScoringWeights::default(),
            bounds: ScoringBounds::default(),
            fee_bps: 120.0, // 1.2%
            sizing: SizingConfig::default(),
        }
    }
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            w1: 0.45, // yield velocity
            w2: 0.25, // net yield
            w3: 0.15, // liquidity
            w4: 0.10, // definition risk (penalty)
            w5: 0.05, // staleness (penalty)
        }
    }
}

impl Default for ScoringBounds {
    fn default() -> Self {
        Self {
            min_t_remaining_sec: 3600,      // 1 hour
            max_t_remaining_sec: 1_209_600, // 14 days
            quote_stale_max_sec: 180,       // 3 minutes
            min_t_days: 0.25,               // 6 hours
            spread_target: 0.02,            // 2%
        }
    }
}

impl Default for SizingConfig {
    fn default() -> Self {
        Self {
            base_position_pct: 0.10, // 10% NAV max
        }
    }
}
