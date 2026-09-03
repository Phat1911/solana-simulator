import { STAKING_PROGRAM_ADDRESS, DEMO_FAUCET_PROGRAM_ADDRESS } from "@/lib/addresses";

export type DeploymentConfig = {
  cluster: "devnet" | "localnet";
  endpoint: string;
  stakingProgram?: string;
  demoFaucetProgram?: string;
  pool?: string;
  poolAuthority?: string;
  stakeMint?: string;
  rewardMint?: string;
  stakeVault?: string;
  rewardVault?: string;
  admins: readonly string[];
};

const DEFAULT_DEVNET_ENDPOINT = "https://api.devnet.solana.com";
const DEFAULT_LOCAL_ENDPOINT = "http://127.0.0.1:8899";

function clean(value: unknown): string | undefined {
  if (typeof value !== "string") {
    return undefined;
  }

  if (!value || value.startsWith("REPLACE_")) {
    return undefined;
  }

  return value;
}

function env(name: string): string | undefined {
  return clean(process.env[name]);
}

export function loadDeploymentConfig(): DeploymentConfig {
  const cluster = process.env.NEXT_PUBLIC_SOLANA_CLUSTER === "localnet" ? "localnet" : "devnet";

  return {
    cluster,
    endpoint: env("NEXT_PUBLIC_SOLANA_RPC_URL") ?? (cluster === "localnet" ? DEFAULT_LOCAL_ENDPOINT : DEFAULT_DEVNET_ENDPOINT),
    stakingProgram:
      env("NEXT_PUBLIC_STAKING_PROGRAM") ??
      STAKING_PROGRAM_ADDRESS?.toString(),
    demoFaucetProgram:
      env("NEXT_PUBLIC_DEMO_FAUCET_PROGRAM") ??
      DEMO_FAUCET_PROGRAM_ADDRESS?.toString(),
    pool: env("NEXT_PUBLIC_POOL"),
    poolAuthority: env("NEXT_PUBLIC_POOL_AUTHORITY"),
    stakeMint: env("NEXT_PUBLIC_STAKE_MINT"),
    rewardMint: env("NEXT_PUBLIC_REWARD_MINT"),
    stakeVault: env("NEXT_PUBLIC_STAKE_VAULT"),
    rewardVault: env("NEXT_PUBLIC_REWARD_VAULT"),
    admins: [
      env("NEXT_PUBLIC_ADMIN_1"),
      env("NEXT_PUBLIC_ADMIN_2"),
      env("NEXT_PUBLIC_ADMIN_3"),
    ].filter((admin): admin is string => Boolean(admin)),
  };
}
