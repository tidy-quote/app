import type { Page } from "@playwright/test";

export async function signUpAndLogin(page: Page): Promise<void> {
  await page.goto("/signup");
  await page.getByLabel("Email").fill("test@example.com");
  await page.getByLabel("Password", { exact: true }).fill("password123");
  await page.getByLabel("Confirm Password").fill("password123");
  await page.getByRole("button", { name: "Sign Up" }).click();
  await page.waitForURL("/verify");
  // In mock mode, go directly to dashboard (no real email verification)
  await page.goto("/");
}
