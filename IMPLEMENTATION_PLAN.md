# Implementation Plan - PM Endgame Sweep v0.1

## Execution Strategy

This plan follows a **schema-first, bottom-up** approach to minimize rework and enforce reliability constraints early.

### Ordering Rationale

1. **Schema + Storage** first (unblocks all downstream work)
2. **Ingest** second (gets real data flowing)
3. **Scoring** third (turns data into decisions)
4. **API** fourth (stabilizes JSON contracts)
5. **UI** fifth (visualizes opportunities)
6. **Hardening** sixth (observability + resilience)

## Phases and Estimates

| Phase | Description          | Duration | Depends On |
| ----- | -------------------- | -------- | ---------- |
| E0    | Repo skeleton + CI   | 1-2 days | -          |
| E1    | Storage + schema     | 2-4 days | E0         |
| E2    | Polymarket ingestion | 4-7 days | E1         |
| E3    | Scoring + risk       | 4-7 days | E1, E2     |
| E4    | API service          | 3-5 days | E1, E3     |
| E5    | Next.js UI           | 4-7 days | E4         |
| E6    | Hardening            | 3-6 days | All        |

**Total estimate**: 21-38 days for first usable system.

## Epic E0: Repo Skeleton + CI

**Goal**: Establish workspace that enforces performance and correctness from day one.

### E0.1: Create Workspace Layout

**Tasks**:

- Create Cargo workspace with crates: `domain`, `storage`, `ingest`, `scoring`, `api`
- Add `.gitignore` (Rust + Node.js + IDE)
- Create `web/` directory for Next.js
- Add `config/` with `default.yaml` and `env.example`
- Create `migrations/` for SQLx

**Acceptance Criteria**:

- `cargo test --workspace` passes (even with no tests yet)
- Workspace has consistent MSRV and shared dependencies

**Files**:

```
Cargo.toml (workspace root)
crates/domain/Cargo.toml
crates/storage/Cargo.toml
crates/ingest/Cargo.toml
crates/scoring/Cargo.toml
crates/api/Cargo.toml
.gitignore
```

### E0.2: Add CI Gates

**Tasks**:

- Add GitHub Actions workflow: `.github/workflows/ci.yml`
- Configure: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo audit`
- Add `deny.toml` for dependency auditing
- Enforce no "full" feature flags without justification

**Acceptance Criteria**:

- CI fails on clippy warnings
- CI fails on missing `cargo fmt`
- CI runs on all PRs and main branch

**Files**:

```
.github/workflows/ci.yml
deny.toml
```

### E0.3: Add Docker Compose

**Tasks**:

- Create `docker/Dockerfile` for Rust services
- Create `docker/Dockerfile.ui` for Next.js
- Create `docker-compose.yml` with postgres, pm-ingest, pm-score, pm-api, ui
- Add health check endpoints to API

**Acceptance Criteria**:

- `docker compose up` yields healthy services
- `GET /healthz` returns 200
- `GET /readyz` checks DB connectivity

**Files**:

```
docker/Dockerfile
docker/Dockerfile.ui
docker-compose.yml
```

---

## Epic E1: Storage + Schema

**Goal**: Lock canonical data model and DB schema so all services can proceed independently.

### E1.1: Add Migrations

**Tasks**:

- Install SQLx CLI: `cargo install sqlx-cli`
- Create migrations for all tables in SPEC.md:
  - `markets`
  - `market_outcomes`
  - `quotes_latest`
  - `quotes_5m`
  - `rules_latest`
  - `scores_latest`
  - `recs_latest`
- Add indexes per SPEC.md
- Add migration: `sqlx migrate add initial_schema`

**Acceptance Criteria**:

- `sqlx migrate run` applies cleanly on empty DB
- All indexes created
- Foreign key constraints work

**Files**:

```
migrations/20260101000001_initial_schema.sql
```

### E1.2: Implement Storage Crate

**Tasks**:

- Add `sqlx` dependency with `postgres`, `runtime-tokio-native-tls`, `macros`, `offline` features
- Implement typed query APIs in `crates/storage/src/`:
  - `markets.rs` - CRUD for markets
  - `quotes.rs` - CRUD for quotes_latest and quotes_5m
  - `rules.rs` - CRUD for rules_latest
  - `scores.rs` - CRUD for scores_latest
  - `recs.rs` - CRUD for recs_latest
- Use SQLx compile-time checked queries (offline mode)
- Add connection pool with bounded size

**Acceptance Criteria**:

- No raw SQL strings in handlers (all queries use `sqlx::query!` macro)
- Queries compile with offline mode
- No N+1 reads in list endpoints

**Files**:

```
crates/storage/src/lib.rs
crates/storage/src/markets.rs
crates/storage/src/quotes.rs
crates/storage/src/rules.rs
crates/storage/src/scores.rs
crates/storage/src/recs.rs
```

### E1.3: Implement Domain Types

**Tasks**:

- Add `crates/domain/src/` with core types:
  - `Market`
  - `Outcome`
  - `Quote`
  - `RuleSnapshot`
  - `Score`
  - `Recommendation`
  - `RiskFlag`
- Add rustdoc on all public items
- Implement `Display` and `Debug` where appropriate
- Add property tests for numeric bounds (no NaNs, clamped ranges)

**Acceptance Criteria**:

- `cargo doc` has no warnings for missing docs
- All numeric types have validated constructors
- Property tests pass

**Files**:

```
crates/domain/src/lib.rs
crates/domain/src/market.rs
crates/domain/src/quote.rs
crates/domain/src/score.rs
crates/domain/src/risk.rs
```

---

## Epic E2: Polymarket Ingestion

**Goal**: Reliably ingest market metadata, quotes, and rules from Polymarket with bounded resources.

### E2.1: Define VenueClient Trait

**Tasks**:

- Create `crates/ingest/src/venue.rs` with `VenueClient` trait:
  - `async fn fetch_markets(&self, cursor: Option<String>) -> Result<Vec<Market>>`
  - `async fn fetch_quotes(&self, market_ids: &[String]) -> Result<Vec<Quote>>`
  - `async fn fetch_rules(&self, market_id: &str) -> Result<String>`
- Add Polymarket implementation: `crates/ingest/src/polymarket.rs`
- Add retry logic with exponential backoff + jitter
- Add circuit breaker state

**Acceptance Criteria**:

- Fixture tests for Polymarket response parsing
- Retry logic tested with mock failures
- Circuit breaker opens after N consecutive failures

**Files**:

```
crates/ingest/src/lib.rs
crates/ingest/src/venue.rs
crates/ingest/src/polymarket.rs
crates/ingest/src/retry.rs
```

### E2.2: Discovery Loop (30m Cadence)

**Tasks**:

- Create `crates/ingest/src/discovery.rs`
- Implement loop that runs every 30 minutes
- Fetch markets in batches
- Upsert markets and market_outcomes
- Track discovery metrics (markets discovered, errors)

**Acceptance Criteria**:

- Updates `markets` table in batch (not row-by-row)
- Bounded batch size (max 1000 markets per fetch)
- Loop continues even if one batch fails

**Files**:

```
crates/ingest/src/discovery.rs
```

### E2.3: Quote Loop (60s Cadence)

**Tasks**:

- Create `crates/ingest/src/quotes.rs`
- Implement loop that runs every 60 seconds
- Fetch quotes for active markets
- Use bounded mpsc channel for market_ids
- Update `quotes_latest` in batches
- Optionally insert `quotes_5m` samples

**Acceptance Criteria**:

- Bounded channel size (e.g., 10000)
- Batch writes (max 100 quotes per transaction)
- Does not fetch quotes for closed/resolved markets

**Files**:

```
crates/ingest/src/quotes.rs
```

### E2.4: Rule Snapshot Loop

**Tasks**:

- Create `crates/ingest/src/rules.rs`
- Fetch rules on market creation
- Re-fetch if rule hash changes
- Compute rule hash (SHA256 of text)
- Store in `rules_latest`

**Acceptance Criteria**:

- Only fetches when hash differs
- Avoids re-fetching unchanged rules

**Files**:

```
crates/ingest/src/rules.rs
```

---

## Epic E3: Scoring + Risk

**Goal**: Compute explainable endgame-sweep opportunities with risk decomposition.

### E3.1: Feature Extraction

**Tasks**:

- Create `crates/scoring/src/features.rs`
- Implement eligibility gates
- Compute:
  - `t_remaining_sec`
  - `gross_yield`
  - `net_yield`
  - `yield_velocity`
  - `staleness_sec`, `staleness_penalty`
  - `liquidity_score`
- Add unit tests for each calculation

**Acceptance Criteria**:

- Deterministic output for given input
- No NaNs or infinities in output
- Tests cover edge cases (zero spread, stale quotes, expired markets)

**Files**:

```
crates/scoring/src/lib.rs
crates/scoring/src/features.rs
```

### E3.2: Definition Risk Extraction

**Tasks**:

- Create `crates/scoring/src/risk.rs`
- Implement deterministic rule parser
- Detect ambiguous phrases:
  - "at our discretion"
  - "generally accepted"
  - "credible sources"
  - timezone ambiguity
  - unclear timestamps
- Compute `definition_risk_score` in [0.0, 1.0]
- Emit `risk_flags` with `code`, `severity`, `evidence_spans`

**Acceptance Criteria**:

- Golden tests for representative rule texts
- Same input → same flags
- Flags include char offsets for highlighting

**Files**:

```
crates/scoring/src/risk.rs
```

### E3.3: Sizing Guidance

**Tasks**:

- Create `crates/scoring/src/sizing.rs`
- Implement sizing curve from SPEC.md:
  - `base = 0.10`
  - `haircut = 1.0 - risk_score`
  - `liq = 0.5 + 0.5 * liquidity_score`
  - `max_position_pct = clamp(base * haircut * liq, 0.01, 0.10)`
- Add property tests for bounds

**Acceptance Criteria**:

- Output always in [0.01, 0.10]
- Worst risk/liquidity → 0.01
- Best risk/liquidity → 0.10

**Files**:

```
crates/scoring/src/sizing.rs
```

### E3.4: Overall Score Computation

**Tasks**:

- Create `crates/scoring/src/score.rs`
- Implement weighted scoring formula
- Load weights from config
- Persist to `scores_latest` and `recs_latest`
- Run every 120 seconds

**Acceptance Criteria**:

- Bounded batch processing (max 10000 markets per tick)
- Batch upserts to DB
- Scoring completes in <10s for 10k markets

**Files**:

```
crates/scoring/src/score.rs
crates/scoring/src/main.rs
```

---

## Epic E4: API Service

**Goal**: Stable JSON contracts for UI and future automation.

### E4.1: Implement /v1/opportunities

**Tasks**:

- Create `crates/api/src/opportunities.rs`
- Implement endpoint with filters:
  - `min_score`
  - `max_t_remaining_sec`
  - `max_risk_score`
  - `has_flags`
  - `cursor`, `limit`
- Add pagination (cursor-based)
- Return JSON per SPEC.md contract

**Acceptance Criteria**:

- Does not load full table into memory
- Uses streaming or bounded batches
- Pagination works correctly

**Files**:

```
crates/api/src/lib.rs
crates/api/src/opportunities.rs
```

### E4.2: Implement /v1/market/:id and /v1/config

**Tasks**:

- Create `crates/api/src/market.rs`
- Fetch market + rules + quote + score + rec
- Create `crates/api/src/config.rs`
- Return current configuration

**Acceptance Criteria**:

- Matches JSON contracts in SPEC.md
- Returns 404 if market not found

**Files**:

```
crates/api/src/market.rs
crates/api/src/config.rs
```

### E4.3: Add /metrics

**Tasks**:

- Add Prometheus metrics:
  - `pm_ingest_markets_total`
  - `pm_ingest_quote_latency_seconds`
  - `pm_score_opportunities_total`
  - `pm_api_requests_total`
- Expose at `GET /metrics`

**Acceptance Criteria**:

- Metrics exposed in Prometheus format
- Includes quote freshness and scoring lag gauges

**Files**:

```
crates/api/src/metrics.rs
crates/api/src/main.rs
```

---

## Epic E5: Next.js UI

**Goal**: Simple, fast UI for monitoring opportunities.

### E5.1: Opportunities Page

**Tasks**:

- Create `web/app/opportunities/page.tsx`
- Fetch from `/v1/opportunities`
- Display table with columns:
  - Score
  - Side
  - Net Yield
  - Time Remaining
  - Liquidity
  - Definition Risk
  - Flags
  - Recommended Size
- Add filters (min score, max risk, etc.)
- Add auto-refresh (SWR, every 30s)

**Acceptance Criteria**:

- Table loads in <1s
- Filters update without page reload
- Score breakdown visible per row

**Files**:

```
web/app/opportunities/page.tsx
web/components/OpportunityTable.tsx
```

### E5.2: Market Detail Page

**Tasks**:

- Create `web/app/market/[id]/page.tsx`
- Fetch from `/v1/market/:id`
- Display:
  - Rule text with highlighted ambiguous phrases
  - Score breakdown bars
  - Quote snapshot
  - Risk flags

**Acceptance Criteria**:

- Evidence spans highlighted in rule text
- Score breakdown uses visual bars

**Files**:

```
web/app/market/[id]/page.tsx
web/components/RuleTextHighlight.tsx
web/components/ScoreBreakdown.tsx
```

---

## Epic E6: Hardening + Observability

**Goal**: Make it reliable enough to run continuously.

### E6.1: Tracing + Structured Logs

**Tasks**:

- Add `tracing` + `tracing-subscriber`
- Emit JSON logs
- Add spans for:
  - Ingest cycle
  - Scoring cycle
  - DB write batches
- No INFO logs in hot paths

**Acceptance Criteria**:

- All logs structured JSON
- Spans include duration

**Files**:

```
crates/*/src/main.rs (add tracing_subscriber init)
```

### E6.2: Graceful Shutdown

**Tasks**:

- Handle SIGTERM/SIGINT
- Finish in-flight work
- Close DB connections cleanly

**Acceptance Criteria**:

- No panics on shutdown
- DB transactions committed or rolled back

**Files**:

```
crates/*/src/main.rs (add shutdown handler)
```

### E6.3: Load Testing

**Tasks**:

- Test with 5k, 10k, 50k markets
- Ensure scoring completes in acceptable time
- Ensure memory bounded

**Acceptance Criteria**:

- Scoring completes <30s for 50k markets
- Memory usage stable under load

**Files**:

```
scripts/load_test.sh
```

---

## Acceptance Tests Summary

| Epic | Key Test                                            |
| ---- | --------------------------------------------------- |
| E0   | `docker compose up` yields healthy services         |
| E1   | `sqlx migrate run` applies cleanly                  |
| E2   | Discovery loop populates markets table              |
| E3   | Golden tests for risk extraction produce same flags |
| E4   | `/v1/opportunities` returns valid JSON              |
| E5   | UI loads opportunities in <1s                       |
| E6   | No panics under load; graceful shutdown             |

---

## Risk Mitigation

| Risk                    | Mitigation                                      |
| ----------------------- | ----------------------------------------------- |
| Polymarket API changes  | VenueClient trait isolates API details          |
| DB migration conflicts  | SQLx offline mode + compile-time checks         |
| Unbounded memory growth | Explicit caps on batch sizes and channels       |
| Scoring takes too long  | Bounded batches; parallel processing where safe |
| UI performance          | Server-side pagination; SWR caching             |

---

## Next Steps

1. Create GitHub issues for each epic/ticket
2. Implement Epic E0 (repo skeleton + CI)
3. Implement Epic E1 (storage + schema)
4. Begin parallel work on E2 (ingest) and E3 (scoring) once E1 done
5. Implement E4 (API) once E3 has stable contracts
6. Implement E5 (UI) once E4 stable
7. Harden in E6

---

## Definition of Done

- All tickets in epic completed
- Tests pass (unit + integration)
- CI green
- Code reviewed
- Documentation updated
- No unbounded resource usage
- No panics in production paths
