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
  nextProposalId: bigint;
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

export type ProposalActionState =
  | { kind: "set-reward-rate"; newRate: bigint }
  | { kind: "unpause-pool" }
  | { kind: "replace-admin"; oldAdmin: string; newAdmin: string };

export type ProposalState = {
  version: number;
  pool: string;
  proposalId: bigint;
  creator: string;
  adminEpoch: bigint;
  action: ProposalActionState;
  approvals: readonly boolean[];
  approvalCount: number;
  createdSlot: bigint;
  expiresAtSlot: bigint;
  executed: boolean;
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
    nextProposalId: u64(data, 283),
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

function decodeProposalAction(data: Uint8Array, offset: number): { action: ProposalActionState; nextOffset: number } {
  const variant = data[offset];

  switch (variant) {
    case 0:
      return {
        action: { kind: "set-reward-rate", newRate: u64(data, offset + 1) },
        nextOffset: offset + 9,
      };
    case 1:
      return {
        action: { kind: "unpause-pool" },
        nextOffset: offset + 1,
      };
    case 2:
      return {
        action: {
          kind: "replace-admin",
          oldAdmin: pubkey(data, offset + 1),
          newAdmin: pubkey(data, offset + 33),
        },
        nextOffset: offset + 65,
      };
    default:
      throw new Error("unknown proposal action variant");
  }
}

// Milestone 22: proposal reads expose action, approval, epoch, expiry, and execution status.
export function decodeProposalState(data: Uint8Array): ProposalState {
  if (data.length < 107) {
    throw new Error("proposal account data is too short");
  }

  const decodedAction = decodeProposalAction(data, 89);
  const approvalsOffset = decodedAction.nextOffset;
  if (data.length < approvalsOffset + 21) {
    throw new Error("proposal account data is too short");
  }

  return {
    version: data[8] ?? 0,
    pool: pubkey(data, 9),
    proposalId: u64(data, 41),
    creator: pubkey(data, 49),
    adminEpoch: u64(data, 81),
    action: decodedAction.action,
    approvals: [data[approvalsOffset] === 1, data[approvalsOffset + 1] === 1, data[approvalsOffset + 2] === 1],
    approvalCount: data[approvalsOffset + 3] ?? 0,
    createdSlot: u64(data, approvalsOffset + 4),
    expiresAtSlot: u64(data, approvalsOffset + 12),
    executed: data[approvalsOffset + 20] === 1,
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
