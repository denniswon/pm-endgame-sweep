# PM Endgame Sweep - Web UI

## Project Status

**Backend Services: ✅ COMPLETE**
- Phase E0: Repository skeleton + CI ✅
- Phase E1: Storage layer (PostgreSQL + SQLx) ✅
- Phase E2: Polymarket ingestion service ✅
- Phase E3: Scoring service with yield velocity algorithms ✅
- Phase E4: REST API with opportunities and market endpoints ✅
- Phase E4.3: Prometheus metrics infrastructure ✅

**Frontend UI: 📋 READY FOR IMPLEMENTATION**

## Quick Start (Backend)

```bash
# Setup database
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/pm_endgame"
cd crates/storage
cargo sqlx migrate run

# Run services
cargo run --bin pm-ingest    # Ingestion service
cargo run --bin pm-score      # Scoring service
cargo run --bin pm-api        # API service (port 3000)
```

## Next.js UI Implementation Guide

### 1. Initialize Next.js Project

```bash
cd web
npx create-next-app@latest . \
  --typescript \
  --tailwind \
  --app \
  --no-src-dir \
  --import-alias "@/*" \
  --turbopack
```

### 2. Install Dependencies

```bash
yarn add swr zod lucide-react nuqs
yarn add -D @types/node @types/react @types/react-dom
```

### 3. Create API Client (`lib/api.ts`)

```typescript
const API_BASE = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3000';

export async function getOpportunities(params?: {
  min_score?: number;
  max_risk_score?: number;
  limit?: number;
  offset?: number;
}) {
  const query = new URLSearchParams();
  if (params?.min_score) query.set('min_score', params.min_score.toString());
  if (params?.max_risk_score) query.set('max_risk_score', params.max_risk_score.toString());
  if (params?.limit) query.set('limit', params.limit.toString());
  if (params?.offset) query.set('offset', params.offset.toString());

  const res = await fetch(`${API_BASE}/v1/opportunities?${query}`);
  if (!res.ok) throw new Error('Failed to fetch opportunities');
  return res.json();
}

export async function getMarket(id: string) {
  const res = await fetch(`${API_BASE}/v1/market/${id}`);
  if (!res.ok) throw new Error('Failed to fetch market');
  return res.json();
}
```

### 4. Create Opportunities Page (`app/page.tsx`)

```typescript
import { Suspense } from 'react';
import { OpportunitiesTable } from '@/components/features/opportunities-table';
import { Filters } from '@/components/features/filters';

export default function Home() {
  return (
    <main className="container mx-auto px-4 py-8">
      <h1 className="text-4xl font-bold mb-8">PM Endgame Sweep</h1>

      <Filters />

      <Suspense fallback={<div>Loading...</div>}>
        <OpportunitiesTable />
      </Suspense>
    </main>
  );
}
```

### 5. Create Opportunities Table Component

Use `swr` for data fetching with 30s refresh interval:

```typescript
'use client';

import useSWR from 'swr';
import { getOpportunities } from '@/lib/api';

export function OpportunitiesTable() {
  const { data, error, isLoading } = useSWR(
    'opportunities',
    () => getOpportunities(),
    { refreshInterval: 30000 }
  );

  if (isLoading) return <TableSkeleton />;
  if (error) return <ErrorState error={error} />;

  return (
    <table className="w-full">
      <thead>
        <tr>
          <th>Score</th>
          <th>Side</th>
          <th>Net Yield</th>
          <th>Time Remaining</th>
          <th>Liquidity</th>
          <th>Risk</th>
        </tr>
      </thead>
      <tbody>
        {data.opportunities.map(opp => (
          <OpportunityRow key={opp.market_id} opportunity={opp} />
        ))}
      </tbody>
    </table>
  );
}
```

### 6. Key Components to Build

```
components/
├── ui/                          # shadcn/ui components
│   ├── table.tsx
│   ├── badge.tsx
│   ├── card.tsx
│   ├── skeleton.tsx
│   └── button.tsx
└── features/
    ├── opportunities-table.tsx   # Main table with SWR
    ├── opportunity-row.tsx       # Single table row
    ├── market-card.tsx           # Market details card
    ├── filters.tsx               # Filter controls
    ├── score-breakdown.tsx       # Score visualization
    └── risk-badges.tsx           # Risk flag display
```

### 7. Environment Variables

Create `.env.local`:

```
NEXT_PUBLIC_API_URL=http://localhost:3000
```

## API Endpoints Available

- `GET /health` - Health check
- `GET /metrics` - Prometheus metrics
- `GET /v1/opportunities` - List recommendations
  - Query params: `min_score`, `max_risk_score`, `max_t_remaining_sec`, `has_flags`, `limit`, `offset`
- `GET /v1/market/:id` - Market details

## Type Definitions

Create `types/api.ts`:

```typescript
export interface Opportunity {
  market_id: string;
  as_of: string;
  recommended_side: string;
  entry_price: number;
  expected_payout: number;
  max_position_pct: number;
  risk_score: number;
  risk_flags: RiskFlag[];
  notes?: string;
}

export interface RiskFlag {
  code: string;
  severity: string;
  evidence_spans: any[];
}

export interface OpportunitiesResponse {
  opportunities: Opportunity[];
  total: number;
  limit: number;
  offset: number;
}
```

## Deployment

### Development
```bash
yarn dev  # Runs on port 3001 (to avoid conflict with API on 3000)
```

### Production
```bash
yarn build
yarn start
```

## Features to Implement

- [ ] Opportunities dashboard with real-time updates
- [ ] Market details page
- [ ] Filtering and sorting
- [ ] Pagination
- [ ] Error boundaries
- [ ] Loading skeletons
- [ ] Dark mode toggle
- [ ] Export to CSV
- [ ] Mobile responsive design

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

## Current Implementation Status

✅ All backend services operational
✅ API endpoints functional
✅ Database schema complete
✅ Metrics infrastructure ready
📋 Frontend UI ready for development

**Next Step**: Run the Next.js initialization command above and follow the implementation guide.
