import { test, expect } from "@playwright/test";
import { signUpAndLogin } from "./helpers";

test.beforeEach(async ({ page }) => {
  await page.goto("/");
  await page.evaluate(() => localStorage.clear());
});

test("app renders without crashing after login", async ({ page }) => {
  await signUpAndLogin(page);
  await expect(page.getByRole("heading", { name: "Welcome to Tidy-Quote" })).toBeVisible();
  // Error boundary fallback should NOT be visible
  await expect(page.locator(".error-fallback")).not.toBeVisible();
});
