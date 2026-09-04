# Slot Staking Research Narrative

Slot Staking is an educational Solana Devnet research project. It implements a
funded staking pool in Rust and Anchor, exercises the original SPL Token
Program through CPIs, and documents how proportional slot-based rewards can be
distributed without iterating over every user.

The project is not production-ready and has not received an independent
professional audit. Its purpose is to make the accounting model, account model,
and testing evidence easy to inspect.

## Research Question

How can a Solana staking pool distribute slot-based rewards proportionally
without iterating over every staker, while keeping principal and reward
obligations solvent?

The design answer is accumulated reward-per-stake accounting:

```text
delta_acc = emitted_reward_scaled / total_staked
position_earned = position_stake * (pool_accumulator - position_reward_debt)
```

The pool updates one global accumulator at checkpoints. Each user position
settles against that accumulator when the user acts. This is the same broad
idea as many Solidity staking contracts that use `accRewardPerShare`, but here
the state is split across Solana accounts and token movement happens by CPI to
the original SPL Token Program.

## Account Model

The Staking Program owns custom Pool, Position, and Proposal accounts. The SPL
Token Program owns mint and token-account data.

```text
Pool PDA        stores global accounting and immutable mint/vault addresses
Position PDA    stores one user's principal and reward debt
Proposal PDA    stores one immutable admin action and approvals
Pool Authority  PDA signer only; owns both vault ATAs
Stake Vault     Pool Authority's STAKE ATA
Reward Vault    Pool Authority's REWARD ATA
```

The Demo Faucet is a separate program. It can mint STAKE once per wallet by
creating a permanent Faucet Claim receipt PDA. It has no access to staking
vaults or reward accounting.

## Reward Accounting

Both STAKE and REWARD use six decimals. On-chain token amounts are stored as
base units, so `1 REWARD = 1_000_000` base units. Reward accounting uses
`PRECISION = 1_000_000_000` and `u128` scaled values to preserve fractional
entitlements.

The pool emits rewards by trusted Solana slot, not by wall-clock time. A
checkpoint calculates elapsed slots, desired emission, and the funded-budget
cap. If no one is staked, no rewards are consumed. If funding is exhausted,
emission stops.

Two solvency invariants anchor the design:

```text
stake_vault.amount >= pool.total_staked

remaining_reward_budget_scaled + allocated_liability_scaled
  <= reward_vault.amount * PRECISION
```

Direct token transfers into vaults are treated as surplus. They do not create
stake, budget, or claimable rewards.

## Administration

The pool has three admins. Any current admin can pause immediately. Unpause,
reward-rate changes, and admin replacement require a Proposal PDA with two
distinct current-admin approvals.

Each proposal stores exactly one allowlisted action:

```text
SetRewardRate { new_rate }
UnpausePool
ReplaceAdmin { old_admin, new_admin }
```

There is no arbitrary CPI, batching, cancellation, admin sweep, or vault
withdrawal instruction. Admin replacement increments `admin_epoch`, which makes
old proposals stale.

## Frontend

The frontend is a Next.js 16 and React 19 app using Wallet Standard and
`@solana/kit`. It derives the same PDAs and canonical ATAs as the programs,
decodes Pool, Position, and Proposal account data, and prepares transaction
messages for user and admin actions.

The frontend never sends trusted reward math to the program. Pending rewards
are labeled as estimates; on-chain checkpoint and settlement logic is
authoritative.

## Evidence

The repository includes:

- Pure Rust arithmetic tests for overflow, underflow, rounding, and settlement.
- LiteSVM tests for initialization, vault authority, staking, unstaking,
  claiming, pausing, emergency withdrawal, faucet claims, governance, rollback,
  and cross-instruction invariants.
- Frontend unit and browser tests for amount parsing, PDA derivation, account
  decoding, transaction builders, and action surfaces.
- A local E2E harness that starts a fresh validator, loads both programs,
  creates fixture mints/accounts, initializes and funds the pool, then runs the
  current program and browser suites.
- Devnet setup and smoke tooling that validates public deployment metadata
  against live RPC account data.

## Known Limits

This is Devnet educational software. Important limitations remain:

- No independent professional audit.
- Upgrade authority remains a trusted Devnet key.
- The faucet is not Sybil-resistant.
- The frontend currently prepares canonical transactions; live wallet
  submission is a future strengthening step.
- Devnet may reset or behave differently from a production mainnet environment.
- No fees, locks, compounding, oracle, admin sweep, pool closure, or production
  governance upgrade path is included.
