# Slot Staking Frontend

The frontend is a Next.js 16, React 19, TypeScript, Wallet Standard,
`@solana/kit`, Tailwind, Vitest, and Playwright app for the Slot Staking
Devnet console.

Implemented surfaces:

- Wallet discovery and connection.
- Pool, Position, Proposal, token-account, and balance reads.
- User transaction preparation for faucet claim, open, stake, unstake, claim,
  emergency withdraw, and close.
- Admin transaction preparation for funding, pause, proposal creation,
  approval, execution, and closure.

The app currently prepares canonical transactions and displays status. Live
wallet signing/submission is a future strengthening step.
