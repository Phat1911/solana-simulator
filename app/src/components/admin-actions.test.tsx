import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { AdminActions } from "@/components/admin-actions";
import { type PoolState, type ProposalState } from "@/lib/account-decoders";

const ADMIN = "11111111111111111111111111111111";
const OTHER = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const THIRD = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

const poolState: PoolState = {
  version: 1,
  initializer: ADMIN,
  poolId: 0n,
  stakeMint: "So11111111111111111111111111111111111111112",
  rewardMint: "SysvarRent111111111111111111111111111111111",
  stakeVault: "SysvarC1ock11111111111111111111111111111111",
  rewardVault: "Sysvar1nstructions1111111111111111111111111",
  admins: [ADMIN, OTHER, THIRD],
  adminEpoch: 4n,
  nextProposalId: 8n,
  paused: false,
  maxRewardRatePerSlot: 100_000_000n,
  rewardRatePerSlot: 0n,
  lastUpdateSlot: 0n,
  totalStaked: 0n,
  accRewardPerStakeScaled: 0n,
  remainingRewardBudgetScaled: 0n,
  allocatedLiabilityScaled: 0n,
};

const proposalState: ProposalState = {
  version: 1,
  pool: poolState.initializer,
  proposalId: 7n,
  creator: ADMIN,
  adminEpoch: 4n,
  action: { kind: "set-reward-rate", newRate: 3_000_000n },
  approvals: [true, true, false],
  approvalCount: 2,
  createdSlot: 10n,
  expiresAtSlot: 216_010n,
  executed: false,
};

describe("AdminActions", () => {
  it("shows admin eligibility, proposal parameters, and enabled execution state", () => {
    render(
      <AdminActions
        context={{
          user: ADMIN,
          stakingProgram: poolState.initializer,
          pool: poolState.initializer,
          rewardMint: poolState.rewardMint,
          rewardVault: poolState.rewardVault,
        }}
        inspectedProposalId="7"
        onInspectedProposalIdChange={vi.fn()}
        poolState={poolState}
        proposalState={proposalState}
      />,
    );

    expect(screen.getByRole("heading", { name: "Proposal Actions" })).toBeTruthy();
    expect(screen.getByText("Admin 1")).toBeTruthy();
    expect(screen.getByText("8")).toBeTruthy();
    expect(screen.getByText("Set rate to 3 REWARD/slot")).toBeTruthy();
    expect(screen.getByText("2/2")).toBeTruthy();
    expect((screen.getByRole("button", { name: "Execute" }) as HTMLButtonElement).disabled).toBe(false);
  });

  it("disables admin-only actions for a non-admin wallet", () => {
    render(
      <AdminActions
        context={{
          user: poolState.rewardMint,
          stakingProgram: poolState.initializer,
          pool: poolState.initializer,
          rewardMint: poolState.rewardMint,
          rewardVault: poolState.rewardVault,
        }}
        inspectedProposalId="7"
        onInspectedProposalIdChange={vi.fn()}
        poolState={poolState}
      />,
    );

    expect(screen.getByText("Not admin")).toBeTruthy();
    expect((screen.getByRole("button", { name: "Pause Pool" }) as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByRole("button", { name: "Create Proposal" }) as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByRole("button", { name: "Approve" }) as HTMLButtonElement).disabled).toBe(true);
  });
});
