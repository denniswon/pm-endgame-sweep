import { test, expect } from '@playwright/test';

test.describe('Market Details Page', () => {
  test('displays comprehensive market information', async ({ page, context }) => {
    // Mock API response for market details
    await context.route('**/v1/market/*', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          market: {
            market_id: 'test-market-123',
            question: 'Will the test pass?',
            description: 'This is a comprehensive test of the market details page',
            end_date: '2024-12-31T23:59:59Z',
            close_time: '2024-12-31T23:59:59Z',
            volume: 250000,
            liquidity: 125000,
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
            staleness_sec: 30,
          },
          score: {
            overall_score: 0.75,
            yield_velocity: 0.8,
            net_yield: 0.15,
            liquidity_score: 0.7,
            definition_risk_score: 0.2,
            staleness_penalty: 0.05,
            t_remaining_sec: 2592000, // 30 days
            as_of: '2024-01-01T00:00:00Z',
          },
          recommendation: {
            market_id: 'test-market-123',
            as_of: '2024-01-01T00:00:00Z',
            recommended_side: 'YES',
            entry_price: 0.65,
            expected_payout: 0.85,
            max_position_pct: 0.12,
            risk_score: 0.3,
            risk_flags: [
              { code: 'LOW_LIQUIDITY', severity: 'medium' },
            ],
            notes: 'Strong opportunity with moderate risk',
          },
        }),
      });
    });

    // Navigate to market details
    await page.goto('/market/test-market-123');

    // Verify market question is displayed
    await expect(page.getByText('Will the test pass?')).toBeVisible();
    await expect(page.getByText('This is a comprehensive test of the market details page')).toBeVisible();

    // Verify market info
    await expect(page.getByText('Active')).toBeVisible();
    await expect(page.getByText(/Market ID: test-market-123/)).toBeVisible();
  });

  test('displays current quotes section', async ({ page, context }) => {
    await context.route('**/v1/market/*', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          market: {
            market_id: 'test-market-123',
            question: 'Test Market',
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
          score: null,
          recommendation: null,
        }),
      });
    });

    await page.goto('/market/test-market-123');

    // Verify quotes section
    await expect(page.getByText('Current Quotes')).toBeVisible();
    await expect(page.getByText('YES')).toBeVisible();
    await expect(page.getByText('NO')).toBeVisible();
    await expect(page.getByText('Bid:')).toBeVisible();
    await expect(page.getByText('Ask:')).toBeVisible();
  });

  test('displays score breakdown section', async ({ page, context }) => {
    await context.route('**/v1/market/*', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          market: {
            market_id: 'test-market-123',
            question: 'Test Market',
            end_date: '2024-12-31T23:59:59Z',
            close_time: '2024-12-31T23:59:59Z',
            volume: 100000,
            liquidity: 50000,
            active: true,
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:00Z',
          },
          quote: null,
          score: {
            overall_score: 0.75,
            yield_velocity: 0.8,
            net_yield: 0.15,
            liquidity_score: 0.7,
            definition_risk_score: 0.2,
            staleness_penalty: 0.05,
            t_remaining_sec: 2592000,
            as_of: '2024-01-01T00:00:00Z',
          },
          recommendation: null,
        }),
      });
    });

    await page.goto('/market/test-market-123');

    // Verify score breakdown section
    await expect(page.getByText('Score Breakdown')).toBeVisible();
    await expect(page.getByText('Overall Score')).toBeVisible();
    await expect(page.getByText('Yield Velocity:')).toBeVisible();
    await expect(page.getByText('Net Yield:')).toBeVisible();
    await expect(page.getByText('Liquidity Score:')).toBeVisible();
  });

  test('displays recommendation section with risk flags', async ({ page, context }) => {
    await context.route('**/v1/market/*', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          market: {
            market_id: 'test-market-123',
            question: 'Test Market',
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
          recommendation: {
            market_id: 'test-market-123',
            as_of: '2024-01-01T00:00:00Z',
            recommended_side: 'YES',
            entry_price: 0.65,
            expected_payout: 0.85,
            max_position_pct: 0.12,
            risk_score: 0.3,
            risk_flags: [
              { code: 'LOW_LIQUIDITY', severity: 'medium' },
              { code: 'HIGH_STALENESS', severity: 'high' },
            ],
            notes: 'Test recommendation notes',
          },
        }),
      });
    });

    await page.goto('/market/test-market-123');

    // Verify recommendation section
    await expect(page.getByText('Recommendation')).toBeVisible();
    await expect(page.getByText('Recommended Side:')).toBeVisible();
    await expect(page.getByText('YES')).toBeVisible();

    // Verify risk flags
    await expect(page.getByText(/Risk Flags \(2\)/)).toBeVisible();
    await expect(page.getByText('LOW_LIQUIDITY')).toBeVisible();
    await expect(page.getByText('HIGH_STALENESS')).toBeVisible();

    // Verify notes
    await expect(page.getByText('Test recommendation notes')).toBeVisible();
  });

  test('back button returns to opportunities page', async ({ page, context }) => {
    await context.route('**/v1/market/*', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          market: {
            market_id: 'test-market-123',
            question: 'Test Market',
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
        }),
      });
    });

    await page.goto('/market/test-market-123');

    // Click back button
    await page.getByText('Back to opportunities').click();

    // Verify navigation
    await expect(page).toHaveURL('/');
  });

  test('handles 404 error gracefully', async ({ page, context }) => {
    await context.route('**/v1/market/*', async (route) => {
      await route.fulfill({ status: 404 });
    });

    await page.goto('/market/nonexistent');

    // Verify error state
    await expect(page.getByText(/Failed to load market/)).toBeVisible({ timeout: 10000 });
  });

  test('is mobile responsive', async ({ page, context }) => {
    await context.route('**/v1/market/*', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          market: {
            market_id: 'test-market-123',
            question: 'Test Market',
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
        }),
      });
    });

    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });

    await page.goto('/market/test-market-123');

    // Verify content is visible on mobile
    await expect(page.getByText('Test Market')).toBeVisible();
    await expect(page.getByText('Back to opportunities')).toBeVisible();
  });
});
