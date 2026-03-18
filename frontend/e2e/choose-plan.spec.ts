import { test, expect } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.goto("/");
  await page.evaluate(() => localStorage.clear());
});

test("shows three plan options", async ({ page }) => {
  await page.goto("/choose-plan");

  await expect(page.getByText("Choose your plan")).toBeVisible();
  await expect(page.locator(".plan-card")).toHaveCount(3);
  await expect(page.getByRole("heading", { name: "Starter" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Solo" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Pro" })).toBeVisible();
});

test("shows pricing for each plan", async ({ page }) => {
  await page.goto("/choose-plan");

  await expect(page.getByText("$1.99")).toBeVisible();
  await expect(page.getByText("$8.99")).toBeVisible();
  await expect(page.getByText("$19.99")).toBeVisible();
});

test("highlights Solo as most popular", async ({ page }) => {
  await page.goto("/choose-plan");

  await expect(page.getByText("Most Popular")).toBeVisible();
  await expect(page.locator(".plan-card--featured")).toHaveCount(1);
});

test("shows Get Started buttons for all plans", async ({ page }) => {
  await page.goto("/choose-plan");

  const buttons = page.getByRole("button", { name: "Get Started" });
  await expect(buttons).toHaveCount(3);
});
