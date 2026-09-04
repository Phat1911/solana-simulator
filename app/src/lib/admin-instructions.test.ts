import { AccountRole } from "@solana/kit";
import { describe, expect, it } from "vitest";

import { parseBaseUnits } from "@/lib/amounts";
import {
  buildApproveProposalTransaction,
  buildCloseProposalTransaction,
  buildCreateProposalTransaction,
  buildExecuteProposalTransaction,
  buildFundRewardsTransaction,
  buildPausePoolTransaction,
  encodeProposalAction,
  type AdminActionContext,
} from "@/lib/admin-instructions";
import { SYSTEM_PROGRAM_ADDRESS, TOKEN_PROGRAM_ADDRESS } from "@/lib/pdas";

const context: AdminActionContext = {
  user: "11111111111111111111111111111111",
  stakingProgram: "8Dkwd74ntycfAMWKeudjELGroj5pqUpWk4MyLijuf1W7",
  pool: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
  rewardMint: "SysvarRent111111111111111111111111111111111",
  rewardVault: "SysvarC1ock11111111111111111111111111111111",
};

function accountStrings(plan: Awaited<ReturnType<typeof buildCreateProposalTransaction>>, index = 0): string[] {
  return plan.instructions[index].accounts?.map((account) => account.address.toString()) ?? [];
}

describe("admin transaction builders", () => {
  it("encodes funding with the source reward account and token program", async () => {
    const sourceRewardAccount = "So11111111111111111111111111111111111111112";
    const plan = await buildFundRewardsTransaction(context, parseBaseUnits("25"), sourceRewardAccount);
    const data = plan.instructions[0].data ?? new Uint8Array();

    expect(Array.from(data.slice(0, 8))).toEqual([114, 64, 163, 112, 175, 167, 19, 121]);
    expect(new DataView(data.buffer).getBigUint64(8, true)).toBe(25_000_000n);
    expect(accountStrings(plan)).toContain(sourceRewardAccount);
    expect(accountStrings(plan)).toContain(TOKEN_PROGRAM_ADDRESS.toString());
  });

  it("prepares immediate pause with current wallet as writable signer", async () => {
    const plan = await buildPausePoolTransaction(context);

    expect(Array.from(plan.instructions[0].data ?? [])).toEqual([160, 15, 12, 189, 160, 0, 243, 245]);
    expect(plan.instructions[0].accounts?.[0]?.role).toBe(AccountRole.WRITABLE_SIGNER);
  });

  it("encodes create proposal id and action payload", async () => {
    const plan = await buildCreateProposalTransaction(context, 7n, {
      kind: "set-reward-rate",
      newRate: parseBaseUnits("3"),
    });
    const data = plan.instructions[0].data ?? new Uint8Array();

    expect(Array.from(data.slice(0, 8))).toEqual([132, 116, 68, 174, 216, 160, 198, 22]);
    expect(new DataView(data.buffer).getBigUint64(8, true)).toBe(7n);
    expect(data[16]).toBe(0);
    expect(new DataView(data.buffer).getBigUint64(17, true)).toBe(3_000_000n);
    expect(accountStrings(plan)).toContain(SYSTEM_PROGRAM_ADDRESS.toString());
    expect(plan.derivedAccounts.proposal).toBeDefined();
  });

  it("uses readonly signer approval and writable proposal lifecycle accounts", async () => {
    const [approve, execute, close] = await Promise.all([
      buildApproveProposalTransaction(context, 7n),
      buildExecuteProposalTransaction(context, 7n),
      buildCloseProposalTransaction(context, 7n, context.user),
    ]);

    expect(approve.instructions[0].accounts?.[0]?.role).toBe(AccountRole.READONLY_SIGNER);
    expect(approve.instructions[0].accounts?.[2]?.role).toBe(AccountRole.WRITABLE);
    expect(execute.instructions[0].accounts?.map((account) => account.role)).toEqual([AccountRole.WRITABLE, AccountRole.WRITABLE]);
    expect(close.instructions[0].accounts?.[0]?.role).toBe(AccountRole.WRITABLE_SIGNER);
    expect(close.instructions[0].accounts?.[3]?.address.toString()).toBe(context.user);
  });

  it("encodes all proposal action variants", () => {
    expect(Array.from(encodeProposalAction({ kind: "unpause-pool" }))).toEqual([1]);
    expect(encodeProposalAction({ kind: "replace-admin", oldAdmin: context.user, newAdmin: context.rewardMint })).toHaveLength(65);
  });
});
