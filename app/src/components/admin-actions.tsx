"use client";

import { CheckCircle2, ClipboardCheck, ShieldAlert } from "lucide-react";
import { useMemo, useState } from "react";

import { formatBaseUnits, parseBaseUnits, shortAddress } from "@/lib/amounts";
import { type PoolState, type ProposalState } from "@/lib/account-decoders";
import {
  buildApproveProposalTransaction,
  buildCloseProposalTransaction,
  buildCreateProposalTransaction,
  buildExecuteProposalTransaction,
  buildFundRewardsTransaction,
  buildPausePoolTransaction,
  type AdminActionContext,
  type AdminProposalAction,
  type PreparedAdminTransaction,
} from "@/lib/admin-instructions";

type AdminStatus =
  | { phase: "idle" }
  | { phase: "preparing"; label: string }
  | { phase: "prepared"; label: string; plan: PreparedAdminTransaction }
  | { phase: "failed"; label: string; message: string };

type AdminActionsProps = {
  context?: AdminActionContext;
  poolState?: PoolState;
  proposalState?: ProposalState;
  inspectedProposalId: string;
  onInspectedProposalIdChange: (value: string) => void;
};

function instructionSummary(plan: PreparedAdminTransaction): string {
  const accountCount = plan.instructions.reduce((total, instruction) => total + (instruction.accounts?.length ?? 0), 0);
  return `${plan.instructions.length} instruction${plan.instructions.length === 1 ? "" : "s"}, ${accountCount} accounts`;
}

function actionLabel(action?: AdminProposalAction | ProposalState["action"]): string {
  if (!action) {
    return "None";
  }

  if (action.kind === "set-reward-rate") {
    return `Set rate to ${formatBaseUnits(action.newRate)} REWARD/slot`;
  }

  if (action.kind === "unpause-pool") {
    return "Unpause pool";
  }

  return `Replace ${shortAddress(action.oldAdmin, 4)} with ${shortAddress(action.newAdmin, 4)}`;
}

export function AdminActions({
  context,
  poolState,
  proposalState,
  inspectedProposalId,
  onInspectedProposalIdChange,
}: AdminActionsProps) {
  const [fundAmountInput, setFundAmountInput] = useState("100");
  const [newRateInput, setNewRateInput] = useState("1");
  const [proposalKind, setProposalKind] = useState<AdminProposalAction["kind"]>("set-reward-rate");
  const [oldAdminInput, setOldAdminInput] = useState("");
  const [newAdminInput, setNewAdminInput] = useState("");
  const [status, setStatus] = useState<AdminStatus>({ phase: "idle" });

  const currentUser = context?.user;
  const adminIndex = currentUser && poolState ? poolState.admins.findIndex((admin) => admin === currentUser) : -1;
  const isAdmin = adminIndex >= 0;
  const nextProposalId = poolState?.nextProposalId ?? 0n;

  const parsedFundAmount = useMemo(() => {
    try {
      return parseBaseUnits(fundAmountInput);
    } catch {
      return undefined;
    }
  }, [fundAmountInput]);

  const parsedNewRate = useMemo(() => {
    try {
      return parseBaseUnits(newRateInput);
    } catch {
      return undefined;
    }
  }, [newRateInput]);

  const inspectedProposalIdBigint = useMemo(() => {
    try {
      return inspectedProposalId.trim() === "" ? undefined : BigInt(inspectedProposalId);
    } catch {
      return undefined;
    }
  }, [inspectedProposalId]);

  const selectedAction = useMemo<AdminProposalAction | undefined>(() => {
    if (proposalKind === "set-reward-rate") {
      return parsedNewRate === undefined ? undefined : { kind: "set-reward-rate", newRate: parsedNewRate };
    }

    if (proposalKind === "unpause-pool") {
      return { kind: "unpause-pool" };
    }

    if (!oldAdminInput || !newAdminInput) {
      return undefined;
    }

    return { kind: "replace-admin", oldAdmin: oldAdminInput, newAdmin: newAdminInput };
  }, [newAdminInput, oldAdminInput, parsedNewRate, proposalKind]);

  async function prepare(label: string, build: (safeContext: AdminActionContext) => Promise<PreparedAdminTransaction>) {
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

  async function prepareProposal(label: string, build: (safeContext: AdminActionContext, proposalId: bigint) => Promise<PreparedAdminTransaction>) {
    if (inspectedProposalIdBigint === undefined) {
      setStatus({ phase: "failed", label, message: "Enter a proposal id" });
      return;
    }

    await prepare(label, (safeContext) => build(safeContext, inspectedProposalIdBigint));
  }

  const canCreateProposal = Boolean(context && isAdmin && selectedAction);

  return (
    <section className="panel admin-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">Admin</p>
          <h2>Proposal Actions</h2>
        </div>
        <ClipboardCheck size={19} />
      </div>

      <dl className="detail-list horizontal">
        <div>
          <dt>Eligibility</dt>
          <dd>{isAdmin ? `Admin ${adminIndex + 1}` : "Not admin"}</dd>
        </div>
        <div>
          <dt>Admin Epoch</dt>
          <dd>{poolState?.adminEpoch?.toString() ?? "0"}</dd>
        </div>
        <div>
          <dt>Next Proposal</dt>
          <dd>{nextProposalId.toString()}</dd>
        </div>
      </dl>

      <div className="admin-form-grid">
        <label className="amount-field">
          <span>Fund</span>
          <input inputMode="decimal" onChange={(event) => setFundAmountInput(event.target.value)} value={fundAmountInput} />
        </label>
        <button
          disabled={!context || parsedFundAmount === undefined}
          onClick={() => void prepare("Fund Rewards", (safeContext) => buildFundRewardsTransaction(safeContext, parsedFundAmount ?? 0n))}
          type="button"
        >
          Fund Rewards
        </button>
        <button disabled={!context || !isAdmin || poolState?.paused} onClick={() => void prepare("Pause Pool", buildPausePoolTransaction)} type="button">
          Pause Pool
        </button>
      </div>

      <div className="proposal-editor">
        <label className="amount-field">
          <span>Proposal</span>
          <input inputMode="numeric" onChange={(event) => onInspectedProposalIdChange(event.target.value)} value={inspectedProposalId} />
        </label>
        <label className="amount-field">
          <span>Action</span>
          <select onChange={(event) => setProposalKind(event.target.value as AdminProposalAction["kind"])} value={proposalKind}>
            <option value="set-reward-rate">Set reward rate</option>
            <option value="unpause-pool">Unpause pool</option>
            <option value="replace-admin">Replace admin</option>
          </select>
        </label>
        {proposalKind === "set-reward-rate" ? (
          <label className="amount-field">
            <span>New Rate</span>
            <input inputMode="decimal" onChange={(event) => setNewRateInput(event.target.value)} value={newRateInput} />
          </label>
        ) : null}
        {proposalKind === "replace-admin" ? (
          <>
            <label className="amount-field">
              <span>Old Admin</span>
              <input onChange={(event) => setOldAdminInput(event.target.value)} value={oldAdminInput} />
            </label>
            <label className="amount-field">
              <span>New Admin</span>
              <input onChange={(event) => setNewAdminInput(event.target.value)} value={newAdminInput} />
            </label>
          </>
        ) : null}
        <div className="proposal-action-row">
          <span>{actionLabel(selectedAction)}</span>
          <button
            disabled={!canCreateProposal}
            onClick={() => void prepare("Create Proposal", (safeContext) => buildCreateProposalTransaction(safeContext, nextProposalId, selectedAction as AdminProposalAction))}
            type="button"
          >
            Create Proposal
          </button>
        </div>
      </div>

      <dl className="detail-list horizontal">
        <div>
          <dt>Loaded Action</dt>
          <dd>{actionLabel(proposalState?.action)}</dd>
        </div>
        <div>
          <dt>Approvals</dt>
          <dd>{proposalState ? `${proposalState.approvalCount}/2` : "Missing"}</dd>
        </div>
        <div>
          <dt>Expires Slot</dt>
          <dd>{proposalState?.expiresAtSlot.toString() ?? "0"}</dd>
        </div>
        <div>
          <dt>Proposal Epoch</dt>
          <dd>{proposalState?.adminEpoch.toString() ?? "0"}</dd>
        </div>
        <div>
          <dt>Executed</dt>
          <dd>{proposalState?.executed ? "Yes" : "No"}</dd>
        </div>
        <div>
          <dt>Creator</dt>
          <dd>{proposalState ? shortAddress(proposalState.creator, 6) : "None"}</dd>
        </div>
      </dl>

      <div className="action-grid admin-action-grid">
        <button disabled={!context || !isAdmin} onClick={() => void prepareProposal("Approve Proposal", buildApproveProposalTransaction)} type="button">
          Approve
        </button>
        <button disabled={!context || !proposalState || proposalState.approvalCount < 2 || proposalState.executed} onClick={() => void prepareProposal("Execute Proposal", buildExecuteProposalTransaction)} type="button">
          Execute
        </button>
        <button
          disabled={!context || !proposalState}
          onClick={() => void prepareProposal("Close Proposal", (safeContext, proposalId) => buildCloseProposalTransaction(safeContext, proposalId, proposalState?.creator ?? safeContext.user))}
          type="button"
        >
          Close
        </button>
      </div>

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
            <ShieldAlert size={17} />
            <span>{status.label}: {status.message}</span>
          </>
        ) : null}
      </div>
    </section>
  );
}
