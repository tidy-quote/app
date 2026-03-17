import { test, expect } from "@playwright/test";

const TEST_EMAIL = `smoke-quote-${Date.now()}@test.tidy-quote.app`;
const TEST_PASSWORD = "SmokeTest123!";

test.describe("Quote generation flow", () => {
  test("signup, set up pricing, generate quote, and verify result", async ({
    page,
  }) => {
    await page.goto("/signup");
    await page.getByLabel("Email").fill(TEST_EMAIL);
    await page.getByLabel("Password", { exact: true }).fill(TEST_PASSWORD);
    await page.getByLabel("Confirm Password").fill(TEST_PASSWORD);
    await page.getByRole("button", { name: "Sign Up" }).click();
    await expect(page).toHaveURL("/");

    await page.getByRole("link", { name: "Pricing", exact: true }).click();
    await expect(page).toHaveURL("/pricing");

    await page.getByPlaceholder("Category name").first().fill("Deep Clean");
    await page.getByPlaceholder("Price").first().fill("120");
    await page
      .getByPlaceholder("Description")
      .first()
      .fill("Full deep cleaning service");

    await page
      .getByRole("button", { name: "Save Pricing Template" })
      .click();
    await expect(
      page.getByText("Pricing template saved successfully!"),
    ).toBeVisible();

    await page.getByRole("link", { name: "New Quote", exact: true }).click();
    await expect(page).toHaveURL("/quote/new");

    await page
      .getByPlaceholder("Paste the lead message here...")
      .fill(
        "Hi, I need a deep clean for my 3-bedroom house next Monday morning please. The address is 10 High Street.",
      );

    await page.getByRole("button", { name: "Generate Quote" }).click();

    await expect(
      page.getByRole("heading", { name: "Job Summary" }),
    ).toBeVisible({ timeout: 30_000 });
    await expect(
      page.getByRole("heading", { name: "Price Breakdown" }),
    ).toBeVisible();
    await expect(
      page.getByRole("heading", { name: "Follow-Up Message" }),
    ).toBeVisible();
  });
});
