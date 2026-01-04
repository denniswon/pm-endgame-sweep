# CLAUDE.md - AI Agent Guidelines

## Project Overview

PM Endgame Sweep is a systematic prediction market scanner and opportunity detector focused on "endgame sweep" convergence harvesting strategies.
The system identifies late-stage prediction market opportunities where outcomes have effectively converged but pricing still offers small residual spreads.

This codebase prioritizes:

- **Bounded resources**: no unbounded memory growth, queues, or database tables
- **Resilient operation**: continues serving last-known data during outages
- **Explainable scoring**: every recommendation includes risk flags and breakdown
- **Production-grade quality**: no panics in production paths, comprehensive error handling

---

## Architecture

Four services compose the system:

| Service       | Description                                        | Tech         |
| ------------- | -------------------------------------------------- | ------------ |
| **pm-ingest** | Discovers markets, polls quotes, extracts rules    | Rust + Tokio |
| **pm-score**  | Computes opportunity scores and risk decomposition | Rust + Tokio |
| **pm-api**    | Read-only REST API                                 | Rust + Axum  |
| **ui**        | Dashboard for opportunities and market detail      | Next.js 15+  |

All Rust services share a PostgreSQL database with bounded retention policies.

```
pm-endgame-sweep/
├── crates/
│   ├── domain/       # Core types, traits, scoring models
│   ├── storage/      # PostgreSQL layer (SQLx, compile-time checked)
│   ├── ingest/       # Market discovery and quote polling
│   ├── scoring/      # Opportunity scoring engine
│   └── api/          # REST API service (Axum)
├── web/              # Next.js dashboard (see web/CLAUDE.md)
├── migrations/       # Database schema (SQLx migrations)
├── config/           # Configuration templates
└── docs/             # Detailed documentation (SPEC.md, etc.)
```

---

## Development Commands

### Rust

```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace --all-features

# Lint
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run individual services
cargo run --bin pm-ingest
cargo run --bin pm-score
cargo run --bin pm-api

# Security audit
cargo audit
cargo deny check
```

### Database

```bash
# Start PostgreSQL
docker compose up -d postgres

# Run migrations
sqlx migrate run

# Create test database
sqlx database create
```

### Web (Next.js)

```bash
cd web
yarn install
yarn dev          # Development server
yarn build        # Production build
yarn test         # Unit tests (Vitest)
yarn test:e2e     # E2E tests (Playwright)
yarn lint         # ESLint
yarn tsc --noEmit # Type check
```

### Docker

```bash
docker compose up           # All services
docker compose up -d postgres  # PostgreSQL only
```

---

## Language and Framework Stack

### Backend (Rust)

- **Rust**: nightly (see `rust-toolchain.toml`)
- **Async Runtime**: Tokio
- **Web Framework**: Axum 0.7
- **Database**: PostgreSQL 16 with SQLx (compile-time checked queries)
- **Serialization**: serde, serde_json
- **HTTP Client**: reqwest
- **Observability**: tracing, prometheus
- **Error Handling**: thiserror, anyhow

### Frontend (TypeScript)

- **Framework**: Next.js 15+ (App Router)
- **React**: 19+
- **Styling**: Tailwind CSS
- **Components**: shadcn/ui patterns
- **Testing**: Vitest (unit), Playwright (E2E)
- **Validation**: zod

---

## Code Style and Formatting

### General Rules

- Never use emojis or exclamation marks in code, comments, or commit messages
- No PR-specific comments in code; those belong in PR descriptions
- Write comments that explain _why_, not _what_
- Maximum line length: 100 characters

### Rust Naming Conventions

- `snake_case` for functions, methods, variables, and modules
- `PascalCase` for types, traits, and enums
- `SCREAMING_SNAKE_CASE` for constants and statics
- Prefix unused variables with underscore: `_unused`
- Avoid abbreviations unless universally understood (e.g., `tx`, `rx`, `ctx`)
- Use descriptive names: `market_close_time` not `mct`

### Rust Formatting

- Run `cargo fmt --all` before committing
- Run `cargo clippy -- -D warnings`; treat all warnings as errors
- Follow `rustfmt` defaults; do not override unless documented in `rustfmt.toml`

### TypeScript/React Style

- Server Components first; only add `'use client'` for leaf nodes requiring hooks or browser APIs
- Use Tailwind theme values; avoid arbitrary values like `text-[#333]`
- Place components in `@/components/ui` (shadcn) or `@/components/features`
- Use `lucide-react` for icons

### Comment Style

```rust
// Good: explains why
// Skip markets with zero liquidity to avoid division errors in spread calculation
if market.liquidity.is_zero() {
    continue;
}

// Bad: restates the obvious
// Check if liquidity is zero and continue if true
if market.liquidity.is_zero() {
    continue;
}
```

### Documentation Comments

```rust
/// Computes the overall opportunity score for a market.
///
/// Returns early if the market fails eligibility gates.
/// Score components are weighted according to configuration.
pub fn compute_score(&self, market: &Market, quote: &Quote) -> Option<Score> {
    // ...
}
```

---

## Architecture Principles

### Design for Production

- Assume all external calls can fail; implement retries with exponential backoff and jitter
- Design for horizontal scalability from day one
- Prefer stateless components; externalize state to PostgreSQL
- Use dependency injection for testability
- Implement graceful shutdown for all long-running services

### Bounded Resources

- Collections must have size limits to prevent unbounded growth (DoS risk)
- Use bounded channels (`tokio::sync::mpsc`) with explicit backpressure handling
- Never load unbounded data into memory; stream large datasets
- Retention policies enforce bounded database growth

### Concurrency

- Prefer `tokio::spawn` for CPU-light async tasks
- Use `tokio::task::spawn_blocking` for CPU-intensive or blocking operations
- Avoid `Arc<Mutex<T>>` when possible; prefer message passing
- Long-running tasks must check for shutdown signals
- All `.await` points are cancellation points: handle gracefully

---

## Performance Requirements

### Hot Path Optimization

- Minimize allocations in critical sections
- Pre-allocate buffers with `Vec::with_capacity(expected_size)`
- Avoid O(n^2) or worse in any production code path
- Batch database operations to amortize overhead
- Use connection pooling for all external services

### Patterns

```rust
// Prefer: pre-allocated buffers
let mut buffer = Vec::with_capacity(expected_size);

// Prefer: batch database operations
sqlx::query("INSERT INTO quotes_latest ... ON CONFLICT DO UPDATE ...")
    .execute_many(&mut *tx, quotes.iter())
    .await?;

// Avoid: allocating in hot loops
for market in markets.iter() {
    let data = market.to_string(); // allocation per iteration
}
```

---

## Error Handling

- Use `thiserror` for library errors, `anyhow` sparingly in binaries
- Define domain-specific error types; avoid stringly-typed errors
- Propagate errors with context using `.context()` or custom error variants
- Never use `.unwrap()` or `.expect()` in production code paths
- Use `.expect()` only in initialization code with descriptive messages
- Log errors at the point of handling, not propagation

```rust
#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error("failed to fetch quotes for market {market_id}: {source}")]
    QuoteFetch {
        market_id: String,
        #[source]
        source: reqwest::Error,
    },

    #[error("market {market_id} not found in venue response")]
    MarketNotFound { market_id: String },
}
```

---

## Testing Strategy

### Rust Tests

- Unit tests in the same file under `#[cfg(test)]`
- Integration tests in `tests/` directory
- Property-based tests for serialization using `proptest`
- Use `#[tokio::test]` for async tests

### Test Naming

```rust
#[test]
fn compute_score_with_stale_quote_returns_penalty() {}

#[test]
fn parse_rule_text_with_ambiguous_source_flags_definition_risk() {}
```

### Web Tests

- Unit tests with Vitest in same directory as components
- E2E tests with Playwright in `web/e2e/`
- Test files named `*.test.ts` or `*.spec.ts`

### Coverage Requirements

- All public APIs must have unit tests
- Critical scoring logic: happy path, error cases, edge cases
- Risk extraction: test known ambiguous phrases

---

## Security Practices

### General Security

- Never log secrets, private keys, or sensitive data
- Validate all external input at system boundaries
- Pin dependency versions in `Cargo.lock` and `yarn.lock`
- Run `cargo audit` and `cargo deny check` in CI
- Use `zod` for TypeScript input validation

### Input Validation

- All data from external APIs (Polymarket) MUST be validated
- Use defensive parsing: handle malformed data gracefully
- Validate ranges for numeric inputs (prevent overflow)
- Sanitize all data before logging to prevent log injection

### Secrets Management

- Never commit `.env` files or secrets
- Use environment variables for configuration
- Document required environment variables in `config/env.example`

---

## Observability

### Logging

- Use `tracing` for structured logging
- Include correlation IDs in all log spans
- Log levels:
  - ERROR: failures requiring attention; operation did not complete
  - WARN: degraded state; operation succeeded with concerns
  - INFO: significant state changes; startup/shutdown events
  - DEBUG: request flow; intermediate steps
  - TRACE: verbose debugging; data dumps
- Never log at INFO or above in hot paths

```rust
tracing::info!(
    market_id = %market.id,
    quote_age_sec = staleness,
    "quote refreshed"
);

tracing::error!(
    market_id = %market.id,
    error = %e,
    "failed to fetch quote"
);
```

### Metrics

- Prometheus metrics exposed at `/metrics`
- Key metrics:
  - `pm_ingest_markets_total` - Number of markets discovered
  - `pm_ingest_quote_latency_seconds` - Quote fetch latency
  - `pm_score_opportunities_total` - Number of opportunities scored
  - `pm_api_requests_total` - API request count
- Use consistent naming: `pm_<service>_<metric>_<unit>`

---

## CI/CD Guidelines

### Workflows

| Workflow            | Purpose                                    |
| ------------------- | ------------------------------------------ |
| `rust-ci.yml`       | Rust lint, test, build, coverage           |
| `web-ci.yml`        | Web lint, test, build, E2E                 |
| `pr-checks.yml`     | PR metadata, dependency review             |
| `security-scan.yml` | cargo audit, cargo deny, npm audit, CodeQL |

### Requirements

- All PRs must pass: `cargo fmt --check`, `cargo clippy`, `cargo test`
- All PRs must pass: `yarn lint`, `yarn tsc --noEmit`, `yarn test`
- Conventional commit format: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`
- Branch naming: `feat/<description>`, `fix/<description>`, `refactor/<description>`

### Commit Messages

```text
feat: add definition risk extraction from rule text

Parses market resolution rules for ambiguous phrases and
discretionary settlement indicators. Produces risk_flags
for UI display.

Closes #42
```

---

## Database Guidelines

### SQLx

- Use compile-time checked queries with `sqlx::query!` or `sqlx::query_as!`
- Run `sqlx prepare` to generate offline query metadata for CI
- Migrations in `migrations/` directory with timestamped filenames

### Bounded Retention

- `quotes_latest`: only latest snapshot per market
- `quotes_5m`: 5-minute samples retained for 7 days
- Daily retention job cleans old samples

### Query Patterns

```rust
// Batch upserts for bulk operations
sqlx::query!(
    r#"
    INSERT INTO quotes_latest (market_id, as_of, yes_bid, yes_ask, no_bid, no_ask)
    VALUES ($1, $2, $3, $4, $5, $6)
    ON CONFLICT (market_id) DO UPDATE SET
        as_of = EXCLUDED.as_of,
        yes_bid = EXCLUDED.yes_bid,
        yes_ask = EXCLUDED.yes_ask,
        no_bid = EXCLUDED.no_bid,
        no_ask = EXCLUDED.no_ask,
        updated_at = NOW()
    "#,
    market_id, as_of, yes_bid, yes_ask, no_bid, no_ask
)
.execute(&pool)
.await?;
```

---

## API Design

### Endpoints

- `GET /v1/opportunities` - Paginated opportunities with filters
- `GET /v1/market/{market_id}` - Market detail with scores and rules
- `GET /v1/config` - Current scoring configuration
- `GET /healthz` - Liveness probe
- `GET /readyz` - Readiness probe
- `GET /metrics` - Prometheus metrics

### Query Parameters

- `min_score` - Minimum overall score filter
- `max_risk_score` - Maximum risk score filter
- `max_t_remaining_sec` - Maximum time to expiry filter
- `has_flags` - Filter markets with risk flags
- `cursor` - Pagination cursor
- `limit` - Results per page (max 100)

### Response Contracts

See `docs/SPEC.md` for detailed JSON schemas.

---

## Domain-Specific Context

### Key Entities

- **Market**: Polymarket market with metadata, close time, settlement rules
- **Quote**: Top-of-book snapshot (YES/NO bid/ask, spread, staleness)
- **Score**: Computed features, risk decomposition, overall score
- **Recommendation**: Suggested side, entry price, max sizing guidance

### Scoring Model

The system ranks contracts where probability has effectively converged but price still leaves residual spread.
This is treated as carry with embedded jump risk.

**Core features:**

- `gross_yield`: `1.0 - entry_price`
- `net_yield`: yield after fees
- `yield_velocity`: annualized yield based on time remaining
- `liquidity_score`: normalized from spread and freshness
- `definition_risk_score`: extracted from rule text analysis

**Overall score formula:**

```
overall_score = w1 * yield_velocity + w2 * net_yield + w3 * liquidity_score
              - w4 * definition_risk_score - w5 * staleness_penalty
```

### Risk Flags

- `SETTLEMENT_DISCRETION` - Rule contains discretionary language
- `AMBIGUOUS_SOURCE` - Data source not clearly specified
- `UNCLEAR_TIMESTAMP` - Timezone or timing ambiguity
- `MISSING_DEFINITION` - Key terms not defined
- `LIQUIDITY_WARNING` - Wide spread or stale quotes

---

## Anti-Patterns to Avoid

### Rust

- Do not use `clone()` to satisfy the borrow checker without understanding the cost
- Do not use `Box<dyn Error>` in library code
- Do not silence warnings with `#[allow(...)]` without justification
- Do not use `lazy_static!`; prefer `std::sync::OnceLock`
- Do not mix sync and async code without explicit boundaries
- Do not use panics for control flow
- Do not allocate in hot loops; hoist allocations outside
- Do not use unbounded channels or collections
- Do not block the async runtime with synchronous I/O

### TypeScript

- Do not use `useEffect` for data fetching; use Server Components or SWR
- Do not use arbitrary Tailwind values; use theme tokens
- Do not add `'use client'` to components that don't need interactivity
- Do not skip error boundaries; every route needs `error.tsx`

### General

- Do not commit `.env` files or secrets
- Do not leave commented-out code; delete it
- Do not write comments that restate the obvious

---

## Code Review Standards

### Priority Order

1. **Security**: Input validation, secret handling, unsafe code
2. **Correctness**: Business logic, edge cases, data integrity
3. **Reliability**: Error handling, resource management, panic-free operation
4. **Performance**: Scalability, efficient data structures, batching
5. **Maintainability**: Documentation, testing, code clarity

### Author Checklist

Before requesting review:

- [ ] Run `cargo fmt` and `cargo clippy`
- [ ] All tests pass (`cargo test --workspace`)
- [ ] Run `yarn lint` and `yarn tsc --noEmit` for web changes
- [ ] Critical paths have test coverage
- [ ] Public APIs are documented
- [ ] No sensitive data in code or logs

### Blocking Issues

- Security vulnerabilities (input validation, secret exposure)
- Unbounded resource growth (DoS risk)
- Panic in production paths
- Data corruption or integrity issues

### Request Changes

- Poor error handling (unwrap/expect in non-test code)
- Missing tests for critical paths
- Performance issues (N+1 queries, excessive cloning)
- Logging sensitive data

### Suggestions

- Code duplication that could be extracted
- Naming clarity improvements
- Additional metrics or observability
- Rust/TypeScript idioms

---

## AI Agent Guidelines

### Code Generation

- Generate complete, compilable code; no placeholder comments like "implement this"
- Include all necessary imports
- Match existing code style in the repository
- Prefer incremental changes over large rewrites
- Always consider performance implications of generated code

### Performance Review

Before finalizing any code, verify:

- [ ] No unnecessary allocations in hot paths
- [ ] Appropriate data structures for access patterns
- [ ] Bounded queues and collections
- [ ] Proper error handling without panics
- [ ] Connection/resource pooling where applicable
- [ ] Batch operations where possible

### When Uncertain

- Ask clarifying questions before generating complex architectural changes
- Propose multiple approaches for non-trivial decisions
- Flag potential performance or security implications explicitly
- Default to the more robust solution when trade-offs are unclear

### Output Format

- For new files: provide complete file content
- For modifications: provide clear diff context or full replacement
- Match the formatting of surrounding code

---

## Getting Help

- **Architecture decisions**: See `docs/SPEC.md` for system design
- **Scoring model**: See "Scoring Model" section in `docs/SPEC.md`
- **API contracts**: See "JSON Output Contracts" in `docs/SPEC.md`
- **Frontend specifics**: See `web/CLAUDE.md`
- **Performance concerns**: Run benchmarks and profile before optimizing
