# Testing Strategy for PM Endgame Sweep Next.js UI

## Overview

This document outlines the testing strategy for the Next.js frontend application, including frameworks, patterns, and best practices.

## Testing Philosophy

1. **Test behavior, not implementation** - Focus on what users experience
2. **Test pyramid approach** - More unit tests, fewer integration tests, minimal E2E
3. **Fast feedback loops** - Tests should run quickly during development
4. **Maintainable tests** - Tests should be easy to understand and update
5. **Real-world scenarios** - Test actual user workflows and edge cases

## Recommended Testing Stack

### Core Testing Framework: Vitest + React Testing Library

**Why Vitest?**
- ✅ Native ESM support (works perfectly with Next.js 15)
- ✅ Compatible with Vite/Turbopack ecosystem
- ✅ Jest-compatible API (easy migration if needed)
- ✅ Fast execution with native TypeScript support
- ✅ Built-in coverage reporting
- ✅ Watch mode with HMR-like experience

**Why React Testing Library?**
- ✅ Encourages testing user behavior over implementation details
- ✅ Works seamlessly with React 19 and Server Components
- ✅ Industry standard for React testing
- ✅ Excellent documentation and community support

### Additional Tools

1. **@testing-library/user-event** - Simulate real user interactions
2. **MSW (Mock Service Worker)** - Mock API requests at network level
3. **@vitejs/plugin-react** - Vitest React plugin
4. **happy-dom** - Fast DOM implementation for Node.js (lighter than jsdom)
5. **Playwright** (E2E only) - For critical user journeys

## Test Categories

### 1. Unit Tests (70% of test coverage)

**What to test:**
- Pure utility functions (`lib/utils.ts`)
- Data transformations
- Type guards and validators
- Formatting functions

**Example:**
```typescript
// lib/utils.test.ts
import { describe, it, expect } from 'vitest';
import { formatPercent, formatDuration, getRiskColor } from './utils';

describe('formatPercent', () => {
  it('formats decimal to percentage with 2 decimals', () => {
    expect(formatPercent(0.1234)).toBe('12.34%');
  });

  it('handles zero correctly', () => {
    expect(formatPercent(0)).toBe('0.00%');
  });

  it('handles 1.0 correctly', () => {
    expect(formatPercent(1.0)).toBe('100.00%');
  });
});

describe('formatDuration', () => {
  it('formats days and hours', () => {
    expect(formatDuration(90000)).toBe('1d 1h'); // 25 hours
  });

  it('formats hours and minutes', () => {
    expect(formatDuration(3700)).toBe('1h 1m');
  });

  it('formats only minutes', () => {
    expect(formatDuration(120)).toBe('2m');
  });
});

describe('getRiskColor', () => {
  it('returns red for high risk', () => {
    expect(getRiskColor(0.8)).toBe('text-red-600 bg-red-50');
  });

  it('returns yellow for medium risk', () => {
    expect(getRiskColor(0.5)).toBe('text-yellow-600 bg-yellow-50');
  });

  it('returns green for low risk', () => {
    expect(getRiskColor(0.2)).toBe('text-green-600 bg-green-50');
  });
});
```

### 2. Component Tests (25% of test coverage)

**What to test:**
- Component rendering with different props
- User interactions (clicks, form inputs)
- Conditional rendering
- Error states and loading states
- Accessibility (a11y)

**Example:**
```typescript
// components/ui/badge.test.tsx
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Badge } from './badge';

describe('Badge', () => {
  it('renders children correctly', () => {
    render(<Badge>Test Badge</Badge>);
    expect(screen.getByText('Test Badge')).toBeInTheDocument();
  });

  it('applies default variant styles', () => {
    render(<Badge>Default</Badge>);
    const badge = screen.getByText('Default');
    expect(badge).toHaveClass('bg-primary');
  });

  it('applies destructive variant styles', () => {
    render(<Badge variant="destructive">Error</Badge>);
    const badge = screen.getByText('Error');
    expect(badge).toHaveClass('bg-destructive');
  });

  it('merges custom className', () => {
    render(<Badge className="custom-class">Custom</Badge>);
    const badge = screen.getByText('Custom');
    expect(badge).toHaveClass('custom-class');
  });
});
```

**Example: Testing with SWR**
```typescript
// components/features/opportunities-table.test.tsx
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { SWRConfig } from 'swr';
import { OpportunitiesTable } from './opportunities-table';
import * as api from '@/lib/api';

// Mock the API
vi.mock('@/lib/api');

const mockOpportunities = {
  opportunities: [
    {
      market_id: 'test-market-1',
      as_of: '2024-01-01T00:00:00Z',
      recommended_side: 'YES',
      entry_price: 0.65,
      expected_payout: 0.85,
      max_position_pct: 0.1,
      risk_score: 0.3,
      risk_flags: [],
    },
  ],
  total: 1,
  limit: 50,
  offset: 0,
};

describe('OpportunitiesTable', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('displays loading state initially', () => {
    vi.mocked(api.getOpportunities).mockImplementation(
      () => new Promise(() => {}) // Never resolves
    );

    render(
      <SWRConfig value={{ provider: () => new Map() }}>
        <OpportunitiesTable />
      </SWRConfig>
    );

    expect(screen.getAllByRole('status')).toHaveLength(5); // 5 skeleton rows
  });

  it('displays opportunities when loaded', async () => {
    vi.mocked(api.getOpportunities).mockResolvedValue(mockOpportunities);

    render(
      <SWRConfig value={{ provider: () => new Map() }}>
        <OpportunitiesTable />
      </SWRConfig>
    );

    await waitFor(() => {
      expect(screen.getByText(/test-market-1/)).toBeInTheDocument();
    });

    expect(screen.getByText('YES')).toBeInTheDocument();
    expect(screen.getByText('65.00%')).toBeInTheDocument();
  });

  it('displays error state on fetch failure', async () => {
    vi.mocked(api.getOpportunities).mockRejectedValue(
      new Error('API Error')
    );

    render(
      <SWRConfig value={{ provider: () => new Map() }}>
        <OpportunitiesTable />
      </SWRConfig>
    );

    await waitFor(() => {
      expect(screen.getByText(/Failed to load opportunities/)).toBeInTheDocument();
    });
  });

  it('displays empty state when no opportunities', async () => {
    vi.mocked(api.getOpportunities).mockResolvedValue({
      ...mockOpportunities,
      opportunities: [],
      total: 0,
    });

    render(
      <SWRConfig value={{ provider: () => new Map() }}>
        <OpportunitiesTable />
      </SWRConfig>
    );

    await waitFor(() => {
      expect(screen.getByText(/No opportunities available/)).toBeInTheDocument();
    });
  });
});
```

### 3. Integration Tests with MSW (Mock Service Worker)

**What to test:**
- API integration flows
- Data fetching and caching (SWR)
- Error handling with real HTTP responses
- Loading states during data fetching

**Setup:**
```typescript
// tests/setup.ts
import { afterAll, afterEach, beforeAll } from 'vitest';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';

const handlers = [
  http.get('http://localhost:3000/v1/opportunities', () => {
    return HttpResponse.json({
      opportunities: [
        {
          market_id: 'test-market-1',
          as_of: '2024-01-01T00:00:00Z',
          recommended_side: 'YES',
          entry_price: 0.65,
          expected_payout: 0.85,
          max_position_pct: 0.1,
          risk_score: 0.3,
          risk_flags: [],
        },
      ],
      total: 1,
      limit: 50,
      offset: 0,
    });
  }),

  http.get('http://localhost:3000/v1/market/:id', ({ params }) => {
    return HttpResponse.json({
      market: {
        market_id: params.id,
        question: 'Test Market?',
        end_date: '2024-12-31T23:59:59Z',
        close_time: '2024-12-31T23:59:59Z',
        volume: 100000,
        liquidity: 50000,
        active: true,
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      },
      quote: null,
      score: null,
      recommendation: null,
    });
  }),
];

export const server = setupServer(...handlers);

beforeAll(() => server.listen({ onUnhandledRequest: 'error' }));
afterEach(() => server.resetHandlers());
afterAll(() => server.close());
```

**Example Integration Test:**
```typescript
// lib/api.integration.test.ts
import { describe, it, expect } from 'vitest';
import { server } from '../tests/setup';
import { http, HttpResponse } from 'msw';
import { getOpportunities, getMarket } from './api';

describe('API Integration', () => {
  it('fetches opportunities successfully', async () => {
    const data = await getOpportunities();

    expect(data.opportunities).toHaveLength(1);
    expect(data.opportunities[0].market_id).toBe('test-market-1');
    expect(data.total).toBe(1);
  });

  it('handles opportunities fetch error', async () => {
    server.use(
      http.get('http://localhost:3000/v1/opportunities', () => {
        return new HttpResponse(null, { status: 500 });
      })
    );

    await expect(getOpportunities()).rejects.toThrow('Failed to fetch opportunities');
  });

  it('fetches market details successfully', async () => {
    const data = await getMarket('test-market-1');

    expect(data.market.market_id).toBe('test-market-1');
    expect(data.market.question).toBe('Test Market?');
  });

  it('passes query parameters correctly', async () => {
    let capturedUrl: URL | undefined;

    server.use(
      http.get('http://localhost:3000/v1/opportunities', ({ request }) => {
        capturedUrl = new URL(request.url);
        return HttpResponse.json({
          opportunities: [],
          total: 0,
          limit: 10,
          offset: 5,
        });
      })
    );

    await getOpportunities({
      min_score: 0.5,
      max_risk_score: 0.7,
      limit: 10,
      offset: 5,
    });

    expect(capturedUrl?.searchParams.get('min_score')).toBe('0.5');
    expect(capturedUrl?.searchParams.get('max_risk_score')).toBe('0.7');
    expect(capturedUrl?.searchParams.get('limit')).toBe('10');
    expect(capturedUrl?.searchParams.get('offset')).toBe('5');
  });
});
```

### 4. E2E Tests with Playwright (5% - Critical Paths Only)

**What to test:**
- Complete user journeys
- Cross-browser compatibility
- Performance and load times
- Visual regression (screenshots)

**Example:**
```typescript
// e2e/opportunities.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Opportunities Dashboard', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:3001');
  });

  test('loads and displays opportunities', async ({ page }) => {
    // Wait for data to load
    await expect(page.getByRole('table')).toBeVisible();

    // Check table headers
    await expect(page.getByRole('columnheader', { name: 'Market ID' })).toBeVisible();
    await expect(page.getByRole('columnheader', { name: 'Side' })).toBeVisible();
    await expect(page.getByRole('columnheader', { name: 'Risk Score' })).toBeVisible();

    // Check at least one row is present
    const rows = page.getByRole('row');
    await expect(rows).toHaveCount.greaterThan(1); // Header + at least 1 data row
  });

  test('navigates to market details', async ({ page }) => {
    // Wait for table to load
    await expect(page.getByRole('table')).toBeVisible();

    // Click on first market link
    const firstMarketLink = page.getByRole('link').first();
    await firstMarketLink.click();

    // Verify navigation
    await expect(page).toHaveURL(/\/market\/.+/);

    // Verify market details are displayed
    await expect(page.getByText('Back to opportunities')).toBeVisible();
  });

  test('auto-refreshes data every 30 seconds', async ({ page }) => {
    // Wait for initial load
    await expect(page.getByRole('table')).toBeVisible();

    // Note: This is a simplified test. In practice, you'd mock time or
    // intercept network requests to verify refresh behavior
    await page.waitForTimeout(31000);

    // Verify network request was made (using Playwright's network monitoring)
    // This would require setting up request interception
  });

  test('displays error state on API failure', async ({ page, context }) => {
    // Intercept API requests and return error
    await context.route('**/v1/opportunities*', (route) => {
      route.fulfill({ status: 500 });
    });

    await page.goto('http://localhost:3001');

    // Verify error message is displayed
    await expect(page.getByText(/Failed to load opportunities/)).toBeVisible();
  });

  test('is mobile responsive', async ({ page }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });

    await expect(page.getByRole('table')).toBeVisible();

    // Verify table is scrollable on mobile
    const table = page.getByRole('table');
    const boundingBox = await table.boundingBox();
    expect(boundingBox?.width).toBeLessThanOrEqual(375);
  });
});

test.describe('Market Details Page', () => {
  test('displays comprehensive market information', async ({ page }) => {
    // Navigate to a specific market
    await page.goto('http://localhost:3001/market/test-market-id');

    // Verify different sections are present
    await expect(page.getByText('Current Quotes')).toBeVisible();
    await expect(page.getByText('Score Breakdown')).toBeVisible();
    await expect(page.getByText('Recommendation')).toBeVisible();
  });

  test('back button returns to opportunities', async ({ page }) => {
    await page.goto('http://localhost:3001/market/test-market-id');

    await page.getByText('Back to opportunities').click();

    await expect(page).toHaveURL('http://localhost:3001');
  });
});
```

## Test Configuration

### Vitest Config

```typescript
// vitest.config.ts
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: 'happy-dom',
    setupFiles: ['./tests/setup.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: [
        'node_modules/',
        'tests/',
        '**/*.config.ts',
        '**/*.d.ts',
        '**/types/',
        '.next/',
      ],
      thresholds: {
        lines: 80,
        functions: 80,
        branches: 75,
        statements: 80,
      },
    },
    include: ['**/*.{test,spec}.{ts,tsx}'],
    exclude: ['node_modules', '.next', 'e2e'],
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './'),
    },
  },
});
```

### Playwright Config

```typescript
// playwright.config.ts
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',

  use: {
    baseURL: 'http://localhost:3001',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
    {
      name: 'Mobile Chrome',
      use: { ...devices['Pixel 5'] },
    },
  ],

  webServer: {
    command: 'yarn dev',
    url: 'http://localhost:3001',
    reuseExistingServer: !process.env.CI,
  },
});
```

## Coverage Goals

- **Overall Coverage**: 80%+
- **Critical Paths**: 95%+ (API client, utilities)
- **Components**: 75%+
- **Types**: Not counted in coverage

## Testing Workflow

### Local Development

```bash
# Run unit and component tests in watch mode
yarn test

# Run tests with coverage
yarn test:coverage

# Run E2E tests (requires running dev server)
yarn test:e2e

# Run E2E tests in UI mode
yarn test:e2e:ui
```

### CI/CD Pipeline

```yaml
# .github/workflows/test.yml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '22'
          cache: 'yarn'

      - name: Install dependencies
        run: yarn install --frozen-lockfile

      - name: Run unit tests
        run: yarn test:coverage

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./coverage/coverage-final.json

      - name: Install Playwright browsers
        run: npx playwright install --with-deps

      - name: Run E2E tests
        run: yarn test:e2e

      - name: Upload E2E test results
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: playwright-report
          path: playwright-report/
```

## Best Practices

### 1. Test Isolation
- Each test should be independent
- Use `beforeEach` to reset state
- Don't rely on test execution order

### 2. Descriptive Test Names
```typescript
// ❌ Bad
it('works', () => { ... });

// ✅ Good
it('displays error message when API returns 500', () => { ... });
```

### 3. Arrange-Act-Assert Pattern
```typescript
it('formats percentage correctly', () => {
  // Arrange
  const input = 0.1234;

  // Act
  const result = formatPercent(input);

  // Assert
  expect(result).toBe('12.34%');
});
```

### 4. Test User Behavior, Not Implementation
```typescript
// ❌ Bad - Testing implementation
expect(component.state.isLoading).toBe(true);

// ✅ Good - Testing user-visible behavior
expect(screen.getByRole('status')).toBeInTheDocument();
```

### 5. Use Testing Library Queries Properly
```typescript
// Priority order (from most to least preferred):
// 1. getByRole (most accessible)
screen.getByRole('button', { name: 'Submit' });

// 2. getByLabelText (forms)
screen.getByLabelText('Email address');

// 3. getByPlaceholderText
screen.getByPlaceholderText('Enter email');

// 4. getByText
screen.getByText('Welcome');

// 5. getByTestId (last resort)
screen.getByTestId('custom-element');
```

## Dependencies to Add

```json
{
  "devDependencies": {
    "vitest": "^2.1.8",
    "@vitejs/plugin-react": "^4.3.4",
    "@testing-library/react": "^16.1.0",
    "@testing-library/user-event": "^14.5.2",
    "@testing-library/jest-dom": "^6.6.3",
    "happy-dom": "^15.11.7",
    "msw": "^2.7.0",
    "@playwright/test": "^1.49.1"
  }
}
```

## Next Steps

1. Install testing dependencies
2. Configure Vitest and Playwright
3. Write tests for utility functions (quick wins)
4. Add component tests for UI primitives
5. Write integration tests for API client
6. Add E2E tests for critical user journeys
7. Set up CI/CD pipeline with test automation
8. Monitor and maintain coverage thresholds

## Resources

- [Vitest Documentation](https://vitest.dev/)
- [React Testing Library](https://testing-library.com/react)
- [Playwright Documentation](https://playwright.dev/)
- [MSW Documentation](https://mswjs.io/)
- [Testing Next.js Applications](https://nextjs.org/docs/app/building-your-application/testing)
