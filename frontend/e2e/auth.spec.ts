import { test, expect } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.goto("/");
  await page.evaluate(() => localStorage.clear());
});

test("redirects to login when not authenticated", async ({ page }) => {
  await page.goto("/");
  await expect(page).toHaveURL("/login");
});

test("can sign up and access dashboard", async ({ page }) => {
  await page.goto("/signup");
  await page.getByLabel("Email").fill("test@example.com");
  await page.getByLabel("Password", { exact: true }).fill("password123");
  await page.getByLabel("Confirm Password").fill("password123");
  await page.getByRole("button", { name: "Sign Up" }).click();

  await expect(page).toHaveURL("/");
  await expect(page.getByRole("heading", { name: "Welcome to TidyQuote" })).toBeVisible();
});

test("shows error when passwords do not match", async ({ page }) => {
  await page.goto("/signup");
  await page.getByLabel("Email").fill("test@example.com");
  await page.getByLabel("Password", { exact: true }).fill("password123");
  await page.getByLabel("Confirm Password").fill("differentpassword");
  await page.getByRole("button", { name: "Sign Up" }).click();

  await expect(page.getByText("Passwords do not match")).toBeVisible();
});

test("can navigate between login and signup", async ({ page }) => {
  await page.goto("/login");
  await page.getByRole("link", { name: "Sign up" }).click();
  await expect(page).toHaveURL("/signup");

  await page.getByRole("link", { name: "Log in" }).click();
  await expect(page).toHaveURL("/login");
});

test("can log out", async ({ page }) => {
  await page.goto("/signup");
  await page.getByLabel("Email").fill("test@example.com");
  await page.getByLabel("Password", { exact: true }).fill("password123");
  await page.getByLabel("Confirm Password").fill("password123");
  await page.getByRole("button", { name: "Sign Up" }).click();
  await expect(page).toHaveURL("/");

  await page.getByRole("button", { name: "Log out" }).click();
  await expect(page).toHaveURL("/login");
});
