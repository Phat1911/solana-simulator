# Slot Staking: Technical Specification

Status: Draft approved for implementation  
Target network: Solana Devnet  
Production status: Educational and research-only; not production-ready

## 1. Purpose

Slot Staking is an end-to-end Solana staking research project built with Rust and Anchor. Users deposit a Devnet SPL token (`STAKE`) and earn a separately funded SPL token (`REWARD`) according to a pool-wide reward rate measured per Solana slot.

The project investigates this question:

> How can a Solana staking pool distribute slot-based rewards proportionally without iterating over every staker, while keeping principal and reward obligations solvent?

The deliverable includes real Anchor programs, original SPL Token execution, a wallet-connected frontend, local and Devnet testing, diagrams, deployment evidence, and a documented security self-audit.

## 2. Goals

- Implement a real Anchor staking program in Rust.
- Execute token minting and transfers through the original SPL Token Program.
- Deploy the programs and test assets to Solana Devnet.
- Provide a public, wallet-connected operational frontend.
- Use accumulated reward-per-stake accounting without iterating over all positions.
- Keep all recorded rewards backed by the reward vault.
- Preserve principal withdrawal while the pool is paused.
- Use PDA-controlled vault authority and explicit Anchor account constraints.
- Implement a scoped educational `2-of-3` administrative proposal system.
- Build security evidence continuously from tests and manual review.
- Present the result as a portfolio research article with diagrams.

## 3. Non-Goals

- Mainnet deployment or production-readiness claims
- An independent professional security audit
- Token-2022 and token extensions
- Oracles, token pricing, or APR guarantees
- Deposit or withdrawal fees
- Lock periods or reward multipliers
- Automatic compounding
- Transferable staking receipt tokens
- Arbitrary governance calls to external programs
- A pool directory or multi-pool frontend
- Admin withdrawal of stake or reward vault assets
- Pool closure or recovery of remaining funded rewards

## 4. System Components

### 4.1 Staking Program

An Anchor program that owns pool, position, and proposal state. It validates user and admin authorization, performs reward accounting, and calls the original SPL Token Program through CPIs.

### 4.2 Demo Faucet Program

A separate Anchor program that mints a fixed amount of Devnet `STAKE` to each wallet once. It cannot access pool state or either staking vault.

### 4.3 Original SPL Token Program

The original SPL Token Program owns all mint and token-account data. The project does not accept Token-2022 mints.

### 4.4 Frontend

A Next.js 16, React 19, and TypeScript application using `@solana/kit`, Wallet Standard, and a generated client derived from the Anchor IDL.

### 4.5 Off-Chain Scripts

Explicit scripts create the Devnet mints, establish mint authorities, initialize the official pool, mint the fixed REWARD supply, fund the pool, and record deployment addresses. Deployment does not imply automatic initialization.

## 5. Token Model

### 5.1 Mints

| Mint | Decimals | Mint authority | Freeze authority |
|---|---:|---|---|
| STAKE | 6 | Faucet Authority PDA | None |
| REWARD | 6 | Removed after initial mint | None |

The setup script mints `1,000,000 REWARD` to the deployment wallet's REWARD Treasury ATA and then permanently removes the REWARD mint authority.

The Demo Faucet mints `1,000 STAKE` (`1_000_000_000` base units) per wallet. Its one-wallet limit is not Sybil-resistant and is explicitly Devnet-only.

### 5.2 Token Accounts

- User staking deposits originate from the user's canonical STAKE ATA.
- Normal and emergency withdrawals return principal only to the user's canonical STAKE ATA.
- Claims pay only the user's canonical REWARD ATA.
- The Stake Vault is the Pool Authority PDA's STAKE ATA.
- The Reward Vault is the Pool Authority PDA's REWARD ATA.
- `fund_rewards` accepts any signer-controlled original SPL token account whose mint is the configured REWARD mint. The frontend defaults to the funder's REWARD ATA.
- Token CPIs use checked token instructions and validate the mint and six-decimal configuration.

Direct token transfers into either vault bypass program bookkeeping. They are treated as uncredited surplus and create no stake position or reward budget. No surplus sweep is provided.

## 6. Addresses and Accounts

All seed byte strings below are ASCII. Integer seed encoding is little-endian.

| Address/account | Seeds or derivation | Data owner | Payer | Purpose |
|---|---|---|---|---|
| Pool State PDA | `["pool", initializer, pool_id]` | Staking Program | Initializer | Shared pool state |
| Pool Authority PDA | `["pool-authority", pool]` | No data account required | N/A | Authority of both vault ATAs |
| Position PDA | `["position", pool, user]` | Staking Program | User | One user's stake and rewards |
| Proposal PDA | `["proposal", pool, proposal_id]` | Staking Program | Proposal creator | One exact admin command |
| Faucet Authority PDA | `["faucet-authority", stake_mint]` | No data account required | N/A | STAKE mint authority |
| Faucet Claim PDA | `["faucet-claim", stake_mint, user]` | Faucet Program | User | Permanent one-claim receipt |
| Stake Vault | ATA of `(Pool Authority PDA, STAKE mint)` | SPL Token Program | Pool initializer | Custodies principal |
| Reward Vault | ATA of `(Pool Authority PDA, REWARD mint)` | SPL Token Program | Pool initializer | Custodies reward funding |

The program permits multiple independently derived pools, but the deployment record, frontend, article, and audit target one configured official pool.

### 6.1 Pool State

The fixed-size Pool State contains at least:

```text
version: u8
initializer: Pubkey
pool_id: u64
pool_bump: u8
pool_authority_bump: u8
stake_mint: Pubkey
reward_mint: Pubkey
stake_vault: Pubkey
reward_vault: Pubkey
admins: [Pubkey; 3]
admin_epoch: u64
next_proposal_id: u64
paused: bool
max_reward_rate_per_slot: u64
reward_rate_per_slot: u64
last_update_slot: u64
total_staked: u64
acc_reward_per_stake_scaled: u128
remaining_reward_budget_scaled: u128
allocated_liability_scaled: u128
```

The mint and vault relationships, initializer, pool ID, and maximum reward rate are immutable. The pool starts paused with a zero reward rate and empty reward budget.

The Devnet setup uses a maximum reward rate of `100 REWARD` per slot (`100_000_000` base units per slot).

### 6.2 Position State

```text
version: u8
pool: Pubkey
owner: Pubkey
bump: u8
staked_amount: u64
reward_debt_scaled: u128
pending_reward_scaled: u128
```

There is one Position PDA for each `(pool, user)` pair.

### 6.3 Proposal State

```text
version: u8
pool: Pubkey
proposal_id: u64
creator: Pubkey
admin_epoch: u64
action: ProposalAction
approvals: [bool; 3]
approval_count: u8
created_slot: u64
expires_at_slot: u64
executed: bool
bump: u8
```

`ProposalAction` has exactly these variants:

```text
SetRewardRate { new_rate: u64 }
UnpausePool
ReplaceAdmin { old_admin: Pubkey, new_admin: Pubkey }
```

One proposal contains one action. Batching, arbitrary account lists, and arbitrary external program calls are prohibited.

### 6.4 Faucet Claim State

```text
version: u8
stake_mint: Pubkey
claimant: Pubkey
amount: u64
claimed_slot: u64
bump: u8
```

The account is permanent and cannot be closed by the claimant.

### 6.5 Account Layout Policy

- Custom state accounts include `version: u8`.
- Account layouts are fixed-size and calculated with Anchor's account sizing support.
- Runtime reallocation is outside scope.
- Public keys, seed relationships, discriminators, data ownership, mint, authority, and Token Program ID are validated for every instruction.

## 7. Authorization and Trust

### 7.1 User Authorization

User actions use ordinary Solana transaction signer authorization. The Solana runtime verifies transaction signatures and supplies trusted signer flags; the program then verifies that the signer matches the Position owner and required token authority.

The project does not implement detached signed commands, custom Ed25519 verification, ECDSA verification, or application nonces for user actions. Exact transaction replay is handled by the Solana transaction layer; repeated business actions are constrained by program state.

### 7.2 Vault Authorization

Both vaults store the Pool Authority PDA as token authority. No wallet has a private key for this PDA. Outgoing vault transfers require the Staking Program to call the Token Program with `invoke_signed` after all instruction rules pass.

Knowledge of PDA seeds and bumps does not grant authority. The runtime derives signer privilege using the currently executing Staking Program ID.

### 7.3 Pool Administration

The pool stores three distinct, non-default admin public keys. The normal threshold is two approvals.

- Any one current admin may pause immediately.
- Setting the reward rate, unpausing, and replacing an admin require Proposal PDA approval.
- Funding is permissionless and is not an admin action.
- Admins cannot withdraw either vault.

### 7.4 Program Upgrade Authority

The deployment wallet remains upgrade authority on Devnet to permit learning and bug fixes. This authority can replace program code and is more powerful than pool administration. Its public key must be disclosed in deployment records and the self-audit.

## 8. Reward Model

### 8.1 Constants and Units

```text
TOKEN_DECIMALS = 6
REWARD_PRECISION = 1_000_000_000
PROPOSAL_TTL_SLOTS = 216_000
ADMIN_COUNT = 3
ADMIN_THRESHOLD = 2
```

Token balances, rates, and slots use `u64`. Scaled rewards and intermediate arithmetic use `u128`. All arithmetic and conversions are checked; an error leaves all accounts unchanged.

### 8.2 Trusted Time

Reward time is `Clock.slot`. Frontend time, local system time, and user-supplied slots are never accepted. Slot-based rewards make no exact wall-clock or APR guarantee.

### 8.3 Pool Checkpoint

`checkpoint_pool` is an internal function, never a public instruction. When the pool is active:

```text
elapsed_slots = current_slot - last_update_slot
wanted_scaled = elapsed_slots * reward_rate_per_slot * REWARD_PRECISION
candidate_scaled = min(wanted_scaled, remaining_reward_budget_scaled)
```

If `total_staked == 0`, no reward budget is consumed. Otherwise:

```text
delta_acc = candidate_scaled / total_staked
distributed_scaled = delta_acc * total_staked

acc_reward_per_stake_scaled += delta_acc
remaining_reward_budget_scaled -= distributed_scaled
allocated_liability_scaled += distributed_scaled
```

The division remainder remains in `remaining_reward_budget_scaled` and participates in later updates. `last_update_slot` advances to the trusted current slot after a successful active checkpoint, including when the pool is empty or the budget is exhausted.

When paused, no rewards accrue and checkpointing does not consume budget.

### 8.4 Position Settlement

Before changing a user's stake or rewards:

```text
accrued_scaled = staked_amount * acc_reward_per_stake_scaled
newly_earned_scaled = accrued_scaled - reward_debt_scaled
pending_reward_scaled += newly_earned_scaled
reward_debt_scaled = staked_amount * acc_reward_per_stake_scaled
```

After changing `staked_amount`, `reward_debt_scaled` is reset using the new amount and current accumulator.

### 8.5 Claims

```text
claimable_base_units = pending_reward_scaled / REWARD_PRECISION
paid_scaled = claimable_base_units * REWARD_PRECISION
```

A claim transfers all whole claimable base units, subtracts `paid_scaled` from the position and `allocated_liability_scaled`, and preserves the fractional remainder. A zero claim returns `NothingToClaim`.

Users claim independently. Claim order must not change entitlement.

### 8.6 Funding and Exhaustion

`fund_rewards` checkpoints before transferring funds so new funding cannot pay retroactively for previously unfunded slots. After a successful checked token transfer:

```text
remaining_reward_budget_scaled += funded_base_units * REWARD_PRECISION
```

Emission stops when the remaining budget reaches zero. If the remaining budget cannot fund the full nominal emission, the final allocation is partial.

### 8.7 Solvency Invariants

```text
stake_vault.amount >= total_staked

remaining_reward_budget_scaled
  + allocated_liability_scaled
  <= reward_vault.amount * REWARD_PRECISION
```

The inequalities permit unsolicited token surplus. Program-mediated flows should preserve equality except for such surplus.

## 9. Pause and Emergency Policy

Pausing performs an active checkpoint at the current slot and then sets `paused = true`.

While paused:

- `stake` is blocked.
- `claim_rewards` is blocked.
- Normal `unstake` is allowed.
- `emergency_withdraw` is allowed.
- `fund_rewards`, proposal operations, and administrative configuration remain available.
- No reward budget is consumed.

Normal unstaking settles accrued rewards into the Position and transfers only the requested principal. Pending rewards remain claimable after unpause.

Emergency withdrawal settles the Position, returns all principal, deletes the complete scaled reward entitlement, subtracts that entitlement from allocated liability, and adds it back to the remaining reward budget. The operation is irreversible.

`emergency_withdraw` is valid when either principal or a pending scaled reward entitlement remains. This permits a fully unstaked user to explicitly forfeit an otherwise unclaimable fractional remainder before calling `close_position`. The frontend must present this as a separate irreversible confirmation; it must never discard the fraction automatically.

Unpause requires an approved proposal and sets `last_update_slot` to the current slot before setting `paused = false`. Paused slots never receive retroactive rewards. A rate configured during pause becomes active from unpause.

## 10. Proposal Governance

### 10.1 Creation

- The creator must be a current admin signer.
- The supplied proposal ID must equal `pool.next_proposal_id`.
- Creation increments `next_proposal_id` atomically.
- The action and parameters become immutable.
- The proposal copies `pool.admin_epoch` internally; the caller cannot choose it.
- The creator counts as the first approval.
- Expiry is `created_slot + PROPOSAL_TTL_SLOTS` using checked arithmetic.

### 10.2 Approval

- The approver must be a distinct current admin signer.
- The proposal must be unexecuted, unexpired, and from the current `admin_epoch`.
- An admin may approve only once.
- Approvals cannot be revoked.

### 10.3 Execution

Execution is permissionless and unrewarded. Before applying the action, the program checks:

```text
proposal.pool == pool.key()
proposal.approval_count >= 2
proposal.executed == false
current_slot <= proposal.expires_at_slot
proposal.admin_epoch == pool.admin_epoch
```

`SetRewardRate` checkpoints using the old rate and requires `new_rate <= max_reward_rate_per_slot`. `UnpausePool` applies the unpause policy. `ReplaceAdmin` requires that the old admin exists and the new admin is non-default, distinct, and not already an admin.

Successful admin replacement increments `pool.admin_epoch`, invalidating every other proposal from the previous admin set. Execution and `proposal.executed = true` are atomic.

### 10.4 Cleanup

Anyone may close an executed, expired, or stale proposal. The rent deposit always returns to the recorded creator. Proposal IDs are never reused.

There is no cancellation instruction. Unapproved proposals are harmless; approved proposals remain executable until they execute, expire, or become stale.

## 11. Instruction Catalogue

### 11.1 Staking Program

| Instruction | Authorization | Summary |
|---|---|---|
| `initialize_pool` | Initializer signer | Creates paused Pool State and both vault ATAs; validates mints and three admins |
| `open_position` | User signer | Creates the user's empty Position PDA |
| `stake(amount)` | Position owner signer | Checkpoints, settles, transfers STAKE into vault, and increases principal |
| `unstake(amount)` | Position owner signer | Checkpoints, settles, transfers requested principal, and preserves rewards |
| `claim_rewards` | Position owner signer | Settles and transfers all whole claimable REWARD |
| `emergency_withdraw` | Position owner signer | Returns all principal and forfeits all scaled rewards |
| `close_position` | Position owner signer | Closes only an empty, reward-free Position and returns rent to owner |
| `fund_rewards(amount)` | Funding source authority signer | Checkpoints, transfers REWARD, and increases remaining budget |
| `pause_pool` | Any one current admin signer | Checkpoints and freezes the pool |
| `create_proposal(action)` | Current admin signer | Creates Proposal PDA and records first approval |
| `approve_proposal` | Distinct current admin signer | Adds one approval |
| `execute_proposal` | Permissionless transaction payer | Executes one approved internal admin action |
| `close_proposal` | Permissionless transaction payer | Closes executed, expired, or stale proposal to its creator |

The frontend bundles `open_position` and the first `stake` into one atomic transaction. It creates missing canonical user ATAs when appropriate.

### 11.2 Demo Faucet Program

| Instruction | Authorization | Summary |
|---|---|---|
| `claim_test_stake` | Claimant signer | Creates claimant ATA and permanent Claim PDA, then mints exactly `1,000 STAKE` |

The Faucet Program has no mutable configuration account. It requires a six-decimal mint whose authority is the expected Faucet Authority PDA.

## 12. Validation and Errors

Every instruction must validate all relevant seed, bump, signer, owner, mint, vault, token authority, Token Program, pool, position, and proposal relationships before mutation.

The error catalogue must include at least:

```text
ArithmeticOverflow
InvalidAmount
InvalidTokenDecimals
InvalidAdminSet
Unauthorized
PoolPaused
PoolNotPaused
InsufficientStake
NothingToClaim
InsufficientRewardBacking
RewardRateAboveMaximum
ProposalExpired
ProposalAlreadyExecuted
ProposalNotApproved
DuplicateApproval
StaleProposal
InvalidProposalId
InvalidAdminReplacement
PositionNotEmpty
FaucetAlreadyClaimed
InvalidMintAuthority
```

Failed instructions emit no success event and leave pool, position, proposal, mint, and token-account state unchanged.

## 13. Events

Every successful state-changing instruction emits an Anchor event. All events include the pool or faucet identity, actor, and trusted slot where applicable.

Required event families:

```text
PoolInitialized
PositionOpened
Staked
Unstaked
RewardsClaimed
EmergencyWithdrawn
PositionClosed
RewardsFunded
PoolPaused
PoolUnpaused
ProposalCreated
ProposalApproved
ProposalExecuted
ProposalClosed
AdminReplaced
RewardRateChanged
FaucetClaimed
```

Amount events include action amounts and resulting important totals. Proposal events include proposal ID and admin epoch. Events support receipts and research analysis; account data remains authoritative.

## 14. Frontend Requirements

### 14.1 Stack

- Next.js 16, React 19, and TypeScript
- `@solana/kit`
- Wallet Standard browser-wallet discovery
- Generated client from Anchor IDL
- Tailwind CSS
- Vitest and Testing Library
- Playwright for local browser workflows

The public build is hard-targeted to Devnet. Localnet is selected only through development environment configuration. The RPC endpoint is configurable and defaults to the public Devnet endpoint.

### 14.2 User Interface

The operational dashboard displays:

- Devnet status, connected wallet, SOL balance, and Explorer links
- Pool active/paused state
- Total staked principal
- Reward rate and immutable maximum
- Remaining reward budget and allocated liability
- Mint, Pool State, authority, and vault addresses
- Wallet STAKE and REWARD balances
- User principal and estimated pending rewards

Pending rewards are an off-chain estimate using fetched account data and the current slot. The UI labels the value as estimated; on-chain claim execution is authoritative.

Required actions:

- Connect and disconnect wallet
- Claim demo STAKE
- Open position and stake
- Partially or fully unstake
- Claim all rewards
- Emergency withdraw with explicit forfeiture confirmation
- Explicitly forfeit a final fractional remainder before Position closure when necessary
- Close an empty position
- Fund rewards
- Pause as one admin
- Create, approve, execute, and close admin proposals

Transaction status proceeds through preparing, wallet signature, submitted, and confirmed/failed states. Confirmed transactions expose Explorer links and trigger account refreshes. Amount handling uses integer base units and `bigint`, never JavaScript floating-point arithmetic for transaction values.

No backend or application database is required. On-chain accounts are the source of truth.

## 15. Testing Strategy

### 15.1 Pure Rust Unit Tests

Test the reward math without a VM:

- Staggered Alice/Bob deposits
- Partial final funding
- Empty-pool slots
- Zero reward rate
- Paused slots
- Accumulator division remainder
- User fractional carry
- Emergency forfeiture recycling
- Overflow and underflow rejection
- Reserve/liability conservation

### 15.2 LiteSVM Integration Tests

Execute compiled programs and real SPL instructions:

- All successful instruction paths
- PDA and canonical ATA derivation
- Runtime signer checks
- Pool Authority `invoke_signed`
- Wrong pool, position, mint, vault, authority, and Token Program substitution
- Cross-pool isolation
- Atomic rollback after CPI failure
- Pause behavior
- Proposal creation, duplicate approval, expiry, replay, and stale epoch
- Admin replacement constraints
- Faucet one-claim enforcement
- Direct vault surplus behavior

### 15.3 Local End-to-End Tests

- Build and deploy both programs locally
- Run mint and pool setup scripts
- Generate and consume the program client
- Run complete faucet-to-stake-to-claim workflows through RPC
- Complete separate-wallet proposal approval and execution
- Verify frontend amount conversion and account refresh
- Exercise local Playwright workflows without storing real wallet seed phrases

### 15.4 Devnet Smoke Tests

- Confirm deployed program IDs and upgrade authorities
- Confirm mint authorities and removed REWARD authority
- Claim faucet STAKE from a test wallet
- Stake, claim, normal unstake, and emergency withdraw small values
- Execute one full admin proposal lifecycle
- Validate public account and transaction Explorer links
- Record transaction signatures in deployment evidence

## 16. Security Self-Audit

`AUDIT.md` is maintained continuously. Each milestone defines invariants and abuse cases, adds normal and adversarial tests, runs the full regression suite, and records evidence.

Each finding includes:

```text
risk ID
severity
affected instruction/accounts
attack scenario
numeric example where useful
implemented defense
test evidence
status
residual risk
```

Mandatory review areas include authorization, account substitution, PDA relationships, token authorities, mint authorities, proposal freshness, arithmetic, rounding, reward backing, pause behavior, emergency withdrawal, direct donations, upgrade authority, frontend transaction construction, dependencies, RPC assumptions, and Devnet faucet separation.

The public wording must state:

> This project received a documented internal security review and adversarial test suite. It has not received an independent professional audit and is not production-ready.

## 17. Known and Accepted Risks

- The Devnet upgrade authority can replace code and access existing PDA authority paths.
- A single compromised admin can grief users by pausing, although principal remains withdrawable.
- Claims remain unavailable while paused.
- Two compromised admins can change rates, unpause, or replace the third admin.
- Admins can time rate changes around their own staking activity.
- Direct vault transfers become uncredited and potentially locked surplus.
- Abandoned fractional user rewards can remain reserved indefinitely.
- Faucet limits are per wallet and are trivially Sybilable.
- Public Devnet RPC is rate-limited and Devnet state may reset.
- Slot duration is variable, so slot emissions do not guarantee wall-clock yield.
- There is no safe pool retirement or remaining-budget recovery path.
- Self-testing cannot prove the absence of vulnerabilities.

## 18. Repository Layout

```text
programs/
  staking_pool/
    src/
      instructions/
      state/
      constants.rs
      error.rs
      events.rs
      math.rs
      lib.rs
  demo_faucet/
    src/
      instructions/
      state/
      constants.rs
      error.rs
      events.rs
      lib.rs
app/
  src/
  tests/
tests/
  litesvm/
  local-e2e/
scripts/
  setup-devnet/
  smoke-devnet/
docs/
  diagrams/
  RESEARCH.md
deployments/
  devnet.json
SPEC.md
README.md
AUDIT.md
Anchor.toml
Cargo.toml
```

Beginner explanations and interview notes belong in a gitignored private notes path such as `.private/LEARNING_NOTES.md`. They must not be copied into the public-ready specification, README, research article, or audit.

## 19. Build, CI, and Deployment Evidence

GitHub Actions must run formatting, Rust checks, pure unit tests, LiteSVM tests, frontend type checking, frontend tests, and a production frontend build. Secrets and wallet keypairs must never enter the repository or CI logs.

Devnet deployment evidence records:

```text
staking program ID
faucet program ID
program upgrade authority
STAKE mint
REWARD mint
Faucet Authority PDA
official Pool State PDA
Pool Authority PDA
Stake Vault ATA
Reward Vault ATA
Treasury ATA
deployment slots
verification status
smoke-test transaction signatures
source commit
```

The frontend must be publicly hostable. The hosting vendor is selected during implementation and is not part of the protocol contract.

## 20. Implementation Sequence

1. Prepare Windows/WSL storage and pin compatible Rust, Solana, Anchor, and Node toolchains.
2. Scaffold the Anchor workspace and pure reward math.
3. Implement and test Pool State, authority, vault creation, and funding.
4. Implement positions, staking, checkpointing, claims, and withdrawals.
5. Implement pause and emergency behavior.
6. Implement proposal governance and admin replacement.
7. Implement the separate faucet and mint setup.
8. Build local setup scripts and complete local end-to-end tests.
9. Build the wallet-connected frontend.
10. Deploy and verify on Devnet, then run smoke tests.
11. Consolidate `AUDIT.md`, diagrams, and the research article.

## 21. Acceptance Criteria

The project is complete when:

- Both Anchor programs build reproducibly.
- Every instruction and error path has proportionate automated coverage.
- Reward calculations satisfy the documented numeric scenarios and invariants.
- Malicious signer, PDA, mint, vault, and proposal substitutions fail atomically.
- The local end-to-end workflow passes from token claim through position closure.
- The programs and assets are deployed and recorded on Devnet.
- A browser wallet can complete the public user workflow.
- Admin wallets can complete pause and proposal workflows.
- CI is green from a clean checkout.
- `README.md`, diagrams, deployment evidence, `AUDIT.md`, and the research article match deployed behavior.
- Public documentation clearly states that the project is educational, self-audited, and not production-ready.
