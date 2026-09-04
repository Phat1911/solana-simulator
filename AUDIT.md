# Security Self-Audit

Status: Milestones 24-25 tooling and documentation prepared; Devnet evidence pending user handoff
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

### Milestones 9-12 - Reward Funding, Stake, Normal Unstake, And Claim

- Scope implemented: Added `fund_rewards`, `stake`, `unstake`, and
  `claim_rewards` Anchor instructions with checked SPL Token Program CPIs,
  canonical user ATA validation, Pool Authority PDA signer seeds for vault
  outflows, pool checkpointing, position settlement, and success events.
- Security and economic rules touched: Funding is permissionless but requires
  the source token-account authority signer and credits budget only after a
  successful REWARD transfer. Staking is blocked while paused, settles the
  position before increasing principal, and preserves
  `stake_vault.amount >= pool.total_staked`. Unstaking is allowed while paused,
  returns principal only to the signer-owned canonical STAKE ATA, and preserves
  pending rewards. Claiming is blocked while paused, pays only whole REWARD base
  units to the canonical REWARD ATA, decrements allocated liability by the exact
  paid scaled amount, and preserves fractional scaled remainder.
- Tests run: `cargo test -p litesvm_baseline milestone9_12_token_flows --
  --nocapture`; `cargo fmt --all -- --check`; `anchor build -p staking_pool`;
  `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D
  warnings`; `git diff --check`.
- Result: Passed. Focused LiteSVM tests cover valid reward funding, direct
  reward-vault donation as surplus only, zero and wrong-authority funding
  failures with rollback, first stake, paused/zero/insufficient-balance stake
  rejection, wrong stake ATA rejection, paused unstake preserving principal and
  rewards, excessive unstake rejection, bad Pool Authority PDA rejection, claim
  payout with double-payment prevention, paused claim rejection, wrong reward
  account rejection, simulated insufficient backing rejection, and direct
  principal/reward solvency assertions. The full workspace test suite passed
  with 28 LiteSVM tests, staking math/state tests, and doc-tests.
- Residual risk or limitation: Pause/unpause instructions, emergency
  withdrawal, governance proposals, faucet behavior, cross-instruction
  state-machine tests, local-validator workflows, frontend flows, and Devnet
  smoke evidence remain future milestones. Some LiteSVM checks use direct
  account mutation to simulate future governance states or corrupted backing;
  later milestones should add end-to-end paths that create those states only
  through public instructions.

### Milestones 13-14 - Pause Semantics And Emergency Withdrawal

- Scope implemented: Added `pause_pool` and `emergency_withdraw` Anchor
  instructions, `PoolPaused` and `EmergencyWithdrawn` events, admin membership
  validation, and Pool Authority PDA-signed principal return for emergency
  withdrawal.
- Security and economic rules touched: Any current admin can pause only an
  active pool, and pause checkpoints before flipping `paused = true` so rewards
  stop exactly at the trusted pause slot. While paused, existing stake and claim
  guards still block stake and claim while normal unstake and emergency
  withdrawal remain available. Emergency withdrawal checkpoints, settles the
  position, returns all principal to the canonical STAKE ATA, zeroes stake,
  reward debt, and pending reward, subtracts the forfeited scaled reward from
  allocated liability, and adds it back to remaining reward budget.
- Tests run: `cargo test -p litesvm_baseline milestone9_12_token_flows --
  --nocapture`; `cargo test -p staking_pool --test position_math`.
- Result: Passed. Focused LiteSVM coverage now includes admin pause
  checkpointing, paused-slot non-accrual, unauthorized pause rejection,
  redundant pause rejection, active emergency withdrawal with full principal
  return and reward recycling, and paused fraction-only forfeiture for position
  cleanup. Pure forfeiture math tests continue to pass.
- Residual risk or limitation: Proposal-based unpause is intentionally completed
  with proposal execution in Milestone 16; there is still no public direct
  unpause instruction. The new LiteSVM tests share the existing token-flow
  fixture and still use direct pool/position mutation for a few future-state
  simulations until governance and broader state-machine tests exist.

### Milestones 15-16 - Proposal Governance And Execution

- Scope implemented: Added `create_proposal`, `approve_proposal`,
  `execute_proposal`, and `close_proposal` Anchor instructions for the
  allowlisted actions `SetRewardRate`, `UnpausePool`, and `ReplaceAdmin`.
  Added proposal lifecycle events plus reward-rate, unpause, and admin-rotation
  events.
- Security and economic rules touched: Proposal PDAs are monotonic per pool and
  bound to the current `next_proposal_id`; action data is stored immutably in
  the proposal account. The creator must be a current admin and receives the
  first approval. A second distinct current admin is required before execution.
  Execution rejects expired, stale, already-executed, or under-approved
  proposals. Reward-rate execution checkpoints at the old rate first. Unpause
  sets `last_update_slot` to the trusted current slot before resuming rewards.
  Admin replacement validates the old admin and distinct non-default new admin,
  then increments `admin_epoch` so old proposals become stale.
- Tests run: `cargo test -p litesvm_baseline milestone15_16_governance --
  --nocapture`.
- Result: Passed. Focused LiteSVM coverage includes proposal creation,
  immutable action storage, proposal ID sequencing, invalid ID/rate/admin
  replacement rejection, duplicate approval rejection, threshold enforcement,
  old-rate checkpointing, execution replay rejection, unpause without paused
  backpay, admin replacement with epoch increment, stale old proposal rejection,
  and proposal closure after execution or expiry with rent returned to creator.
- Residual risk or limitation: Governance is now implemented for the staking
  program, but broader adversarial state-machine scenarios across funding,
  staking, pause, emergency withdrawal, and governance remain Milestone 18.

### Milestone 17 - Demo Faucet Program

- Scope implemented: Added the `claim_test_stake` instruction to the separate
  `demo_faucet` Anchor program, fixed faucet constants, Faucet Authority PDA
  derivation, permanent Faucet Claim receipt PDA derivation, canonical claimant
  STAKE ATA creation, SPL Token `mint_to` CPI, `FaucetClaimed` event, and a
  project-specific demo faucet program ID.
- Security and economic rules touched: The faucet accepts only six-decimal
  original SPL Token mints whose mint authority is the Faucet Authority PDA and
  whose freeze authority is absent. Each wallet can create only one canonical
  receipt for a given STAKE mint, so replay fails before any second mint. The
  faucet does not read or mutate staking Pool, Position, Proposal, vault, admin,
  or reward accounting state.
- Tests run: `cargo test -p litesvm_baseline milestone17_demo_faucet --
  --nocapture`.
- Result: Passed. Focused LiteSVM coverage includes first claim with canonical
  ATA creation and receipt recording, replay rejection without extra minting,
  alternate token-account rejection, wrong receipt rejection, wrong faucet
  authority rejection, wrong mint-decimal rejection, and rollback checks for
  failed claims.
- Residual risk or limitation: The faucet is intentionally Devnet-only and is
  not Sybil-resistant; one human can still claim from many wallets. Devnet setup
  scripts that create the configured STAKE mint and assign/revoke authorities
  remain Milestone 19.

### Milestone 18 - Cross-Instruction Invariant Suite

- Scope implemented: Added a self-contained LiteSVM state-machine suite that
  uses only public staking-program instructions to combine reward funding,
  two-user staking, slot advancement, claims, normal unstaking, admin pause,
  emergency withdrawal, governance rate changes, and proposal-based unpause.
  Added adversarial substitutions and three repeatable fixed-seed generated
  operation sequences.
- Security and economic rules touched: Every successful transition checks exact
  principal accounting, aggregate position entitlement against allocated scaled
  liability, reward reserve-plus-liability against the real reward vault, and
  complete STAKE and REWARD token conservation. Expected failures snapshot all
  pool, position, and token balances and require atomic rollback. Malicious
  reward-vault, position-owner, Token Program, and pause-authority substitutions
  are rejected without changing protocol state.
- Tests run: `cargo test -p litesvm_baseline milestone18 -- --nocapture`.
- Result: Passed. Three focused tests cover the deterministic two-user economic
  timeline, adversarial account and signer replacement, and 48 generated
  operations replayed from seeds `0x18`, `0x5eed`, and `0xc0ffee`. The timeline
  verifies old-rate checkpointing, paused claim rollback, emergency reward
  recycling, whole-unit payout with fractional carry, and final zero liability.
- Residual risk or limitation: These tests use deterministic LiteSVM execution,
  not parallel validator scheduling or unbounded coverage-guided fuzzing. Local
  validator workflows, browser transactions, Devnet deployment, and external
  security review remain future work.

### Milestone 19 - Devnet Setup And Deployment Scripts

- Scope implemented: Added a milestone-marked Rust deployment helper under
  `scripts/deployment_tools`, a thin `scripts/devnet/setup.sh` wrapper, Devnet
  config and deployment metadata examples, and setup documentation. The helper
  validates config, derives the canonical Pool, Pool Authority, Faucet Authority,
  vault, and treasury addresses, creates six-decimal STAKE and REWARD mints,
  mints the fixed `1_000_000 REWARD` supply, revokes REWARD mint authority,
  initializes the staking pool, funds rewards through `fund_rewards`, and writes
  public metadata only.
- Security and economic rules touched: Setup refuses duplicate/default admins,
  excessive reward-rate caps, zero initial funding, stake/reward mint reuse,
  private output paths, and config text that looks like embedded key material.
  Mint keypairs are created or reused from ignored local paths, private RPC URLs
  are not committed, STAKE mint authority is set to the Faucet Authority PDA,
  and REWARD mint authority is revoked after the fixed initial supply.
- Tests run: `cargo test -p deployment_tools`; `cargo run -p deployment_tools
  -- validate --config /tmp/rust-smc-m19/config.json --output
  /tmp/rust-smc-m19/localnet.json`; `cargo run -p deployment_tools -- dry-run
  --config /tmp/rust-smc-m19/config.json --output
  /tmp/rust-smc-m19/localnet.json`; local validator setup using
  `cargo run -p deployment_tools -- setup --config
  /tmp/rust-smc-m19/config.json --output /tmp/rust-smc-m19/localnet.json`;
  repeated local setup to verify idempotent reuse.
- Result: Passed. Focused unit tests cover admin uniqueness/default rejection,
  secret-like config rejection, private output-path rejection, and public RPC
  labeling. Dry-run produced public deployment metadata without writing secrets.
  Fresh local setup created the mints/accounts and initialized/funded the pool;
  a second setup run completed without recreating mints or double-crediting the
  configured initial reward funding.
- Residual risk or limitation: The successful setup was verified against a
  temporary local validator. Real Devnet deployment, Explorer signatures,
  upgrade-authority disclosure, and smoke-test evidence remain Milestone 24.

### Milestone 20 - Generated Client And Frontend Foundation

- Scope implemented: Replaced the placeholder frontend package with a Next.js
  16, React 19, TypeScript, Tailwind CSS, Wallet Standard, Solana Kit, Vitest,
  and Playwright foundation. Added generated staking and faucet IDL/type
  artifacts under `app/src/generated`, a read-only Devnet console, wallet
  discovery/connect UI, deployment/account status display, Solana RPC account
  reads, and shared bigint helpers for six-decimal token base units.
- Security and economic rules touched: Frontend arithmetic helpers parse and
  format token amounts as integer base units, not JavaScript floating point.
  The Milestone 20 UI is read-only except wallet connection; no transaction
  builders, signing, CPI assumptions, or trusted client-side accounting were
  introduced.
- Tests run: `npm run typecheck`; `npm test`; `npm run build`; localhost smoke
  with `NEXT_PUBLIC_POOL=Fg6PaFpoGXkYsidMpWxTWqkFrnDRBTTnyW6m9n6eGJZ npm run
  dev -- --hostname 127.0.0.1 --port 3000` and `curl -I
  http://127.0.0.1:3000`.
- Result: Passed. TypeScript completed with `tsc --noEmit`; Vitest ran 2 test
  files and 6 tests; `next build --webpack` compiled and prerendered `/`; the
  dev server returned HTTP 200 for the dashboard route. The default Turbopack
  production build path panicked in this WSL/container environment while
  PostCSS tried to bind an internal port, so the build script uses Next's
  supported `--webpack` flag.
- Residual risk or limitation: The app currently renders deployment status and
  wallet connectivity only. User transaction construction, decoded Pool and
  Position account data, browser E2E coverage, and real Devnet Explorer smoke
  evidence remain later milestones.

### Milestone 21 - User Transaction Flows

- Scope implemented: Added milestone-marked frontend PDA/ATA derivation helpers,
  Anchor instruction encoders, prepared user transaction builders for faucet
  claim, open position, stake, open-plus-stake bundling, unstake, claim rewards,
  emergency withdraw, and close position, plus a Solana Kit transaction-message
  assembly helper. The dashboard now lifts wallet connection state, derives
  user token accounts, decodes Pool and Position account data, reads token
  balances, displays principal, paused state, and estimated pending rewards,
  and shows a transaction preparation status panel. Emergency withdrawal
  requires an explicit forfeiture checkbox.
- Security and economic rules touched: Frontend derivations mirror the
  canonical on-chain seed recipes for Position, Pool Authority, Faucet
  Authority, Faucet Claim, and user ATAs. Amount parsing rejects malformed,
  negative, zero, over-precision, scientific-notation, and oversized `u64`
  values before instruction data is encoded. Pending rewards are displayed as
  an estimate only; on-chain settlement and claim execution remain
  authoritative.
- Tests run: `npm run typecheck`; `npm test`; `npm run e2e`; `npm run build`;
  `cargo fmt --all -- --check`; `git diff --check`.
- Result: Passed. TypeScript completed with `tsc --noEmit`; Vitest ran 6 test
  files and 17 tests covering amount parsing, PDA/ATA derivation, account
  decoding, user instruction account lists, Anchor discriminators, u64 amount
  encoding, Pool Authority PDA use, and transaction-message assembly.
  Playwright ran 2 Chromium tests covering the disconnected action surface and
  connected fake-wallet first-stake preparation. `next build --webpack`
  compiled and prerendered `/`.
- Residual risk or limitation: The milestone now prepares canonical user
  transactions but does not yet submit signed wallet transactions to a live
  local validator. Playwright uses a fake Wallet Standard wallet and configured
  account addresses for browser coverage. Full local-validator transaction
  journeys, real wallet differences, RPC staleness handling, and Devnet
  Explorer evidence remain future milestones.

### Milestone 22 - Admin And Proposal Flows

- Scope implemented: Added milestone-marked frontend builders for reward
  funding, immediate pause, proposal creation, proposal approval, proposal
  execution, and proposal closure. Added Proposal PDA derivation, Proposal
  account decoding, Proposal RPC reads, and an admin dashboard panel that shows
  wallet eligibility, `admin_epoch`, next proposal ID, exact selected action
  parameters, loaded proposal action, approval count, expiry slot, execution
  state, and creator.
- Security and economic rules touched: Frontend builders use the same
  canonical Proposal PDA seed recipe as the staking program and keep all token
  amounts as integer base units. Admin-only buttons are gated by current Pool
  account data, but this remains advisory UI gating; on-chain admin membership,
  epoch, expiry, approval, and execution checks remain authoritative.
- Tests run: `npm run typecheck`; `npm test`; `npm run e2e`.
- Result: Passed. TypeScript completed with `tsc --noEmit`; Vitest ran 8 test
  files and 26 tests covering admin instruction discriminators, proposal action
  encoding, account roles, proposal PDA derivation, variable Proposal account
  decoding, and admin component display/gating. Playwright ran 3 Chromium tests
  covering the disconnected user/admin surface, connected fake-wallet stake
  preparation, and connected admin surface gating before live signatures.
- Residual risk or limitation: The browser tests still use fake Wallet Standard
  accounts and verify transaction preparation, not signed submission to a live
  validator. Two-real-wallet approval/execution, stale/expired proposal
  browser journeys, and transaction confirmation UX remain future hardening.

### Milestone 23 - Local End-To-End System Test

- Scope implemented: Added `tests/local-e2e/run.sh`, a one-command local
  harness that creates ignored fixture wallets, builds both Anchor programs,
  starts a fresh `solana-test-validator`, loads the staking and faucet program
  shared objects, airdrops the local payer, generates setup config, initializes
  and funds a local pool through the deployment helper, then runs the current
  LiteSVM invariant suite plus frontend unit and browser suites. Added local
  E2E documentation and ignore rules for reproducible generated local metadata.
- Security and economic rules touched: The harness keeps keypairs under
  `.private/local-e2e`, writes only public machine-specific metadata to the
  ignored `deployments/local-e2e.generated.json`, and reuses the Milestone 18
  invariant suite for Alice/Bob reward distribution, pause behavior, emergency
  forfeiture, governance rate change, admin replacement, proposal invalidation,
  solvency, conservation, and rollback checks.
- Tests run: `bash -n tests/local-e2e/run.sh`.
- Result: Passed syntax validation. The full harness is intentionally handed
  off because it can take a while: `tests/local-e2e/run.sh`.
- Residual risk or limitation: Full local-validator execution evidence is
  pending the handoff command. The browser segment currently validates rendered
  action surfaces and prepared transaction plans; live wallet-signed browser
  transaction submission remains future work.

### Milestone 24 - Devnet Deployment And Smoke Evidence

- Scope implemented: Added a `smoke` mode to the Rust deployment helper and a
  `scripts/devnet/smoke.sh` wrapper. The smoke path reads
  `deployments/devnet.json`, rejects secret-like metadata, checks live RPC
  account data, verifies both program accounts are executable, validates
  six-decimal original SPL Token mints, proves the STAKE mint authority is the
  Faucet Authority PDA, proves the REWARD mint authority is revoked, checks
  disabled freeze authorities, checks Pool fields against metadata, validates
  both vault token accounts are owned by the Pool Authority PDA, and asserts
  principal and reward solvency.
- Security and economic rules touched: The smoke command is read-only and does
  not sign transactions. It reduces false-evidence risk by checking that public
  frontend/deployment metadata matches the real deployed accounts and the
  fixed-supply reward rule.
- Tests run: `cargo test -p deployment_tools`; `cargo test --workspace`;
  `cargo clippy --workspace --all-targets -- -D warnings`; `cargo fmt --all
  -- --check`; `bash -n scripts/devnet/smoke.sh`; `bash -n
  tests/local-e2e/run.sh`; `npm run typecheck`; `npm test`; `npm run build`;
  `npm run e2e`; `git diff --check`.
- Result: Passed. The helper test suite now includes Milestone 24 coverage for
  RPC label resolution and Devnet Explorer link generation in addition to the
  existing Milestone 19 secret-safety tests.
- Residual risk or limitation: Actual Devnet deployment, setup signatures,
  smoke output, and Explorer evidence require the user's funded Devnet wallet
  and have not been recorded yet. Program deployment signatures are produced by
  `anchor deploy` and must be copied into public evidence manually.

### Milestone 25 - Final Self-Audit And Research Package

- Scope implemented: Replaced placeholder public research and diagram files
  with a portfolio-ready research narrative, account map, user flow,
  governance flow, and reward-solvency diagram. Added a Devnet evidence
  template and updated deployment/script documentation for the final evidence
  workflow.
- Security and economic rules touched: The final package repeats the
  non-production disclaimer, distinguishes internal self-audit from independent
  audit, documents trusted upgrade authority, Sybilable faucet limits, public
  RPC assumptions, direct vault surplus behavior, and the current frontend
  limitation that transactions are prepared but not live-submitted.
- Tests run: Documentation edits are covered by `cargo fmt --all -- --check`,
  `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D
  warnings`, `npm run typecheck`, `npm test`, `npm run build`, `npm run e2e`,
  shell syntax checks, and `git diff --check`.
- Result: Prepared but not final. The final audit cannot be closed until
  Milestone 24 Devnet evidence and the applicable full verification commands
  are complete.
- Residual risk or limitation: The public package is ready for evidence
  insertion, but the repository must not claim Milestone 25 complete until the
  Devnet smoke evidence is recorded.

## Consolidated Threat Model

This self-audit assumes honest Solana runtime signature checks, honest Clock
sysvar slots, correct execution of the original SPL Token Program, and no
private-key compromise. The deployment wallet remains upgrade authority on
Devnet, so it can replace program code and is more powerful than pool admins.

Primary assets:

- User STAKE principal in the Stake Vault.
- Funded REWARD budget and allocated reward liability in the Reward Vault.
- Pool, Position, Proposal, and Faucet Claim account integrity.
- Admin authority and proposal state.
- Public deployment metadata used by the frontend.

Primary risks and controls:

- Principal theft: vault authority is a PDA controlled only by staking-program
  signer seeds; all vault outflows validate canonical accounts first.
- Reward overpayment: checkpoint, settlement, and claim math use checked
  integer arithmetic and assert reward backing.
- Backpay or paused-slot accrual: pause and unpause update checkpoint
  boundaries using trusted slots.
- Account substitution: Anchor constraints and explicit checks bind pool,
  position, proposal, mint, vault, token authority, and Token Program IDs.
- Governance abuse: proposals are allowlisted, immutable, epoch-bound,
  expiring, threshold-approved, and one-time executable.
- Faucet replay: each wallet/mint pair creates one permanent claim receipt PDA.
- False deployment evidence: Milestone 24 smoke checks public metadata against
  live RPC account data.

## Consolidated Evidence Matrix

```text
Arithmetic and reward math        staking_pool unit tests
PDA/account schemas               state_pda tests
Token CPI flows                   LiteSVM milestone9_12_token_flows
Pause/emergency behavior          LiteSVM milestone13_14 coverage
Governance lifecycle              LiteSVM milestone15_16_governance
Faucet replay and mint authority  LiteSVM milestone17_demo_faucet
Cross-instruction invariants      LiteSVM milestone18
Setup validation                  deployment_tools unit tests
Frontend preparation              Vitest and Playwright app tests
Local harness                     tests/local-e2e/run.sh
Devnet smoke                      scripts/devnet/smoke.sh, pending live run
```

## Final Limitations

This repository is educational Devnet software. It is not production-ready, has
not received an independent professional audit, and must not custody assets
with real value. The faucet has no Sybil resistance, Devnet may reset, public
RPC can be stale or unavailable, frontend estimates are not authoritative, and
the current frontend prepares transactions without completing live wallet
submission.
