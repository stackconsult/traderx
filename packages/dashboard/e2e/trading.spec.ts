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
});
