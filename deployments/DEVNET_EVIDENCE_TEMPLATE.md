# Devnet Evidence Template

Milestone 24 records public evidence only. Do not paste keypair files, seed
phrases, private RPC URLs, or wallet screenshots that expose secrets.

## Deployment

```text
staking_program:
demo_faucet_program:
upgrade_authority:
deployment_wallet:
deployment_date_utc:
```

## Setup Metadata

```text
devnet_metadata_file: deployments/devnet.json
stake_mint:
reward_mint:
reward_treasury_ata:
pool:
pool_authority:
stake_vault:
reward_vault:
faucet_authority:
admins:
```

## Transaction Signatures

```text
staking_program_deploy_signature:
demo_faucet_deploy_signature:
setup_or_initialization_signatures:
compact_smoke_flow_signatures:
```

## Smoke Assertions

Paste the output of:

```bash
scripts/devnet/smoke.sh
```

Expected assertions:

- Staking Program account exists and is executable.
- Demo Faucet Program account exists and is executable.
- STAKE and REWARD mints use six decimals.
- STAKE mint authority is the Faucet Authority PDA.
- REWARD mint authority is revoked.
- Both freeze authorities are disabled.
- Pool state matches `deployments/devnet.json`.
- Stake and reward vaults are original SPL Token accounts owned by the Pool
  Authority PDA.
- Principal solvency holds.
- Reward solvency holds.

## Explorer Links

```text
staking_program:
demo_faucet_program:
stake_mint:
reward_mint:
pool:
pool_authority:
stake_vault:
reward_vault:
faucet_authority:
```
