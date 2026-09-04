# Local E2E Tests

Milestone 23 adds a one-command local harness:

```bash
tests/local-e2e/run.sh
```

The script creates ignored fixture wallets under `.private/local-e2e`, builds
the Anchor programs, starts a fresh `solana-test-validator`, loads both program
shared objects, airdrops the local payer, runs the setup helper, and then runs:

```bash
cargo test -p litesvm_baseline milestone18 -- --nocapture
cd app && npm test
cd app && npm run e2e
```

The generated local deployment metadata is written to
`deployments/local-e2e.generated.json`, which is ignored because it is
machine-specific and reproducible. The file contains public addresses only.

Current limitation: the browser suite verifies rendered transaction surfaces
and prepared canonical account lists. Real wallet-signed local browser
submissions remain a later strengthening task because the frontend still stops
at transaction preparation.
