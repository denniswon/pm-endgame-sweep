import { test, expect } from "@playwright/test";

test.describe("Opportunities Dashboard", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
  });

  test("loads and displays page header", async ({ page }) => {
    await expect(
      page.getByRole("heading", { name: "PM Endgame Sweep" })
    ).toBeVisible();
    await expect(
      page.getByText("Polymarket Opportunity Scanner")
    ).toBeVisible();
  });

  test("displays opportunities section", async ({ page }) => {
    await expect(
      page.getByRole("heading", { name: "Opportunities" })
    ).toBeVisible();
    await expect(
      page.getByText("Live opportunities updated every 30 seconds")
    ).toBeVisible();
  });

  test("loads and displays opportunities table", async ({ page }) => {
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
  }) => {
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

  test("displays loading skeletons initially", async ({ page }) => {
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

  test("is mobile responsive", async ({ page }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });

    await expect(
      page.getByRole("heading", { name: "PM Endgame Sweep" })
    ).toBeVisible();
    await expect(page.getByRole("table")).toBeVisible({ timeout: 10000 });

    // Verify layout is not broken
    const table = page.getByRole("table");
    const boundingBox = await table.boundingBox();
    expect(boundingBox?.width).toBeLessThanOrEqual(375);
  });
});

test.describe("Opportunities Table Content", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.getByRole("table")).toBeVisible({ timeout: 10000 });
  });

  test("displays market data correctly", async ({ page }) => {
    // Check that at least one row exists
    const rows = page.getByRole("row");
    const count = await rows.count();
    expect(count).toBeGreaterThan(1); // Header + at least 1 data row
  });

  test("shows pagination information", async ({ page }) => {
    // Look for pagination text at bottom of table
    await expect(
      page.getByText(/Showing \d+ of \d+ opportunities/)
    ).toBeVisible();
  });
});
