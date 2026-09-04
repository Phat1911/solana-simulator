import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  timeout: 30_000,
  use: {
    baseURL: "http://127.0.0.1:3100",
    trace: "on-first-retry",
  },
  webServer: {
    command: "npm run dev -- --hostname 127.0.0.1 --port 3100",
    env: {
      NEXT_PUBLIC_POOL: "8Dkwd74ntycfAMWKeudjELGroj5pqUpWk4MyLijuf1W7",
      NEXT_PUBLIC_REWARD_MINT: "SysvarRent111111111111111111111111111111111",
      NEXT_PUBLIC_REWARD_VAULT: "SysvarC1ock11111111111111111111111111111111",
      NEXT_PUBLIC_STAKE_MINT: "So11111111111111111111111111111111111111112",
      NEXT_PUBLIC_STAKE_VAULT: "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
    },
    reuseExistingServer: true,
    timeout: 30_000,
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
});
