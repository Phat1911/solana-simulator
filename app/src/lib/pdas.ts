import {
  address,
  getAddressEncoder,
  getProgramDerivedAddress,
  type Address,
  type ProgramDerivedAddress,
} from "@solana/kit";

export const TOKEN_PROGRAM_ADDRESS = address("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
export const ASSOCIATED_TOKEN_PROGRAM_ADDRESS = address("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
export const SYSTEM_PROGRAM_ADDRESS = address("11111111111111111111111111111111");

const POSITION_SEED = "position";
const POOL_AUTHORITY_SEED = "pool-authority";
const PROPOSAL_SEED = "proposal";
const FAUCET_AUTHORITY_SEED = "faucet-authority";
const FAUCET_CLAIM_SEED = "faucet-claim";

const addressEncoder = getAddressEncoder();

function encodeAddress(value: string | Address) {
  return addressEncoder.encode(typeof value === "string" ? address(value) : value);
}

// Milestone 21: frontend PDA recipes mirror the on-chain seed rules exactly.
export async function derivePositionPda(stakingProgram: string, pool: string, user: string): Promise<ProgramDerivedAddress> {
  return getProgramDerivedAddress({
    programAddress: address(stakingProgram),
    seeds: [POSITION_SEED, encodeAddress(pool), encodeAddress(user)],
  });
}

export async function derivePoolAuthorityPda(stakingProgram: string, pool: string): Promise<ProgramDerivedAddress> {
  return getProgramDerivedAddress({
    programAddress: address(stakingProgram),
    seeds: [POOL_AUTHORITY_SEED, encodeAddress(pool)],
  });
}

export async function deriveProposalPda(stakingProgram: string, pool: string, proposalId: bigint): Promise<ProgramDerivedAddress> {
  const encodedProposalId = new Uint8Array(8);
  new DataView(encodedProposalId.buffer).setBigUint64(0, proposalId, true);

  return getProgramDerivedAddress({
    programAddress: address(stakingProgram),
    seeds: [PROPOSAL_SEED, encodeAddress(pool), encodedProposalId],
  });
}

export async function deriveFaucetAuthorityPda(faucetProgram: string, stakeMint: string): Promise<ProgramDerivedAddress> {
  return getProgramDerivedAddress({
    programAddress: address(faucetProgram),
    seeds: [FAUCET_AUTHORITY_SEED, encodeAddress(stakeMint)],
  });
}

export async function deriveFaucetClaimPda(
  faucetProgram: string,
  stakeMint: string,
  claimant: string,
): Promise<ProgramDerivedAddress> {
  return getProgramDerivedAddress({
    programAddress: address(faucetProgram),
    seeds: [FAUCET_CLAIM_SEED, encodeAddress(stakeMint), encodeAddress(claimant)],
  });
}

export async function deriveAssociatedTokenAccount(owner: string, mint: string): Promise<ProgramDerivedAddress> {
  return getProgramDerivedAddress({
    programAddress: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
    seeds: [encodeAddress(owner), encodeAddress(TOKEN_PROGRAM_ADDRESS), encodeAddress(mint)],
  });
}
