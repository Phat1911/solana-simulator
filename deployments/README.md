# Deployments

Milestone 1 reserves this directory for public deployment metadata.

Never store wallet keypairs, seed phrases, private RPC URLs, or other secrets
here.

Milestone 19 writes `deployments/devnet.json` after an approved Devnet setup.
That file is intended to contain only public addresses, transaction-independent
configuration, and non-secret status notes. Use `deployments/devnet.example.json`
as the public shape before the real Devnet smoke milestone.

## Milestone 24 Evidence Flow

1. Build and deploy both programs to Devnet with the configured deploy wallet.
2. Run the approved setup helper so `deployments/devnet.json` is written.
3. Run:

```bash
scripts/devnet/smoke.sh
```

4. Copy public addresses, transaction signatures, smoke output, and Explorer
   links into a final evidence note using
   `deployments/DEVNET_EVIDENCE_TEMPLATE.md`.

The committed `deployments/devnet.json` may contain public addresses and public
RPC labels only. It must never contain keypair arrays, private RPC URLs, seed
phrases, or local wallet paths.
