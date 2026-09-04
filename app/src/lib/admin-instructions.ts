import { AccountRole, address, getAddressEncoder, type AccountMeta, type Instruction } from "@solana/kit";

import { toU64Amount } from "@/lib/amounts";
import {
  SYSTEM_PROGRAM_ADDRESS,
  TOKEN_PROGRAM_ADDRESS,
  deriveAssociatedTokenAccount,
  deriveProposalPda,
} from "@/lib/pdas";

export type AdminProposalAction =
  | { kind: "set-reward-rate"; newRate: bigint }
  | { kind: "unpause-pool" }
  | { kind: "replace-admin"; oldAdmin: string; newAdmin: string };

export type AdminActionKind =
  | "fund-rewards"
  | "pause-pool"
  | "create-proposal"
  | "approve-proposal"
  | "execute-proposal"
  | "close-proposal";

export type AdminActionContext = {
  user: string;
  stakingProgram: string;
  pool: string;
  rewardMint: string;
  rewardVault: string;
};

export type PreparedAdminTransaction = {
  kind: AdminActionKind;
  instructions: readonly Instruction[];
  derivedAccounts: Record<string, string>;
};

const DISCRIMINATORS = {
  fundRewards: [114, 64, 163, 112, 175, 167, 19, 121],
  pausePool: [160, 15, 12, 189, 160, 0, 243, 245],
  createProposal: [132, 116, 68, 174, 216, 160, 198, 22],
  approveProposal: [136, 108, 102, 85, 98, 114, 7, 147],
  executeProposal: [186, 60, 116, 133, 108, 128, 111, 28],
  closeProposal: [213, 178, 139, 19, 50, 191, 82, 245],
} as const;

const addressEncoder = getAddressEncoder();

function addressBytes(value: string): Uint8Array {
  return new Uint8Array(addressEncoder.encode(address(value)));
}

function meta(value: string, role: AccountRole): AccountMeta {
  return { address: address(value), role };
}

function readonly(value: string): AccountMeta {
  return meta(value, AccountRole.READONLY);
}

function writable(value: string): AccountMeta {
  return meta(value, AccountRole.WRITABLE);
}

function readonlySigner(value: string): AccountMeta {
  return meta(value, AccountRole.READONLY_SIGNER);
}

function writableSigner(value: string): AccountMeta {
  return meta(value, AccountRole.WRITABLE_SIGNER);
}

function u64Bytes(value: bigint): Uint8Array {
  const encoded = new Uint8Array(8);
  new DataView(encoded.buffer).setBigUint64(0, toU64Amount(value), true);
  return encoded;
}

function instruction(programAddress: string, accounts: readonly AccountMeta[], encodedData: Uint8Array): Instruction {
  return {
    accounts,
    data: encodedData,
    programAddress: address(programAddress),
  };
}

function data(discriminator: readonly number[], ...parts: readonly Uint8Array[]): Uint8Array {
  const encoded = new Uint8Array(discriminator.length + parts.reduce((total, part) => total + part.length, 0));
  encoded.set(discriminator);
  let offset = discriminator.length;

  for (const part of parts) {
    encoded.set(part, offset);
    offset += part.length;
  }

  return encoded;
}

export function encodeProposalAction(action: AdminProposalAction): Uint8Array {
  if (action.kind === "set-reward-rate") {
    return data([0], u64Bytes(action.newRate));
  }

  if (action.kind === "unpause-pool") {
    return new Uint8Array([1]);
  }

  return data([2], addressBytes(action.oldAdmin), addressBytes(action.newAdmin));
}

// Milestone 22: admin builders mirror the staking program's governance account order.
export async function buildFundRewardsTransaction(
  context: AdminActionContext,
  amount: bigint,
  sourceRewardAccount?: string,
): Promise<PreparedAdminTransaction> {
  const [defaultSourceRewardAccount] = await deriveAssociatedTokenAccount(context.user, context.rewardMint);
  const resolvedSourceRewardAccount = sourceRewardAccount ?? defaultSourceRewardAccount.toString();

  return {
    kind: "fund-rewards",
    instructions: [
      instruction(
        context.stakingProgram,
        [
          writableSigner(context.user),
          writable(context.pool),
          writable(resolvedSourceRewardAccount),
          readonly(context.rewardMint),
          writable(context.rewardVault),
          readonly(TOKEN_PROGRAM_ADDRESS.toString()),
        ],
        data(DISCRIMINATORS.fundRewards, u64Bytes(amount)),
      ),
    ],
    derivedAccounts: { sourceRewardAccount: resolvedSourceRewardAccount },
  };
}

export async function buildPausePoolTransaction(context: AdminActionContext): Promise<PreparedAdminTransaction> {
  return {
    kind: "pause-pool",
    instructions: [
      instruction(context.stakingProgram, [writableSigner(context.user), writable(context.pool)], data(DISCRIMINATORS.pausePool)),
    ],
    derivedAccounts: {},
  };
}

export async function buildCreateProposalTransaction(
  context: AdminActionContext,
  proposalId: bigint,
  action: AdminProposalAction,
): Promise<PreparedAdminTransaction> {
  const [proposal] = await deriveProposalPda(context.stakingProgram, context.pool, proposalId);

  return {
    kind: "create-proposal",
    instructions: [
      instruction(
        context.stakingProgram,
        [
          writableSigner(context.user),
          writable(context.pool),
          writable(proposal.toString()),
          readonly(SYSTEM_PROGRAM_ADDRESS.toString()),
        ],
        data(DISCRIMINATORS.createProposal, u64Bytes(proposalId), encodeProposalAction(action)),
      ),
    ],
    derivedAccounts: { proposal: proposal.toString() },
  };
}

export async function buildApproveProposalTransaction(
  context: AdminActionContext,
  proposalId: bigint,
): Promise<PreparedAdminTransaction> {
  const [proposal] = await deriveProposalPda(context.stakingProgram, context.pool, proposalId);

  return {
    kind: "approve-proposal",
    instructions: [
      instruction(context.stakingProgram, [readonlySigner(context.user), readonly(context.pool), writable(proposal.toString())], data(DISCRIMINATORS.approveProposal)),
    ],
    derivedAccounts: { proposal: proposal.toString() },
  };
}

export async function buildExecuteProposalTransaction(
  context: AdminActionContext,
  proposalId: bigint,
): Promise<PreparedAdminTransaction> {
  const [proposal] = await deriveProposalPda(context.stakingProgram, context.pool, proposalId);

  return {
    kind: "execute-proposal",
    instructions: [
      instruction(context.stakingProgram, [writable(context.pool), writable(proposal.toString())], data(DISCRIMINATORS.executeProposal)),
    ],
    derivedAccounts: { proposal: proposal.toString() },
  };
}

export async function buildCloseProposalTransaction(
  context: AdminActionContext,
  proposalId: bigint,
  creator: string,
): Promise<PreparedAdminTransaction> {
  const [proposal] = await deriveProposalPda(context.stakingProgram, context.pool, proposalId);

  return {
    kind: "close-proposal",
    instructions: [
      instruction(
        context.stakingProgram,
        [writableSigner(context.user), readonly(context.pool), writable(proposal.toString()), writable(creator)],
        data(DISCRIMINATORS.closeProposal),
      ),
    ],
    derivedAccounts: { proposal: proposal.toString(), creator },
  };
}
