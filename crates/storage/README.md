# PM Storage - Storage Layer

SQLx-based PostgreSQL storage layer for PM Endgame Sweep.

## Setup

The storage layer requires running PostgreSQL database and migrations before it can compile.

### Running Migrations

```bash
# Set database URL
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/pm_endgame"

# Run migrations
sqlx migrate run --source migrations

# Generate SQLx offline data for compile-time query checking
cargo sqlx prepare --workspace
```

### SQLx Offline Mode

This crate uses SQLx's compile-time query checking. To enable offline compilation without a live database:

1. Run migrations on a development database
2. Run `cargo sqlx prepare` to generate `.sqlx/` directory
3. Commit `.sqlx/` to version control

## Modules

- `markets` - Market and outcome CRUD
- `quotes` - Quote snapshots (latest + 5m samples)
- `rules` - Rule text and risk extraction
- `scores` - Computed opportunity scores
- `recs` - Trading recommendations

All modules follow the same pattern:
- Batch operations use transactions
- Upserts handle conflicts automatically
- Proper error types with `thiserror`
- No `unwrap`/`expect` in production code
