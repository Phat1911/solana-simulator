import { describe, expect, it } from "vitest";

import {
  ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
  TOKEN_PROGRAM_ADDRESS,
  deriveAssociatedTokenAccount,
  deriveFaucetAuthorityPda,
  deriveFaucetClaimPda,
  derivePoolAuthorityPda,
  derivePositionPda,
  deriveProposalPda,
} from "@/lib/pdas";

const STAKING_PROGRAM = "Fg6PaFpoGXkYsidMpWxTWqkFrnDRBTTnyW6m9n6eGJZ";
const FAUCET_PROGRAM = "J12YAqC7dWbVWVAFveRdJ8SJ3sYc6roP3WJekhy4bDkM";
const POOL = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const USER_A = "11111111111111111111111111111111";
const USER_B = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
const STAKE_MINT = "So11111111111111111111111111111111111111112";
const REWARD_MINT = "SysvarRent111111111111111111111111111111111";

describe("canonical frontend derivations", () => {
  it("derives stable pool and user PDAs", async () => {
    const [poolAuthority] = await derivePoolAuthorityPda(STAKING_PROGRAM, POOL);
    const [position] = await derivePositionPda(STAKING_PROGRAM, POOL, USER_A);
    const [otherPosition] = await derivePositionPda(STAKING_PROGRAM, POOL, USER_B);

    expect(poolAuthority.toString()).not.toBe(POOL);
    expect(position.toString()).not.toBe(otherPosition.toString());
  });

  it("derives faucet authority and one-claim receipt by mint and user", async () => {
    const [faucetAuthority] = await deriveFaucetAuthorityPda(FAUCET_PROGRAM, STAKE_MINT);
    const [receiptA] = await deriveFaucetClaimPda(FAUCET_PROGRAM, STAKE_MINT, USER_A);
    const [receiptB] = await deriveFaucetClaimPda(FAUCET_PROGRAM, STAKE_MINT, USER_B);

    expect(faucetAuthority.toString()).not.toBe(STAKE_MINT);
    expect(receiptA.toString()).not.toBe(receiptB.toString());
  });

  it("derives canonical token accounts from owner and mint", async () => {
    const [stakeAta] = await deriveAssociatedTokenAccount(USER_A, STAKE_MINT);
    const [rewardAta] = await deriveAssociatedTokenAccount(USER_A, REWARD_MINT);

    expect(TOKEN_PROGRAM_ADDRESS.toString()).toBe("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
    expect(ASSOCIATED_TOKEN_PROGRAM_ADDRESS.toString()).toBe("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
    expect(stakeAta.toString()).not.toBe(rewardAta.toString());
  });

  it("derives proposal PDAs by pool and little-endian proposal id", async () => {
    const [proposalZero] = await deriveProposalPda(STAKING_PROGRAM, POOL, 0n);
    const [proposalOne] = await deriveProposalPda(STAKING_PROGRAM, POOL, 1n);

    expect(proposalZero.toString()).not.toBe(proposalOne.toString());
    expect(proposalZero.toString()).not.toBe(POOL);
  });
});
