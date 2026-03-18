import { test, expect } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.goto("/");
  await page.evaluate(() => localStorage.clear());
});

test("navigates to forgot password from login", async ({ page }) => {
  await page.goto("/login");
  await page.getByRole("link", { name: "Forgot password?" }).click();
  await expect(page).toHaveURL("/forgot-password");
  await expect(page.getByText("Reset your password")).toBeVisible();
});

test("shows email input on forgot password page", async ({ page }) => {
  await page.goto("/forgot-password");
  await expect(page.getByLabel("Email")).toBeVisible();
  await expect(page.getByRole("button", { name: /Send reset link/i })).toBeVisible();
});

test("shows back to login link on forgot password page", async ({ page }) => {
  await page.goto("/forgot-password");
  await expect(page.getByRole("link", { name: "Back to login" })).toBeVisible();
});

test("shows invalid link message on reset password without token", async ({ page }) => {
  await page.goto("/reset-password");
  await expect(page.getByText("Invalid reset link")).toBeVisible();
  await expect(page.getByRole("link", { name: "Request a new link" })).toBeVisible();
});

test("shows password fields on reset password with token", async ({ page }) => {
  await page.goto("/reset-password?token=test-token");
  await expect(page.getByText("Set a new password")).toBeVisible();
  await expect(page.getByLabel("New password")).toBeVisible();
  await expect(page.getByLabel("Confirm password")).toBeVisible();
});
