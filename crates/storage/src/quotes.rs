//! Database operations for quotes

use chrono::{DateTime, Timelike, Utc};
use sqlx::PgPool;

use pm_domain::Quote;

/// Error type for quote operations
#[derive(Debug, thiserror::Error)]
pub enum QuoteError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Quote not found for market: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, QuoteError>;

/// Upsert latest quote for a market
pub async fn upsert_quote_latest(pool: &PgPool, quote: &Quote) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO quotes_latest (
            market_id, as_of, yes_bid, yes_ask, no_bid, no_ask,
            spread_yes, spread_no, mid_yes, mid_no, quote_source
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (market_id)
        DO UPDATE SET
            as_of = EXCLUDED.as_of,
            yes_bid = EXCLUDED.yes_bid,
            yes_ask = EXCLUDED.yes_ask,
            no_bid = EXCLUDED.no_bid,
            no_ask = EXCLUDED.no_ask,
            spread_yes = EXCLUDED.spread_yes,
            spread_no = EXCLUDED.spread_no,
            mid_yes = EXCLUDED.mid_yes,
            mid_no = EXCLUDED.mid_no,
            quote_source = EXCLUDED.quote_source,
            updated_at = NOW()
        "#,
        quote.market_id,
        quote.as_of,
        quote.yes_bid,
        quote.yes_ask,
        quote.no_bid,
        quote.no_ask,
        quote.spread_yes,
        quote.spread_no,
        quote.mid_yes,
        quote.mid_no,
        quote.quote_source
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Batch upsert latest quotes
pub async fn upsert_quotes_latest_batch(pool: &PgPool, quotes: &[Quote]) -> Result<()> {
    if quotes.is_empty() {
        return Ok(());
    }

    let mut tx = pool.begin().await?;

    for quote in quotes {
        sqlx::query!(
            r#"
            INSERT INTO quotes_latest (
                market_id, as_of, yes_bid, yes_ask, no_bid, no_ask,
                spread_yes, spread_no, mid_yes, mid_no, quote_source
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (market_id)
            DO UPDATE SET
                as_of = EXCLUDED.as_of,
                yes_bid = EXCLUDED.yes_bid,
                yes_ask = EXCLUDED.yes_ask,
                no_bid = EXCLUDED.no_bid,
                no_ask = EXCLUDED.no_ask,
                spread_yes = EXCLUDED.spread_yes,
                spread_no = EXCLUDED.spread_no,
                mid_yes = EXCLUDED.mid_yes,
                mid_no = EXCLUDED.mid_no,
                quote_source = EXCLUDED.quote_source,
                updated_at = NOW()
            "#,
            quote.market_id,
            quote.as_of,
            quote.yes_bid,
            quote.yes_ask,
            quote.no_bid,
            quote.no_ask,
            quote.spread_yes,
            quote.spread_no,
            quote.mid_yes,
            quote.mid_no,
            quote.quote_source
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

/// Get latest quote for a market
pub async fn get_quote_latest(pool: &PgPool, market_id: &str) -> Result<Quote> {
    let row = sqlx::query!(
        r#"
        SELECT
            market_id, as_of, yes_bid, yes_ask, no_bid, no_ask,
            spread_yes, spread_no, mid_yes, mid_no, quote_source
        FROM quotes_latest
        WHERE market_id = $1
        "#,
        market_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| QuoteError::NotFound(market_id.to_string()))?;

    Ok(Quote {
        market_id: row.market_id,
        as_of: row.as_of,
        yes_bid: row.yes_bid,
        yes_ask: row.yes_ask,
        no_bid: row.no_bid,
        no_ask: row.no_ask,
        spread_yes: row.spread_yes,
        spread_no: row.spread_no,
        mid_yes: row.mid_yes,
        mid_no: row.mid_no,
        quote_source: row.quote_source,
    })
}

/// Get latest quotes for multiple markets
pub async fn get_quotes_latest_batch(
    pool: &PgPool,
    market_ids: &[String],
) -> Result<Vec<Quote>> {
    if market_ids.is_empty() {
        return Ok(Vec::new());
    }

    let rows = sqlx::query!(
        r#"
        SELECT
            market_id, as_of, yes_bid, yes_ask, no_bid, no_ask,
            spread_yes, spread_no, mid_yes, mid_no, quote_source
        FROM quotes_latest
        WHERE market_id = ANY($1)
        "#,
        market_ids
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| Quote {
            market_id: row.market_id,
            as_of: row.as_of,
            yes_bid: row.yes_bid,
            yes_ask: row.yes_ask,
            no_bid: row.no_bid,
            no_ask: row.no_ask,
            spread_yes: row.spread_yes,
            spread_no: row.spread_no,
            mid_yes: row.mid_yes,
            mid_no: row.mid_no,
            quote_source: row.quote_source,
        })
        .collect())
}

/// Insert 5-minute sample quote
pub async fn insert_quote_5m(pool: &PgPool, quote: &Quote) -> Result<()> {
    // Bucket to 5-minute intervals
    let bucket_start = bucket_to_5m(quote.as_of);

    sqlx::query!(
        r#"
        INSERT INTO quotes_5m (
            market_id, bucket_start, as_of, yes_bid, yes_ask, no_bid, no_ask
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (market_id, bucket_start) DO NOTHING
        "#,
        quote.market_id,
        bucket_start,
        quote.as_of,
        quote.yes_bid,
        quote.yes_ask,
        quote.no_bid,
        quote.no_ask
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Delete old 5-minute samples (retention policy)
pub async fn delete_old_quotes_5m(pool: &PgPool, retention_days: i64) -> Result<u64> {
    let cutoff = Utc::now() - chrono::Duration::days(retention_days);

    let result = sqlx::query!(
        r#"
        DELETE FROM quotes_5m
        WHERE bucket_start < $1
        "#,
        cutoff
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Get 5-minute samples for a market
pub async fn get_quotes_5m(
    pool: &PgPool,
    market_id: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<Quote>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            market_id, as_of, yes_bid, yes_ask, no_bid, no_ask,
            bucket_start
        FROM quotes_5m
        WHERE market_id = $1
          AND bucket_start >= $2
          AND bucket_start <= $3
        ORDER BY bucket_start ASC
        "#,
        market_id,
        start,
        end
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| Quote {
            market_id: row.market_id,
            as_of: row.as_of,
            yes_bid: row.yes_bid,
            yes_ask: row.yes_ask,
            no_bid: row.no_bid,
            no_ask: row.no_ask,
            spread_yes: None,
            spread_no: None,
            mid_yes: None,
            mid_no: None,
            quote_source: "polymarket".to_string(),
        })
        .collect())
}

/// Round timestamp down to nearest 5-minute bucket
fn bucket_to_5m(dt: DateTime<Utc>) -> DateTime<Utc> {
    let minutes = dt.minute();
    let bucket_minutes = (minutes / 5) * 5;
    dt.with_minute(bucket_minutes)
        .and_then(|d: DateTime<Utc>| d.with_second(0))
        .and_then(|d: DateTime<Utc>| d.with_nanosecond(0))
        .unwrap_or(dt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bucket_to_5m() {
        let dt = Utc::now()
            .with_minute(17)
            .and_then(|d| d.with_second(42))
            .and_then(|d| d.with_nanosecond(123456789))
            .unwrap();

        let bucketed = bucket_to_5m(dt);
        assert_eq!(bucketed.minute(), 15);
        assert_eq!(bucketed.second(), 0);
        assert_eq!(bucketed.nanosecond(), 0);
    }
}
