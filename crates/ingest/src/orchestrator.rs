//! Ingestion orchestrator with bounded channels and periodic tasks

use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;
use tokio::sync::mpsc;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;

use pm_domain::{Market, Quote, RuleSnapshot};
use pm_storage::{markets, quotes, rules};

use crate::client::VenueClient;
use crate::config::IngestConfig;

/// Error type for orchestrator operations
#[derive(Debug, thiserror::Error)]
pub enum OrchestratorError {
    #[error("Client error: {0}")]
    Client(#[from] crate::client::ClientError),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Channel send failed")]
    ChannelSend,
}

pub type Result<T> = std::result::Result<T, OrchestratorError>;

/// Ingestion orchestrator coordinates discovery, quote polling, and rule extraction
pub struct IngestOrchestrator<C: VenueClient> {
    client: Arc<C>,
    pool: PgPool,
    config: IngestConfig,
    cancellation: CancellationToken,
}

impl<C: VenueClient + 'static> IngestOrchestrator<C> {
    /// Create a new orchestrator
    pub fn new(client: C, pool: PgPool, config: IngestConfig) -> Self {
        Self {
            client: Arc::new(client),
            pool,
            config,
            cancellation: CancellationToken::new(),
        }
    }

    /// Get cancellation token for external shutdown
    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancellation.clone()
    }

    /// Start all ingestion tasks
    pub async fn run(&self) -> Result<()> {
        tracing::info!("Starting ingestion orchestrator");

        // Create bounded channels for work distribution
        let (market_tx, market_rx) = mpsc::channel::<Market>(self.config.max_channel_size);
        let (quote_tx, quote_rx) = mpsc::channel::<Vec<Quote>>(self.config.max_channel_size);
        let (rule_tx, rule_rx) = mpsc::channel::<RuleSnapshot>(self.config.max_channel_size);

        // Spawn worker tasks
        let mut handles = vec![];

        // Market discovery task
        handles.push(tokio::spawn({
            let client = Arc::clone(&self.client);
            let config = self.config.clone();
            let market_tx = market_tx.clone();
            let cancellation = self.cancellation.clone();

            async move {
                Self::discovery_task(client, config, market_tx, cancellation).await;
            }
        }));

        // Quote polling task
        handles.push(tokio::spawn({
            let client = Arc::clone(&self.client);
            let pool = self.pool.clone();
            let config = self.config.clone();
            let quote_tx = quote_tx.clone();
            let cancellation = self.cancellation.clone();

            async move {
                Self::quote_polling_task(client, pool, config, quote_tx, cancellation).await;
            }
        }));

        // Rule extraction task
        handles.push(tokio::spawn({
            let client = Arc::clone(&self.client);
            let pool = self.pool.clone();
            let config = self.config.clone();
            let rule_tx = rule_tx.clone();
            let cancellation = self.cancellation.clone();

            async move {
                Self::rule_extraction_task(client, pool, config, rule_tx, cancellation).await;
            }
        }));

        // Market persistence task
        handles.push(tokio::spawn({
            let pool = self.pool.clone();
            let cancellation = self.cancellation.clone();

            async move {
                Self::market_persistence_task(pool, market_rx, cancellation).await;
            }
        }));

        // Quote persistence task
        handles.push(tokio::spawn({
            let pool = self.pool.clone();
            let cancellation = self.cancellation.clone();

            async move {
                Self::quote_persistence_task(pool, quote_rx, cancellation).await;
            }
        }));

        // Rule persistence task
        handles.push(tokio::spawn({
            let pool = self.pool.clone();
            let cancellation = self.cancellation.clone();

            async move {
                Self::rule_persistence_task(pool, rule_rx, cancellation).await;
            }
        }));

        // Wait for cancellation signal
        self.cancellation.cancelled().await;
        tracing::info!("Shutting down ingestion orchestrator");

        // Wait for all tasks to complete
        for handle in handles {
            let _ = handle.await;
        }

        Ok(())
    }

    /// Stop the orchestrator
    pub fn stop(&self) {
        self.cancellation.cancel();
    }

    /// Market discovery task - periodically discovers new markets
    async fn discovery_task(
        client: Arc<C>,
        config: IngestConfig,
        market_tx: mpsc::Sender<Market>,
        cancellation: CancellationToken,
    ) {
        let mut ticker = interval(Duration::from_secs(config.discovery_cadence_sec));

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    tracing::info!("Running market discovery");

                    let limit = config.max_markets_per_discovery;
                    let mut offset = 0;

                    loop {
                        match client.discover_markets(limit, offset).await {
                            Ok(markets) => {
                                if markets.is_empty() {
                                    break;
                                }

                                tracing::info!(count = markets.len(), "Discovered markets");

                                for market in markets {
                                    if market_tx.send(market).await.is_err() {
                                        tracing::error!("Market channel closed");
                                        return;
                                    }
                                }

                                offset += limit;
                            }
                            Err(e) => {
                                tracing::error!(error = %e, "Market discovery failed");
                                break;
                            }
                        }
                    }
                }
                _ = cancellation.cancelled() => {
                    tracing::info!("Discovery task cancelled");
                    return;
                }
            }
        }
    }

    /// Quote polling task - periodically fetches quotes for active markets
    async fn quote_polling_task(
        client: Arc<C>,
        pool: PgPool,
        config: IngestConfig,
        quote_tx: mpsc::Sender<Vec<Quote>>,
        cancellation: CancellationToken,
    ) {
        let mut ticker = interval(Duration::from_secs(config.quotes_cadence_sec));

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    tracing::info!("Polling quotes");

                    // Get active markets from database
                    let market_ids = match markets::list_active_markets(
                        &pool,
                        3600,          // min 1 hour remaining
                        1209600,       // max 14 days remaining
                        config.max_quotes_per_fetch as i64,
                    )
                    .await
                    {
                        Ok(markets) => markets.into_iter().map(|m| m.market_id).collect::<Vec<_>>(),
                        Err(e) => {
                            tracing::error!(error = %e, "Failed to fetch active markets");
                            continue;
                        }
                    };

                    if market_ids.is_empty() {
                        tracing::debug!("No active markets to poll");
                        continue;
                    }

                    match client.get_quotes(&market_ids).await {
                        Ok(quotes) => {
                            tracing::info!(count = quotes.len(), "Fetched quotes");

                            if quote_tx.send(quotes).await.is_err() {
                                tracing::error!("Quote channel closed");
                                return;
                            }
                        }
                        Err(e) => {
                            tracing::error!(error = %e, "Quote polling failed");
                        }
                    }
                }
                _ = cancellation.cancelled() => {
                    tracing::info!("Quote polling task cancelled");
                    return;
                }
            }
        }
    }

    /// Rule extraction task - periodically extracts rule text and risk flags
    async fn rule_extraction_task(
        client: Arc<C>,
        pool: PgPool,
        config: IngestConfig,
        rule_tx: mpsc::Sender<RuleSnapshot>,
        cancellation: CancellationToken,
    ) {
        let mut ticker = interval(Duration::from_secs(config.rules_refresh_cadence_sec));

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    tracing::info!("Extracting rules");

                    // Get active markets from database
                    let markets = match markets::list_active_markets(
                        &pool,
                        3600,          // min 1 hour remaining
                        1209600,       // max 14 days remaining
                        100,           // limit to 100 markets per cycle
                    )
                    .await
                    {
                        Ok(m) => m,
                        Err(e) => {
                            tracing::error!(error = %e, "Failed to fetch active markets");
                            continue;
                        }
                    };

                    for market in markets {
                        match client.get_rules(&market.market_id).await {
                            Ok(rule) => {
                                // Check if rule hash has changed before sending
                                let has_changed = match rules::has_rule_changed(
                                    &pool,
                                    &rule.market_id,
                                    &rule.rule_hash,
                                )
                                .await
                                {
                                    Ok(changed) => changed,
                                    Err(e) => {
                                        tracing::error!(
                                            market_id = %market.market_id,
                                            error = %e,
                                            "Failed to check rule hash"
                                        );
                                        true // Assume changed on error
                                    }
                                };

                                if has_changed {
                                    tracing::info!(
                                        market_id = %market.market_id,
                                        "Rule text changed"
                                    );

                                    if rule_tx.send(rule).await.is_err() {
                                        tracing::error!("Rule channel closed");
                                        return;
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    market_id = %market.market_id,
                                    error = %e,
                                    "Rule extraction failed"
                                );
                            }
                        }
                    }
                }
                _ = cancellation.cancelled() => {
                    tracing::info!("Rule extraction task cancelled");
                    return;
                }
            }
        }
    }

    /// Market persistence task - saves markets to database
    async fn market_persistence_task(
        pool: PgPool,
        mut market_rx: mpsc::Receiver<Market>,
        cancellation: CancellationToken,
    ) {
        let mut batch = Vec::new();
        let batch_size = 100;

        loop {
            tokio::select! {
                Some(market) = market_rx.recv() => {
                    batch.push(market);

                    if batch.len() >= batch_size {
                        Self::flush_markets(&pool, &mut batch).await;
                    }
                }
                _ = cancellation.cancelled() => {
                    // Flush remaining markets
                    if !batch.is_empty() {
                        Self::flush_markets(&pool, &mut batch).await;
                    }
                    tracing::info!("Market persistence task cancelled");
                    return;
                }
            }
        }
    }

    /// Quote persistence task - saves quotes to database
    async fn quote_persistence_task(
        pool: PgPool,
        mut quote_rx: mpsc::Receiver<Vec<Quote>>,
        cancellation: CancellationToken,
    ) {
        loop {
            tokio::select! {
                Some(quotes) = quote_rx.recv() => {
                    // Save to latest table
                    if let Err(e) = quotes::upsert_quotes_latest_batch(&pool, &quotes).await {
                        tracing::error!(error = %e, "Failed to save latest quotes");
                    }

                    // Sample to 5m table
                    for quote in &quotes {
                        if let Err(e) = quotes::insert_quote_5m(&pool, quote).await {
                            tracing::error!(
                                market_id = %quote.market_id,
                                error = %e,
                                "Failed to save 5m quote sample"
                            );
                        }
                    }

                    tracing::info!(count = quotes.len(), "Persisted quotes");
                }
                _ = cancellation.cancelled() => {
                    tracing::info!("Quote persistence task cancelled");
                    return;
                }
            }
        }
    }

    /// Rule persistence task - saves rules to database
    async fn rule_persistence_task(
        pool: PgPool,
        mut rule_rx: mpsc::Receiver<RuleSnapshot>,
        cancellation: CancellationToken,
    ) {
        loop {
            tokio::select! {
                Some(rule) = rule_rx.recv() => {
                    if let Err(e) = rules::upsert_rule(&pool, &rule).await {
                        tracing::error!(
                            market_id = %rule.market_id,
                            error = %e,
                            "Failed to save rule"
                        );
                    } else {
                        tracing::info!(market_id = %rule.market_id, "Persisted rule");
                    }
                }
                _ = cancellation.cancelled() => {
                    tracing::info!("Rule persistence task cancelled");
                    return;
                }
            }
        }
    }

    /// Flush market batch to database
    async fn flush_markets(pool: &PgPool, batch: &mut Vec<Market>) {
        if batch.is_empty() {
            return;
        }

        match markets::upsert_markets_batch(pool, batch).await {
            Ok(()) => {
                tracing::info!(count = batch.len(), "Persisted markets");
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to save markets");
            }
        }

        batch.clear();
    }
}
