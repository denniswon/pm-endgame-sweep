//! Database operations for recommendations

use sqlx::PgPool;

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
        rec.entry_price,
        rec.expected_payout,
        rec.max_position_pct,
        rec.risk_score,
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
            rec.entry_price,
            rec.expected_payout,
            rec.max_position_pct,
            rec.risk_score,
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
        max_risk_score,
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
