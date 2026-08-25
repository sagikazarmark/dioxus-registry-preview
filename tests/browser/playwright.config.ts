import { defineConfig, devices } from "@playwright/test";

const port = 4173;
const baseURL = `http://127.0.0.1:${port}`;
const customHomePort = 4174;
const customHomeBaseURL = `http://127.0.0.1:${customHomePort}`;

export default defineConfig({
  testDir: ".",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  reporter: [["list"]],
  timeout: 60 * 1000,
  expect: { timeout: 15 * 1000 },

  use: {
    baseURL,
    trace: "on-first-retry",
  },

  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],

  webServer: [
    {
      command: "node ./serve.mjs",
      env: { PORT: String(port) },
      url: baseURL,
      reuseExistingServer: !process.env.CI,
      stdout: "pipe",
    },
    {
      command: "node ./serve.mjs",
      env: {
        DIST: "../../target/dx/custom-home-preview/release/web/public",
        PORT: String(customHomePort),
      },
      url: customHomeBaseURL,
      reuseExistingServer: !process.env.CI,
      stdout: "pipe",
    },
  ],
});
