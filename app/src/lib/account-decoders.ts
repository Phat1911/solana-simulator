import { getAddressDecoder } from "@solana/kit";

import { formatBaseUnits, TOKEN_DECIMALS } from "@/lib/amounts";

export type PoolState = {
  version: number;
  initializer: string;
  poolId: bigint;
  stakeMint: string;
  rewardMint: string;
  stakeVault: string;
  rewardVault: string;
  admins: readonly string[];
  adminEpoch: bigint;
  paused: boolean;
  maxRewardRatePerSlot: bigint;
  rewardRatePerSlot: bigint;
  lastUpdateSlot: bigint;
  totalStaked: bigint;
  accRewardPerStakeScaled: bigint;
  remainingRewardBudgetScaled: bigint;
  allocatedLiabilityScaled: bigint;
};

export type PositionState = {
  version: number;
  pool: string;
  owner: string;
  stakedAmount: bigint;
  rewardDebtScaled: bigint;
  pendingRewardScaled: bigint;
};

export const REWARD_PRECISION = 1_000_000_000n;

const addressDecoder = getAddressDecoder();

function pubkey(bytes: Uint8Array, offset: number): string {
  return addressDecoder.decode(bytes.slice(offset, offset + 32)).toString();
}

function u64(bytes: Uint8Array, offset: number): bigint {
  return new DataView(bytes.buffer, bytes.byteOffset + offset, 8).getBigUint64(0, true);
}

function u128(bytes: Uint8Array, offset: number): bigint {
  const view = new DataView(bytes.buffer, bytes.byteOffset + offset, 16);
  return view.getBigUint64(0, true) + (view.getBigUint64(8, true) << 64n);
}

export function decodePoolState(data: Uint8Array): PoolState {
  if (data.length < 372) {
    throw new Error("pool account data is too short");
  }

  return {
    version: data[8] ?? 0,
    initializer: pubkey(data, 9),
    poolId: u64(data, 41),
    stakeMint: pubkey(data, 51),
    rewardMint: pubkey(data, 83),
    stakeVault: pubkey(data, 115),
    rewardVault: pubkey(data, 147),
    admins: [pubkey(data, 179), pubkey(data, 211), pubkey(data, 243)],
    adminEpoch: u64(data, 275),
    paused: data[291] === 1,
    maxRewardRatePerSlot: u64(data, 292),
    rewardRatePerSlot: u64(data, 300),
    lastUpdateSlot: u64(data, 308),
    totalStaked: u64(data, 316),
    accRewardPerStakeScaled: u128(data, 324),
    remainingRewardBudgetScaled: u128(data, 340),
    allocatedLiabilityScaled: u128(data, 356),
  };
}

export function decodePositionState(data: Uint8Array): PositionState {
  if (data.length < 114) {
    throw new Error("position account data is too short");
  }

  return {
    version: data[8] ?? 0,
    pool: pubkey(data, 9),
    owner: pubkey(data, 41),
    stakedAmount: u64(data, 74),
    rewardDebtScaled: u128(data, 82),
    pendingRewardScaled: u128(data, 98),
  };
}

export function estimatePendingRewardScaled(pool: PoolState, position: PositionState): bigint {
  const accrued = position.stakedAmount * pool.accRewardPerStakeScaled;
  const newlyEarned = accrued > position.rewardDebtScaled ? accrued - position.rewardDebtScaled : 0n;
  return position.pendingRewardScaled + newlyEarned;
}

export function formatScaledReward(scaled: bigint): string {
  return formatBaseUnits(scaled / REWARD_PRECISION, TOKEN_DECIMALS);
}
