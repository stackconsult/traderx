import { test, expect } from "@playwright/test";

test.describe("Trading flow", () => {
  test.beforeEach(async ({ page }) => {
    await page.route("**/api/auth/login", async (route) => {
      await route.fulfill({
        status: 200,
        body: JSON.stringify({ token: "mock-jwt-token", user: { id: "1", username: "testuser", role: "trader" } }),
      });
    });
    await page.route("**/api/positions", async (route) => {
      await route.fulfill({
        status: 200,
        body: JSON.stringify([]),
      });
    });

    await page.goto("/login");
    await page.getByLabel(/username/i).fill("testuser");
    await page.getByLabel(/password/i).fill("testpass");
    await page.getByRole("button", { name: /sign in/i }).click();
    await page.waitForURL(/\/dashboard/);
  });

  test("order entry submits and shows success toast", async ({ page }) => {
    await page.route("**/api/orders", async (route) => {
      await route.fulfill({
        status: 200,
        body: JSON.stringify({
          orderId: "ord-001",
          symbol: "AAPL",
          side: "buy",
          quantity: 10,
          status: "pending",
        }),
      });
    });

    await page.getByPlaceholder(/symbol/i).fill("AAPL");
    await page.getByPlaceholder(/quantity/i).fill("10");
    await page.getByRole("button", { name: /place order/i }).click();

    await expect(page.getByText(/order submitted/i)).toBeVisible();
  });

  test("position table renders when positions exist", async ({ page }) => {
    await page.route("**/api/positions", async (route) => {
      await route.fulfill({
        status: 200,
        body: JSON.stringify([
          { symbol: "AAPL", quantity: 100, avgPrice: 150, marketPrice: 178, unrealizedPnl: 2800, unrealizedPnlPct: 18.67, realizedPnl: 0, marketValue: 17800 },
        ]),
      });
    });

    await page.reload();
    await expect(page.getByText("AAPL")).toBeVisible();
  });

  test("signal card Trade Signal prefills order entry symbol and side", async ({ page }) => {
    await page.route("**/api/orders", async (route) => {
      await route.fulfill({
        status: 200,
        body: JSON.stringify({ orderId: "ord-002", symbol: "TSLA", side: "sell", quantity: 5, status: "pending" }),
      });
    });

    await page.evaluate(() => {
      window.dispatchEvent(
        new CustomEvent("traderx:prefill-order", {
          detail: { symbol: "TSLA", side: "sell" },
        })
      );
    });

    await expect(page.getByPlaceholder(/symbol/i)).toHaveValue("TSLA");
    const sellTab = page.getByRole("button", { name: /sell/i });
    await expect(sellTab).toHaveAttribute("data-active", "true");
  });

  test("watchlist price flashes on market data update", async ({ page }) => {
    await page.evaluate(() => {
      const { useMarketDataStore } = window as unknown as {
        useMarketDataStore: { getState: () => { setPrice: (d: unknown) => void } };
      };
      if (useMarketDataStore) {
        useMarketDataStore.getState().setPrice({
          symbol: "AAPL",
          price: 195.5,
          bid: 195.45,
          ask: 195.55,
          volume: 1000000,
          change: 2.5,
          changePercent: 1.3,
          timestamp: new Date().toISOString(),
        });
      }
    });

    const aapl = page.locator('[data-symbol="AAPL"]');
    await expect(aapl).toBeVisible();
    await expect(aapl.getByText("195.50")).toBeVisible();
  });
});
