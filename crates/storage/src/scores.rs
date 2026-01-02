//! Database operations for scores

use bigdecimal::BigDecimal;
use sqlx::PgPool;
use std::str::FromStr;

use pm_domain::Score;

/// Error type for score operations
#[derive(Debug, thiserror::Error)]
pub enum ScoreError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Score not found for market: {0}")]
    NotFound(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ScoreError>;

/// Convert f64 to BigDecimal
fn f64_to_bigdecimal(val: f64) -> BigDecimal {
    BigDecimal::from_str(&val.to_string()).unwrap_or_else(|_| BigDecimal::from(0))
}

/// Convert Option<f64> to Option<BigDecimal>
fn opt_f64_to_bigdecimal(val: Option<f64>) -> Option<BigDecimal> {
    val.map(f64_to_bigdecimal)
}

/// Convert BigDecimal to f64
#[allow(dead_code)]
fn bigdecimal_to_f64(val: BigDecimal) -> f64 {
    val.to_string().parse().unwrap_or(0.0)
}

/// Upsert score for a market
pub async fn upsert_score(pool: &PgPool, score: &Score) -> Result<()> {
    let score_breakdown_json = serde_json::to_value(&score.score_breakdown)?;

    sqlx::query!(
        r#"
        INSERT INTO scores_latest (
            market_id, as_of, t_remaining_sec, gross_yield, fee_bps,
            net_yield, yield_velocity, liquidity_score,
            staleness_sec, staleness_penalty, definition_risk_score,
            overall_score, score_breakdown
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        ON CONFLICT (market_id)
        DO UPDATE SET
            as_of = EXCLUDED.as_of,
            t_remaining_sec = EXCLUDED.t_remaining_sec,
            gross_yield = EXCLUDED.gross_yield,
            fee_bps = EXCLUDED.fee_bps,
            net_yield = EXCLUDED.net_yield,
            yield_velocity = EXCLUDED.yield_velocity,
            liquidity_score = EXCLUDED.liquidity_score,
            staleness_sec = EXCLUDED.staleness_sec,
            staleness_penalty = EXCLUDED.staleness_penalty,
            definition_risk_score = EXCLUDED.definition_risk_score,
            overall_score = EXCLUDED.overall_score,
            score_breakdown = EXCLUDED.score_breakdown,
            updated_at = NOW()
        "#,
        score.market_id,
        score.as_of,
        score.t_remaining_sec,
        f64_to_bigdecimal(score.gross_yield),
        f64_to_bigdecimal(score.fee_bps),
        f64_to_bigdecimal(score.net_yield),
        f64_to_bigdecimal(score.yield_velocity),
        f64_to_bigdecimal(score.liquidity_score),
        score.staleness_sec,
        f64_to_bigdecimal(score.staleness_penalty),
        f64_to_bigdecimal(score.definition_risk_score),
        f64_to_bigdecimal(score.overall_score),
        score_breakdown_json
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Batch upsert scores
pub async fn upsert_scores_batch(pool: &PgPool, scores: &[Score]) -> Result<()> {
    if scores.is_empty() {
        return Ok(());
    }

    let mut tx = pool.begin().await?;

    for score in scores {
        let score_breakdown_json = serde_json::to_value(&score.score_breakdown)?;

        sqlx::query!(
            r#"
            INSERT INTO scores_latest (
                market_id, as_of, t_remaining_sec, gross_yield, fee_bps,
                net_yield, yield_velocity, liquidity_score,
                staleness_sec, staleness_penalty, definition_risk_score,
                overall_score, score_breakdown
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (market_id)
            DO UPDATE SET
                as_of = EXCLUDED.as_of,
                t_remaining_sec = EXCLUDED.t_remaining_sec,
                gross_yield = EXCLUDED.gross_yield,
                fee_bps = EXCLUDED.fee_bps,
                net_yield = EXCLUDED.net_yield,
                yield_velocity = EXCLUDED.yield_velocity,
                liquidity_score = EXCLUDED.liquidity_score,
                staleness_sec = EXCLUDED.staleness_sec,
                staleness_penalty = EXCLUDED.staleness_penalty,
                definition_risk_score = EXCLUDED.definition_risk_score,
                overall_score = EXCLUDED.overall_score,
                score_breakdown = EXCLUDED.score_breakdown,
                updated_at = NOW()
            "#,
            score.market_id,
            score.as_of,
            score.t_remaining_sec,
            f64_to_bigdecimal(score.gross_yield),
            f64_to_bigdecimal(score.fee_bps),
            f64_to_bigdecimal(score.net_yield),
            f64_to_bigdecimal(score.yield_velocity),
            f64_to_bigdecimal(score.liquidity_score),
            score.staleness_sec,
            f64_to_bigdecimal(score.staleness_penalty),
            f64_to_bigdecimal(score.definition_risk_score),
            f64_to_bigdecimal(score.overall_score),
            score_breakdown_json
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

/// Get score for a market
pub async fn get_score(pool: &PgPool, market_id: &str) -> Result<Score> {
    let row = sqlx::query!(
        r#"
        SELECT
            market_id, as_of, t_remaining_sec, gross_yield, fee_bps,
            net_yield, yield_velocity, liquidity_score,
            staleness_sec, staleness_penalty, definition_risk_score,
            overall_score, score_breakdown
        FROM scores_latest
        WHERE market_id = $1
        "#,
        market_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| ScoreError::NotFound(market_id.to_string()))?;

    Ok(Score {
        market_id: row.market_id,
        as_of: row.as_of,
        t_remaining_sec: row.t_remaining_sec,
        gross_yield: row.gross_yield.to_string().parse().unwrap_or(0.0),
        fee_bps: row.fee_bps.to_string().parse().unwrap_or(0.0),
        net_yield: row.net_yield.to_string().parse().unwrap_or(0.0),
        yield_velocity: row.yield_velocity.to_string().parse().unwrap_or(0.0),
        liquidity_score: row.liquidity_score.to_string().parse().unwrap_or(0.0),
        staleness_sec: row.staleness_sec,
        staleness_penalty: row.staleness_penalty.to_string().parse().unwrap_or(0.0),
        definition_risk_score: row
            .definition_risk_score
            .to_string()
            .parse()
            .unwrap_or(0.0),
        overall_score: row.overall_score.to_string().parse().unwrap_or(0.0),
        score_breakdown: row.score_breakdown,
    })
}

/// List top scoring opportunities
pub async fn list_top_scores(
    pool: &PgPool,
    min_score: Option<f64>,
    max_t_remaining: Option<i64>,
    limit: i64,
    offset: i64,
) -> Result<Vec<Score>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            market_id, as_of, t_remaining_sec, gross_yield, fee_bps,
            net_yield, yield_velocity, liquidity_score,
            staleness_sec, staleness_penalty, definition_risk_score,
            overall_score, score_breakdown
        FROM scores_latest
        WHERE ($1::numeric IS NULL OR overall_score >= $1)
          AND ($2::bigint IS NULL OR t_remaining_sec <= $2)
        ORDER BY overall_score DESC
        LIMIT $3
        OFFSET $4
        "#,
        opt_f64_to_bigdecimal(min_score),
        max_t_remaining,
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| Score {
            market_id: row.market_id,
            as_of: row.as_of,
            t_remaining_sec: row.t_remaining_sec,
            gross_yield: row.gross_yield.to_string().parse().unwrap_or(0.0),
            fee_bps: row.fee_bps.to_string().parse().unwrap_or(0.0),
            net_yield: row.net_yield.to_string().parse().unwrap_or(0.0),
            yield_velocity: row.yield_velocity.to_string().parse().unwrap_or(0.0),
            liquidity_score: row.liquidity_score.to_string().parse().unwrap_or(0.0),
            staleness_sec: row.staleness_sec,
            staleness_penalty: row.staleness_penalty.to_string().parse().unwrap_or(0.0),
            definition_risk_score: row
                .definition_risk_score
                .to_string()
                .parse()
                .unwrap_or(0.0),
            overall_score: row.overall_score.to_string().parse().unwrap_or(0.0),
            score_breakdown: row.score_breakdown,
        })
        .collect())
}
