import { AccountRole } from "@solana/kit";
import { describe, expect, it } from "vitest";

import { parseBaseUnits } from "@/lib/amounts";
import { TOKEN_PROGRAM_ADDRESS } from "@/lib/pdas";
import {
  buildClaimRewardsTransaction,
  buildClaimTestStakeTransaction,
  buildEmergencyWithdrawTransaction,
  buildStakeTransaction,
  buildUnstakeTransaction,
  type UserActionContext,
} from "@/lib/user-instructions";

const context: UserActionContext = {
  user: "11111111111111111111111111111111",
  stakingProgram: "Fg6PaFpoGXkYsidMpWxTWqkFrnDRBTTnyW6m9n6eGJZ",
  demoFaucetProgram: "J12YAqC7dWbVWVAFveRdJ8SJ3sYc6roP3WJekhy4bDkM",
  pool: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
  stakeMint: "So11111111111111111111111111111111111111112",
  rewardMint: "SysvarRent111111111111111111111111111111111",
  stakeVault: "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
  rewardVault: "SysvarC1ock11111111111111111111111111111111",
};

function accountStrings(plan: Awaited<ReturnType<typeof buildStakeTransaction>>, index = 0): string[] {
  return plan.instructions[index].accounts?.map((account) => account.address.toString()) ?? [];
}

describe("user transaction builders", () => {
  it("encodes stake as Anchor discriminator plus u64 base units", async () => {
    const plan = await buildStakeTransaction(context, parseBaseUnits("2.5"));
    const data = plan.instructions[0].data ?? new Uint8Array();

    expect(Array.from(data.slice(0, 8))).toEqual([206, 176, 202, 18, 200, 209, 179, 108]);
    expect(new DataView(data.buffer).getBigUint64(8, true)).toBe(2_500_000n);
    expect(accountStrings(plan)).toContain(context.stakeVault);
    expect(accountStrings(plan)).toContain(context.rewardVault);
    expect(accountStrings(plan)).toContain(TOKEN_PROGRAM_ADDRESS.toString());
  });

  it("bundles open position before first stake when requested", async () => {
    const plan = await buildStakeTransaction(context, 1_000_000n, { includeOpenPosition: true });

    expect(plan.instructions).toHaveLength(2);
    expect(Array.from(plan.instructions[0].data ?? [])).toEqual([135, 128, 47, 77, 15, 152, 240, 49]);
    expect(Array.from((plan.instructions[1].data ?? new Uint8Array()).slice(0, 8))).toEqual([
      206, 176, 202, 18, 200, 209, 179, 108,
    ]);
  });

  it("uses pool authority PDA for vault outflows", async () => {
    const [unstake, claim, emergency] = await Promise.all([
      buildUnstakeTransaction(context, 1_000_000n),
      buildClaimRewardsTransaction(context),
      buildEmergencyWithdrawTransaction(context),
    ]);

    expect(unstake.derivedAccounts.poolAuthority).toBe(claim.derivedAccounts.poolAuthority);
    expect(emergency.derivedAccounts.poolAuthority).toBe(claim.derivedAccounts.poolAuthority);
    expect(unstake.instructions[0].accounts?.[2]?.role).toBe(AccountRole.READONLY);
    expect(claim.instructions[0].accounts?.[2]?.role).toBe(AccountRole.READONLY);
  });

  it("prepares faucet claim with receipt and claimant stake ATA", async () => {
    const plan = await buildClaimTestStakeTransaction(context);
    const accounts = accountStrings(plan);

    expect(plan.instructions).toHaveLength(1);
    expect(Array.from(plan.instructions[0].data ?? [])).toEqual([160, 5, 137, 252, 241, 44, 153, 233]);
    expect(accounts).toContain(plan.derivedAccounts.claimReceipt);
    expect(accounts).toContain(plan.derivedAccounts.claimantStakeAccount);
  });
});
