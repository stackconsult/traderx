import { test, expect } from "@playwright/test";

test.describe("Authentication flow", () => {
  test("unauthenticated user is redirected to /login", async ({ page }) => {
    await page.goto("/dashboard");
    await expect(page).toHaveURL(/\/login/);
  });

  test("login form shows validation error on empty submit", async ({ page }) => {
    await page.goto("/login");
    await page.getByRole("button", { name: /sign in/i }).click();
    await expect(page.getByText(/required|invalid/i).first()).toBeVisible();
  });

  test("successful login redirects to /dashboard", async ({ page }) => {
    await page.goto("/login");
    await page.getByLabel(/username/i).fill("testuser");
    await page.getByLabel(/password/i).fill("testpass");

    await page.route("**/api/auth/login", async (route) => {
      await route.fulfill({
        status: 200,
        body: JSON.stringify({ token: "mock-jwt-token", user: { id: "1", username: "testuser", role: "trader" } }),
      });
    });

    await page.getByRole("button", { name: /sign in/i }).click();
    await expect(page).toHaveURL(/\/dashboard/);
  });
});
