//! PM Endgame Sweep - API service
//!
//! Read-only REST API for opportunities and market detail.

use pm_api::{ApiConfig, ApiServer};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pm_api=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    tracing::info!("pm-api starting");

    // Load configuration
    let config = ApiConfig::default();

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

    // Create and run server
    let server = ApiServer::new(pool, config);
    server.run().await?;

    tracing::info!("pm-api shutdown complete");
    Ok(())
}
