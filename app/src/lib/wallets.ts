"use client";

import { getWallets } from "@wallet-standard/app";

type WalletAccountLike = {
  readonly address: string;
};

export type WalletLike = {
  readonly name: string;
  readonly icon?: string;
  readonly accounts?: readonly WalletAccountLike[];
  readonly features?: Record<string, unknown>;
};

export type ConnectedWalletState = {
  walletName: string;
  account: string;
};

type ConnectFeature = {
  connect(input?: { silent?: boolean }): Promise<{ accounts?: readonly WalletAccountLike[] }>;
};

function canConnect(feature: unknown): feature is ConnectFeature {
  return typeof feature === "object" && feature !== null && "connect" in feature;
}

export function getDetectedWallets(): readonly WalletLike[] {
  return getWallets().get() as readonly WalletLike[];
}

export function subscribeToWalletChanges(onChange: () => void): () => void {
  const wallets = getWallets();
  const offRegister = wallets.on("register", onChange);
  const offUnregister = wallets.on("unregister", onChange);

  return () => {
    offRegister();
    offUnregister();
  };
}

export async function connectWallet(wallet: WalletLike): Promise<readonly WalletAccountLike[]> {
  const feature = wallet.features?.["standard:connect"];

  if (!canConnect(feature)) {
    return wallet.accounts ?? [];
  }

  const result = await feature.connect();
  return result.accounts ?? wallet.accounts ?? [];
}
