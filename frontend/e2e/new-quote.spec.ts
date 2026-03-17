import { test, expect } from "@playwright/test";
import { signUpAndLogin } from "./helpers";

test.beforeEach(async ({ page }) => {
  await page.goto("/");
  await page.evaluate(() => localStorage.clear());
  await signUpAndLogin(page);
});

test("navigates to new quote page", async ({ page }) => {
  await page.getByRole("link", { name: "New Quote", exact: true }).click();
  await expect(page).toHaveURL("/quote/new");
  await expect(page.getByRole("heading", { name: "New Quote" })).toBeVisible();
});

test("shows generate button disabled when no input", async ({ page }) => {
  await page.goto("/quote/new");

  await expect(
    page.getByRole("button", { name: "Generate Quote" })
  ).toBeDisabled();
});

test("enables generate button when text is entered", async ({ page }) => {
  await page.goto("/quote/new");

  await page
    .getByPlaceholder("Paste the lead message here...")
    .fill("I need a deep clean for my 3-bedroom house");

  await expect(
    page.getByRole("button", { name: "Generate Quote" })
  ).toBeEnabled();
});

test("can select tone options", async ({ page }) => {
  await page.goto("/quote/new");

  for (const tone of ["Friendly", "Direct", "Premium"]) {
    await page.getByText(tone, { exact: true }).click();
    await expect(page.getByLabel(tone)).toBeChecked();
  }
});

test("generates quote from text input", async ({ page }) => {
  await page.goto("/quote/new");

  await page
    .getByPlaceholder("Paste the lead message here...")
    .fill(
      "Hi, I need a deep clean for my 3-bedroom house next Monday morning please. The address is 10 High Street."
    );

  await page.getByRole("button", { name: "Generate Quote" }).click();

  // Wait for the mock API delay (1.5s) and result to appear
  await expect(
    page.getByRole("heading", { name: "Job Summary" })
  ).toBeVisible({ timeout: 10_000 });
  await expect(
    page.getByRole("heading", { name: "Price Breakdown" })
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Follow-Up Message" })
  ).toBeVisible();
});

test("can copy follow-up message", async ({ page, context }) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await page.goto("/quote/new");

  await page
    .getByPlaceholder("Paste the lead message here...")
    .fill("Deep clean for 3-bedroom house next Monday morning please");

  await page.getByRole("button", { name: "Generate Quote" }).click();

  await expect(
    page.getByRole("button", { name: "Copy Message" })
  ).toBeVisible({ timeout: 10_000 });

  await page.getByRole("button", { name: "Copy Message" }).click();
  await expect(page.getByRole("button", { name: "Copied!" })).toBeVisible();
});

test("can start a new quote after generating", async ({ page }) => {
  await page.goto("/quote/new");

  await page
    .getByPlaceholder("Paste the lead message here...")
    .fill("Deep clean for 3-bedroom house next Monday morning please");

  await page.getByRole("button", { name: "Generate Quote" }).click();

  await expect(
    page.getByRole("heading", { name: "Quote Generated" })
  ).toBeVisible({ timeout: 10_000 });

  await page.getByRole("button", { name: "New Quote" }).click();

  // Form should be reset
  await expect(
    page.getByRole("heading", { name: "New Quote", exact: true })
  ).toBeVisible();
  await expect(
    page.getByPlaceholder("Paste the lead message here...")
  ).toHaveValue("");
  await expect(
    page.getByRole("button", { name: "Generate Quote" })
  ).toBeDisabled();
});
