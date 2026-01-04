//! Scoring engine with yield, liquidity, and risk calculations

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use pm_domain::{Market, Quote, Recommendation, RuleSnapshot, Score};
use serde_json::json;

use crate::config::ScoringConfig;

/// Error type for scoring operations
#[derive(Debug, thiserror::Error)]
pub enum ScoringError {
    #[error("Missing quote for market: {0}")]
    MissingQuote(String),

    #[error("Missing rule for market: {0}")]
    MissingRule(String),

    #[error("Invalid market state: {0}")]
    InvalidMarket(String),
}

pub type Result<T> = std::result::Result<T, ScoringError>;

/// Scoring engine for computing opportunity scores
pub struct ScoringEngine {
    config: ScoringConfig,
}

impl ScoringEngine {
    /// Create a new scoring engine
    pub fn new(config: ScoringConfig) -> Self {
        Self { config }
    }

    /// Compute score for a market
    pub fn compute_score(
        &self,
        market: &Market,
        quote: &Quote,
        rule: Option<&RuleSnapshot>,
        now: DateTime<Utc>,
    ) -> Result<Score> {
        // Validate market has close_time
        let close_time = market
            .close_time
            .ok_or_else(|| ScoringError::InvalidMarket(market.market_id.clone()))?;

        // Calculate time remaining
        let t_remaining_sec = (close_time - now).num_seconds();

        // Check eligibility bounds
        if t_remaining_sec < self.config.bounds.min_t_remaining_sec
            || t_remaining_sec > self.config.bounds.max_t_remaining_sec
        {
            return Err(ScoringError::InvalidMarket(format!(
                "Time remaining {} outside bounds",
                t_remaining_sec
            )));
        }

        // Calculate staleness
        let staleness_sec = (now - quote.as_of).num_seconds();
        let staleness_penalty = self.calculate_staleness_penalty(staleness_sec);

        // Calculate NO side pricing (we're selling volatility on low-prob outcomes)
        let no_bid = quote
            .no_bid
            .ok_or_else(|| ScoringError::MissingQuote(market.market_id.clone()))?;
        let no_ask = quote
            .no_ask
            .ok_or_else(|| ScoringError::MissingQuote(market.market_id.clone()))?;

        // Gross yield = selling NO at bid price
        let entry_price = no_bid;
        let gross_yield = entry_price;

        // Net yield after fees
        let fee_rate = self.config.fee_bps / 10000.0;
        let net_yield = gross_yield * (1.0 - fee_rate);

        // Yield velocity (annualized)
        let t_days = t_remaining_sec as f64 / 86400.0;
        let t_days_clamped = t_days.max(self.config.bounds.min_t_days);
        let yield_velocity = net_yield / t_days_clamped;

        // Liquidity score
        let liquidity_score = self.calculate_liquidity_score(no_bid, no_ask, staleness_penalty);

        // Definition risk score
        let definition_risk_score = rule.map(|r| r.definition_risk_score).unwrap_or(0.0);

        // Overall score (weighted combination)
        let overall_score = self.calculate_overall_score(
            yield_velocity,
            net_yield,
            liquidity_score,
            definition_risk_score,
            staleness_penalty,
        );

        // Score breakdown for transparency
        let score_breakdown = json!({
            "yield_velocity": yield_velocity,
            "net_yield": net_yield,
            "liquidity_score": liquidity_score,
            "definition_risk_score": definition_risk_score,
            "staleness_penalty": staleness_penalty,
            "gross_yield": gross_yield,
            "fee_rate": fee_rate,
            "t_days": t_days,
            "entry_price": entry_price,
        });

        Ok(Score {
            market_id: market.market_id.clone(),
            as_of: now,
            t_remaining_sec,
            gross_yield,
            fee_bps: self.config.fee_bps,
            net_yield,
            yield_velocity,
            liquidity_score,
            staleness_sec,
            staleness_penalty,
            definition_risk_score,
            overall_score,
            score_breakdown,
        })
    }

    /// Generate recommendation from score
    pub fn generate_recommendation(
        &self,
        market: &Market,
        score: &Score,
        quote: &Quote,
        rule: Option<&RuleSnapshot>,
    ) -> Recommendation {
        // Entry price (NO bid)
        let entry_price = quote.no_bid.unwrap_or(0.0);

        // Expected payout (NO side pays 1.0 if outcome is NO)
        let expected_payout = 1.0;

        // Calculate position size with risk haircuts
        let max_position_pct = self.calculate_position_size(score, rule);

        // Aggregate risk score
        let risk_score = score.definition_risk_score + score.staleness_penalty;

        // Risk flags from rule analysis
        let risk_flags = rule.map(|r| r.risk_flags.clone()).unwrap_or_default();

        // Notes with key metrics
        let notes = format!(
            "Yield: {:.2}% | Velocity: {:.2}% | Liquidity: {:.2} | Risk: {:.2}",
            score.net_yield * 100.0,
            score.yield_velocity * 100.0,
            score.liquidity_score,
            risk_score
        );

        Recommendation {
            market_id: market.market_id.clone(),
            as_of: score.as_of,
            recommended_side: "NO".to_string(),
            entry_price,
            expected_payout,
            max_position_pct,
            risk_score,
            risk_flags,
            notes: Some(notes),
        }
    }

    /// Calculate staleness penalty
    fn calculate_staleness_penalty(&self, staleness_sec: i64) -> f64 {
        let ratio = staleness_sec as f64 / self.config.bounds.quote_stale_max_sec as f64;
        ratio.clamp(0.0, 1.0)
    }

    /// Calculate liquidity score
    fn calculate_liquidity_score(&self, no_bid: f64, no_ask: f64, staleness_penalty: f64) -> f64 {
        let spread_no = no_ask - no_bid;
        let spread_ratio = spread_no / self.config.bounds.spread_target;
        let raw_score = (1.0 - spread_ratio).clamp(0.0, 1.0);

        // Apply staleness penalty
        raw_score * (1.0 - staleness_penalty)
    }

    /// Calculate overall score (weighted combination)
    fn calculate_overall_score(
        &self,
        yield_velocity: f64,
        net_yield: f64,
        liquidity_score: f64,
        definition_risk_score: f64,
        staleness_penalty: f64,
    ) -> f64 {
        let w = &self.config.weights;

        // Normalize yield metrics (assuming max ~1.0 for yield velocity, ~0.5 for net
        // yield)
        let norm_velocity = (yield_velocity / 1.0).clamp(0.0, 1.0);
        let norm_net_yield = (net_yield / 0.5).clamp(0.0, 1.0);

        // Weighted combination
        let score = w.w1 * norm_velocity + w.w2 * norm_net_yield + w.w3 * liquidity_score
            - w.w4 * definition_risk_score
            - w.w5 * staleness_penalty;

        score.clamp(0.0, 1.0)
    }

    /// Calculate position size with risk haircuts
    fn calculate_position_size(&self, score: &Score, _rule: Option<&RuleSnapshot>) -> f64 {
        let base = self.config.sizing.base_position_pct;

        // Risk haircut based on definition risk
        let risk_haircut = 1.0 - score.definition_risk_score;

        // Liquidity haircut
        let liq_haircut = 0.5 + 0.5 * score.liquidity_score;

        // Compute final position size
        let position_pct = base * risk_haircut * liq_haircut;

        position_pct.clamp(0.01, 0.10) // Min 1%, max 10%
    }

    /// Batch score computation for multiple markets
    pub fn compute_scores_batch(
        &self,
        markets: &[Market],
        quotes: &HashMap<String, Quote>,
        rules: &HashMap<String, RuleSnapshot>,
        now: DateTime<Utc>,
    ) -> Vec<Score> {
        markets
            .iter()
            .filter_map(|market| {
                let quote = quotes.get(&market.market_id)?;
                let rule = rules.get(&market.market_id);

                match self.compute_score(market, quote, rule, now) {
                    Ok(score) => Some(score),
                    Err(e) => {
                        tracing::debug!(
                            market_id = %market.market_id,
                            error = %e,
                            "Skipping market in scoring"
                        );
                        None
                    }
                }
            })
            .collect()
    }

    /// Batch recommendation generation
    pub fn generate_recommendations_batch(
        &self,
        markets: &[Market],
        scores: &HashMap<String, Score>,
        quotes: &HashMap<String, Quote>,
        rules: &HashMap<String, RuleSnapshot>,
    ) -> Vec<Recommendation> {
        markets
            .iter()
            .filter_map(|market| {
                let score = scores.get(&market.market_id)?;
                let quote = quotes.get(&market.market_id)?;
                let rule = rules.get(&market.market_id);

                Some(self.generate_recommendation(market, score, quote, rule))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_staleness_penalty() {
        let config = ScoringConfig::default();
        let engine = ScoringEngine::new(config);

        // No staleness
        assert_eq!(engine.calculate_staleness_penalty(0), 0.0);

        // Half staleness
        assert_eq!(engine.calculate_staleness_penalty(90), 0.5);

        // Full staleness
        assert_eq!(engine.calculate_staleness_penalty(180), 1.0);

        // Over staleness threshold
        assert_eq!(engine.calculate_staleness_penalty(360), 1.0);
    }

    #[test]
    fn test_liquidity_score() {
        let config = ScoringConfig::default();
        let engine = ScoringEngine::new(config);

        // Perfect liquidity (no spread)
        let score = engine.calculate_liquidity_score(0.95, 0.95, 0.0);
        assert_eq!(score, 1.0);

        // Target spread (2%)
        let score = engine.calculate_liquidity_score(0.94, 0.96, 0.0);
        assert_eq!(score, 0.0);

        // With staleness penalty
        let score = engine.calculate_liquidity_score(0.95, 0.95, 0.5);
        assert_eq!(score, 0.5);
    }
}
