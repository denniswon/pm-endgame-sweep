# Project Memory: Next.js Production App

## Tech Stack

- Next.js 15+ (App Router), React 19+, Tailwind CSS, TypeScript (Strict).
- State: Server Actions + URL State (nuqs) + React 'use' for data.

## Architecture Guidelines

- **Server First:** Favor Server Components. Only use 'use client' for leaf nodes requiring interactivity.
- **Data Fetching:** No `useEffect` for data. Use async Server Components or SWR for client-side.
- **Safety:** Use `zod` for all Server Action inputs and environment variables.
- **Error Handling:** Every route segment must have an `error.tsx` and `loading.tsx`.

## Standards & Commands

- Build: `yarn build` | Test: `yarn test` | Lint: `yarn lint` (I prefer yarn over npm)
- Components: Use `shadcn/ui` patterns. Place in `@/components/ui` or `@/components/features`.
- **Framework:** React 19 / Next.js App Router.
- **Client Directives:** Only add 'use client' if the component uses hooks (useState, useEffect) or browser APIs.
- **Styling:** Use Tailwind CSS utility classes. Avoid arbitrary values; use the theme (e.g., `text-primary` not `text-[#333]`).
- **Icons:** Use `lucide-react`.
- **Performance:** Memoize expensive components only if requested. Favor `next/image` for all images.
- **Accessibility:** Ensure all interactive elements have ARIA labels and focus states.

## PM Endgame Sweep UI Specifics

### API Integration

- Base URL: `http://localhost:3000` (configurable via `NEXT_PUBLIC_API_URL`)
- Endpoints:
  - `GET /v1/opportunities` - Paginated recommendations with filters
  - `GET /v1/market/:id` - Market details
  - `GET /health` - Health check
  - `GET /metrics` - Prometheus metrics

### Key Features

1. **Opportunities Dashboard**

   - Real-time opportunities table with auto-refresh (30s)
   - Filters: min_score, max_risk, has_flags, time_remaining
   - Sortable columns: Score, Yield, Liquidity, Risk
   - Pagination with cursor-based navigation

2. **Market Details View**

   - Comprehensive market information
   - Quote spreads (YES/NO bid/ask)
   - Rule text and risk flags
   - Score breakdown visualization
   - Recommendation summary

3. **Data Refresh Strategy**
   - Use SWR with 30-second refresh interval
   - Show loading states with skeleton components
   - Error boundaries for failed requests
   - Optimistic updates where applicable

### Component Structure

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
    ├── opportunities-table.tsx
    ├── market-card.tsx
    ├── filters.tsx
    └── score-breakdown.tsx
```

### Type Safety

- Generate TypeScript types from API responses
- Use zod schemas for validation
- Strict mode enabled in tsconfig.json
