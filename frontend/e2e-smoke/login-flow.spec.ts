import { test, expect } from "@playwright/test";

const TEST_EMAIL = `smoke-${Date.now()}@test.tidyquote.app`;
const TEST_PASSWORD = "SmokeTest123!";

test.describe("Login flow", () => {
  test("signup, logout, and login again", async ({ page }) => {
    await page.goto("/signup");

    await page.getByLabel("Email").fill(TEST_EMAIL);
    await page.getByLabel("Password", { exact: true }).fill(TEST_PASSWORD);
    await page.getByLabel("Confirm Password").fill(TEST_PASSWORD);
    await page.getByRole("button", { name: "Sign Up" }).click();

    await expect(page).toHaveURL("/");
    await expect(
      page.getByRole("heading", { name: "Welcome to TidyQuote" }),
    ).toBeVisible();

    await page.getByRole("button", { name: "Log out" }).click();
    await expect(page).toHaveURL("/login");

    await page.getByLabel("Email").fill(TEST_EMAIL);
    await page.getByLabel("Password").fill(TEST_PASSWORD);
    await page.getByRole("button", { name: "Log In" }).click();

    await expect(page).toHaveURL("/");
    await expect(
      page.getByRole("heading", { name: "Welcome to TidyQuote" }),
    ).toBeVisible();
  });
});
