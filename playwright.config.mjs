/** @type {import('@playwright/test').PlaywrightTestConfig} */
export default {
  testDir: "tests",
  testMatch: "gui-smoke.mjs",
  timeout: 120_000,
  use: {
    headless: true,
    viewport: { width: 900, height: 600 },
  },
};
