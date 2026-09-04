import { getAddressEncoder } from "@solana/kit";
import { describe, expect, it } from "vitest";

import {
  decodePoolState,
  decodePositionState,
  decodeProposalState,
  estimatePendingRewardScaled,
  REWARD_PRECISION,
} from "@/lib/account-decoders";

const encoder = getAddressEncoder();
const A = "11111111111111111111111111111111";
const B = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const C = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
const D = "So11111111111111111111111111111111111111112";
const E = "SysvarRent111111111111111111111111111111111";

function writeAddress(data: Uint8Array, offset: number, value: string) {
  data.set(encoder.encode(value as never), offset);
}

function writeU64(data: Uint8Array, offset: number, value: bigint) {
  new DataView(data.buffer).setBigUint64(offset, value, true);
}

function writeU128(data: Uint8Array, offset: number, value: bigint) {
  const view = new DataView(data.buffer);
  view.setBigUint64(offset, value & ((1n << 64n) - 1n), true);
  view.setBigUint64(offset + 8, value >> 64n, true);
}

describe("account decoders", () => {
  it("decodes pool fields used by the dashboard", () => {
    const data = new Uint8Array(372);
    data[8] = 1;
    writeAddress(data, 9, A);
    writeU64(data, 41, 7n);
    writeAddress(data, 51, B);
    writeAddress(data, 83, C);
    writeAddress(data, 115, D);
    writeAddress(data, 147, E);
    writeAddress(data, 179, A);
    writeAddress(data, 211, B);
    writeAddress(data, 243, C);
    writeU64(data, 275, 2n);
    writeU64(data, 283, 9n);
    data[291] = 1;
    writeU64(data, 292, 100_000_000n);
    writeU64(data, 300, 5_000_000n);
    writeU64(data, 316, 2_000_000n);
    writeU128(data, 324, 3n * REWARD_PRECISION);

    const pool = decodePoolState(data);

    expect(pool.version).toBe(1);
    expect(pool.poolId).toBe(7n);
    expect(pool.adminEpoch).toBe(2n);
    expect(pool.nextProposalId).toBe(9n);
    expect(pool.paused).toBe(true);
    expect(pool.totalStaked).toBe(2_000_000n);
    expect(pool.accRewardPerStakeScaled).toBe(3_000_000_000n);
  });

  it("decodes position fields and estimates pending rewards", () => {
    const poolData = new Uint8Array(372);
    writeU128(poolData, 324, 5n * REWARD_PRECISION);
    const pool = decodePoolState(poolData);

    const positionData = new Uint8Array(114);
    positionData[8] = 1;
    writeAddress(positionData, 9, B);
    writeAddress(positionData, 41, A);
    writeU64(positionData, 74, 2_000_000n);
    writeU128(positionData, 82, 6_000_000n * REWARD_PRECISION);
    writeU128(positionData, 98, 1_000_000n * REWARD_PRECISION);

    const position = decodePositionState(positionData);

    expect(position.stakedAmount).toBe(2_000_000n);
    expect(estimatePendingRewardScaled(pool, position)).toBe(5_000_000n * REWARD_PRECISION);
  });

  it("decodes proposal status and variable action payloads", () => {
    const setRateData = new Uint8Array(162);
    setRateData[8] = 1;
    writeAddress(setRateData, 9, B);
    writeU64(setRateData, 41, 3n);
    writeAddress(setRateData, 49, A);
    writeU64(setRateData, 81, 2n);
    setRateData[89] = 0;
    writeU64(setRateData, 90, 42_000_000n);
    setRateData[98] = 1;
    setRateData[101] = 1;
    writeU64(setRateData, 102, 50n);
    writeU64(setRateData, 110, 216_050n);

    const setRate = decodeProposalState(setRateData);

    expect(setRate.proposalId).toBe(3n);
    expect(setRate.action).toEqual({ kind: "set-reward-rate", newRate: 42_000_000n });
    expect(setRate.approvals).toEqual([true, false, false]);
    expect(setRate.approvalCount).toBe(1);
    expect(setRate.expiresAtSlot).toBe(216_050n);

    const replaceData = new Uint8Array(186);
    writeAddress(replaceData, 9, B);
    replaceData[89] = 2;
    writeAddress(replaceData, 90, A);
    writeAddress(replaceData, 122, D);
    replaceData[174] = 1;

    const replace = decodeProposalState(replaceData);

    expect(replace.action).toEqual({ kind: "replace-admin", oldAdmin: A, newAdmin: D });
    expect(replace.executed).toBe(true);
  });
});
