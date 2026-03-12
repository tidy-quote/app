import { test, expect } from "@playwright/test";

const TEST_EMAIL = `smoke-pricing-${Date.now()}@test.tidyquote.app`;
const TEST_PASSWORD = "SmokeTest123!";

test.describe("Pricing setup flow", () => {
  test("signup, configure pricing, save, and verify persistence", async ({
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

    await page.getByRole("button", { name: "Save Pricing Template" }).click();
    await expect(
      page.getByText("Pricing template saved successfully!"),
    ).toBeVisible();

    await page.getByRole("link", { name: "Dashboard" }).click();
    await expect(page).toHaveURL("/");

    await page.getByRole("link", { name: "Pricing", exact: true }).click();
    await expect(page).toHaveURL("/pricing");

    await expect(page.getByPlaceholder("Category name").first()).toHaveValue(
      "Deep Clean",
    );
    await expect(page.getByPlaceholder("Price").first()).toHaveValue("120");
  });
});
