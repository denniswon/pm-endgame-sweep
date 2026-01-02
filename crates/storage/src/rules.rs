//! Database operations for rules

use sqlx::PgPool;

use pm_domain::RuleSnapshot;

/// Error type for rule operations
#[derive(Debug, thiserror::Error)]
pub enum RuleError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Rule not found for market: {0}")]
    NotFound(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, RuleError>;

/// Upsert rule snapshot for a market
pub async fn upsert_rule(pool: &PgPool, rule: &RuleSnapshot) -> Result<()> {
    let risk_flags_json = serde_json::to_value(&rule.risk_flags)?;

    sqlx::query!(
        r#"
        INSERT INTO rules_latest (
            market_id, as_of, rule_text, rule_hash,
            settlement_source, settlement_window,
            definition_risk_score, risk_flags
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (market_id)
        DO UPDATE SET
            as_of = EXCLUDED.as_of,
            rule_text = EXCLUDED.rule_text,
            rule_hash = EXCLUDED.rule_hash,
            settlement_source = EXCLUDED.settlement_source,
            settlement_window = EXCLUDED.settlement_window,
            definition_risk_score = EXCLUDED.definition_risk_score,
            risk_flags = EXCLUDED.risk_flags,
            updated_at = NOW()
        "#,
        rule.market_id,
        rule.as_of,
        rule.rule_text,
        rule.rule_hash,
        rule.settlement_source,
        rule.settlement_window,
        rule.definition_risk_score,
        risk_flags_json
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Get rule snapshot for a market
pub async fn get_rule(pool: &PgPool, market_id: &str) -> Result<RuleSnapshot> {
    let row = sqlx::query!(
        r#"
        SELECT
            market_id, as_of, rule_text, rule_hash,
            settlement_source, settlement_window,
            definition_risk_score, risk_flags
        FROM rules_latest
        WHERE market_id = $1
        "#,
        market_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| RuleError::NotFound(market_id.to_string()))?;

    let risk_flags = serde_json::from_value(row.risk_flags)?;

    Ok(RuleSnapshot {
        market_id: row.market_id,
        as_of: row.as_of,
        rule_text: row.rule_text,
        rule_hash: row.rule_hash,
        settlement_source: row.settlement_source,
        settlement_window: row.settlement_window,
        definition_risk_score: row.definition_risk_score
            .to_string()
            .parse()
            .unwrap_or(1.0),
        risk_flags,
    })
}

/// Get rule snapshots for multiple markets
pub async fn get_rules_batch(
    pool: &PgPool,
    market_ids: &[String],
) -> Result<Vec<RuleSnapshot>> {
    if market_ids.is_empty() {
        return Ok(Vec::new());
    }

    let rows = sqlx::query!(
        r#"
        SELECT
            market_id, as_of, rule_text, rule_hash,
            settlement_source, settlement_window,
            definition_risk_score, risk_flags
        FROM rules_latest
        WHERE market_id = ANY($1)
        "#,
        market_ids
    )
    .fetch_all(pool)
    .await?;

    let mut results = Vec::new();
    for row in rows {
        let risk_flags = serde_json::from_value(row.risk_flags)?;

        results.push(RuleSnapshot {
            market_id: row.market_id,
            as_of: row.as_of,
            rule_text: row.rule_text,
            rule_hash: row.rule_hash,
            settlement_source: row.settlement_source,
            settlement_window: row.settlement_window,
            definition_risk_score: row
                .definition_risk_score
                .to_string()
                .parse()
                .unwrap_or(1.0),
            risk_flags,
        });
    }

    Ok(results)
}

/// Check if rule text has changed (by hash comparison)
pub async fn has_rule_changed(
    pool: &PgPool,
    market_id: &str,
    new_hash: &str,
) -> Result<bool> {
    let row = sqlx::query!(
        r#"
        SELECT rule_hash
        FROM rules_latest
        WHERE market_id = $1
        "#,
        market_id
    )
    .fetch_optional(pool)
    .await?;

    match row {
        None => Ok(true), // No existing rule, so it has "changed"
        Some(r) => Ok(r.rule_hash != new_hash),
    }
}
