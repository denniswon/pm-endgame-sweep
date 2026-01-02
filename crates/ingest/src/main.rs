//! PM Endgame Sweep - Ingestion service
//!
//! Discovers markets, polls quotes, and extracts rules from Polymarket.

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    // TODO: Initialize service

    Ok(())
}
