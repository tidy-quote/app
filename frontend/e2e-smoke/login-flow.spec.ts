import { test, expect } from "@playwright/test";

const TEST_EMAIL = `smoke-${Date.now()}@test.tidy-quote.app`;
const TEST_PASSWORD = "SmokeTest123!";

test.describe("Login flow", () => {
  test("signup redirects to verify email page", async ({ page }) => {
    await page.goto("/signup");

    await page.getByLabel("Email").fill(TEST_EMAIL);
    await page.getByLabel("Password", { exact: true }).fill(TEST_PASSWORD);
    await page.getByLabel("Confirm Password").fill(TEST_PASSWORD);
    await page.getByRole("button", { name: "Sign Up" }).click();

    await page.waitForURL("/verify", { timeout: 30_000 });
    await expect(page.getByText("Check your email")).toBeVisible();
  });

  test("login page renders correctly", async ({ page }) => {
    await page.goto("/login");

    await expect(page.getByText("Log in to your account")).toBeVisible();
    await expect(page.getByLabel("Email")).toBeVisible();
    await expect(page.getByLabel("Password")).toBeVisible();
    await expect(page.getByRole("link", { name: "Forgot password?" })).toBeVisible();
    await expect(page.getByRole("link", { name: "Sign up" })).toBeVisible();
  });

  test("forgot password page renders correctly", async ({ page }) => {
    await page.goto("/forgot-password");

    await expect(page.getByText("Reset your password")).toBeVisible();
    await expect(page.getByLabel("Email")).toBeVisible();
    await expect(page.getByRole("button", { name: /Send reset link/i })).toBeVisible();
  });

  test("choose plan page shows all plans", async ({ page }) => {
    await page.goto("/choose-plan");

    await expect(page.getByText("Choose your plan")).toBeVisible();
    await expect(page.getByRole("heading", { name: "Starter" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Solo" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Pro" })).toBeVisible();
  });
});
