# Project Agent Guide

## Working Context

- In your implementation, don't forget to mark (comment) what is milestone this code relate.
- Update current status section in README.md for progression.
- Always communicate with the user in English don't care what language i use.
- The user is new to Rust and Solana, but understands basic Solidity/Ethereum ideas.
- Explain each new core concept from first principles and include a small numeric example when useful.
- At a core-logic milestone, confirm the user's understanding before implementing it. Supporting work only needs a quick review.
- `SPEC.md` is authoritative. Do not silently change an approved security rule, economic rule, account model, or scope decision.
- Keep beginner notes private and gitignored. Public documentation must be portfolio-ready.

## Project Structure

```text
programs/staking_pool/   Anchor staking program
programs/demo_faucet/    Separate devnet-only faucet program
app/                     Next.js frontend and frontend tests
tests/litesvm/           Fast program integration tests
tests/local-e2e/         Local validator and browser workflows
scripts/                 Devnet setup and smoke-test scripts
docs/diagrams/           Architecture and flow diagrams
docs/RESEARCH.md         Public research narrative
deployments/             Public deployment metadata; never secrets
SPEC.md                  Approved product and protocol specification
PLAN.md                  Ordered implementation milestones
AUDIT.md                 Living self-audit and test evidence
```

## Coding Conventions

- Follow the module boundaries in `SPEC.md`: instruction handlers, state, constants, errors, events, and pure math.
- Keep reward math in pure functions. Keep account validation and SPL Token CPIs in instruction handlers.
- Use checked `u64` arithmetic for token/slot values and checked `u128` arithmetic for scaled intermediates. Never use floating point or a decimal crate for protocol accounting.
- Represent frontend token values with integer base units and TypeScript `bigint`.
- Validate all accounts and authorization before changing state or invoking a CPI. Return explicit custom errors; do not panic or `unwrap` user-controlled input.
- Use fixed-size, versioned account layouts. Use canonical PDA seed recipes and canonical user ATAs exactly as specified.
- Emit events only after successful instructions. Keep comments brief and explain reasons, invariants, or non-obvious Solana behavior.
- Use the original SPL Token Program, not Token-2022, for this project.

## Testing Conventions

- Add tests in the same milestone as behavior; a milestone is not complete while its required tests fail or are missing.
- Test happy paths, authorization failures, boundary arithmetic, malicious account substitution, and atomic rollback.
- Use the lightest sufficient layer: pure Rust unit tests, then LiteSVM, local-validator E2E, browser E2E, and finally a small Devnet smoke test.
- Assert economic invariants directly, including principal solvency, reward solvency, and conservation across users and vaults.
- Use behavior-oriented test names such as `claim_rejects_wrong_reward_vault`.
- Run formatting, compilation, and the relevant test layers before marking a milestone complete.
- Update `AUDIT.md` continuously with the milestone's risks, controls, tests, results, and remaining limitations.

## Rules

### Security

- This is educational Devnet software. Never present it as production-ready, audited, or suitable for mainnet funds.
- Trust the Solana Clock sysvar for slots; never trust a caller-supplied current slot.
- Validate PDA seeds and bumps, signers, account owners and discriminators, mint relationships, vault addresses and authorities, and the original Token Program ID.
- Pool Authority PDAs must own the staking and reward vaults. Vault outflows occur only through validated staking-program instructions using `invoke_signed` or Anchor signer seeds.
- Admins have no vault withdrawal, sweep, mint, or arbitrary-transfer instruction.
- All failed instructions must be atomic: no retained state changes, token movement, or success events.
- Use three distinct admins with threshold two. A single current admin may pause; unpause, reward-rate changes, and admin replacement require a proposal.
- A proposal contains exactly one immutable allowlisted action, is bound to one pool and the current `admin_epoch`, requires two distinct current approvals, expires after `216_000` slots, and executes at most once. It cannot contain arbitrary CPI or batched actions.
- Increment `admin_epoch` after admin replacement so all old proposals become stale.
- Do not commit or log wallet keypairs, seed phrases, private RPC URLs, or other secrets. Publish only addresses, transaction signatures, and non-secret configuration.

### Economics

- Both demo mints use six decimals. Store all amounts in base units and use `PRECISION = 1_000_000_000` for reward accounting.
- Preserve principal solvency: `stake_vault.amount >= pool.total_staked`.
- Preserve reward solvency: `remaining_budget_scaled + allocated_liability_scaled <= reward_vault.amount * PRECISION`.
- Checkpoint rewards before every reward-sensitive mutation. A rate change settles at the old rate; funding settles before adding the new budget.
- Emit no rewards while paused, while no stake exists, or after the funded budget is exhausted. Allow a partial final emission and preserve scaled rounding fractions.
- Normal unstake returns principal and preserves earned rewards. Emergency withdrawal returns principal and forfeits all pending rewards back to the unallocated reward reserve.
- While paused, freeze reward generation, allow normal unstake and emergency withdrawal, and block stake and claim. Unpause must not pay for paused slots.
- Funding is permissionless but the source token-account authority must sign. Direct vault transfers are surplus only and never become accounting credit.
- User stake, payout, and claim accounts are canonical ATAs. The funding source may be any correctly owned reward token account.
- Pool mints, vault relationships, and maximum reward rate are immutable. A pool starts paused with reward rate zero.
- The faucet gives each wallet `1_000 STAKE` once. Its receipt is permanent, Sybil resistance is out of scope, and the fixed `1_000_000 REWARD` supply is created during setup before reward mint authority is revoked.
- No fees, compounding, oracle, lock-up period, pool closure, admin sweep, or production upgrade-governance claim is in scope.

## Default Workflow

1. Read `AGENTS.md`, `SPEC.md`, `PLAN.md`, `AUDIT.md`, and the current worktree before changing code.
2. Select only the next incomplete milestone and restate its scope and required verification.
3. For a **Core logic** milestone, explain the concept, walk through a concrete example, state the invariant, and wait for explicit user confirmation before implementation.
4. For a **Supporting/boilerplate** milestone, give a concise preview and proceed after a quick review; no deep checkpoint is required.
5. Implement the smallest coherent change without unrelated refactoring or silent specification changes.
6. Format, compile, and run the milestone's required tests. Investigate failures instead of weakening assertions.
7. Update `PLAN.md` status and `AUDIT.md` evidence, then report changed files, test results, and known limitations.
8. Do not combine or skip core milestones unless the user explicitly asks. Stop and ask when an implementation choice would alter an approved rule.
9. Commit, tag (if needed), and push the change onto github on wsl environment.