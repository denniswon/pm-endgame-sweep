import { afterAll, afterEach, beforeAll } from 'vitest';
import { cleanup } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';

// Mock API handlers
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
        market_id: params.id as string,
        question: 'Test Market?',
        description: 'This is a test market description',
        end_date: '2024-12-31T23:59:59Z',
        close_time: '2024-12-31T23:59:59Z',
        volume: 100000,
        liquidity: 50000,
        active: true,
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      },
      quote: {
        yes_bid: 0.45,
        yes_ask: 0.47,
        no_bid: 0.53,
        no_ask: 0.55,
        as_of: '2024-01-01T00:00:00Z',
        staleness_sec: 0,
      },
      score: {
        overall_score: 0.75,
        yield_velocity: 0.8,
        net_yield: 0.15,
        liquidity_score: 0.7,
        definition_risk_score: 0.2,
        staleness_penalty: 0.0,
        t_remaining_sec: 86400 * 30, // 30 days
        as_of: '2024-01-01T00:00:00Z',
      },
      recommendation: {
        market_id: params.id as string,
        as_of: '2024-01-01T00:00:00Z',
        recommended_side: 'YES',
        entry_price: 0.65,
        expected_payout: 0.85,
        max_position_pct: 0.1,
        risk_score: 0.3,
        risk_flags: [],
        notes: 'Strong opportunity with low risk',
      },
    });
  }),

  http.get('http://localhost:3000/health', () => {
    return HttpResponse.json({ status: 'ok' });
  }),
];

export const server = setupServer(...handlers);

// Start MSW server before all tests
beforeAll(() => {
  server.listen({ onUnhandledRequest: 'error' });
});

// Reset handlers after each test
afterEach(() => {
  server.resetHandlers();
  cleanup();
});

// Clean up after all tests
afterAll(() => {
  server.close();
});
