import { address, type Address } from "@solana/kit";

import demoFaucetIdl from "@/generated/idl/demo_faucet.json";
import stakingPoolIdl from "@/generated/idl/staking_pool.json";

type AnchorIdlMetadata = {
  address?: string;
};

type AnchorIdl = {
  address?: string;
  metadata?: AnchorIdlMetadata;
};

function programAddressFromIdl(idl: AnchorIdl): Address | undefined {
  const raw = idl.address ?? idl.metadata?.address;
  return raw ? address(raw) : undefined;
}

// Milestone 20: generated IDLs are the frontend source for program addresses.
export const STAKING_PROGRAM_ADDRESS = programAddressFromIdl(stakingPoolIdl as AnchorIdl);
export const DEMO_FAUCET_PROGRAM_ADDRESS = programAddressFromIdl(demoFaucetIdl as AnchorIdl);

export function asSolanaAddress(value: string): Address {
  return address(value);
}
