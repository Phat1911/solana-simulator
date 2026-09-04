"use client";

import { AlertTriangle, CheckCircle2, ClipboardList } from "lucide-react";
import { useMemo, useState } from "react";

import { formatBaseUnits, parseBaseUnits } from "@/lib/amounts";
import { formatScaledReward } from "@/lib/account-decoders";
import {
  buildClaimRewardsTransaction,
  buildClaimTestStakeTransaction,
  buildClosePositionTransaction,
  buildEmergencyWithdrawTransaction,
  buildOpenPositionTransaction,
  buildStakeTransaction,
  buildUnstakeTransaction,
  type PreparedUserTransaction,
  type UserActionContext,
} from "@/lib/user-instructions";

type ActionStatus =
  | { phase: "idle" }
  | { phase: "preparing"; label: string }
  | { phase: "prepared"; label: string; plan: PreparedUserTransaction }
  | { phase: "failed"; label: string; message: string };

type UserActionsProps = {
  context?: UserActionContext;
  positionExists: boolean;
  poolPaused?: boolean;
  pendingRewardScaled?: bigint;
  stakedAmount?: bigint;
  stakeBalance?: bigint;
  rewardBalance?: bigint;
};

function instructionSummary(plan: PreparedUserTransaction): string {
  const accountCount = plan.instructions.reduce((total, instruction) => total + (instruction.accounts?.length ?? 0), 0);
  return `${plan.instructions.length} instruction${plan.instructions.length === 1 ? "" : "s"}, ${accountCount} accounts`;
}

export function UserActions({
  context,
  positionExists,
  poolPaused,
  pendingRewardScaled,
  stakedAmount,
  stakeBalance,
  rewardBalance,
}: UserActionsProps) {
  const [amountInput, setAmountInput] = useState("1");
  const [confirmEmergency, setConfirmEmergency] = useState(false);
  const [status, setStatus] = useState<ActionStatus>({ phase: "idle" });

  const parsedAmount = useMemo(() => {
    try {
      return parseBaseUnits(amountInput);
    } catch {
      return undefined;
    }
  }, [amountInput]);

  const canPrepare = Boolean(context);

  async function prepare(label: string, build: (safeContext: UserActionContext) => Promise<PreparedUserTransaction>) {
    if (!context) {
      setStatus({ phase: "failed", label, message: "Connect wallet and configure deployment first" });
      return;
    }

    setStatus({ phase: "preparing", label });
    try {
      setStatus({ phase: "prepared", label, plan: await build(context) });
    } catch (error) {
      setStatus({ phase: "failed", label, message: error instanceof Error ? error.message : "Could not prepare transaction" });
    }
  }

  async function prepareAmount(label: string, build: (safeContext: UserActionContext, amount: bigint) => Promise<PreparedUserTransaction>) {
    if (parsedAmount === undefined) {
      setStatus({ phase: "failed", label, message: "Enter a six-decimal token amount" });
      return;
    }

    await prepare(label, (safeContext) => build(safeContext, parsedAmount));
  }

  return (
    <section className="panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">User</p>
          <h2>Actions</h2>
        </div>
        <ClipboardList size={19} />
      </div>

      <dl className="detail-list action-balances">
        <div>
          <dt>Principal</dt>
          <dd>{stakedAmount === undefined ? "0" : formatBaseUnits(stakedAmount)} STAKE</dd>
        </div>
        <div>
          <dt>Estimated Pending</dt>
          <dd>{pendingRewardScaled === undefined ? "0" : formatScaledReward(pendingRewardScaled)} REWARD</dd>
        </div>
        <div>
          <dt>STAKE Balance</dt>
          <dd>{stakeBalance === undefined ? "0" : formatBaseUnits(stakeBalance)} STAKE</dd>
        </div>
        <div>
          <dt>REWARD Balance</dt>
          <dd>{rewardBalance === undefined ? "0" : formatBaseUnits(rewardBalance)} REWARD</dd>
        </div>
      </dl>

      <label className="amount-field">
        <span>Amount</span>
        <input inputMode="decimal" onChange={(event) => setAmountInput(event.target.value)} value={amountInput} />
      </label>

      <div className="action-grid">
        <button disabled={!canPrepare} onClick={() => void prepare("Claim Faucet", buildClaimTestStakeTransaction)} type="button">
          Claim Faucet
        </button>
        <button disabled={!canPrepare || positionExists} onClick={() => void prepare("Open Position", buildOpenPositionTransaction)} type="button">
          Open Position
        </button>
        <button
          disabled={!canPrepare || poolPaused}
          onClick={() =>
            void prepareAmount("Stake", (safeContext, amount) =>
              buildStakeTransaction(safeContext, amount, { includeOpenPosition: !positionExists }),
            )
          }
          type="button"
        >
          Stake
        </button>
        <button disabled={!canPrepare || !positionExists} onClick={() => void prepareAmount("Unstake", buildUnstakeTransaction)} type="button">
          Unstake
        </button>
        <button disabled={!canPrepare || !positionExists || poolPaused} onClick={() => void prepare("Claim Rewards", buildClaimRewardsTransaction)} type="button">
          Claim Rewards
        </button>
        <button disabled={!canPrepare || !positionExists} onClick={() => void prepare("Close Position", buildClosePositionTransaction)} type="button">
          Close Position
        </button>
      </div>

      <label className="confirm-row">
        <input checked={confirmEmergency} onChange={(event) => setConfirmEmergency(event.target.checked)} type="checkbox" />
        <span>Forfeit pending rewards</span>
      </label>
      <button
        className="danger-button"
        disabled={!canPrepare || !positionExists || !confirmEmergency}
        onClick={() => void prepare("Emergency Withdraw", buildEmergencyWithdrawTransaction)}
        type="button"
      >
        Emergency Withdraw
      </button>

      <div className="transaction-status">
        {status.phase === "idle" ? <span>Idle</span> : null}
        {status.phase === "preparing" ? <span>Preparing {status.label}</span> : null}
        {status.phase === "prepared" ? (
          <>
            <CheckCircle2 size={17} />
            <span>{status.label}: {instructionSummary(status.plan)}</span>
          </>
        ) : null}
        {status.phase === "failed" ? (
          <>
            <AlertTriangle size={17} />
            <span>{status.label}: {status.message}</span>
          </>
        ) : null}
      </div>
    </section>
  );
}
