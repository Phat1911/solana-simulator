# Slot Staking

Slot Staking is an educational Solana DeFi project that implements a real slot-based staking pool with Rust, Anchor, the original SPL Token Program, a wallet-connected frontend, and a public Devnet deployment.

> This repository is a research and portfolio project. It is not production-ready, has not received an independent professional audit, and must not custody assets with real value.

## Research Question

> How can a Solana staking pool distribute slot-based rewards proportionally without iterating over every staker, while keeping principal and reward obligations solvent?

The project focuses on accumulated reward-per-stake accounting, integer precision, funded reward limits, PDA-controlled token vaults, pause safety, and scoped multisig administration.

## Current Status

Milestones 1 through 23 are complete. The repository now has the baseline
workspace, staking and faucet program crates, placeholder frontend and
evidence directories, private-note ignore rules, the first audit/test evidence
structure, and pure checked arithmetic helpers for six-decimal token units and
scaled reward units. It also has pure global reward checkpoint math for
slot-based emissions, funded-budget caps, pause gaps, zero-stake checkpoints,
exhaustion, partial final emission, and scaled rounding remainders. Position
settlement and claim math now preserve reward debt, pending scaled rewards,
whole-token claims, fractional carry, and emergency forfeiture conservation.
The staking program now defines fixed-size Anchor-serializable Pool, Position,
and Proposal account schemas plus canonical PDA derivation helpers. Pool
initialization now creates the canonical Pool PDA and Pool Authority-owned stake
and reward vault ATAs, validates three distinct admins, distinct six-decimal
mints, the original SPL Token Program, and starts each pool paused with reward
emission set to zero. Users can now open their canonical Position PDA for a
pool, fund rewards through validated SPL Token transfers, stake from canonical
STAKE ATAs, unstake principal through the Pool Authority PDA, claim whole REWARD
base units, and close a position only when it contains no stake or reward
accounting. Any current admin can now pause a pool after checkpointing, paused
slots do not generate rewards, and users can emergency-withdraw principal while
forfeiting their complete scaled reward entitlement back into the unallocated
reward budget.
The staking program now also supports the scoped 2-of-3 proposal system for
reward-rate changes, proposal-based unpause, and one-admin replacement, with
proposal expiry, stale-epoch invalidation, replay prevention, and safe proposal
closure back to the recorded creator.
The separate demo faucet program now mints exactly `1_000 STAKE` once per
wallet through a Faucet Authority PDA, creates the user's canonical STAKE ATA
when needed, and records a permanent claim receipt PDA.
The cross-instruction LiteSVM suite now drives two users through funding,
staking, reward claims, governance rate changes, pause/unpause, normal unstake,
and emergency forfeiture. It also replays fixed-seed operation sequences and
checks principal solvency, exact scaled reward conservation, canonical account
binding, token-supply conservation, and atomic rollback after rejected calls.
The repository now also includes Milestone 19 setup automation for localnet and
Devnet configuration validation, mint creation, REWARD supply minting,
REWARD mint-authority revocation, pool initialization, reward funding, and
public deployment metadata generation without storing key material. The
frontend is now a real Next.js 16 and React 19 app with generated Anchor IDL
artifacts, Wallet Standard discovery, Solana Kit read-only RPC plumbing,
deployment/account status display, Tailwind CSS styling, and reusable bigint
formatting helpers for six-decimal token base units. The frontend also prepares
canonical user transactions for faucet claims, position opening, first-stake
bundling, stake, unstake, claim, emergency withdraw, and position close; it
decodes Pool and Position account data, displays user balances and estimated
pending rewards, and keeps transaction amounts as integer base units.
The admin console now prepares funding, immediate pause, proposal creation,
approval, execution, and closure transactions; it displays exact proposal
parameters, approval count, admin epoch, expiry, execution state, and whether
the connected wallet is currently eligible as an admin. A local end-to-end
harness now starts a fresh validator, loads both programs, creates fixture
wallets and mints, initializes/funds the pool, and runs the current program and
browser suites from one command.

Milestones 24 and 25 now have Devnet smoke tooling and public research/audit
package drafts prepared. Signed browser submission, the real Devnet deploy,
Devnet smoke output, and final evidence insertion remain planned handoff work.

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

## Stack

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

Package and toolchain versions are documented in the baseline workspace:

- Rust toolchain: `stable` via `rust-toolchain.toml`
- Anchor CLI target: `0.31.1`
- Solana CLI target: `2.1.21`
- Node.js target: `22.18.0`
- npm target: `10.9.3`

The WSL `crypto` environment has the Rust stable toolchain plus the pinned
Anchor and Solana toolchains available. The original Windows workspace still
has a rustup shim temp-file limitation, so WSL is the recommended development
environment.

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

## Development Environment

Anchor development on Windows will use WSL. The storage migration/setup will be handled as a separate preparation task.

The current baseline workflow is:

```bash
cargo fmt-all
cargo lint
cargo test-all
cd app && npm run test
```

Later milestones will add the full Anchor and frontend workflows:

```bash
anchor build
anchor test
npm run build
```

## Local And Devnet Setup

Milestone 19 adds a deployment helper with a safe dry-run path:

```bash
cp scripts/devnet/config.example.json scripts/devnet/config.local.json
scripts/devnet/setup.sh validate
scripts/devnet/setup.sh dry-run
```

For real setup, keep mint keypairs under `.private/`, keep private RPC URLs out
of committed config, and write only public addresses to `deployments/devnet.json`.
The helper creates six-decimal STAKE and REWARD mints, mints the fixed REWARD
supply, revokes REWARD mint authority, initializes the pool, funds rewards
through the staking program, and can be rerun without double-funding the
configured initial amount.

After deploying both programs to Devnet and running setup, Milestone 24 smoke
verification is:

```bash
scripts/devnet/smoke.sh
```

If Devnet SOL runs out after one program deploys, top up the same wallet and
deploy only the missing program, for example:

```bash
anchor deploy -p staking_pool --provider.cluster devnet
```

Record public addresses, transaction signatures, smoke output, and Explorer
links using [DEVNET_EVIDENCE_TEMPLATE.md](./deployments/DEVNET_EVIDENCE_TEMPLATE.md).

## Local E2E Harness

Milestone 23 adds:

```bash
tests/local-e2e/run.sh
```

This script uses `.private/local-e2e` for fixture keypairs and writes
machine-specific public metadata to `deployments/local-e2e.generated.json`.
It can take a while because it builds Anchor programs, starts a validator,
runs setup, and then runs program plus browser checks.

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
