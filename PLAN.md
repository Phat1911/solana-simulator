# Implementation Plan

This plan turns `SPEC.md` into small, reviewable increments. Milestones are completed in order unless the user explicitly changes the order.

## Milestones

### 1. Minimal Workspace And Toolchain

**Label:** Supporting/boilerplate — I need a quick review, not a deep checkpoint.

- [x] Create the Anchor workspace, the two program crates, shared test directories, frontend placeholder, and documented version pins.
- [x] Add formatting, linting, build, and test commands without implementing protocol behavior.
- **Verification:** Both empty programs compile through Cargo; Rust formatting and the initial test command pass. Anchor/Solana CLI verification is documented as pending local toolchain installation.

### 2. Test And Audit Baseline

**Label:** Supporting/boilerplate — I need a quick review, not a deep checkpoint.

- [x] Add pure Rust test modules, a minimal LiteSVM harness, `AUDIT.md`, and private-notes ignore rules.
- [x] Record tool versions, test layers, and the evidence format used by later milestones.
- **Verification:** One harmless test runs in each available lightweight harness; private-note paths are ignored.

### 3. Units And Checked Arithmetic

**Label:** Core logic

- [x] Define six-decimal token base units, `PRECISION = 1_000_000_000`, maximum reward rate, and checked conversion helpers.
- [x] Implement only pure arithmetic helpers for `u64` values and `u128` scaled intermediates.
- **Invariant:** Protocol math never uses floating point, never silently overflows or underflows, and never mixes token units with scaled-token units.
- **Verification:** Unit tests cover zero, one-base-unit, fractional-scaled, maximum-safe, overflow, underflow, and narrowing cases.

### 4. Global Reward Checkpoint Math

**Label:** Core logic

- [x] Implement the pure calculation for elapsed slots, desired emission, partial final emission, budget reduction, and `acc_reward_per_share_scaled` growth.
- [x] Model paused, zero-stake, and exhausted-budget checkpoints without account access or CPIs.
- **Invariant:** A checkpoint emits at most the unallocated funded budget; it emits nothing while paused, with zero total stake, or after exhaustion; `last_update_slot` advances without retroactive accrual.
- **Verification:** Table-driven unit tests cover normal accrual, zero stake, pause gaps, exact exhaustion, partial final emission, and rounding remainder.

### 5. Position Settlement And Claim Math

**Label:** Core logic

- [x] Implement pure position settlement, reward debt updates, pending rewards, claimable whole tokens, and scaled remainder preservation.
- [x] Implement the accounting effect of normal claim and emergency forfeiture.
- **Invariant:** Settling repeatedly at the same accumulator creates no reward; claims cannot exceed allocated liability; fractional rewards are preserved; forfeiture returns exactly the position's pending scaled liability to the unallocated reserve.
- **Verification:** Unit tests cover multiple users, staggered entry, repeated settlement, partial-token claims, post-claim accrual, and forfeiture conservation.

### 6. Staking Account Schemas And PDA Recipes

**Label:** Core logic

- [x] Define versioned, fixed-size `Pool`, `Position`, and `Proposal` layouts plus all canonical PDA derivations. The Pool Authority PDA has no data account.
- [x] Encode immutable mint/vault relationships, admin epoch, proposal action data, and position ownership.
- **Invariant:** Every state account has one canonical address for its semantic identity, and account substitution across pools, users, or proposals is rejected.
- **Verification:** PDA derivation and account-size tests prove deterministic addresses and separation between pools, users, and proposal IDs.

### 7. Pool Initialization And Vault Authority

**Label:** Core logic

- [x] Implement pool initialization with three distinct admins, threshold two, original Token Program checks, and separate SPL Token vaults controlled by the Pool Authority PDA.
- [x] Require distinct six-decimal stake and reward mints; start paused with rate zero and immutable maximum rate.
- **Invariant:** No human keypair, including an admin, can sign directly for a vault outflow; pool configuration is canonical and begins in a non-emitting state.
- **Verification:** LiteSVM tests cover valid initialization, duplicate admins, wrong decimals, same mint, wrong token program, wrong vault authority, and duplicate initialization.

### 8. Position Open And Close Lifecycle

**Label:** Core logic

- [x] Implement permissionless opening of the canonical user position and safe closure by its owning user.
- [x] Permit closure only when stake, pending reward, reward debt, and scaled remainder are all zero.
- **Invariant:** One user has at most one position per pool, another signer cannot control it, and closing an account cannot erase principal or reward liability.
- **Verification:** LiteSVM tests cover canonical creation, duplicate creation, wrong user/pool seeds, unauthorized close, non-empty close, and successful empty close.

### 9. Reward Funding

**Label:** Core logic

- [x] Implement permissionless funding from any valid reward token account whose authority signs.
- [x] Checkpoint before transfer, then increase unallocated scaled budget by exactly the received base units times `PRECISION`.
- **Invariant:** Accounting credit is created only by a successful validated SPL Token transfer; direct vault donations remain surplus and funding cannot erase already earned liabilities.
- **Verification:** LiteSVM tests cover valid third-party funding, wrong mint/vault/authority/program, direct donation behavior, rollback, and reward-solvency equations.

### 10. Stake

**Label:** Core logic

- [x] Implement staking from the user's canonical stake ATA after checkpointing and settling the existing position.
- [x] Transfer principal through the original SPL Token Program and update position and pool totals atomically.
- **Invariant:** Newly deposited principal earns no reward for earlier slots, and successful stake preserves `stake_vault.amount >= total_staked` with exact principal accounting.
- **Verification:** LiteSVM tests cover first and repeated stake, wrong ATA/mint/vault/position, zero amount, pause rejection, insufficient balance, and rollback.

### 11. Normal Unstake

**Label:** Core logic

- [x] Implement partial and full unstake after checkpointing and settling rewards, including while paused.
- [x] Return principal only to the user's canonical stake ATA using Pool Authority signer seeds.
- **Invariant:** Unstake never forfeits or pays rewards, never returns another user's principal, and decreases position stake, pool total, and vault balance by the same amount.
- **Verification:** LiteSVM tests cover active and paused unstake, partial/full amounts, excessive amount, substituted accounts, PDA signer failure, and atomic rollback.

### 12. Claim

**Label:** Core logic

- [x] Implement claim to the user's canonical reward ATA after checkpointing and settling the position.
- [x] Transfer only whole base units, decrement scaled liability by the exact transferred amount, and retain fractional scaled remainder.
- **Invariant:** A user can claim only their own allocated reward, each paid unit is removed from liability exactly once, and reward solvency holds after payout.
- **Verification:** LiteSVM tests cover repeated claims, no-op fractional claim, wrong claimant/ATA/vault, insufficient backing, pause rejection, and rollback.

### 13. Pause And Unpause Semantics

**Label:** Core logic

- [x] Implement immediate pause by any current admin and reserve unpause for the proposal execution path.
- [x] Checkpoint at pause and enforce paused-slot accounting; proposal execution will set the unpause boundary in Milestone 16.
- **Invariant:** Pausing freezes generation at the pause slot; paused pools block stake and claim but allow both withdrawal paths; unpause resumes from its current trusted slot without back pay.
- **Verification:** LiteSVM timeline tests measure rewards immediately before, during, and after pause and exercise unauthorized and redundant transitions.

### 14. Emergency Withdraw And Forfeiture

**Label:** Core logic

- [x] Implement emergency principal withdrawal with no reward payout in both active and paused states.
- [x] Settle first, return all principal, and move the position's complete pending scaled reward from allocated liability to unallocated reserve.
- **Invariant:** Emergency withdrawal cannot lose principal or create or destroy funded rewards; the user's reward becomes claimable by nobody and is again available for future emissions.
- **Verification:** LiteSVM tests cover whole and fractional pending rewards, active/paused calls, empty position, substituted accounts, and conservation before/after forfeiture.

### 15. Proposal Creation And Approval

**Label:** Core logic

- [x] Implement monotonically identified proposal PDAs for one immutable allowlisted action: set rate, unpause, or replace one admin.
- [x] Store pool, proposer approval, creation/expiry slots, and the current `admin_epoch`; allow one approval per distinct current admin.
- **Invariant:** No proposal can change meaning after an approval, collect duplicate/non-admin approvals, target another pool, exceed the maximum rate, or remain valid past expiry.
- **Verification:** LiteSVM tests cover all action encodings, immutable data, threshold counting, expiry boundaries, wrong epoch/pool, and invalid replacement keys.

### 16. Proposal Execution And Admin Rotation

**Label:** Core logic

- [x] Execute threshold-approved proposals exactly once; checkpoint before rate changes; apply unpause timing; rotate one admin and increment `admin_epoch`.
- [x] Add safe proposal closure only after execution, expiry, or stale-epoch invalidation, returning rent to the recorded proposer.
- **Invariant:** Execution performs exactly the stored action, stale proposals cannot execute after admin rotation, rate changes do not rewrite past rewards, and a proposal can never be replayed.
- **Verification:** LiteSVM tests cover each action, execution replay, stale epoch, expiry, wrong accounts, old-rate checkpointing, rent destination, and transaction rollback.

### 17. Demo Faucet Program

**Label:** Core logic

- [x] Implement fixed-config faucet initialization and one `1_000 STAKE` claim per wallet using canonical claim-receipt and faucet-authority PDAs.
- [x] Keep the receipt non-closable and separate all faucet authority from staking administration.
- **Invariant:** One wallet can receive at most one faucet allocation, only the configured stake mint can be issued, and the faucet has no authority over staking or reward vaults.
- **Verification:** LiteSVM tests cover first claim, replay, alternate token accounts, wrong mint/receipt/authority, and atomic rollback.

### 18. Cross-Instruction Invariant Suite

**Label:** Core logic

- [x] Build multi-user and adversarial state-machine scenarios across funding, staking, slot advancement, claims, pause, emergency withdrawal, and governance.
- [x] Assert vault balances and all scaled budget/liability quantities after every successful transition and every expected failure.
- **Invariant:** Principal and reward solvency, reward conservation, authorization, canonical account binding, and atomic rollback hold across arbitrary valid operation sequences.
- **Verification:** Deterministic scenario tests plus property/fuzz-style sequences run repeatedly with recorded seeds and no invariant failure.


### 19. Devnet Setup And Deployment Scripts

**Label:** Supporting/boilerplate

- [x] Add idempotent scripts to create six-decimal mints, mint `1_000_000 REWARD`, revoke reward mint authority, initialize the faucet and pool, fund rewards, and write public addresses to `deployments/devnet.json`.
- [x] Add secret-safety checks and clear wallet-balance prerequisites; never embed key material.
- **Verification:** Scripts support dry-run/config validation and successfully create a fresh local deployment before Devnet use.

### 20. Generated Client And Frontend Foundation

**Label:** Supporting/boilerplate

- [x] Generate the TypeScript client and IDL bindings and create the Next.js 16, React 19, Wallet Standard, `@solana/kit`, Tailwind CSS, Vitest, and Playwright foundation.
- [x] Add wallet connection, network display, pool discovery, transaction status, and reusable integer amount formatting.
- **Verification:** Frontend type-checks, builds, unit tests run, and account reads render against a known local deployment.

### 21. User Transaction Flows

**Label:** Core logic

- [ ] Implement faucet claim, position opening, stake, unstake, emergency withdrawal, and claim transaction builders using derived PDAs and canonical ATAs.
- [ ] Display principal, pending rewards, paused state, available balances, and explorer links without floating-point transaction math.
- **Invariant:** The UI derives and submits the same canonical accounts and integer base-unit amounts enforced by the programs; displayed estimates never become trusted on-chain inputs.
- **Verification:** Vitest covers amount parsing and derivation; Playwright covers successful and rejected user workflows against the local validator.

### 22. Admin And Proposal Flows

**Label:** Supporting/boilerplate.

- [ ] Add funding, immediate pause, proposal creation, approval, execution, and proposal-status views.
- [ ] Clearly show exact action parameters, approvals, epoch, expiry, execution state, and wallet eligibility before signing.
- **Verification:** Component tests cover state display; Playwright covers two-wallet approval and execution plus stale, expired, and unauthorized cases.

### 23. Local End-To-End System Test

**Label:** Supporting/boilerplate.

- [ ] Automate local validator startup, deployments, fixture wallets, mint setup, pool initialization, and full browser journeys.
- [ ] Include Alice/Bob reward distribution, pause behavior, emergency forfeiture, multisig rate change, admin replacement, and proposal invalidation.
- **Verification:** One documented command recreates the environment and passes program and browser E2E suites from a clean state.

### 24. Devnet Deployment And Smoke Evidence

**Label:** Supporting/boilerplate.

- [ ] Deploy both programs to Devnet, run the approved setup, execute a compact real-wallet smoke flow, and verify reward mint authority revocation.
- [ ] Record program IDs, mint/vault/PDA addresses, transaction signatures, explorer links, slot observations, and upgrade-authority status.
- **Verification:** The deployment metadata is reproducible, contains no secrets, and all recorded links and smoke assertions are valid.

### 25. Final Self-Audit And Research Package

**Label:** Supporting/boilerplate.

- [ ] Consolidate milestone audit entries into the final `AUDIT.md`, including threat model, controls, evidence, known limitations, and explicit non-audit disclaimer.
- [ ] Complete architecture/flow diagrams, `docs/RESEARCH.md`, and `README.md` with reproducible local and Devnet instructions.
- [ ] Run formatting, linting, unit, LiteSVM, local-validator, frontend, browser, and Devnet smoke checks applicable to the finished repository.
- **Verification:** Every acceptance criterion in `SPEC.md` maps to code, a test/evidence item, or a clearly documented out-of-scope limitation.

## Completion Rule

A core milestone is complete only when the user has confirmed the concept, the implementation preserves its stated invariant, the required tests pass, and `AUDIT.md` contains the evidence. Supporting milestones require implementation, verification, and a concise review, but no deep conceptual checkpoint.
