# Scripts

Milestone 1 reserves this directory for setup and smoke-test automation.

Scripts must be idempotent where practical and must avoid logging secrets.

## Milestone 19 Setup

The Devnet/localnet setup helper is split into a thin shell wrapper and a Rust
CLI:

```bash
cp scripts/devnet/config.example.json scripts/devnet/config.local.json
scripts/devnet/setup.sh validate
scripts/devnet/setup.sh dry-run
scripts/devnet/setup.sh setup
```

`CONFIG` and `OUTPUT` can override the default paths:

```bash
CONFIG=/path/to/config.json OUTPUT=/tmp/localnet.json scripts/devnet/setup.sh setup
```

The helper creates or reuses ignored mint keypairs, creates six-decimal original
SPL Token mints, assigns the STAKE mint authority to the Faucet Authority PDA,
mints exactly `1_000_000 REWARD`, revokes the REWARD mint authority, initializes
the staking pool, funds rewards through the staking program, and writes public
metadata only.

Never commit `scripts/devnet/config.local.json`, `.private/`, wallet keypairs,
seed phrases, or private RPC URLs.
