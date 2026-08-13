import { defineConfig } from "@playwright/test";

const port = process.env.PLAYWRIGHT_PORT ?? "4173";
if (!/^\d{1,5}$/.test(port) || Number(port) === 0 || Number(port) > 65535) {
  throw new Error("PLAYWRIGHT_PORT must be an integer between 1 and 65535");
}

const executablePath = process.env.BROWSER_EXECUTABLE_PATH;

export default defineConfig({
  testDir: "./tests/browser",
  outputDir: "test-results",
  forbidOnly: Boolean(process.env.CI),
  retries: process.env.CI ? 1 : 0,
  reporter: "line",
  use: {
    baseURL: `http://127.0.0.1:${port}`,
    launchOptions: executablePath ? { executablePath } : {},
    screenshot: "only-on-failure",
    trace: "retain-on-failure"
  },
  webServer: {
    command: `npx --no-install http-server . -a 127.0.0.1 -p ${port} -c-1 --silent`,
    url: `http://127.0.0.1:${port}`,
    reuseExistingServer: false
  },
  projects: [
    { name: "mobile_375", use: { viewport: { width: 375, height: 812 }, isMobile: true, hasTouch: true } },
    { name: "tablet_768", use: { viewport: { width: 768, height: 1024 }, hasTouch: true } },
    { name: "desktop_1024", use: { viewport: { width: 1024, height: 768 } } },
    { name: "desktop_1440", use: { viewport: { width: 1440, height: 900 } } }
  ]
});
