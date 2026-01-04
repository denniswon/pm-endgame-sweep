//! Scoring orchestrator that periodically computes scores and recommendations

use std::{collections::HashMap, time::Duration};

use pm_domain::{Quote, RuleSnapshot, Score};
use pm_storage::{markets, quotes, recs, rules, scores};
use sqlx::PgPool;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;

use crate::{config::ScoringConfig, engine::ScoringEngine};

/// Error type for orchestrator operations
#[derive(Debug, thiserror::Error)]
pub enum OrchestratorError {
    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Scoring error: {0}")]
    Scoring(#[from] crate::engine::ScoringError),
}

pub type Result<T> = std::result::Result<T, OrchestratorError>;

/// Scoring orchestrator coordinates periodic scoring runs
pub struct ScoringOrchestrator {
    engine: ScoringEngine,
    pool: PgPool,
    config: ScoringConfig,
    cancellation: CancellationToken,
}

impl ScoringOrchestrator {
    /// Create a new orchestrator
    pub fn new(pool: PgPool, config: ScoringConfig) -> Self {
        let engine = ScoringEngine::new(config.clone());

        Self {
            engine,
            pool,
            config,
            cancellation: CancellationToken::new(),
        }
    }

    /// Get cancellation token for external shutdown
    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancellation.clone()
    }

    /// Start scoring loop
    pub async fn run(&self) -> Result<()> {
        tracing::info!("Starting scoring orchestrator");

        let mut ticker = interval(Duration::from_secs(self.config.cadence_sec));

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    if let Err(e) = self.run_scoring_cycle().await {
                        tracing::error!(error = %e, "Scoring cycle failed");
                    }
                }
                _ = self.cancellation.cancelled() => {
                    tracing::info!("Scoring orchestrator cancelled");
                    return Ok(());
                }
            }
        }
    }

    /// Run a single scoring cycle
    async fn run_scoring_cycle(&self) -> Result<()> {
        let now = chrono::Utc::now();
        tracing::info!("Running scoring cycle");

        // Fetch active markets
        let markets = markets::list_active_markets(
            &self.pool,
            self.config.bounds.min_t_remaining_sec,
            self.config.bounds.max_t_remaining_sec,
            1000, // Process up to 1000 markets per cycle
        )
        .await
        .map_err(|e| OrchestratorError::Storage(e.to_string()))?;

        if markets.is_empty() {
            tracing::debug!("No active markets to score");
            return Ok(());
        }

        tracing::info!(count = markets.len(), "Fetched active markets");

        // Fetch quotes for these markets
        let market_ids: Vec<String> = markets.iter().map(|m| m.market_id.clone()).collect();

        let quotes_list = quotes::get_quotes_latest_batch(&self.pool, &market_ids)
            .await
            .map_err(|e| OrchestratorError::Storage(e.to_string()))?;

        let quotes: HashMap<String, Quote> = quotes_list
            .into_iter()
            .map(|q| (q.market_id.clone(), q))
            .collect();

        tracing::info!(count = quotes.len(), "Fetched latest quotes");

        // Fetch rules for these markets
        let rules_list = rules::get_rules_batch(&self.pool, &market_ids)
            .await
            .map_err(|e| OrchestratorError::Storage(e.to_string()))?;

        let rules: HashMap<String, RuleSnapshot> = rules_list
            .into_iter()
            .map(|r| (r.market_id.clone(), r))
            .collect();

        tracing::info!(count = rules.len(), "Fetched rules");

        // Compute scores
        let computed_scores = self
            .engine
            .compute_scores_batch(&markets, &quotes, &rules, now);

        if computed_scores.is_empty() {
            tracing::debug!("No scores computed");
            return Ok(());
        }

        tracing::info!(count = computed_scores.len(), "Computed scores");

        // Save scores to database
        scores::upsert_scores_batch(&self.pool, &computed_scores)
            .await
            .map_err(|e| OrchestratorError::Storage(e.to_string()))?;

        // Build scores map for recommendation generation
        let scores_map: HashMap<String, Score> = computed_scores
            .into_iter()
            .map(|s| (s.market_id.clone(), s))
            .collect();

        // Generate recommendations
        let recommendations =
            self.engine
                .generate_recommendations_batch(&markets, &scores_map, &quotes, &rules);

        tracing::info!(count = recommendations.len(), "Generated recommendations");

        // Save recommendations to database
        if !recommendations.is_empty() {
            recs::upsert_recs_batch(&self.pool, &recommendations)
                .await
                .map_err(|e| OrchestratorError::Storage(e.to_string()))?;
        }

        tracing::info!("Scoring cycle complete");
        Ok(())
    }
}
