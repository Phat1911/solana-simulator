# Slot Staking

Slot Staking is an educational Solana DeFi project that implements a real slot-based staking pool with Rust, Anchor, the original SPL Token Program, a wallet-connected frontend, and a public Devnet deployment.

> This repository is a research and portfolio project. It is not production-ready, has not received an independent professional audit, and must not custody assets with real value.

## Research Question

> How can a Solana staking pool distribute slot-based rewards proportionally without iterating over every staker, while keeping principal and reward obligations solvent?

The project focuses on accumulated reward-per-stake accounting, integer precision, funded reward limits, PDA-controlled token vaults, pause safety, and scoped multisig administration.

## Current Status

Specification drafted; implementation has not started.

The authoritative behavior and acceptance criteria are in [SPEC.md](./SPEC.md).

## Architecture

```text
Browser wallet and Next.js frontend
                |
                | Solana transactions and RPC reads
                v
        Anchor Staking Program
          |       |        |
          |       |        +-- Proposal PDAs
          |       +----------- Position PDAs
          +------------------- Pool State PDA
                |
                | CPI with Pool Authority PDA
                v
        Original SPL Token Program
          |                       |
          +-- STAKE Vault ATA     +-- REWARD Vault ATA

        Anchor Demo Faucet Program
                |
                | SPL Token mint_to CPI
                v
        User STAKE ATA + Claim Receipt PDA
```

The Staking Program cannot mint tokens. A separate Devnet-only faucet gives each wallet `1,000 STAKE` once. REWARD has a fixed initial supply and can enter the pool only through voluntary funding transfers.

## Core Behavior

- Users stake STAKE and earn separately funded REWARD.
- Rewards are emitted pool-wide per trusted Solana slot.
- Distribution is proportional and does not loop over every staker.
- Token amounts use six decimals and checked integer arithmetic.
- Reward precision and liabilities use scaled `u128` accounting.
- Empty pools consume no rewards.
- Emissions stop at the funded limit and may end with a partial allocation.
- Users claim independently; fractional rewards carry forward.
- Normal unstaking preserves earned rewards for later claim.
- Emergency withdrawal returns principal and forfeits rewards back to the emission budget.
- Pausing freezes emissions and blocks stake and claim while preserving withdrawal.

## Administration

The pool has three admins and a `2-of-3` approval threshold.

- Any one admin may pause immediately.
- Reward-rate changes, unpause, and admin replacement use on-chain Proposal PDAs.
- Proposal execution is permissionless after two valid approvals.
- Proposal expiry, one-time execution, and `admin_epoch` checks prevent stale administrative authority.
- Admins cannot directly withdraw stake or reward vault assets.

## Solana Accounts

```text
Pool State PDA      shared configuration and accounting
Pool Authority PDA token authority for both vaults
Position PDA        one user's stake and reward state
Proposal PDA        one exact approved admin command
Faucet Claim PDA    one-time Devnet faucet receipt
```

The Stake and Reward Vaults are canonical ATAs owned by the Pool Authority PDA. User deposits and payouts use the user's canonical token ATAs.

## Planned Stack

### Programs

- Rust
- Anchor
- Original SPL Token Program
- LiteSVM
- Solana Devnet

### Frontend

- Next.js 16
- React 19
- TypeScript
- `@solana/kit`
- Wallet Standard
- Tailwind CSS
- Vitest, Testing Library, and Playwright

Package and toolchain versions will be pinned together during workspace setup to preserve Anchor and Solana compatibility.

## Testing

The project uses four levels of evidence:

1. Pure Rust tests for reward arithmetic and invariants
2. LiteSVM tests for compiled programs, PDAs, signers, SPL CPIs, and attacks
3. Local RPC and frontend end-to-end workflows
4. Public Devnet smoke tests with Explorer evidence

Security review happens throughout implementation. Each milestone adds adversarial tests and updates the public self-audit. The final audit will state its limitations and will not claim independent review.

## Planned Repository Layout

```text
programs/staking_pool/   Anchor staking program
programs/demo_faucet/    Devnet-only faucet program
app/                     wallet-connected frontend
tests/                   LiteSVM and local end-to-end tests
scripts/                 setup and Devnet smoke scripts
docs/diagrams/           account and transaction diagrams
docs/RESEARCH.md         portfolio research article
deployments/devnet.json  public deployment evidence
SPEC.md                  authoritative specification
AUDIT.md                 living security self-audit
```

Beginner learning notes are intentionally kept in a private, gitignored path instead of the public specification and portfolio article.

## Development Environment

Anchor development on Windows will use WSL. The current machine has limited free space on `C:`, so WSL storage, toolchains, dependency caches, and build artifacts should be placed on `D:` before installation. The storage migration/setup will be handled as a separate preparation task.

Once the workspace is scaffolded, the expected workflow will include:

```bash
anchor build
cargo test
anchor test
npm run test
npm run build
```

These commands are targets for the implementation phase and are not yet available in this specification-only repository.

## Safety Notice

Known trust assumptions include the Devnet program upgrade authority, a single-admin pause capability, `2-of-3` administrative control, a Sybilable test faucet, public RPC availability, and possible Devnet resets. Direct token transfers into vaults are not credited and may become locked surplus.

See [SPEC.md](./SPEC.md) for the complete threat model, formulas, instruction contracts, and acceptance criteria.

## References

- [Solana programs](https://solana.com/docs/core/programs)
- [Solana program deployment](https://solana.com/docs/programs/deploying)
- [Calling the Token Program via CPI](https://solana.com/docs/tokens/advanced/cpi)
- [Anchor documentation](https://www.anchor-lang.com/docs)
- [Anchor LiteSVM testing](https://www.anchor-lang.com/docs/testing/litesvm)
- [Official Next.js + Anchor template](https://solana.com/developers/templates/nextjs-anchor)
