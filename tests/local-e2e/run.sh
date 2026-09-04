#!/usr/bin/env bash
# Milestone 23: one-command local validator setup plus program/browser checks.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

WORK_DIR="${LOCAL_E2E_DIR:-$ROOT_DIR/.private/local-e2e}"
LEDGER_DIR="$WORK_DIR/ledger"
CONFIG_PATH="$WORK_DIR/config.json"
DEPLOYMENT_PATH="${LOCAL_E2E_DEPLOYMENT:-$ROOT_DIR/deployments/local-e2e.generated.json}"
RPC_URL="${LOCAL_E2E_RPC_URL:-http://127.0.0.1:8899}"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

require_command anchor
require_command cargo
require_command npm
require_command solana
require_command solana-keygen
require_command solana-test-validator

mkdir -p "$WORK_DIR" "$(dirname "$DEPLOYMENT_PATH")"

keygen() {
  local path="$1"
  if [[ ! -f "$path" ]]; then
    solana-keygen new --no-bip39-passphrase --silent --outfile "$path" >/dev/null
  fi
}

keygen "$WORK_DIR/payer.json"
keygen "$WORK_DIR/admin-a.json"
keygen "$WORK_DIR/admin-b.json"
keygen "$WORK_DIR/admin-c.json"

PAYER="$(solana-keygen pubkey "$WORK_DIR/payer.json")"
ADMIN_A="$(solana-keygen pubkey "$WORK_DIR/admin-a.json")"
ADMIN_B="$(solana-keygen pubkey "$WORK_DIR/admin-b.json")"
ADMIN_C="$(solana-keygen pubkey "$WORK_DIR/admin-c.json")"
STAKING_PROGRAM_ID="Fg6PaFpoGXkYsidMpWxTWqkFrnDRBTTnyW6m9n6eGJZ"
DEMO_FAUCET_PROGRAM_ID="J12YAqC7dWbVWVAFveRdJ8SJ3sYc6roP3WJekhy4bDkM"

anchor build

rm -rf "$LEDGER_DIR"
solana-test-validator \
  --reset \
  --quiet \
  --ledger "$LEDGER_DIR" \
  --bpf-program "$STAKING_PROGRAM_ID" target/deploy/staking_pool.so \
  --bpf-program "$DEMO_FAUCET_PROGRAM_ID" target/deploy/demo_faucet.so \
  >"$WORK_DIR/validator.log" 2>&1 &
VALIDATOR_PID=$!
trap 'kill "$VALIDATOR_PID" >/dev/null 2>&1 || true' EXIT

for _ in {1..40}; do
  if solana --url "$RPC_URL" cluster-version >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

solana --url "$RPC_URL" airdrop 10 "$PAYER" >/dev/null

cat >"$CONFIG_PATH" <<JSON
{
  "cluster": "localnet",
  "rpc_url": "$RPC_URL",
  "payer_keypair": "$WORK_DIR/payer.json",
  "keypair_dir": "$WORK_DIR/mints",
  "pool_id": 0,
  "admins": [
    "$ADMIN_A",
    "$ADMIN_B",
    "$ADMIN_C"
  ],
  "max_reward_rate_per_slot_base_units": 100000000,
  "initial_reward_funding_base_units": 500000000000
}
JSON

cargo run -p deployment_tools -- setup --config "$CONFIG_PATH" --output "$DEPLOYMENT_PATH"
cargo test -p litesvm_baseline milestone18 -- --nocapture

pushd app >/dev/null
npm test
npm run e2e
popd >/dev/null

echo "Milestone 23 local E2E harness completed."
echo "Deployment metadata: $DEPLOYMENT_PATH"
echo "Validator log: $WORK_DIR/validator.log"
