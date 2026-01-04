//! Venue client trait and Polymarket implementation

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use pm_domain::{Market, MarketStatus, Outcome, Quote, RiskFlag, RuleSnapshot};
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{config::RetryConfig, retry::retry_with_backoff};

/// Error type for venue client operations
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON deserialization failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Market not found: {0}")]
    MarketNotFound(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

pub type Result<T> = std::result::Result<T, ClientError>;

/// Generic trait for prediction market venue clients
#[async_trait]
pub trait VenueClient: Send + Sync {
    /// Discover active markets (with pagination)
    async fn discover_markets(&self, limit: usize, offset: usize) -> Result<Vec<Market>>;

    /// Get top-of-book quotes for a list of markets
    async fn get_quotes(&self, market_ids: &[String]) -> Result<Vec<Quote>>;

    /// Get rule text and extract risk flags for a market
    async fn get_rules(&self, market_id: &str) -> Result<RuleSnapshot>;

    /// Get market outcomes (for binary or multi-outcome markets)
    async fn get_outcomes(&self, market_id: &str) -> Result<Vec<Outcome>>;
}

/// Polymarket client implementation
pub struct PolymarketClient {
    http: Client,
    base_url: String,
    retry_config: RetryConfig,
}

impl PolymarketClient {
    /// Create a new Polymarket client
    pub fn new(retry_config: RetryConfig) -> Self {
        Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client"),
            base_url: "https://gamma-api.polymarket.com".to_string(),
            retry_config,
        }
    }

    /// Compute SHA-256 hash of rule text for change detection
    fn compute_rule_hash(text: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Extract risk flags from rule text using simple heuristics
    fn extract_risk_flags(rule_text: &str) -> Vec<RiskFlag> {
        let mut flags = Vec::new();
        let lower = rule_text.to_lowercase();

        // High-risk patterns
        if lower.contains("subjective") || lower.contains("discretion") {
            flags.push(RiskFlag {
                code: "SUBJECTIVE_RESOLUTION".to_string(),
                severity: "high".to_string(),
                evidence_spans: vec![],
            });
        }

        if lower.contains("unnamed") || lower.contains("anonymous") {
            flags.push(RiskFlag {
                code: "UNNAMED_SOURCE".to_string(),
                severity: "high".to_string(),
                evidence_spans: vec![],
            });
        }

        if lower.contains("may") || lower.contains("might") || lower.contains("could") {
            flags.push(RiskFlag {
                code: "AMBIGUOUS_LANGUAGE".to_string(),
                severity: "medium".to_string(),
                evidence_spans: vec![],
            });
        }

        flags
    }

    /// Calculate definition risk score based on flags
    fn calculate_risk_score(flags: &[RiskFlag]) -> f64 {
        flags
            .iter()
            .map(|f| match f.severity.as_str() {
                "high" => 0.3,
                "medium" => 0.15,
                "low" => 0.05,
                _ => 0.0,
            })
            .sum::<f64>()
            .min(1.0)
    }
}

#[async_trait]
impl VenueClient for PolymarketClient {
    async fn discover_markets(&self, limit: usize, offset: usize) -> Result<Vec<Market>> {
        let url = format!(
            "{}/markets?limit={}&offset={}&active=true",
            self.base_url, limit, offset
        );

        let response = retry_with_backoff(&self.retry_config, || async {
            self.http.get(&url).send().await?.error_for_status()
        })
        .await?;

        let markets_json: Vec<PolymarketMarketResponse> = response.json().await?;

        Ok(markets_json
            .into_iter()
            .map(|m| {
                let url = format!("https://polymarket.com/event/{}", m.slug);
                Market {
                    market_id: m.condition_id,
                    venue: "polymarket".to_string(),
                    title: m.question,
                    slug: Some(m.slug),
                    category: m.category,
                    status: if m.closed {
                        MarketStatus::Closed
                    } else {
                        MarketStatus::Active
                    },
                    open_time: m.start_date,
                    close_time: m.end_date,
                    resolved_time: None,
                    url: Some(url),
                }
            })
            .collect())
    }

    async fn get_quotes(&self, market_ids: &[String]) -> Result<Vec<Quote>> {
        let mut quotes = Vec::new();
        let now = Utc::now();

        for market_id in market_ids {
            let url = format!("{}/markets/{}/book", self.base_url, market_id);

            let response = retry_with_backoff(&self.retry_config, || async {
                self.http.get(&url).send().await?.error_for_status()
            })
            .await;

            match response {
                Ok(resp) => {
                    let book: PolymarketBookResponse = resp.json().await?;

                    // For binary markets, extract YES/NO quotes
                    let yes_bid = book.bids.first().map(|b| b.price);
                    let yes_ask = book.asks.first().map(|a| a.price);
                    let no_bid = yes_ask.map(|p| 1.0 - p);
                    let no_ask = yes_bid.map(|p| 1.0 - p);

                    let spread_yes = match (yes_bid, yes_ask) {
                        (Some(bid), Some(ask)) => Some(ask - bid),
                        _ => None,
                    };

                    let spread_no = match (no_bid, no_ask) {
                        (Some(bid), Some(ask)) => Some(ask - bid),
                        _ => None,
                    };

                    let mid_yes = match (yes_bid, yes_ask) {
                        (Some(bid), Some(ask)) => Some((bid + ask) / 2.0),
                        _ => None,
                    };

                    let mid_no = match (no_bid, no_ask) {
                        (Some(bid), Some(ask)) => Some((bid + ask) / 2.0),
                        _ => None,
                    };

                    quotes.push(Quote {
                        market_id: market_id.clone(),
                        as_of: now,
                        yes_bid,
                        yes_ask,
                        no_bid,
                        no_ask,
                        spread_yes,
                        spread_no,
                        mid_yes,
                        mid_no,
                        quote_source: "polymarket".to_string(),
                    });
                }
                Err(e) => {
                    tracing::warn!(
                        market_id,
                        error = %e,
                        "Failed to fetch quote for market"
                    );
                }
            }
        }

        Ok(quotes)
    }

    async fn get_rules(&self, market_id: &str) -> Result<RuleSnapshot> {
        let url = format!("{}/markets/{}", self.base_url, market_id);

        let response = retry_with_backoff(&self.retry_config, || async {
            self.http.get(&url).send().await?.error_for_status()
        })
        .await?;

        let market: PolymarketMarketDetailResponse = response.json().await?;

        let rule_text = market
            .description
            .unwrap_or_else(|| "No rules provided".to_string());
        let rule_hash = Self::compute_rule_hash(&rule_text);
        let risk_flags = Self::extract_risk_flags(&rule_text);
        let definition_risk_score = Self::calculate_risk_score(&risk_flags);

        Ok(RuleSnapshot {
            market_id: market_id.to_string(),
            as_of: Utc::now(),
            rule_text,
            rule_hash,
            settlement_source: market.resolution_source,
            settlement_window: None,
            definition_risk_score,
            risk_flags,
        })
    }

    async fn get_outcomes(&self, market_id: &str) -> Result<Vec<Outcome>> {
        let url = format!("{}/markets/{}", self.base_url, market_id);

        let response = retry_with_backoff(&self.retry_config, || async {
            self.http.get(&url).send().await?.error_for_status()
        })
        .await?;

        let _market: PolymarketMarketDetailResponse = response.json().await?;

        // For binary markets, create YES/NO outcomes
        Ok(vec![
            Outcome {
                market_id: market_id.to_string(),
                outcome: "YES".to_string(),
                token_id: None,
            },
            Outcome {
                market_id: market_id.to_string(),
                outcome: "NO".to_string(),
                token_id: None,
            },
        ])
    }
}

// Polymarket API response types

#[derive(Debug, Deserialize)]
struct PolymarketMarketResponse {
    #[serde(rename = "conditionId")]
    condition_id: String,
    question: String,
    slug: String,
    category: Option<String>,
    closed: bool,
    #[serde(rename = "startDate")]
    start_date: Option<DateTime<Utc>>,
    #[serde(rename = "endDate")]
    end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct PolymarketMarketDetailResponse {
    description: Option<String>,
    #[serde(rename = "resolutionSource")]
    resolution_source: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PolymarketBookResponse {
    bids: Vec<PolymarketOrderLevel>,
    asks: Vec<PolymarketOrderLevel>,
}

#[derive(Debug, Deserialize)]
struct PolymarketOrderLevel {
    price: f64,
    #[allow(dead_code)]
    size: f64,
}
