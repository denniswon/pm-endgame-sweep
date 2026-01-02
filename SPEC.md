# PM Endgame Sweep - System Specification v0.1

## Overview

This system scans Polymarket markets, computes "endgame sweep" style opportunity scores, and serves results via an API to a Next.js dashboard.
v0.1 supports Polymarket only.
The architecture is modular so additional venues (Kalshi, etc.) can be added behind a shared `VenueClient` interface.
Cadence is fixed for v0.1: quotes every 60s, scoring every 120s, market discovery every 30m.
Retention is bounded: `*_latest` only plus optional 5-minute samples retained for 7 days.
The UI shows sizing guidance but does not track executed positions in v0.1.

## Goals and Non-Goals

### Goals

- Provide a continuously updated ranked list of high-probability, late-stage contracts with small remaining spreads.
- Provide explainable scoring breakdown and explicit risk flags (definition risk, settlement risk, liquidity risk, freshness risk).
- Guarantee bounded storage growth and bounded in-memory queues.
- Be resilient to API failures and degrade gracefully while serving the last known "latest" dataset.

### Non-Goals (v0.1)

- No automated trade execution.
- No portfolio position tracking or PnL.
- No cross-venue arbitrage execution.
- No user authentication.
- No advanced charting beyond basic sparklines on 5-minute samples.

## System Architecture

### Services

**pm-ingest**

Discovers markets and fetches quotes and rule snapshots.
Writes normalized data into Postgres.
Runs three loops: discovery (30m), quotes (60s), and rule refresh (event-driven + periodic).

**pm-score**

Reads `*_latest` snapshots and computes scores and recommendations every 120s.
Writes to `scores_latest` and `recs_latest`.

**pm-api**

Read-only API for the dashboard and external tooling.
Serves opportunities, market detail, and current config.

**ui (Next.js)**

Renders ranked opportunities, detail pages, and filters.

### Key Constraints

- No unbounded channels or collections.
- No panics in production paths (no `unwrap`/`expect` outside tests).
- All external calls assumed to fail and retried with exponential backoff and jitter.
- SQL uses compile-time checked SQLx queries.
- Hot paths minimize allocations and batch DB writes.

## Data Model

### Logical Entities

**Market**

A Polymarket market with metadata, close time, and settlement rules.

**Outcome**

A binary outcome (YES/NO) with best bid/ask and last update time.

**Quote Snapshot**

Top-of-book and minimal liquidity proxies at a point in time.

**Rule Snapshot**

Market resolution rules, extracted features, and a hash for change detection.

**Score Snapshot**

Computed features, risk decomposition, and an overall score.

**Recommendation Snapshot**

Suggested side (usually NO), max sizing guidance, and key execution notes.

### Postgres Schema (DDL)

```sql
-- Core markets.
CREATE TABLE IF NOT EXISTS markets (
  market_id TEXT PRIMARY KEY,
  venue TEXT NOT NULL DEFAULT 'polymarket',
  title TEXT NOT NULL,
  slug TEXT NULL,
  category TEXT NULL,
  status TEXT NOT NULL, -- e.g., active, closed, resolved
  open_time TIMESTAMPTZ NULL,
  close_time TIMESTAMPTZ NULL,
  resolved_time TIMESTAMPTZ NULL,
  url TEXT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS markets_status_close_time_idx
  ON markets (status, close_time);

-- Binary outcomes.
CREATE TABLE IF NOT EXISTS market_outcomes (
  market_id TEXT NOT NULL REFERENCES markets(market_id) ON DELETE CASCADE,
  outcome TEXT NOT NULL, -- 'YES' | 'NO'
  token_id TEXT NULL,
  PRIMARY KEY (market_id, outcome)
);

-- Latest quotes (top-of-book + minimal liquidity proxies).
CREATE TABLE IF NOT EXISTS quotes_latest (
  market_id TEXT PRIMARY KEY REFERENCES markets(market_id) ON DELETE CASCADE,
  as_of TIMESTAMPTZ NOT NULL,
  yes_bid NUMERIC(10,6) NULL,
  yes_ask NUMERIC(10,6) NULL,
  no_bid NUMERIC(10,6) NULL,
  no_ask NUMERIC(10,6) NULL,
  spread_yes NUMERIC(10,6) NULL,
  spread_no NUMERIC(10,6) NULL,
  mid_yes NUMERIC(10,6) NULL,
  mid_no NUMERIC(10,6) NULL,
  quote_source TEXT NOT NULL DEFAULT 'polymarket',
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS quotes_latest_as_of_idx
  ON quotes_latest (as_of);

-- Optional 5-minute samples retained for 7 days.
CREATE TABLE IF NOT EXISTS quotes_5m (
  market_id TEXT NOT NULL REFERENCES markets(market_id) ON DELETE CASCADE,
  bucket_start TIMESTAMPTZ NOT NULL,
  as_of TIMESTAMPTZ NOT NULL,
  yes_bid NUMERIC(10,6) NULL,
  yes_ask NUMERIC(10,6) NULL,
  no_bid NUMERIC(10,6) NULL,
  no_ask NUMERIC(10,6) NULL,
  PRIMARY KEY (market_id, bucket_start)
);

CREATE INDEX IF NOT EXISTS quotes_5m_bucket_idx
  ON quotes_5m (bucket_start);

-- Latest rule text and extracted risk features.
CREATE TABLE IF NOT EXISTS rules_latest (
  market_id TEXT PRIMARY KEY REFERENCES markets(market_id) ON DELETE CASCADE,
  as_of TIMESTAMPTZ NOT NULL,
  rule_text TEXT NOT NULL,
  rule_hash TEXT NOT NULL,
  settlement_source TEXT NULL,
  settlement_window TEXT NULL,
  definition_risk_score NUMERIC(10,6) NOT NULL,
  risk_flags JSONB NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Latest computed scoring features and risk decomposition.
CREATE TABLE IF NOT EXISTS scores_latest (
  market_id TEXT PRIMARY KEY REFERENCES markets(market_id) ON DELETE CASCADE,
  as_of TIMESTAMPTZ NOT NULL,
  t_remaining_sec BIGINT NOT NULL,
  gross_yield NUMERIC(10,6) NOT NULL,
  fee_bps NUMERIC(10,6) NOT NULL,
  net_yield NUMERIC(10,6) NOT NULL,
  yield_velocity NUMERIC(10,6) NOT NULL,
  liquidity_score NUMERIC(10,6) NOT NULL,
  staleness_sec BIGINT NOT NULL,
  staleness_penalty NUMERIC(10,6) NOT NULL,
  definition_risk_score NUMERIC(10,6) NOT NULL,
  overall_score NUMERIC(10,6) NOT NULL,
  score_breakdown JSONB NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS scores_latest_overall_idx
  ON scores_latest (overall_score DESC);

-- Latest recommendations for UI.
CREATE TABLE IF NOT EXISTS recs_latest (
  market_id TEXT PRIMARY KEY REFERENCES markets(market_id) ON DELETE CASCADE,
  as_of TIMESTAMPTZ NOT NULL,
  recommended_side TEXT NOT NULL, -- typically 'NO'
  entry_price NUMERIC(10,6) NOT NULL,
  expected_payout NUMERIC(10,6) NOT NULL DEFAULT 1.0,
  max_position_pct NUMERIC(10,6) NOT NULL,
  risk_score NUMERIC(10,6) NOT NULL,
  risk_flags JSONB NOT NULL,
  notes TEXT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS recs_latest_risk_idx
  ON recs_latest (risk_score ASC);
```

## Scoring Model

### Core Concept

We are not "predicting outcomes".
We are ranking contracts where probability has effectively converged, but the price still leaves a small residual spread to harvest.
We treat this as carry with embedded jump risk, so sizing is driven primarily by tail-risk and definition risk.

### Eligibility Gates

A market is eligible for scoring if all conditions hold:

- `market.status` in `('active')`
- `close_time` is not null
- `t_remaining_sec` between `MIN_T_REMAINING_SEC` and `MAX_T_REMAINING_SEC`
- `quotes_latest.as_of` within `QUOTE_STALE_MAX_SEC`
- `rules_latest` exists

### Feature Definitions

Let `p` be the entry price of the recommended side.
For endgame sweep, `p` is usually `no_ask` when recommending NO.
Let `fee_bps` be the configured effective fee load in basis points (config).
Let `T_days = t_remaining_sec / 86400.0`.

**Gross yield**

```
gross_yield = (1.0 - p)
```

**Fee model**

```
fee = p * fee_bps / 10000.0
```

This is deliberately conservative and can be updated once venue fee mechanics are modeled precisely.

**Net yield**

```
net_yield = max(gross_yield - fee, 0.0)
```

**Yield velocity**

```
yield_velocity = net_yield / max(T_days, MIN_T_DAYS)
```

**Staleness**

```
staleness_sec = now() - quotes_latest.as_of
staleness_penalty = clamp(staleness_sec / QUOTE_STALE_MAX_SEC, 0.0, 1.0)
```

**Liquidity score (v0.1 proxy)**

We compute a normalized score from spread and freshness.

```
spread_no = (no_ask - no_bid) if both exist else 1.0
liquidity_score = clamp(1.0 - (spread_no / SPREAD_TARGET), 0.0, 1.0) * (1.0 - staleness_penalty)
```

**Definition risk score**

Pulled from `rules_latest.definition_risk_score`.
See "Risk scoring" below.

### Overall Score

```
overall_score = w1 * norm(yield_velocity) + w2 * norm(net_yield) + w3 * liquidity_score - w4 * definition_risk_score - w5 * staleness_penalty
```

`norm(x)` is min-max normalization using configured bounds and clamping.
All weights are configuration-driven and exposed via `GET /v1/config`.

**Default weights (v0.1)**

- `w1 = 0.45`
- `w2 = 0.25`
- `w3 = 0.15`
- `w4 = 0.10`
- `w5 = 0.05`

## Risk Scoring

### Risk Components

We compute a `risk_score` in `[0.0, 1.0]` where higher is worse.
We also emit discrete `risk_flags` for explainability.

**Components (v0.1)**

- Definition risk (rules ambiguity and settlement discretion)
- Operational risk (platform outages, halts, rule changes)
- Liquidity risk (wide spreads, stale quotes)
- Event jump risk (binary tail events, discontinuous settlement disputes)

Operational risk and event jump risk are not fully observable on-chain in v0.1.
We proxy them via conservative base rates and always-on tail-risk budgeting in sizing.

### Definition Risk Extraction

We compute `definition_risk_score` from rule text using deterministic phrase patterns and missing-field checks.

**Examples of high-risk indicators**

- Ambiguous sources ("credible sources", "generally accepted")
- Discretionary settlement ("at our discretion", "we may decide")
- Unclear timestamps or timezones
- Unclear treatment of delays, reversals, or partial data
- Unclear "what counts" for the outcome (definitions of "reach", "touch", "close", "official")

We produce `risk_flags` as a JSON array of objects, each with `code`, `severity`, and `evidence_spans`.

### Sizing Guidance

We emit `max_position_pct` for each recommendation.
This is not a command and does not track execution.
Sizing is a function of total risk and liquidity score.

**Default sizing curve (v0.1)**

```
Start with base = 0.10 (10% NAV max)
Apply risk haircut: haircut = 1.0 - risk_score
Apply liquidity haircut: liq = 0.5 + 0.5 * liquidity_score
Compute max_position_pct = clamp(base * haircut * liq, 0.01, 0.10)
```

**Interpretation**

Even "best" opportunities cap at 10% NAV in v0.1 to respect jump risk.
Very risky or illiquid opportunities degrade toward 1% NAV.

## JSON Output Contracts

### Opportunity List Item

```json
{
  "market_id": "string",
  "venue": "polymarket",
  "title": "string",
  "url": "string",
  "as_of": "2026-01-01T00:00:00Z",
  "close_time": "2026-01-02T00:00:00Z",
  "t_remaining_sec": 12345,
  "recommended_side": "NO",
  "entry_price": 0.965,
  "gross_yield": 0.035,
  "net_yield": 0.033,
  "yield_velocity": 0.061,
  "liquidity_score": 0.72,
  "definition_risk_score": 0.12,
  "risk_score": 0.22,
  "overall_score": 0.78,
  "max_position_pct": 0.08,
  "risk_flags": [
    {
      "code": "SETTLEMENT_DISCRETION",
      "severity": "medium",
      "evidence_spans": [
        { "start": 120, "end": 168 }
      ]
    }
  ],
  "quote": {
    "no_bid": 0.961,
    "no_ask": 0.965,
    "spread_no": 0.004,
    "as_of": "2026-01-01T00:00:00Z",
    "staleness_sec": 12
  }
}
```

### Market Detail

```json
{
  "market": {
    "market_id": "string",
    "title": "string",
    "status": "active",
    "close_time": "2026-01-02T00:00:00Z",
    "url": "string"
  },
  "rules": {
    "as_of": "2026-01-01T00:00:00Z",
    "rule_text": "string",
    "rule_hash": "string",
    "definition_risk_score": 0.12,
    "risk_flags": []
  },
  "latest_quote": {
    "as_of": "2026-01-01T00:00:00Z",
    "yes_bid": 0.03,
    "yes_ask": 0.04,
    "no_bid": 0.96,
    "no_ask": 0.97
  },
  "latest_score": {
    "as_of": "2026-01-01T00:00:00Z",
    "overall_score": 0.78,
    "score_breakdown": {
      "yield_velocity": 0.061,
      "net_yield": 0.033,
      "liquidity_score": 0.72,
      "definition_risk_score": 0.12,
      "staleness_penalty": 0.02
    }
  },
  "recommendation": {
    "as_of": "2026-01-01T00:00:00Z",
    "recommended_side": "NO",
    "entry_price": 0.965,
    "max_position_pct": 0.08,
    "notes": "string"
  }
}
```

### Config

```json
{
  "version": "0.1.0",
  "cadence": {
    "quotes_sec": 60,
    "scoring_sec": 120,
    "discovery_sec": 1800
  },
  "retention": {
    "latest_only": true,
    "samples_5m_enabled": true,
    "samples_retention_days": 7
  },
  "scoring": {
    "weights": { "w1": 0.45, "w2": 0.25, "w3": 0.15, "w4": 0.10, "w5": 0.05 },
    "bounds": {
      "min_t_remaining_sec": 3600,
      "max_t_remaining_sec": 1209600,
      "quote_stale_max_sec": 180
    },
    "fee_bps": 120
  }
}
```

## API Endpoints

- `GET /v1/opportunities` - Query params: `min_score`, `max_t_remaining_sec`, `max_risk_score`, `has_flags`, `cursor`, `limit`
- `GET /v1/market/{market_id}`
- `GET /v1/config`
- `GET /healthz`, `GET /readyz`, `GET /metrics`

## Background Jobs

### Discovery Loop

Runs every 30 minutes.
Upserts markets and outcomes.
Fetches rule text for new markets.

### Quote Loop

Runs every 60 seconds.
Fetches top-of-book for eligible markets.
Updates `quotes_latest`.
Optionally inserts `quotes_5m` if the current bucket is missing.

### Scoring Loop

Runs every 120 seconds.
Joins markets, `quotes_latest`, `rules_latest`.
Computes scores and recs in memory with bounded batches.
Writes results in batch upserts.

### Retention Job

Runs daily.
Deletes from `quotes_5m` where `bucket_start < now() - 7 days`.

## Quality Bar

- All PRs must pass `cargo fmt --check`, `cargo clippy`, and `cargo test`.
- No unbounded growth in memory, queues, or DB tables.
- No `unwrap`/`expect` in production paths.
- All public items require rustdoc.
