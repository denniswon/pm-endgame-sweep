-- PM Endgame Sweep - Initial Schema
-- Migration: 20260101000001_initial_schema

-- Core markets table
CREATE TABLE IF NOT EXISTS markets (
  market_id TEXT PRIMARY KEY,
  venue TEXT NOT NULL DEFAULT 'polymarket',
  title TEXT NOT NULL,
  slug TEXT NULL,
  category TEXT NULL,
  status TEXT NOT NULL,
  open_time TIMESTAMPTZ NULL,
  close_time TIMESTAMPTZ NULL,
  resolved_time TIMESTAMPTZ NULL,
  url TEXT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS markets_status_close_time_idx
  ON markets (status, close_time);

CREATE INDEX IF NOT EXISTS markets_venue_status_idx
  ON markets (venue, status);

-- Binary outcomes (YES/NO)
CREATE TABLE IF NOT EXISTS market_outcomes (
  market_id TEXT NOT NULL REFERENCES markets(market_id) ON DELETE CASCADE,
  outcome TEXT NOT NULL,
  token_id TEXT NULL,
  PRIMARY KEY (market_id, outcome)
);

-- Latest quotes (top-of-book + minimal liquidity proxies)
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

-- Optional 5-minute samples retained for 7 days
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

-- Latest rule text and extracted risk features
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

CREATE INDEX IF NOT EXISTS rules_latest_rule_hash_idx
  ON rules_latest (rule_hash);

-- Latest computed scoring features and risk decomposition
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

CREATE INDEX IF NOT EXISTS scores_latest_as_of_idx
  ON scores_latest (as_of);

-- Latest recommendations for UI
CREATE TABLE IF NOT EXISTS recs_latest (
  market_id TEXT PRIMARY KEY REFERENCES markets(market_id) ON DELETE CASCADE,
  as_of TIMESTAMPTZ NOT NULL,
  recommended_side TEXT NOT NULL,
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

CREATE INDEX IF NOT EXISTS recs_latest_as_of_idx
  ON recs_latest (as_of);
