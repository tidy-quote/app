import { test, expect } from "@playwright/test";
import { signUpAndLogin } from "./helpers";

test.beforeEach(async ({ page }) => {
  await page.goto("/");
  await page.evaluate(() => localStorage.clear());
  await signUpAndLogin(page);
});

test("navigates to pricing setup page", async ({ page }) => {
  await page.getByRole("link", { name: "Pricing", exact: true }).click();
  await expect(page).toHaveURL("/pricing");
  await expect(
    page.getByRole("heading", { name: "Pricing Setup" })
  ).toBeVisible();
});

test("adds and configures a service category", async ({ page }) => {
  await page.goto("/pricing");

  const categoryName = page.getByPlaceholder("Category name").first();
  const categoryPrice = page.getByPlaceholder("Price").first();
  const categoryDesc = page.getByPlaceholder("Description").first();

  await categoryName.fill("Window Cleaning");
  await categoryPrice.fill("75");
  await categoryDesc.fill("Interior and exterior windows");

  await expect(categoryName).toHaveValue("Window Cleaning");
  await expect(categoryPrice).toHaveValue("75");
  await expect(categoryDesc).toHaveValue("Interior and exterior windows");
});

test("adds an add-on", async ({ page }) => {
  await page.goto("/pricing");

  await page.getByRole("button", { name: "+ Add-On" }).click();

  const addOnName = page.getByPlaceholder("Add-on name").first();
  const addOnPrice = page
    .locator("fieldset")
    .filter({ hasText: "Add-Ons" })
    .getByPlaceholder("Price")
    .first();

  await addOnName.fill("Oven Cleaning");
  await addOnPrice.fill("30");

  await expect(addOnName).toHaveValue("Oven Cleaning");
  await expect(addOnPrice).toHaveValue("30");
});

test("shows error when category name is empty", async ({ page }) => {
  await page.goto("/pricing");

  // Leave category name empty, fill price
  await page.getByPlaceholder("Price").first().fill("100");

  await page.getByRole("button", { name: "Save Pricing Template" }).click();

  await expect(page.getByText("Category 1 needs a name.")).toBeVisible();
});

test("saves pricing template", async ({ page }) => {
  await page.goto("/pricing");

  await page.getByPlaceholder("Category name").first().fill("Deep Clean");
  await page.getByPlaceholder("Price").first().fill("100");
  await page.getByPlaceholder("Description").first().fill("Full deep clean");

  await page.getByRole("button", { name: "Save Pricing Template" }).click();

  await expect(page.getByText("Pricing template saved successfully!")).toBeVisible();
});

test("loads saved pricing template on revisit", async ({ page }) => {
  await page.goto("/pricing");

  await page.getByPlaceholder("Category name").first().fill("Carpet Cleaning");
  await page.getByPlaceholder("Price").first().fill("50");
  await page.getByPlaceholder("Description").first().fill("Steam carpet clean");

  await page.getByRole("button", { name: "Save Pricing Template" }).click();
  await expect(page.getByText("Pricing template saved successfully!")).toBeVisible();

  // Navigate away and come back
  await page.getByRole("link", { name: "Dashboard" }).click();
  await expect(page).toHaveURL("/");

  await page.getByRole("link", { name: "Pricing", exact: true }).click();
  await expect(page).toHaveURL("/pricing");

  await expect(page.getByPlaceholder("Category name").first()).toHaveValue(
    "Carpet Cleaning"
  );
  await expect(page.getByPlaceholder("Price").first()).toHaveValue("50");
  await expect(page.getByPlaceholder("Description").first()).toHaveValue(
    "Steam carpet clean"
  );
});
