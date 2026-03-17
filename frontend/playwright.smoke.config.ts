import { defineConfig } from "@playwright/test";

const deployedUrl = process.env.DEPLOYED_URL;
if (!deployedUrl) throw new Error("DEPLOYED_URL env var is required");

export default defineConfig({
  testDir: "./e2e-smoke",
  fullyParallel: false,
  forbidOnly: true,
  retries: 1,
  workers: 1,
  reporter: "html",
  timeout: 60_000,
  expect: {
    timeout: 30_000,
  },
  use: {
    baseURL: deployedUrl,
    screenshot: "only-on-failure",
    trace: "on-first-retry",
    httpCredentials: {
      username: process.env.BASIC_AUTH_USERNAME ?? "",
      password: process.env.BASIC_AUTH_PASSWORD ?? "",
    },
  },
  projects: [
    {
      name: "chromium",
      use: { browserName: "chromium" },
    },
  ],
});
