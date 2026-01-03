//! Database operations for recommendations

use bigdecimal::BigDecimal;
use sqlx::PgPool;
use std::str::FromStr;

use pm_domain::Recommendation;

/// Error type for recommendation operations
#[derive(Debug, thiserror::Error)]
pub enum RecError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Recommendation not found for market: {0}")]
    NotFound(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, RecError>;

/// Convert f64 to BigDecimal
fn f64_to_bigdecimal(val: f64) -> BigDecimal {
    BigDecimal::from_str(&val.to_string()).unwrap_or_else(|_| BigDecimal::from(0))
}

/// Convert Option<f64> to Option<BigDecimal>
fn opt_f64_to_bigdecimal(val: Option<f64>) -> Option<BigDecimal> {
    val.map(f64_to_bigdecimal)
}

/// Upsert recommendation for a market
pub async fn upsert_rec(pool: &PgPool, rec: &Recommendation) -> Result<()> {
    let risk_flags_json = serde_json::to_value(&rec.risk_flags)?;

    sqlx::query!(
        r#"
        INSERT INTO recs_latest (
            market_id, as_of, recommended_side, entry_price,
            expected_payout, max_position_pct, risk_score,
            risk_flags, notes
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT (market_id)
        DO UPDATE SET
            as_of = EXCLUDED.as_of,
            recommended_side = EXCLUDED.recommended_side,
            entry_price = EXCLUDED.entry_price,
            expected_payout = EXCLUDED.expected_payout,
            max_position_pct = EXCLUDED.max_position_pct,
            risk_score = EXCLUDED.risk_score,
            risk_flags = EXCLUDED.risk_flags,
            notes = EXCLUDED.notes,
            updated_at = NOW()
        "#,
        rec.market_id,
        rec.as_of,
        rec.recommended_side,
        f64_to_bigdecimal(rec.entry_price),
        f64_to_bigdecimal(rec.expected_payout),
        f64_to_bigdecimal(rec.max_position_pct),
        f64_to_bigdecimal(rec.risk_score),
        risk_flags_json,
        rec.notes
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Batch upsert recommendations
pub async fn upsert_recs_batch(pool: &PgPool, recs: &[Recommendation]) -> Result<()> {
    if recs.is_empty() {
        return Ok(());
    }

    let mut tx = pool.begin().await?;

    for rec in recs {
        let risk_flags_json = serde_json::to_value(&rec.risk_flags)?;

        sqlx::query!(
            r#"
            INSERT INTO recs_latest (
                market_id, as_of, recommended_side, entry_price,
                expected_payout, max_position_pct, risk_score,
                risk_flags, notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (market_id)
            DO UPDATE SET
                as_of = EXCLUDED.as_of,
                recommended_side = EXCLUDED.recommended_side,
                entry_price = EXCLUDED.entry_price,
                expected_payout = EXCLUDED.expected_payout,
                max_position_pct = EXCLUDED.max_position_pct,
                risk_score = EXCLUDED.risk_score,
                risk_flags = EXCLUDED.risk_flags,
                notes = EXCLUDED.notes,
                updated_at = NOW()
            "#,
            rec.market_id,
            rec.as_of,
            rec.recommended_side,
            f64_to_bigdecimal(rec.entry_price),
            f64_to_bigdecimal(rec.expected_payout),
            f64_to_bigdecimal(rec.max_position_pct),
            f64_to_bigdecimal(rec.risk_score),
            risk_flags_json,
            rec.notes
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

/// Get recommendation for a market
pub async fn get_rec(pool: &PgPool, market_id: &str) -> Result<Recommendation> {
    let row = sqlx::query!(
        r#"
        SELECT
            market_id, as_of, recommended_side, entry_price,
            expected_payout, max_position_pct, risk_score,
            risk_flags, notes
        FROM recs_latest
        WHERE market_id = $1
        "#,
        market_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| RecError::NotFound(market_id.to_string()))?;

    let risk_flags = serde_json::from_value(row.risk_flags)?;

    Ok(Recommendation {
        market_id: row.market_id,
        as_of: row.as_of,
        recommended_side: row.recommended_side,
        entry_price: row.entry_price.to_string().parse().unwrap_or(0.0),
        expected_payout: row.expected_payout.to_string().parse().unwrap_or(1.0),
        max_position_pct: row.max_position_pct.to_string().parse().unwrap_or(0.0),
        risk_score: row.risk_score.to_string().parse().unwrap_or(0.0),
        risk_flags,
        notes: row.notes,
    })
}

/// List recommendations with filters
pub async fn list_recs(
    pool: &PgPool,
    min_score: Option<f64>,
    max_t_remaining_sec: Option<i64>,
    max_risk_score: Option<f64>,
    has_flags: Option<bool>,
    limit: usize,
    offset: usize,
) -> Result<Vec<Recommendation>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            r.market_id, r.as_of, r.recommended_side, r.entry_price,
            r.expected_payout, r.max_position_pct, r.risk_score,
            r.risk_flags, r.notes
        FROM recs_latest r
        LEFT JOIN scores_latest s ON r.market_id = s.market_id
        WHERE ($1::numeric IS NULL OR s.overall_score >= $1)
          AND ($2::bigint IS NULL OR s.t_remaining_sec <= $2)
          AND ($3::numeric IS NULL OR r.risk_score <= $3)
          AND ($4::boolean IS NULL OR
               ($4 = true AND jsonb_array_length(r.risk_flags) > 0) OR
               ($4 = false AND jsonb_array_length(r.risk_flags) = 0))
        ORDER BY s.overall_score DESC NULLS LAST
        LIMIT $5
        OFFSET $6
        "#,
        opt_f64_to_bigdecimal(min_score),
        max_t_remaining_sec,
        opt_f64_to_bigdecimal(max_risk_score),
        has_flags,
        limit as i64,
        offset as i64
    )
    .fetch_all(pool)
    .await?;

    let mut results = Vec::new();
    for row in rows {
        let risk_flags = serde_json::from_value(row.risk_flags)?;

        results.push(Recommendation {
            market_id: row.market_id,
            as_of: row.as_of,
            recommended_side: row.recommended_side,
            entry_price: row.entry_price.to_string().parse().unwrap_or(0.0),
            expected_payout: row.expected_payout.to_string().parse().unwrap_or(1.0),
            max_position_pct: row.max_position_pct.to_string().parse().unwrap_or(0.0),
            risk_score: row.risk_score.to_string().parse().unwrap_or(0.0),
            risk_flags,
            notes: row.notes,
        });
    }

    Ok(results)
}

/// Count recommendations with filters
pub async fn count_recs(
    pool: &PgPool,
    min_score: Option<f64>,
    max_t_remaining_sec: Option<i64>,
    max_risk_score: Option<f64>,
    has_flags: Option<bool>,
) -> Result<usize> {
    let row = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM recs_latest r
        LEFT JOIN scores_latest s ON r.market_id = s.market_id
        WHERE ($1::numeric IS NULL OR s.overall_score >= $1)
          AND ($2::bigint IS NULL OR s.t_remaining_sec <= $2)
          AND ($3::numeric IS NULL OR r.risk_score <= $3)
          AND ($4::boolean IS NULL OR
               ($4 = true AND jsonb_array_length(r.risk_flags) > 0) OR
               ($4 = false AND jsonb_array_length(r.risk_flags) = 0))
        "#,
        opt_f64_to_bigdecimal(min_score),
        max_t_remaining_sec,
        opt_f64_to_bigdecimal(max_risk_score),
        has_flags
    )
    .fetch_one(pool)
    .await?;

    Ok(row.count.unwrap_or(0) as usize)
}

/// List top recommendations
pub async fn list_top_recs(
    pool: &PgPool,
    max_risk_score: Option<f64>,
    has_flags: Option<bool>,
    limit: i64,
    offset: i64,
) -> Result<Vec<Recommendation>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            market_id, as_of, recommended_side, entry_price,
            expected_payout, max_position_pct, risk_score,
            risk_flags, notes
        FROM recs_latest
        WHERE ($1::numeric IS NULL OR risk_score <= $1)
          AND ($2::boolean IS NULL OR
               ($2 = true AND jsonb_array_length(risk_flags) > 0) OR
               ($2 = false AND jsonb_array_length(risk_flags) = 0))
        ORDER BY risk_score ASC
        LIMIT $3
        OFFSET $4
        "#,
        opt_f64_to_bigdecimal(max_risk_score),
        has_flags,
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;

    let mut results = Vec::new();
    for row in rows {
        let risk_flags = serde_json::from_value(row.risk_flags)?;

        results.push(Recommendation {
            market_id: row.market_id,
            as_of: row.as_of,
            recommended_side: row.recommended_side,
            entry_price: row.entry_price.to_string().parse().unwrap_or(0.0),
            expected_payout: row.expected_payout.to_string().parse().unwrap_or(1.0),
            max_position_pct: row.max_position_pct.to_string().parse().unwrap_or(0.0),
            risk_score: row.risk_score.to_string().parse().unwrap_or(0.0),
            risk_flags,
            notes: row.notes,
        });
    }

    Ok(results)
}
