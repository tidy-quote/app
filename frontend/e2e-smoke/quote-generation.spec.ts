import { test, expect } from "@playwright/test";

test.describe("Quote generation", () => {
  test("unauthenticated user is redirected to login", async ({ page }) => {
    await page.goto("/quote/new");
    await expect(page).toHaveURL("/login");
  });
});
