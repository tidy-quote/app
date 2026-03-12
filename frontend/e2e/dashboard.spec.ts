import { test, expect } from "@playwright/test";
import { signUpAndLogin } from "./helpers";

test.beforeEach(async ({ page }) => {
  await page.goto("/");
  await page.evaluate(() => localStorage.clear());
  await signUpAndLogin(page);
});

test("shows pricing setup CTA when no template configured", async ({
  page,
}) => {
  await expect(
    page.getByText("Set up your pricing template first")
  ).toBeVisible();
  await expect(page.getByRole("link", { name: "Set Up Pricing" })).toBeVisible();
});

test("shows quick action buttons", async ({ page }) => {
  await expect(
    page.getByRole("link", { name: "New Quote", exact: true })
  ).toBeVisible();
  await expect(page.locator(".action-card__title", { hasText: "Pricing Setup" })).toBeVisible();
});

test("navigates to new quote from dashboard", async ({ page }) => {
  await page.getByRole("link", { name: "New Quote" }).first().click();
  await expect(page).toHaveURL("/quote/new");
  await expect(page.getByRole("heading", { name: "New Quote" })).toBeVisible();
});
