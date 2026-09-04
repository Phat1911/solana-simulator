import { AccountRole, address, type AccountMeta, type Instruction } from "@solana/kit";

import { toU64Amount } from "@/lib/amounts";
import {
  ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
  SYSTEM_PROGRAM_ADDRESS,
  TOKEN_PROGRAM_ADDRESS,
  deriveAssociatedTokenAccount,
  deriveFaucetAuthorityPda,
  deriveFaucetClaimPda,
  derivePoolAuthorityPda,
  derivePositionPda,
} from "@/lib/pdas";

export type UserActionKind =
  | "claim-test-stake"
  | "open-position"
  | "stake"
  | "unstake"
  | "claim-rewards"
  | "emergency-withdraw"
  | "close-position";

export type UserActionContext = {
  user: string;
  stakingProgram: string;
  demoFaucetProgram: string;
  pool: string;
  stakeMint: string;
  rewardMint: string;
  stakeVault: string;
  rewardVault: string;
};

export type PreparedUserTransaction = {
  kind: UserActionKind;
  instructions: readonly Instruction[];
  derivedAccounts: Record<string, string>;
};

const DISCRIMINATORS = {
  claimTestStake: [160, 5, 137, 252, 241, 44, 153, 233],
  openPosition: [135, 128, 47, 77, 15, 152, 240, 49],
  stake: [206, 176, 202, 18, 200, 209, 179, 108],
  unstake: [90, 95, 107, 42, 205, 124, 50, 225],
  claimRewards: [4, 144, 132, 71, 116, 23, 151, 80],
  emergencyWithdraw: [239, 45, 203, 64, 150, 73, 218, 92],
  closePosition: [123, 134, 81, 0, 49, 68, 98, 98],
} as const;

function meta(value: string, role: AccountRole): AccountMeta {
  return { address: address(value), role };
}

function readonly(value: string): AccountMeta {
  return meta(value, AccountRole.READONLY);
}

function writable(value: string): AccountMeta {
  return meta(value, AccountRole.WRITABLE);
}

function writableSigner(value: string): AccountMeta {
  return meta(value, AccountRole.WRITABLE_SIGNER);
}

function data(discriminator: readonly number[], amount?: bigint): Uint8Array {
  if (amount === undefined) {
    return new Uint8Array(discriminator);
  }

  const encoded = new Uint8Array(16);
  encoded.set(discriminator);
  new DataView(encoded.buffer).setBigUint64(8, toU64Amount(amount), true);
  return encoded;
}

function instruction(programAddress: string, accounts: readonly AccountMeta[], encodedData: Uint8Array): Instruction {
  return {
    accounts,
    data: encodedData,
    programAddress: address(programAddress),
  };
}

function createIdempotentAtaInstruction(payer: string, owner: string, mint: string, ata: string): Instruction {
  return instruction(
    ASSOCIATED_TOKEN_PROGRAM_ADDRESS.toString(),
    [
      writableSigner(payer),
      writable(ata),
      readonly(owner),
      readonly(mint),
      readonly(SYSTEM_PROGRAM_ADDRESS.toString()),
      readonly(TOKEN_PROGRAM_ADDRESS.toString()),
    ],
    new Uint8Array([1]),
  );
}

async function commonDerived(context: UserActionContext) {
  const [[position], [poolAuthority], [userStakeAccount], [userRewardAccount]] = await Promise.all([
    derivePositionPda(context.stakingProgram, context.pool, context.user),
    derivePoolAuthorityPda(context.stakingProgram, context.pool),
    deriveAssociatedTokenAccount(context.user, context.stakeMint),
    deriveAssociatedTokenAccount(context.user, context.rewardMint),
  ]);

  return {
    position: position.toString(),
    poolAuthority: poolAuthority.toString(),
    userStakeAccount: userStakeAccount.toString(),
    userRewardAccount: userRewardAccount.toString(),
  };
}

// Milestone 21: user action builders produce canonical account metas and Anchor data.
export async function buildClaimTestStakeTransaction(context: UserActionContext): Promise<PreparedUserTransaction> {
  const [[faucetAuthority], [claimReceipt], [claimantStakeAccount]] = await Promise.all([
    deriveFaucetAuthorityPda(context.demoFaucetProgram, context.stakeMint),
    deriveFaucetClaimPda(context.demoFaucetProgram, context.stakeMint, context.user),
    deriveAssociatedTokenAccount(context.user, context.stakeMint),
  ]);

  const derivedAccounts = {
    faucetAuthority: faucetAuthority.toString(),
    claimReceipt: claimReceipt.toString(),
    claimantStakeAccount: claimantStakeAccount.toString(),
  };

  return {
    kind: "claim-test-stake",
    instructions: [
      instruction(
        context.demoFaucetProgram,
        [
          writableSigner(context.user),
          writable(context.stakeMint),
          readonly(derivedAccounts.faucetAuthority),
          writable(derivedAccounts.claimantStakeAccount),
          writable(derivedAccounts.claimReceipt),
          readonly(TOKEN_PROGRAM_ADDRESS.toString()),
          readonly(ASSOCIATED_TOKEN_PROGRAM_ADDRESS.toString()),
          readonly(SYSTEM_PROGRAM_ADDRESS.toString()),
        ],
        data(DISCRIMINATORS.claimTestStake),
      ),
    ],
    derivedAccounts,
  };
}

export async function buildOpenPositionTransaction(context: UserActionContext): Promise<PreparedUserTransaction> {
  const { position } = await commonDerived(context);

  return {
    kind: "open-position",
    instructions: [
      instruction(
        context.stakingProgram,
        [writableSigner(context.user), readonly(context.pool), writable(position), readonly(SYSTEM_PROGRAM_ADDRESS.toString())],
        data(DISCRIMINATORS.openPosition),
      ),
    ],
    derivedAccounts: { position },
  };
}

export async function buildStakeTransaction(
  context: UserActionContext,
  amount: bigint,
  options: { includeOpenPosition?: boolean } = {},
): Promise<PreparedUserTransaction> {
  const derived = await commonDerived(context);
  const stakeInstruction = instruction(
    context.stakingProgram,
    [
      writableSigner(context.user),
      writable(context.pool),
      writable(derived.position),
      readonly(context.stakeMint),
      readonly(context.rewardMint),
      writable(derived.userStakeAccount),
      writable(context.stakeVault),
      writable(context.rewardVault),
      readonly(TOKEN_PROGRAM_ADDRESS.toString()),
    ],
    data(DISCRIMINATORS.stake, amount),
  );

  const open = options.includeOpenPosition ? await buildOpenPositionTransaction(context) : undefined;

  return {
    kind: "stake",
    instructions: open ? [...open.instructions, stakeInstruction] : [stakeInstruction],
    derivedAccounts: derived,
  };
}

export async function buildUnstakeTransaction(context: UserActionContext, amount: bigint): Promise<PreparedUserTransaction> {
  const derived = await commonDerived(context);

  return {
    kind: "unstake",
    instructions: [
      instruction(
        context.stakingProgram,
        [
          writableSigner(context.user),
          writable(context.pool),
          readonly(derived.poolAuthority),
          writable(derived.position),
          readonly(context.stakeMint),
          readonly(context.rewardMint),
          writable(derived.userStakeAccount),
          writable(context.stakeVault),
          writable(context.rewardVault),
          readonly(TOKEN_PROGRAM_ADDRESS.toString()),
        ],
        data(DISCRIMINATORS.unstake, amount),
      ),
    ],
    derivedAccounts: derived,
  };
}

export async function buildClaimRewardsTransaction(
  context: UserActionContext,
  options: { ensureRewardAta?: boolean } = {},
): Promise<PreparedUserTransaction> {
  const derived = await commonDerived(context);
  const claim = instruction(
    context.stakingProgram,
    [
      writableSigner(context.user),
      writable(context.pool),
      readonly(derived.poolAuthority),
      writable(derived.position),
      readonly(context.rewardMint),
      writable(context.rewardVault),
      writable(derived.userRewardAccount),
      readonly(TOKEN_PROGRAM_ADDRESS.toString()),
    ],
    data(DISCRIMINATORS.claimRewards),
  );

  return {
    kind: "claim-rewards",
    instructions: options.ensureRewardAta
      ? [createIdempotentAtaInstruction(context.user, context.user, context.rewardMint, derived.userRewardAccount), claim]
      : [claim],
    derivedAccounts: derived,
  };
}

export async function buildEmergencyWithdrawTransaction(context: UserActionContext): Promise<PreparedUserTransaction> {
  const derived = await commonDerived(context);

  return {
    kind: "emergency-withdraw",
    instructions: [
      instruction(
        context.stakingProgram,
        [
          writableSigner(context.user),
          writable(context.pool),
          readonly(derived.poolAuthority),
          writable(derived.position),
          readonly(context.stakeMint),
          readonly(context.rewardMint),
          writable(derived.userStakeAccount),
          writable(context.stakeVault),
          writable(context.rewardVault),
          readonly(TOKEN_PROGRAM_ADDRESS.toString()),
        ],
        data(DISCRIMINATORS.emergencyWithdraw),
      ),
    ],
    derivedAccounts: derived,
  };
}

export async function buildClosePositionTransaction(context: UserActionContext): Promise<PreparedUserTransaction> {
  const { position } = await commonDerived(context);

  return {
    kind: "close-position",
    instructions: [
      instruction(context.stakingProgram, [writableSigner(context.user), readonly(context.pool), writable(position)], data(DISCRIMINATORS.closePosition)),
    ],
    derivedAccounts: { position },
  };
}
