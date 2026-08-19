import { defineConfig, devices } from '@playwright/test';

// De motortests draaien tegen de gebouwde bundel, niet tegen de ontwikkelbouw.
// Dat is met opzet: wat hier wordt getoetst is precies wat de gebruiker krijgt,
// inclusief de geëxtraheerde stijlen en de verlaagde syntaxis.
//
// `webkit` benadert zowel WKWebView als WebKitGTK dicht genoeg om vrijwel alle
// verschillen te vinden, en draait op de Linux-machine. Daardoor breekt een
// wijziging al voordat er een macOS-bouw bestaat.
export default defineConfig({
  testDir: './motortests',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: 0,
  reporter: process.env.CI ? 'list' : 'line',
  use: {
    baseURL: 'http://127.0.0.1:4173',
    trace: 'off',
    screenshot: 'off',
  },
  webServer: {
    command: 'pnpm exec vite preview --port 4173 --strictPort',
    url: 'http://127.0.0.1:4173',
    reuseExistingServer: !process.env.CI,
    stdout: 'pipe',
    stderr: 'pipe',
    timeout: 60_000,
  },
  projects: [
    { name: 'webkit', use: { ...devices['Desktop Safari'] } },
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
  ],
});
