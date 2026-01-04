# CLAUDE.md - Next.js Frontend Guidelines

## Tech Stack

| Category      | Technology                        | Version |
| ------------- | --------------------------------- | ------- |
| Framework     | Next.js (App Router)              | 15.x    |
| React         | React (Server Components first)   | 19.x    |
| Language      | TypeScript (Strict mode)          | 5.x     |
| Styling       | Tailwind CSS                      | 4.x     |
| UI Components | shadcn/ui patterns                | -       |
| Icons         | lucide-react                      | -       |
| Validation    | zod                               | -       |
| Unit Testing  | Vitest + React Testing Library    | -       |
| E2E Testing   | Playwright                        | -       |
| State         | Server Actions + URL State (nuqs) | -       |

---

## Architecture Guidelines

### Server Components First

Use Server Components by default. Only add `'use client'` when the component requires:

- React hooks (`useState`, `useEffect`, `useRef`)
- Browser APIs (`window`, `localStorage`, `navigator`)
- Event handlers that need client-side state
- Third-party client-only libraries

### Rendering Strategy Decision Tree

```
Is data fetching needed?
├── Yes → Use async Server Component
│         └── Need real-time updates? → Add SWR in client wrapper
└── No → Does it need interactivity?
         ├── Yes → 'use client' (leaf node only)
         └── No → Keep as Server Component
```

### Data Fetching Rules

- **Server Components**: Use `async/await` directly in the component
- **Client Components**: Use SWR with 30-second refresh interval
- **Never use**: `useEffect` for data fetching
- **Parallel fetching**: Avoid waterfalls by fetching data in parallel

### Error Handling

Every route segment must have:

- `error.tsx` - Error boundary for the segment
- `loading.tsx` - Loading state with skeleton components

---

## Code Style

### File Naming Conventions

| Type         | Convention                 | Example                  |
| ------------ | -------------------------- | ------------------------ |
| Components   | `kebab-case.tsx`           | `market-card.tsx`        |
| Pages/Routes | lowercase                  | `page.tsx`, `layout.tsx` |
| Utilities    | `kebab-case.ts`            | `format-date.ts`         |
| Types        | `kebab-case.ts`            | `api.ts`                 |
| Tests        | `*.test.ts` or `*.spec.ts` | `api.test.ts`            |
| E2E Tests    | `*.spec.ts`                | `opportunities.spec.ts`  |

### Import Ordering

```typescript
// 1. React/Next.js imports
import { Suspense } from "react";
import Link from "next/link";

// 2. Third-party imports
import { z } from "zod";

// 3. Internal imports (absolute paths)
import { fetchOpportunities } from "@/lib/api";
import { Badge } from "@/components/ui/badge";

// 4. Relative imports
import { ScoreBreakdown } from "./score-breakdown";

// 5. Types (if separate)
import type { Opportunity } from "@/types/api";
```

### Component Structure

```typescript
// 1. Imports (ordered as above)
// 2. Types/interfaces
// 3. Component definition
// 4. Helper functions (if small, else extract to lib/)
```

### Styling Rules

- Use Tailwind CSS utility classes exclusively
- Use theme tokens: `text-primary`, `bg-muted`, `border-border`
- Avoid arbitrary values: `text-[#333]`, `p-[13px]`
- Use responsive prefixes: `md:`, `lg:` for breakpoints
- Use state variants: `hover:`, `focus:`, `disabled:`

---

## Testing Strategy

### Unit Tests (Vitest)

Location: Same directory as component or in `tests/`

```typescript
// component.test.tsx
import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { Badge } from "./badge";

describe("Badge", () => {
  it("should render children correctly", () => {
    render(<Badge>Test</Badge>);
    expect(screen.getByText("Test")).toBeInTheDocument();
  });

  it("should apply variant styles", () => {
    render(<Badge variant="destructive">Error</Badge>);
    expect(screen.getByText("Error")).toHaveClass("bg-destructive");
  });
});
```

### E2E Tests (Playwright)

Location: `e2e/` directory

```typescript
// e2e/opportunities.spec.ts
import { test, expect } from "@playwright/test";

test.describe("Opportunities Page", () => {
  test("should display opportunities table", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByRole("table")).toBeVisible();
  });
});
```

### Test Naming Convention

- `describe`: What component/feature
- `it`/`test`: Should + expected behavior

### Commands

```bash
yarn test          # Run unit tests
yarn test:coverage # Run with coverage
yarn test:e2e      # Run Playwright tests
```

---

## Performance Patterns

### Code Splitting

```typescript
// Use next/dynamic for heavy components
import dynamic from "next/dynamic";

const ScoreChart = dynamic(() => import("./score-chart"), {
  loading: () => <Skeleton className="h-64 w-full" />,
  ssr: false, // Only if component uses browser APIs
});
```

### Image Optimization

```typescript
// Always use next/image with explicit dimensions
import Image from "next/image";

<Image
  src="/logo.png"
  alt="PM Endgame Sweep"
  width={120}
  height={40}
  priority // Only for above-the-fold images
/>;
```

### Loading States

```typescript
// Use Suspense boundaries for streaming
import { Suspense } from "react";

export default function Page() {
  return (
    <Suspense fallback={<TableSkeleton />}>
      <OpportunitiesTable />
    </Suspense>
  );
}
```

### Caching

- Use `unstable_cache` for expensive computations
- Set appropriate `revalidate` values for ISR
- Use SWR's `dedupingInterval` to prevent duplicate requests

---

## Security Practices

### Environment Variables

```typescript
// Only NEXT_PUBLIC_ prefixed vars are exposed to client
// Use zod to validate environment variables

import { z } from "zod";

const envSchema = z.object({
  NEXT_PUBLIC_API_URL: z.string().url(),
});

export const env = envSchema.parse({
  NEXT_PUBLIC_API_URL: process.env.NEXT_PUBLIC_API_URL,
});
```

### Input Validation

```typescript
// Always validate external data with zod
const OpportunitySchema = z.object({
  market_id: z.string(),
  overall_score: z.number().min(0).max(1),
  risk_flags: z.array(
    z.object({
      code: z.string(),
      severity: z.enum(["low", "medium", "high"]),
    })
  ),
});

// Validate API responses
const data = OpportunitySchema.parse(await response.json());
```

### XSS Prevention

- Never use `dangerouslySetInnerHTML` without sanitization
- Escape user-generated content before rendering
- Use React's built-in escaping (JSX automatically escapes)

---

## Accessibility

### Required Practices

- All interactive elements must have ARIA labels
- All images must have meaningful `alt` text
- Focus states must be visible for keyboard navigation
- Color contrast must meet WCAG AA standards
- Form inputs must have associated labels

### Focus Management

```typescript
// Use useRef for programmatic focus
const inputRef = useRef<HTMLInputElement>(null);

useEffect(() => {
  if (error) {
    inputRef.current?.focus();
  }
}, [error]);
```

### Keyboard Navigation

- All clickable elements must be focusable
- Custom components must handle `Enter` and `Space` for activation
- Modals must trap focus and restore on close

---

## Anti-Patterns to Avoid

### Data Fetching

```typescript
// BAD: useEffect for data fetching
useEffect(() => {
  fetch("/api/data").then(setData);
}, []);

// GOOD: Server Component with async/await
async function DataComponent() {
  const data = await fetchData();
  return <Display data={data} />;
}
```

### Styling

```typescript
// BAD: Arbitrary values
<div className="text-[#333] p-[13px]">

// GOOD: Theme tokens
<div className="text-foreground p-3">

// BAD: Inline styles
<div style={{ marginTop: '20px' }}>

// GOOD: Tailwind utilities
<div className="mt-5">
```

### TypeScript

```typescript
// BAD: any type
function process(data: any) {}

// GOOD: Explicit types
function process(data: Opportunity) {}

// BAD: Type assertions without validation
const data = response as Opportunity;

// GOOD: Runtime validation
const data = OpportunitySchema.parse(response);
```

### General

- No `console.log` in production code (use proper error handling)
- No commented-out code (use git history)
- No hardcoded API URLs (use environment variables)
- No synchronous data fetching in components

---

## PM Endgame Sweep UI Specifics

### API Integration

- Base URL: Configured via `NEXT_PUBLIC_API_URL`
- Endpoints:
  - `GET /v1/opportunities` - Paginated recommendations with filters
  - `GET /v1/market/:id` - Market details with scores and rules
  - `GET /healthz` - Health check
  - `GET /metrics` - Prometheus metrics

### Key Features

1. **Opportunities Dashboard**

   - Real-time table with 30s auto-refresh (SWR)
   - Filters: min_score, max_risk, has_flags, time_remaining
   - Sortable columns: Score, Yield, Liquidity, Risk
   - Cursor-based pagination

2. **Market Details View**

   - Market metadata and status
   - Quote spreads (YES/NO bid/ask)
   - Rule text with risk flag highlighting
   - Score breakdown visualization
   - Recommendation summary with sizing guidance

3. **Data Refresh Strategy**
   - SWR with `refreshInterval: 30000`
   - Skeleton loading states
   - Error boundaries for failed requests
   - Stale-while-revalidate pattern

### Component Structure

```
app/
├── page.tsx                    # Opportunities dashboard
├── market/[id]/page.tsx        # Market details
├── layout.tsx                  # Root layout with navigation
├── error.tsx                   # Global error boundary
├── loading.tsx                 # Global loading state
└── globals.css                 # Global styles and Tailwind

components/
├── ui/                         # shadcn/ui base components
│   ├── badge.tsx
│   ├── card.tsx
│   ├── skeleton.tsx
│   └── table.tsx
└── features/                   # Domain-specific components
    ├── opportunities-table.tsx
    ├── market-card.tsx
    ├── filters.tsx
    └── score-breakdown.tsx

lib/
├── api.ts                      # API client functions
└── utils.ts                    # Utility functions

types/
└── api.ts                      # TypeScript types for API

e2e/
├── opportunities.spec.ts       # E2E tests
└── market-details.spec.ts
```

---

## Development Commands

```bash
# Development
yarn dev              # Start dev server (port 3000)

# Build & Production
yarn build            # Production build
yarn start            # Start production server

# Quality
yarn lint             # ESLint
yarn tsc --noEmit     # Type check

# Testing
yarn test             # Vitest unit tests
yarn test:coverage    # With coverage report
yarn test:e2e         # Playwright E2E tests
```

---

## AI Agent Collaboration Tips

### Effective Prompting

- Break large features into small, testable chunks
- Provide existing component examples when requesting new ones
- Ask for tests alongside implementation
- Specify the exact file path for new components

### Iteration Pattern

1. Request implementation with clear requirements
2. Review generated code for patterns and conventions
3. Ask for refinements or tests if needed
4. Verify against TypeScript and ESLint

### Example Prompts

```
"Create a filter component for the opportunities table.
Place it in components/features/filters.tsx.
It should filter by min_score (0-1 range) and has_flags (boolean).
Follow the existing shadcn/ui patterns from components/ui/."

"Add unit tests for the Badge component.
Use the same patterns as lib/api.test.ts.
Test variant prop and custom className merging."
```

---

## Getting Help

- **Root guidelines**: See root `CLAUDE.md` for backend and general practices
- **API contracts**: See `docs/SPEC.md` for JSON schemas
- **Testing docs**: See `TESTING.md` for detailed testing guide
- **Component patterns**: Reference existing components in `components/ui/`
