# Deployments

Milestone 1 reserves this directory for public deployment metadata.

Never store wallet keypairs, seed phrases, private RPC URLs, or other secrets
here.

Milestone 19 writes `deployments/devnet.json` after an approved Devnet setup.
That file is intended to contain only public addresses, transaction-independent
configuration, and non-secret status notes. Use `deployments/devnet.example.json`
as the public shape before the real Devnet smoke milestone.
