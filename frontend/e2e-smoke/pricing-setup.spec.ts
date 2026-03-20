import { test, expect } from "@playwright/test";

const TEST_EMAIL = process.env.SMOKE_TEST_EMAIL ?? "";
const TEST_PASSWORD = process.env.SMOKE_TEST_PASSWORD ?? "";

test.describe("Pricing setup", () => {
  test("unauthenticated user is redirected to login", async ({ page }) => {
    await page.goto("/pricing");
    await expect(page).toHaveURL("/login");
  });

  test("saves a new category and add-on", async ({ page }) => {
    test.skip(!TEST_EMAIL, "SMOKE_TEST_EMAIL not set");

    await page.goto("/login");
    await page.getByLabel("Email").fill(TEST_EMAIL);
    await page.getByLabel("Password").fill(TEST_PASSWORD);
    await page.getByRole("button", { name: "Log In" }).click();
    await page.waitForURL("/");

    await page.goto("/pricing");
    await expect(
      page.getByRole("heading", { name: "Pricing Setup" })
    ).toBeVisible();

    const categoryName = page.getByPlaceholder("Category name").first();
    const categoryPrice = page.getByPlaceholder("Price").first();
    const categoryDesc = page.getByPlaceholder("Description").first();

    await categoryName.fill("End of Tenancy");
    await categoryPrice.fill("180");
    await categoryDesc.fill("Full property deep clean");

    const addOnName = page.getByPlaceholder("Add-on name").first();
    const addOnPrice = page
      .locator("fieldset")
      .filter({ hasText: "Add-Ons" })
      .getByPlaceholder("Price")
      .first();

    await addOnName.fill("Oven Cleaning");
    await addOnPrice.fill("35");

    await page.getByRole("button", { name: "Save Pricing Template" }).click();

    await expect(
      page.getByText("Pricing template saved successfully!")
    ).toBeVisible();
  });
});
