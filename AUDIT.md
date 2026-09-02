# Security Self-Audit

Status: Milestone 4 complete  
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
- Pinned Rust toolchain for project compatibility: `1.86.0`
- WSL `crypto` verification: `rustc 1.86.0`, `cargo 1.86.0`,
  `anchor-cli 0.31.1`, `solana-cli 2.1.21`
- Anchor CLI target in `Anchor.toml`: `0.31.1`
- Solana CLI target in `Anchor.toml`: `2.1.21`
- Node/npm available on PATH: Node `22.18.0`, npm `10.9.3`
- Local limitation: rustup shims cannot create temp files under
  `C:\Users\HONG PHAT\.rustup\tmp` in this sandbox, so Rust verification was
  run through the installed stable toolchain binaries with `RUSTC` and
  `RUSTDOC` set directly.
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
