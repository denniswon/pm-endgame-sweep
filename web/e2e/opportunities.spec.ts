import { test, expect, BrowserContext } from "@playwright/test";

// Mock data for opportunities
const mockOpportunities = {
  opportunities: [
    {
      market_id: "test-market-001",
      as_of: "2024-01-01T00:00:00Z",
      recommended_side: "YES",
      entry_price: 0.65,
      expected_payout: 0.85,
      max_position_pct: 0.12,
      risk_score: 0.3,
      risk_flags: [],
      notes: "Test opportunity 1",
    },
    {
      market_id: "test-market-002",
      as_of: "2024-01-01T00:00:00Z",
      recommended_side: "NO",
      entry_price: 0.45,
      expected_payout: 0.75,
      max_position_pct: 0.08,
      risk_score: 0.5,
      risk_flags: [{ code: "LOW_LIQUIDITY", severity: "medium" }],
      notes: "Test opportunity 2",
    },
  ],
  total: 2,
  limit: 50,
  offset: 0,
};

// Helper to setup API mocking
async function setupOpportunitiesMock(
  context: BrowserContext,
  data = mockOpportunities
) {
  await context.route("**/v1/opportunities*", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(data),
    });
  });
}

test.describe("Opportunities Dashboard", () => {
  test("loads and displays page header", async ({ page, context }) => {
    await setupOpportunitiesMock(context);
    await page.goto("/");

    await expect(
      page.getByRole("heading", { name: "PM Endgame Sweep" })
    ).toBeVisible();
    await expect(
      page.getByText("Polymarket Opportunity Scanner")
    ).toBeVisible();
  });

  test("displays opportunities section", async ({ page, context }) => {
    await setupOpportunitiesMock(context);
    await page.goto("/");

    await expect(
      page.getByRole("heading", { name: "Opportunities" })
    ).toBeVisible();
    await expect(
      page.getByText("Live opportunities updated every 30 seconds")
    ).toBeVisible();
  });

  test("loads and displays opportunities table", async ({ page, context }) => {
    await setupOpportunitiesMock(context);
    await page.goto("/");

    // Wait for table to load
    await expect(page.getByRole("table")).toBeVisible({ timeout: 10000 });

    // Check table headers
    await expect(
      page.getByRole("columnheader", { name: "Market ID" })
    ).toBeVisible();
    await expect(
      page.getByRole("columnheader", { name: "Side" })
    ).toBeVisible();
    await expect(
      page.getByRole("columnheader", { name: "Entry Price" })
    ).toBeVisible();
    await expect(
      page.getByRole("columnheader", { name: "Expected Payout" })
    ).toBeVisible();
    await expect(
      page.getByRole("columnheader", { name: "Position %" })
    ).toBeVisible();
    await expect(
      page.getByRole("columnheader", { name: "Risk Score" })
    ).toBeVisible();
    await expect(
      page.getByRole("columnheader", { name: "Flags" })
    ).toBeVisible();
  });

  test("navigates to market details when clicking market link", async ({
    page,
    context,
  }) => {
    await setupOpportunitiesMock(context);

    // Also mock the market details endpoint
    await context.route("**/v1/market/*", async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          market: {
            market_id: "test-market-001",
            question: "Test Market Question",
            end_date: "2024-12-31T23:59:59Z",
            close_time: "2024-12-31T23:59:59Z",
            volume: 100000,
            liquidity: 50000,
            active: true,
            created_at: "2024-01-01T00:00:00Z",
            updated_at: "2024-01-01T00:00:00Z",
          },
          quote: null,
          score: null,
          recommendation: null,
        }),
      });
    });

    await page.goto("/");

    // Wait for table to load
    await expect(page.getByRole("table")).toBeVisible({ timeout: 10000 });

    // Click on first market link
    const firstMarketLink = page.getByRole("link").first();
    await firstMarketLink.click();

    // Verify navigation
    await expect(page).toHaveURL(new RegExp(`/market/.+`));

    // Verify market details page loaded
    await expect(page.getByText("Back to opportunities")).toBeVisible();
  });

  test("displays loading skeletons initially", async ({ page, context }) => {
    // Add a delay to the API response to ensure we see loading state
    await context.route("**/v1/opportunities*", async (route) => {
      await new Promise((resolve) => setTimeout(resolve, 500));
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(mockOpportunities),
      });
    });

    // Navigate to page but don't wait for network
    await page.goto("/", { waitUntil: "domcontentloaded" });

    // Check for skeleton elements (they have animate-pulse class)
    const skeletons = page.locator(".animate-pulse");
    const count = await skeletons.count();
    expect(count).toBeGreaterThan(0);
  });

  test("handles empty state gracefully", async ({ page, context }) => {
    // Intercept API request and return empty data
    await context.route("**/v1/opportunities*", async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          opportunities: [],
          total: 0,
          limit: 50,
          offset: 0,
        }),
      });
    });

    await page.goto("/");

    // Verify empty state is displayed
    await expect(page.getByText("No opportunities available")).toBeVisible();
  });

  test("displays error state on API failure", async ({ page, context }) => {
    // Intercept API requests and return error
    await context.route("**/v1/opportunities*", async (route) => {
      await route.fulfill({ status: 500 });
    });

    await page.goto("/");

    // Verify error message is displayed
    await expect(page.getByText(/Failed to load opportunities/)).toBeVisible({
      timeout: 10000,
    });
  });

  test("is mobile responsive", async ({ page, context }) => {
    await setupOpportunitiesMock(context);

    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });

    await page.goto("/");

    await expect(
      page.getByRole("heading", { name: "PM Endgame Sweep" })
    ).toBeVisible();
    await expect(page.getByRole("table")).toBeVisible({ timeout: 10000 });

    // Verify layout is not broken - table should be scrollable on mobile
    const table = page.getByRole("table");
    await expect(table).toBeVisible();
  });
});

test.describe("Opportunities Table Content", () => {
  test("displays market data correctly", async ({ page, context }) => {
    await setupOpportunitiesMock(context);
    await page.goto("/");
    await expect(page.getByRole("table")).toBeVisible({ timeout: 10000 });

    // Check that at least one row exists
    const rows = page.getByRole("row");
    const count = await rows.count();
    expect(count).toBeGreaterThan(1); // Header + at least 1 data row
  });

  test("shows pagination information", async ({ page, context }) => {
    await setupOpportunitiesMock(context);
    await page.goto("/");
    await expect(page.getByRole("table")).toBeVisible({ timeout: 10000 });

    // Look for pagination text at bottom of table
    await expect(
      page.getByText(/Showing \d+ of \d+ opportunities/)
    ).toBeVisible();
  });
});
