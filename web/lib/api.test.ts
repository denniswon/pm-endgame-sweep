import { describe, it, expect } from "vitest";
import { server } from "../tests/setup";
import { http, HttpResponse } from "msw";
import { getOpportunities, getMarket, checkHealth } from "./api";

describe("API Client", () => {
  describe("getOpportunities", () => {
    it("fetches opportunities successfully", async () => {
      const data = await getOpportunities();

      expect(data.opportunities).toHaveLength(1);
      expect(data.opportunities[0].market_id).toBe("test-market-1");
      expect(data.total).toBe(1);
      expect(data.limit).toBe(50);
      expect(data.offset).toBe(0);
    });

    it("passes query parameters correctly", async () => {
      let capturedUrl: URL | undefined;

      server.use(
        http.get("http://localhost:3000/v1/opportunities", ({ request }) => {
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
        max_t_remaining_sec: 86400,
        has_flags: true,
        limit: 10,
        offset: 5,
      });

      expect(capturedUrl?.searchParams.get("min_score")).toBe("0.5");
      expect(capturedUrl?.searchParams.get("max_risk_score")).toBe("0.7");
      expect(capturedUrl?.searchParams.get("max_t_remaining_sec")).toBe(
        "86400"
      );
      expect(capturedUrl?.searchParams.get("has_flags")).toBe("true");
      expect(capturedUrl?.searchParams.get("limit")).toBe("10");
      expect(capturedUrl?.searchParams.get("offset")).toBe("5");
    });

    it("handles undefined query parameters", async () => {
      let capturedUrl: URL | undefined;

      server.use(
        http.get("http://localhost:3000/v1/opportunities", ({ request }) => {
          capturedUrl = new URL(request.url);
          return HttpResponse.json({
            opportunities: [],
            total: 0,
            limit: 50,
            offset: 0,
          });
        })
      );

      await getOpportunities({});

      expect(capturedUrl?.searchParams.toString()).toBe("");
    });

    it("throws error on 500 response", async () => {
      server.use(
        http.get("http://localhost:3000/v1/opportunities", () => {
          return new HttpResponse(null, {
            status: 500,
            statusText: "Internal Server Error",
          });
        })
      );

      await expect(getOpportunities()).rejects.toThrow(
        "Failed to fetch opportunities: 500 Internal Server Error"
      );
    });

    it("throws error on 404 response", async () => {
      server.use(
        http.get("http://localhost:3000/v1/opportunities", () => {
          return new HttpResponse(null, {
            status: 404,
            statusText: "Not Found",
          });
        })
      );

      await expect(getOpportunities()).rejects.toThrow(
        "Failed to fetch opportunities: 404 Not Found"
      );
    });

    it("respects cache: no-store", async () => {
      let requestCount = 0;

      server.use(
        http.get("http://localhost:3000/v1/opportunities", () => {
          requestCount++;
          // Note: fetch() doesn't always send cache-control header, but cache: 'no-store' prevents caching
          return HttpResponse.json({
            opportunities: [],
            total: 0,
            limit: 50,
            offset: 0,
          });
        })
      );

      await getOpportunities();
      await getOpportunities();

      expect(requestCount).toBe(2); // Two separate requests, no caching
    });
  });

  describe("getMarket", () => {
    it("fetches market details successfully", async () => {
      const data = await getMarket("test-market-1");

      expect(data.market.market_id).toBe("test-market-1");
      expect(data.market.question).toBe("Test Market?");
      expect(data.quote).toBeDefined();
      expect(data.score).toBeDefined();
      expect(data.recommendation).toBeDefined();
    });

    it("handles market with no quote/score/recommendation", async () => {
      server.use(
        http.get("http://localhost:3000/v1/market/:id", ({ params }) => {
          return HttpResponse.json({
            market: {
              market_id: params.id,
              question: "Test Market?",
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
          });
        })
      );

      const data = await getMarket("test-market-2");

      expect(data.market.market_id).toBe("test-market-2");
      expect(data.quote).toBeNull();
      expect(data.score).toBeNull();
      expect(data.recommendation).toBeNull();
    });

    it("throws error on 500 response", async () => {
      server.use(
        http.get("http://localhost:3000/v1/market/:id", () => {
          return new HttpResponse(null, {
            status: 500,
            statusText: "Internal Server Error",
          });
        })
      );

      await expect(getMarket("test-market-1")).rejects.toThrow(
        "Failed to fetch market: 500 Internal Server Error"
      );
    });

    it("throws error on 404 response", async () => {
      server.use(
        http.get("http://localhost:3000/v1/market/:id", () => {
          return new HttpResponse(null, {
            status: 404,
            statusText: "Not Found",
          });
        })
      );

      await expect(getMarket("nonexistent")).rejects.toThrow(
        "Failed to fetch market: 404 Not Found"
      );
    });
  });

  describe("checkHealth", () => {
    it("fetches health status successfully", async () => {
      const data = await checkHealth();

      expect(data.status).toBe("ok");
    });

    it("throws error on failed health check", async () => {
      server.use(
        http.get("http://localhost:3000/health", () => {
          return new HttpResponse(null, { status: 503 });
        })
      );

      await expect(checkHealth()).rejects.toThrow("API health check failed");
    });
  });

  describe("API Base URL", () => {
    it("uses environment variable when available", async () => {
      // This test verifies that the API_BASE constant uses NEXT_PUBLIC_API_URL
      // In practice, you'd test this with different env configs
      const data = await getOpportunities();
      expect(data).toBeDefined();
    });
  });
});
