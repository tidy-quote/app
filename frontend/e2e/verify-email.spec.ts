import { test, expect } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.goto("/");
  await page.evaluate(() => localStorage.clear());
});

test("shows verify email page after signup", async ({ page }) => {
  await page.goto("/signup");
  await page.getByLabel("Email").fill("test@example.com");
  await page.getByLabel("Password", { exact: true }).fill("password123");
  await page.getByLabel("Confirm Password").fill("password123");
  await page.getByRole("button", { name: "Sign Up" }).click();

  await expect(page).toHaveURL("/verify");
  await expect(page.getByText("Check your email")).toBeVisible();
  await expect(page.getByRole("button", { name: /Resend/i })).toBeVisible();
});

test("shows back to login link on verify page", async ({ page }) => {
  await page.goto("/verify");
  await expect(page.getByRole("link", { name: "Back to login" })).toBeVisible();
});
