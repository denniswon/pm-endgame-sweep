# PM Endgame Sweep - Web UI

## Project Status

**Backend Services: ✅ COMPLETE**

- Phase E0: Repository skeleton + CI ✅
- Phase E1: Storage layer (PostgreSQL + SQLx) ✅
- Phase E2: Polymarket ingestion service ✅
- Phase E3: Scoring service with yield velocity algorithms ✅
- Phase E4: REST API with opportunities and market endpoints ✅
- Phase E4.3: Prometheus metrics infrastructure ✅

**Frontend UI: ✅ COMPLETE**

- Phase E5: Next.js 15 UI with real-time updates ✅

## Quick Start (Backend)

```bash
# Setup database
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/pm_endgame"
cargo sqlx migrate run

# Run services
cargo run --bin pm-ingest    # Ingestion service
cargo run --bin pm-score      # Scoring service
cargo run --bin pm-api        # API service (port 3000)
```

## Quick Start (Frontend)

```bash
cd web
yarn install
yarn dev  # Runs on port 3001
```

Open [http://localhost:3001](http://localhost:3001) to view the UI.

## Full Stack Usage

1. Start the backend services:

   ```bash
   # Terminal 1: API service
   cargo run --bin pm-api

   # Terminal 2: Scoring service
   cargo run --bin pm-score

   # Terminal 3: Ingestion service
   cargo run --bin pm-ingest
   ```

2. Start the frontend:

   ```bash
   # Terminal 4: Next.js UI
   cd web && yarn dev
   ```

3. View the app at [http://localhost:3001](http://localhost:3001)

## Implementation Details

### API Client (`lib/api.ts`)

```typescript
const API_BASE = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3000";

export async function getOpportunities(params?: {
  min_score?: number;
  max_risk_score?: number;
  limit?: number;
  offset?: number;
}) {
  const query = new URLSearchParams();
  if (params?.min_score) query.set("min_score", params.min_score.toString());
  if (params?.max_risk_score)
    query.set("max_risk_score", params.max_risk_score.toString());
  if (params?.limit) query.set("limit", params.limit.toString());
  if (params?.offset) query.set("offset", params.offset.toString());

  const res = await fetch(`${API_BASE}/v1/opportunities?${query}`);
  if (!res.ok) throw new Error("Failed to fetch opportunities");
  return res.json();
}

export async function getMarket(id: string) {
  const res = await fetch(`${API_BASE}/v1/market/${id}`);
  if (!res.ok) throw new Error("Failed to fetch market");
  return res.json();
}
```

### Key Components Implemented

```
app/
├── page.tsx                    # Home/Opportunities page
├── market/[id]/page.tsx       # Market details
├── layout.tsx                 # Root layout with nav
├── error.tsx                  # Global error boundary
└── loading.tsx                # Global loading state

components/
├── ui/                        # shadcn/ui components
│   ├── table.tsx
│   ├── badge.tsx
│   ├── card.tsx
│   └── skeleton.tsx
└── features/
    ├── opportunities-table.tsx # Main table with SWR
    └── market-card.tsx         # Market details card

lib/
├── api.ts                     # API client functions
└── utils.ts                   # Utility functions

types/
└── api.ts                     # TypeScript type definitions
```

## API Endpoints Available

- `GET /health` - Health check
- `GET /metrics` - Prometheus metrics
- `GET /v1/opportunities` - List recommendations
  - Query params: `min_score`, `max_risk_score`, `max_t_remaining_sec`, `has_flags`, `limit`, `offset`
- `GET /v1/market/:id` - Market details

## Features Implemented

- ✅ Opportunities dashboard with real-time updates (30s refresh)
- ✅ Market details page with comprehensive information
- ✅ Error boundaries on all routes
- ✅ Loading skeletons for async components
- ✅ Mobile responsive design with Tailwind CSS
- ✅ TypeScript strict mode with full type safety
- ✅ SWR data fetching with automatic revalidation
- ✅ Server Components for optimal performance

## Documentation

See `CLAUDE.md` for detailed Next.js conventions and best practices.

## Architecture Diagram

```
┌─────────────────┐
│   Next.js UI    │
│   (Port 3001)   │
└────────┬────────┘
         │ HTTP
         ↓
┌─────────────────┐
│   API Service   │
│   (Port 3000)   │
└────────┬────────┘
         │
    ┌────┴────┐
    ↓         ↓
┌────────┐ ┌──────────┐
│ Ingest │ │  Scoring │
│ Service│ │  Service │
└───┬────┘ └────┬─────┘
    │           │
    └─────┬─────┘
          ↓
    ┌──────────┐
    │PostgreSQL│
    └──────────┘
```

## Project Status Summary

✅ **Backend Services**: Fully operational

- PostgreSQL database with SQLx
- Polymarket ingestion service
- Scoring engine with yield velocity algorithms
- REST API with filtering and pagination
- Prometheus metrics

✅ **Frontend UI**: Production-ready

- Next.js 15 with App Router
- Real-time data with SWR (30s refresh)
- Responsive design with Tailwind CSS
- Full TypeScript type safety
- Error boundaries and loading states

🚀 **Ready for Production**: All phases (E0-E5) complete!
