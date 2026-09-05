# Devnet Evidence

Milestone 24 public evidence only. No keypair files, seed phrases, private RPC
URLs, or wallet screenshots are included.

## Deployment

```text
staking_program: 8Dkwd74ntycfAMWKeudjELGroj5pqUpWk4MyLijuf1W7
demo_faucet_program: J12YAqC7dWbVWVAFveRdJ8SJ3sYc6roP3WJekhy4bDkM
upgrade_authority: 2W5TpA6NHR5bCuDA7btAy5aZwKk8PkM3KFHefztBdYJp
deployment_wallet: 2W5TpA6NHR5bCuDA7btAy5aZwKk8PkM3KFHefztBdYJp
deployment_date_utc: 2026-09-05
```

## Setup Metadata

```text
devnet_metadata_file: deployments/devnet.json
stake_mint: CdFFrhokPGEG4kuHvwKr7m2JqXpxJ8cbQxB3rQG8cXgx
reward_mint: DxJ7zRQKNu9c6oeBuLuc6QRrMFZerq2hyGYPdxquWZgN
reward_treasury_ata: BqhhFLj8S4igU24jm3tSJuxpDnN3Xea72SysWsnJexfT
pool: EwAD89dRxyGAJZb6yHb5Lba3SjSUJiHsThBBqvgJ8k1T
pool_authority: 4JdgDdmVyCYFXymPab8jRzEsFyZ5trweW9eabth2pgor
stake_vault: 8iMZwCFwG4mp3nmVJfueF9iCEzbvF6w5rm5kHfgQ9THX
reward_vault: 62nWQZSgMXmPzUPVmWvZc2BdeBDRcUQGPD5fP5a5tuXp
faucet_authority: Gq54tVVQaaU1NYR6uG1Czy8eWLA6aqHWMQMi8BPPc6uZ
admins:
  2W5TpA6NHR5bCuDA7btAy5aZwKk8PkM3KFHefztBdYJp
  GHYwa6ADkvizRn8ZzQXLcEFH9z5U27PTw2rHaU6a1w4P
  EK6Ywiern5BgWV4n8wYV4wL8fwqVJKj18RaBqF7F9nZY
```

## Transaction Signatures

```text
staking_program_deploy_signature:
  2kpx6iepQp7VaG2iygiCEHALuFTpiSRVKd3fY4PBy7zbcE4feTm4eLtopArZ2Ty77NKw5tATBtwymvwTLy8TToqf
demo_faucet_deploy_signature:
  5L6W2Kun5Xy4hVmrY9usaHdt2a9t8BD2BtsCcXtsXbGqNJV13sxhPt8nWCr8FQbTnxkc7T2spDDmSkYcJNnaSMiM
setup_or_initialization_signatures:
  not captured by the setup helper output used for this evidence record
compact_smoke_flow_signatures:
  smoke command is read-only and produces no transaction signature
```

## Smoke Assertions

Command:

```bash
scripts/devnet/smoke.sh
```

Observed output:

```text
Milestone 24 smoke passed.
slot: 493276421
staking program: https://explorer.solana.com/address/8Dkwd74ntycfAMWKeudjELGroj5pqUpWk4MyLijuf1W7?cluster=devnet
demo faucet: https://explorer.solana.com/address/J12YAqC7dWbVWVAFveRdJ8SJ3sYc6roP3WJekhy4bDkM?cluster=devnet
pool: https://explorer.solana.com/address/EwAD89dRxyGAJZb6yHb5Lba3SjSUJiHsThBBqvgJ8k1T?cluster=devnet
stake mint: https://explorer.solana.com/address/CdFFrhokPGEG4kuHvwKr7m2JqXpxJ8cbQxB3rQG8cXgx?cluster=devnet
reward mint: https://explorer.solana.com/address/DxJ7zRQKNu9c6oeBuLuc6QRrMFZerq2hyGYPdxquWZgN?cluster=devnet
reward mint authority revoked: true
principal solvency: 0 >= 0
reward solvency: 10000000000000000000 <= 10000000000000000000
```

## Explorer Links

```text
staking_program:
  https://explorer.solana.com/address/8Dkwd74ntycfAMWKeudjELGroj5pqUpWk4MyLijuf1W7?cluster=devnet
demo_faucet_program:
  https://explorer.solana.com/address/J12YAqC7dWbVWVAFveRdJ8SJ3sYc6roP3WJekhy4bDkM?cluster=devnet
stake_mint:
  https://explorer.solana.com/address/CdFFrhokPGEG4kuHvwKr7m2JqXpxJ8cbQxB3rQG8cXgx?cluster=devnet
reward_mint:
  https://explorer.solana.com/address/DxJ7zRQKNu9c6oeBuLuc6QRrMFZerq2hyGYPdxquWZgN?cluster=devnet
pool:
  https://explorer.solana.com/address/EwAD89dRxyGAJZb6yHb5Lba3SjSUJiHsThBBqvgJ8k1T?cluster=devnet
pool_authority:
  https://explorer.solana.com/address/4JdgDdmVyCYFXymPab8jRzEsFyZ5trweW9eabth2pgor?cluster=devnet
stake_vault:
  https://explorer.solana.com/address/8iMZwCFwG4mp3nmVJfueF9iCEzbvF6w5rm5kHfgQ9THX?cluster=devnet
reward_vault:
  https://explorer.solana.com/address/62nWQZSgMXmPzUPVmWvZc2BdeBDRcUQGPD5fP5a5tuXp?cluster=devnet
faucet_authority:
  https://explorer.solana.com/address/Gq54tVVQaaU1NYR6uG1Czy8eWLA6aqHWMQMi8BPPc6uZ?cluster=devnet
```
