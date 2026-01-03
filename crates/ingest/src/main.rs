//! PM Endgame Sweep - Ingestion service
//!
//! Discovers markets, polls quotes, and extracts rules from Polymarket.

use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use pm_ingest::{IngestConfig, IngestOrchestrator, PolymarketClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pm_ingest=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    tracing::info!("pm-ingest starting");

    // Load configuration
    let config = IngestConfig::default();

    // Connect to database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/pm_endgame".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(&database_url)
        .await?;

    tracing::info!("Connected to database");

    // Create Polymarket client
    let client = PolymarketClient::new(config.retry.clone());

    // Create orchestrator
    let orchestrator = IngestOrchestrator::new(client, pool, config);

    // Setup signal handler for graceful shutdown
    tokio::spawn({
        let cancel = orchestrator.cancellation_token();
        async move {
            tokio::signal::ctrl_c().await.ok();
            tracing::info!("Received shutdown signal");
            cancel.cancel();
        }
    });

    orchestrator.run().await?;

    tracing::info!("pm-ingest shutdown complete");
    Ok(())
}
