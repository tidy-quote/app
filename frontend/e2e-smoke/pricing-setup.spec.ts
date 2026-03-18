import { test, expect } from "@playwright/test";

test.describe("Pricing setup", () => {
  test("unauthenticated user is redirected to login", async ({ page }) => {
    await page.goto("/pricing");
    await expect(page).toHaveURL("/login");
  });
});
