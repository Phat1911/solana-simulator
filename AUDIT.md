# Security Self-Audit

Status: Milestone 8 complete
Scope: Educational Solana Devnet staking research project  
Production status: Not production-ready; no independent professional audit

> This project received a documented internal security review and adversarial
> test suite. It has not received an independent professional audit and is not
> production-ready.

## Evidence Format

Each milestone records:

- Scope implemented
- Security and economic rules touched
- Tests run
- Result
- Residual risk or limitation

Future findings use:

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

## Baseline Tool Observations

- Original Windows PATH observation: `rustc 1.93.0`, `cargo 1.93.0`
- Rust toolchain policy updated after Milestone 6: `stable` via
  `rust-toolchain.toml`, so current dependency tests can use modern crates.
- Latest observed WSL `crypto` stable toolchain: `rustc 1.98.0`, `cargo 1.98.0`.
- WSL `crypto` Anchor/Solana verification: `anchor-cli 0.31.1`,
  `solana-cli 2.1.21`
- Anchor SBF build compatibility: selected transitive dependencies are pinned
  in `Cargo.lock` to versions accepted by Solana's SBF Rust 1.79 toolchain
  while the host project continues to use Rust `stable`.
- Anchor CLI target in `Anchor.toml`: `0.31.1`
- Solana CLI target in `Anchor.toml`: `2.1.21`
- Node/npm available on PATH: Node `22.18.0`, npm `10.9.3`
- Historical local limitation: rustup shims could not create temp files under
  `C:\Users\HONG PHAT\.rustup\tmp` in the original Windows sandbox, so early
  Rust verification used direct stable toolchain binaries.
- Recommended local environment: WSL `crypto`, with the project copied under
  `/home/hong_phat/projects/rust-smc`.

## Milestone Evidence

### Milestone 1 - Minimal Workspace And Toolchain

- Scope implemented: Created the Cargo workspace, two program crates, Anchor
  configuration, frontend placeholder, shared test directories, deployment,
  script, diagram, and research placeholders.
- Security and economic rules touched: No protocol behavior was implemented.
  No token movement, account validation, admin action, or reward accounting
  exists yet.
- Tests run: `cargo fmt --all -- --check`; `cargo clippy --workspace
  --all-targets -- -D warnings`; `cargo test --workspace`.
- Result: Passed. Formatting used the installed stable `cargo-fmt` binary.
  Workspace tests passed after pinning `RUSTC` and `RUSTDOC` to the installed
  stable toolchain binaries to avoid the rustup shim permission issue.
- Residual risk or limitation: `anchor` and `solana` CLIs are not installed on
  the original Windows PATH. The pinned versions are now available in WSL
  `crypto`, which is the recommended environment for future Anchor checks.

### Milestone 2 - Test And Audit Baseline

- Scope implemented: Added harmless Rust unit tests in both program crates,
  a baseline integration-test crate under `tests/litesvm`, this audit file,
  and private-note ignore rules.
- Security and economic rules touched: No protocol behavior was implemented.
  The baseline only proves the workspace test targets can execute.
- Tests run: `cargo test --workspace`; `npm run test`; `git -c
  safe.directory=D:/rust-smc check-ignore .private/LEARNING_NOTES.md
  private-notes/example.md LEARNING_NOTES.md`.
- Result: Passed. Program unit tests, the baseline LiteSVM-placeholder harness,
  frontend placeholder check, and private-note ignore checks all passed.
- Residual risk or limitation: The LiteSVM crate itself is not introduced yet
  because on-chain instruction behavior does not exist.

### Milestone 3 - Units And Checked Arithmetic

- Scope implemented: Added milestone-marked staking constants, a small pure
  math error enum, and checked helpers for positive amounts, `u64` token/slot
  arithmetic, `u128` scaled arithmetic, base-unit scaling, emission scaling,
  claimable whole-unit conversion, paid-scaled calculation, and `u128` to `u64`
  narrowing.
- Security and economic rules touched: Token amounts remain six-decimal integer
  base units, reward liabilities use `REWARD_PRECISION = 1_000_000_000`, and
  every helper returns an explicit error for invalid zero amounts, overflow, or
  underflow. No floating point, decimal crate, account access, CPI, or authority
  logic was introduced.
- Tests run: `cargo fmt --all -- --check`; `cargo clippy --workspace
  --all-targets -- -D warnings`; `cargo test --workspace`.
- Result: Passed. Unit tests cover zero amount rejection, one-base-unit scaling,
  fractional scaled carry, maximum safe scaling, overflow, underflow, and
  narrowing rejection.
- Residual risk or limitation: These helpers are pure primitives only. Later
  milestones must still wire them into Anchor account handlers and map these
  pure errors to the final on-chain error catalogue.

### Milestone 4 - Global Reward Checkpoint Math

- Scope implemented: Added milestone-marked pure checkpoint input/output
  structs and `checkpoint_pool_rewards` for elapsed-slot accounting, desired
  emission, funded-budget caps, partial final emission, accumulator growth,
  budget reduction, allocated-liability growth, zero-stake checkpoints, paused
  checkpoints, exhausted-budget checkpoints, and scaled rounding remainders.
- Security and economic rules touched: Checkpoints emit at most the unallocated
  funded budget, emit nothing while paused, with zero stake, or after
  exhaustion, and advance `last_update_slot` to the trusted current slot so
  skipped gaps cannot accrue retroactively. Rounding leftovers stay in the
  remaining budget instead of becoming allocated liability.
- Tests run: `cargo fmt --all -- --check`; `cargo test -p staking_pool --test
  checkpoint_math`; `cargo clippy --workspace --all-targets -- -D warnings`;
  `cargo test --workspace`.
- Result: Passed. Focused tests cover normal accrual, zero stake, pause gaps,
  exact exhaustion, partial final emission, rounding remainder preservation, and
  backward-slot underflow.
- Residual risk or limitation: This remains pure math only. Later Anchor
  handlers must read `Clock.slot`, persist the returned pool fields atomically,
  enforce account relationships, and preserve reward solvency against real SPL
  Token vault balances.

### Milestone 5 - Position Settlement And Claim Math

- Scope implemented: Added milestone-marked pure position settlement,
  reward-debt reset, whole-unit claim, and emergency-forfeiture accounting
  helpers. The functions operate only on integer base units and scaled `u128`
  reward values.
- Security and economic rules touched: Repeated settlement at the same
  accumulator creates no new reward, staggered entrants receive no backpay,
  claims pay only whole base units while preserving scaled fractional
  remainders, and emergency forfeiture moves exactly the position's pending
  scaled liability from allocated liability back to the remaining reward
  budget.
- Tests run: `cargo fmt --all -- --check`; `cargo test -p staking_pool --test
  position_math`; `cargo clippy --workspace --all-targets -- -D warnings`;
  `cargo test --workspace`.
- Result: Passed. Focused tests cover multiple users, staggered entry,
  repeated settlement, partial-token claims, post-claim accrual, forfeiture
  conservation, fractional-only claim rejection, and arithmetic
  overflow/underflow boundaries.
- Residual risk or limitation: This remains pure math only. Later Anchor
  handlers must call these helpers after checkpointing, perform SPL Token
  transfers atomically, reject paused claims, validate canonical user accounts,
  and check real reward-vault backing before payout.

### Milestone 6 - Staking Account Schemas And PDA Recipes

- Scope implemented: Added milestone-marked Anchor-serializable fixed-size
  `Pool`, `Position`, and `Proposal` state layouts, the allowlisted
  `ProposalAction` enum, account space constants including Anchor's
  discriminator, and canonical PDA derivation helpers for pool, pool authority,
  position, and proposal addresses.
- Security and economic rules touched: State now stores pool identity,
  immutable mint and vault relationships, admin epoch and proposal data,
  position owner and pool binding, reward accumulator, funded budget, and
  allocated liability fields needed by later instruction handlers.
- Tests run: `cargo test -p staking_pool --test state_pda`.
- Result: Passed. Focused tests cover exact seed prefixes, little-endian integer
  seed encoding, deterministic PDA derivation, separation across initializers,
  pool IDs, program IDs, pools, users, and proposal IDs, and fixed account
  sizes for Pool, Position, Proposal, and ProposalAction.
- Residual risk or limitation: This milestone defines schemas and derivation
  recipes only. Later Anchor handlers must still enforce account owners,
  discriminators, stored-key relationships, signer authorization, token program
  identity, vault authority, mint configuration, and atomic state/token updates.

### Milestone 7 - Pool Initialization And Vault Authority

- Scope implemented: Added the `initialize_pool` Anchor instruction, `Pool`
  Anchor account discriminators, associated-token vault creation, original SPL
  Token Program wiring, and a `PoolInitialized` event emitted only after
  successful initialization.
- Security and economic rules touched: Initialization requires three distinct
  non-default admins, distinct six-decimal stake and reward mints, a configured
  maximum reward rate no higher than the Devnet cap, canonical Pool PDA seeds,
  and the canonical Pool Authority PDA as owner of both vault ATAs. New pools
  start paused with `reward_rate_per_slot = 0`, `total_staked = 0`, and empty
  reward budget/liability fields.
- Tests run: `cargo fmt --all -- --check`; `anchor build -p staking_pool`;
  `cargo test -p staking_pool --test state_pda`; `cargo test -p
  litesvm_baseline milestone7_initialize_pool -- --nocapture`; `cargo test
  --workspace`; `cargo clippy --workspace --all-targets -- -D warnings`; `git
  diff --check`.
- Result: Passed after pinning transitive crates such as `indexmap`,
  `unicode-segmentation`, `blake3`, `zeroize`, `zeroize_derive`, and
  `proc-macro-crate` to versions compatible with Anchor's Solana SBF build
  toolchain. LiteSVM tests cover valid initialization, duplicate admins, wrong
  mint decimals, same stake/reward mint, excessive maximum reward rate,
  non-canonical Pool Authority PDA, and duplicate initialization.
- Residual risk or limitation: This milestone creates and validates the pool
  and vault accounts only. Later handlers must still implement token funding,
  staking, withdrawal, claim, pause/governance, real vault solvency checks, and
  adversarial account substitution coverage for every token-moving instruction.

### Milestone 8 - Position Open And Close Lifecycle

- Scope implemented: Added `open_position` and `close_position` Anchor
  instructions, canonical Position PDA initialization, signer-owned safe
  closure, and `PositionOpened`/`PositionClosed` events.
- Security and economic rules touched: Each user can create only the canonical
  Position PDA for one `(pool, user)` pair. Closure requires the signer-bound
  canonical position and rejects any nonzero `staked_amount`,
  `reward_debt_scaled`, or `pending_reward_scaled`, so account closure cannot
  erase principal or reward accounting.
- Tests run: `anchor build -p staking_pool`; `cargo test -p litesvm_baseline
  milestone8_position_lifecycle -- --nocapture`.
- Result: Passed. LiteSVM tests cover canonical creation, duplicate creation,
  wrong position seed, wrong pool seed, user/pool separation, unauthorized
  close, non-empty stake, pending reward, reward debt, and successful empty
  close with the account drained and rent returned.
- Residual risk or limitation: Position lifecycle does not yet stake, unstake,
  settle, claim, or transfer SPL tokens. Later token-moving instructions must
  preserve principal solvency, reward solvency, and rollback guarantees while
  updating these position fields.
