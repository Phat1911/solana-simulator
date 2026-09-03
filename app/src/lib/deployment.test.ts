import { afterEach, describe, expect, it } from "vitest";

import { loadDeploymentConfig } from "@/lib/deployment";

const ENV_KEYS = [
  "NEXT_PUBLIC_SOLANA_CLUSTER",
  "NEXT_PUBLIC_SOLANA_RPC_URL",
  "NEXT_PUBLIC_POOL",
  "NEXT_PUBLIC_STAKE_MINT",
  "NEXT_PUBLIC_REWARD_MINT",
] as const;

describe("deployment config", () => {
  afterEach(() => {
    for (const key of ENV_KEYS) {
      delete process.env[key];
    }
  });

  it("defaults to public devnet RPC when deployment metadata is not filled", () => {
    const config = loadDeploymentConfig();

    expect(config.cluster).toBe("devnet");
    expect(config.endpoint).toBe("https://api.devnet.solana.com");
    expect(config.pool).toBeUndefined();
  });

  it("accepts explicit localnet frontend environment", () => {
    process.env.NEXT_PUBLIC_SOLANA_CLUSTER = "localnet";
    process.env.NEXT_PUBLIC_SOLANA_RPC_URL = "http://127.0.0.1:8899";
    process.env.NEXT_PUBLIC_POOL = "Pool111111111111111111111111111111111111111";

    const config = loadDeploymentConfig();

    expect(config.cluster).toBe("localnet");
    expect(config.endpoint).toBe("http://127.0.0.1:8899");
    expect(config.pool).toBe("Pool111111111111111111111111111111111111111");
  });
});
