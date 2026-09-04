import { expect, test, type Page } from "@playwright/test";

async function registerFakeWallet(page: Page) {
  await page.addInitScript(() => {
    const account = {
      address: "11111111111111111111111111111111",
      chains: ["solana:devnet"],
      features: ["solana:signAndSendTransaction"],
      publicKey: new Uint8Array(32),
    };
    const wallet = {
      accounts: [account],
      chains: ["solana:devnet"],
      features: {
        "standard:connect": {
          connect: async () => ({ accounts: [account] }),
          version: "1.0.0",
        },
      },
      icon: "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg'/%3E",
      name: "Test Wallet",
      version: "1.0.0",
    };

    window.addEventListener("wallet-standard:app-ready", (event) => {
      (event as CustomEvent<{ register: (wallet: unknown) => void }>).detail.register(wallet);
    });
    window.dispatchEvent(
      new CustomEvent("wallet-standard:register-wallet", {
        detail: (api: { register: (wallet: unknown) => void }) => api.register(wallet),
      }),
    );
  });
}

test("milestone 21 user action surface renders with disabled transaction buttons before wallet connection", async ({ page }) => {
  await page.goto("/", { waitUntil: "domcontentloaded" });

  await expect(page.getByRole("heading", { name: "User State" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Actions" })).toBeVisible();
  await expect(page.getByLabel("Amount")).toHaveValue("1");
  await expect(page.getByRole("button", { name: "Claim Faucet" })).toBeDisabled();
  await expect(page.getByRole("button", { name: "Stake", exact: true })).toBeDisabled();
  await expect(page.getByRole("button", { name: "Emergency Withdraw" })).toBeDisabled();
  const actionPanel = page.locator("section").filter({ has: page.getByRole("heading", { name: "Actions" }) });
  await expect(actionPanel.getByText("Estimated Pending")).toBeVisible();
});

test("milestone 21 prepares first stake with connected wallet", async ({ page }) => {
  await registerFakeWallet(page);
  await page.goto("/", { waitUntil: "domcontentloaded" });

  await page.getByRole("button", { name: "Test Wallet" }).click();
  await expect(page.getByText("Connected")).toBeVisible();

  await page.getByLabel("Amount").fill("2.5");
  await page.getByRole("button", { name: "Stake", exact: true }).click();

  await expect(page.getByText("Stake: 2 instructions")).toBeVisible();
});
