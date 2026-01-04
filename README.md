# PM Endgame Sweep

A systematic prediction market scanner and opportunity detector focused on "endgame sweep" convergence harvesting strategies.

## Overview

PM Endgame Sweep identifies late-stage prediction market opportunities where outcomes have effectively converged but pricing still offers small residual spreads.
The system treats this as probability carry with embedded jump risk, emphasizing:

- **Explainable scoring**: every recommendation includes risk flags and breakdown
- **Bounded resources**: no unbounded memory growth or queue sizes
- **Resilient operation**: continues serving last-known data during outages
- **Modular venues**: Polymarket in v0.1, extensible to Kalshi and others

## Architecture

The system consists of four services:

1. **pm-ingest** - Discovers markets, polls quotes, extracts rules (Rust + Tokio)
2. **pm-score** - Computes opportunity scores and risk decomposition (Rust + Tokio)
3. **pm-api** - Read-only REST API (Rust + Axum)
4. **ui** - Dashboard for opportunities and market detail (Next.js)

All services share a PostgreSQL database with bounded retention policies.

## Quick Start

### Prerequisites

- Rust 1.93+ (MSRV)
- Node.js 20+
- PostgreSQL 15+
- Docker & Docker Compose (optional)

### Local Development

```bash
# Clone repository
git clone <repo-url>
cd pm-endgame-sweep

# Start PostgreSQL
docker compose up -d postgres

# Run migrations
sqlx migrate run

# Start services
cargo run --bin pm-ingest &
cargo run --bin pm-score &
cargo run --bin pm-api &

# Start UI
cd web
yarn install
yarn dev
```

### Docker Compose

```bash
docker compose up
```

Dashboard will be available at `http://localhost:3000`.
API at `http://localhost:8080`.
Metrics at `http://localhost:8080/metrics`.

## Project Structure

```
.
├── crates/
│   ├── domain/       # Core types and traits
│   ├── storage/      # PostgreSQL layer (SQLx)
│   ├── ingest/       # Market discovery and quote polling
│   ├── scoring/      # Opportunity scoring engine
│   └── api/          # REST API service
├── web/              # Next.js dashboard
├── migrations/       # Database schema
├── config/           # Configuration templates
└── docs/             # Detailed documentation
```

## Configuration

Default configuration lives in `config/default.yaml`.
Override via environment variables or `config/local.yaml`:

```yaml
# config/local.yaml
ingest:
  cadence:
    quotes_sec: 60
    discovery_sec: 1800

scoring:
  cadence_sec: 120
  weights:
    w1: 0.45 # yield velocity
    w2: 0.25 # net yield
    w3: 0.15 # liquidity
    w4: 0.10 # definition risk
    w5: 0.05 # staleness

database:
  url: postgres://localhost/pm_endgame
```

## Strategy

PM Endgame Sweep implements a "probability carry" strategy:

1. **Eligibility filtering**: only markets with <14 days to expiry and fresh quotes
2. **Risk extraction**: parse rule text for ambiguity flags
3. **Yield calculation**: compute net yield after fees and staleness penalties
4. **Liquidity scoring**: penalize wide spreads and thin depth
5. **Sizing guidance**: conservative caps (1-10% NAV) based on risk decomposition

See [SPEC.md](./SPEC.md) for detailed scoring formulas and risk model.

## API Examples

### List Opportunities

```bash
curl http://localhost:8080/v1/opportunities?min_score=0.7&limit=20
```

### Get Market Detail

```bash
curl http://localhost:8080/v1/market/0xabc123
```

### Health Check

```bash
curl http://localhost:8080/healthz
```

## Development Guidelines

- Follow the [Rust style guide](./docs/rust-style.md)
- All public items require rustdoc
- No `unwrap`/`expect` in production paths
- Use `cargo fmt`, `cargo clippy` before committing
- Write tests for scoring logic and risk extraction

## Observability

Metrics are exposed at `/metrics` in Prometheus format:

- `pm_ingest_markets_total` - Number of markets discovered
- `pm_ingest_quote_latency_seconds` - Quote fetch latency
- `pm_score_opportunities_total` - Number of opportunities scored
- `pm_api_requests_total` - API request count

Structured logs use JSON format with tracing spans.

## Roadmap

### v0.1 (Current)

- [x] Polymarket ingestion
- [x] Definition risk extraction
- [x] Endgame sweep scoring
- [x] REST API
- [x] Next.js dashboard

### v0.2 (Planned)

- [ ] Kalshi venue support
- [ ] Portfolio tracking (manual entry)
- [ ] WebSocket live updates
- [ ] Rotation recommendations

### v0.3 (Future)

- [ ] Deribit IV comparison
- [ ] Cross-venue arbitrage detection
- [ ] Advanced depth analysis
- [ ] Alerting system

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md).

## License

MIT
