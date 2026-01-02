//! Database operations for markets

use chrono::Utc;
use sqlx::PgPool;

use pm_domain::{Market, MarketStatus, Outcome};

/// Error type for market operations
#[derive(Debug, thiserror::Error)]
pub enum MarketError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Market not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, MarketError>;

/// Insert or update a market
pub async fn upsert_market(pool: &PgPool, market: &Market) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO markets (
            market_id, venue, title, slug, category, status,
            open_time, close_time, resolved_time, url
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT (market_id)
        DO UPDATE SET
            title = EXCLUDED.title,
            slug = EXCLUDED.slug,
            category = EXCLUDED.category,
            status = EXCLUDED.status,
            open_time = EXCLUDED.open_time,
            close_time = EXCLUDED.close_time,
            resolved_time = EXCLUDED.resolved_time,
            url = EXCLUDED.url,
            updated_at = NOW()
        "#,
        market.market_id,
        market.venue,
        market.title,
        market.slug,
        market.category,
        format!("{:?}", market.status).to_lowercase(),
        market.open_time,
        market.close_time,
        market.resolved_time,
        market.url
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Batch upsert markets
pub async fn upsert_markets_batch(pool: &PgPool, markets: &[Market]) -> Result<()> {
    if markets.is_empty() {
        return Ok(());
    }

    let mut tx = pool.begin().await?;

    for market in markets {
        sqlx::query!(
            r#"
            INSERT INTO markets (
                market_id, venue, title, slug, category, status,
                open_time, close_time, resolved_time, url
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (market_id)
            DO UPDATE SET
                title = EXCLUDED.title,
                slug = EXCLUDED.slug,
                category = EXCLUDED.category,
                status = EXCLUDED.status,
                open_time = EXCLUDED.open_time,
                close_time = EXCLUDED.close_time,
                resolved_time = EXCLUDED.resolved_time,
                url = EXCLUDED.url,
                updated_at = NOW()
            "#,
            market.market_id,
            market.venue,
            market.title,
            market.slug,
            market.category,
            format!("{:?}", market.status).to_lowercase(),
            market.open_time,
            market.close_time,
            market.resolved_time,
            market.url
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

/// Get a market by ID
pub async fn get_market(pool: &PgPool, market_id: &str) -> Result<Market> {
    let row = sqlx::query!(
        r#"
        SELECT
            market_id, venue, title, slug, category, status,
            open_time, close_time, resolved_time, url
        FROM markets
        WHERE market_id = $1
        "#,
        market_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| MarketError::NotFound(market_id.to_string()))?;

    Ok(Market {
        market_id: row.market_id,
        venue: row.venue,
        title: row.title,
        slug: row.slug,
        category: row.category,
        status: parse_market_status(&row.status),
        open_time: row.open_time,
        close_time: row.close_time,
        resolved_time: row.resolved_time,
        url: row.url,
    })
}

/// List markets with optional filters
pub async fn list_markets(
    pool: &PgPool,
    venue: Option<&str>,
    status: Option<MarketStatus>,
    limit: i64,
    offset: i64,
) -> Result<Vec<Market>> {
    let status_str = status.map(|s| format!("{:?}", s).to_lowercase());

    let rows = sqlx::query!(
        r#"
        SELECT
            market_id, venue, title, slug, category, status,
            open_time, close_time, resolved_time, url
        FROM markets
        WHERE ($1::text IS NULL OR venue = $1)
          AND ($2::text IS NULL OR status = $2)
        ORDER BY close_time DESC NULLS LAST
        LIMIT $3
        OFFSET $4
        "#,
        venue,
        status_str.as_deref(),
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| Market {
            market_id: row.market_id,
            venue: row.venue,
            title: row.title,
            slug: row.slug,
            category: row.category,
            status: parse_market_status(&row.status),
            open_time: row.open_time,
            close_time: row.close_time,
            resolved_time: row.resolved_time,
            url: row.url,
        })
        .collect())
}

/// List active markets eligible for scoring
pub async fn list_active_markets(
    pool: &PgPool,
    min_time_remaining: i64,
    max_time_remaining: i64,
    limit: i64,
) -> Result<Vec<Market>> {
    let now = Utc::now();
    let min_close = now + chrono::Duration::seconds(min_time_remaining);
    let max_close = now + chrono::Duration::seconds(max_time_remaining);

    let rows = sqlx::query!(
        r#"
        SELECT
            market_id, venue, title, slug, category, status,
            open_time, close_time, resolved_time, url
        FROM markets
        WHERE status = 'active'
          AND close_time IS NOT NULL
          AND close_time >= $1
          AND close_time <= $2
        ORDER BY close_time ASC
        LIMIT $3
        "#,
        min_close,
        max_close,
        limit
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| Market {
            market_id: row.market_id,
            venue: row.venue,
            title: row.title,
            slug: row.slug,
            category: row.category,
            status: parse_market_status(&row.status),
            open_time: row.open_time,
            close_time: row.close_time,
            resolved_time: row.resolved_time,
            url: row.url,
        })
        .collect())
}

/// Upsert market outcomes
pub async fn upsert_outcomes(pool: &PgPool, outcomes: &[Outcome]) -> Result<()> {
    if outcomes.is_empty() {
        return Ok(());
    }

    let mut tx = pool.begin().await?;

    for outcome in outcomes {
        sqlx::query!(
            r#"
            INSERT INTO market_outcomes (market_id, outcome, token_id)
            VALUES ($1, $2, $3)
            ON CONFLICT (market_id, outcome)
            DO UPDATE SET token_id = EXCLUDED.token_id
            "#,
            outcome.market_id,
            outcome.outcome,
            outcome.token_id
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

/// Get outcomes for a market
pub async fn get_outcomes(pool: &PgPool, market_id: &str) -> Result<Vec<Outcome>> {
    let rows = sqlx::query!(
        r#"
        SELECT market_id, outcome, token_id
        FROM market_outcomes
        WHERE market_id = $1
        ORDER BY outcome
        "#,
        market_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| Outcome {
            market_id: row.market_id,
            outcome: row.outcome,
            token_id: row.token_id,
        })
        .collect())
}

/// Parse market status from string
fn parse_market_status(s: &str) -> MarketStatus {
    match s.to_lowercase().as_str() {
        "active" => MarketStatus::Active,
        "closed" => MarketStatus::Closed,
        "resolved" => MarketStatus::Resolved,
        "halted" => MarketStatus::Halted,
        _ => MarketStatus::Active, // Default fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_market_status() {
        assert!(matches!(
            parse_market_status("active"),
            MarketStatus::Active
        ));
        assert!(matches!(
            parse_market_status("closed"),
            MarketStatus::Closed
        ));
        assert!(matches!(
            parse_market_status("resolved"),
            MarketStatus::Resolved
        ));
        assert!(matches!(
            parse_market_status("halted"),
            MarketStatus::Halted
        ));
    }
}
