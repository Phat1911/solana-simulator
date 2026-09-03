#!/usr/bin/env bash
# Milestone 19: safe wrapper around the Rust localnet/devnet setup helper.
set -euo pipefail

MODE="${1:-validate}"
CONFIG="${CONFIG:-scripts/devnet/config.local.json}"
OUTPUT="${OUTPUT:-deployments/devnet.json}"

case "$MODE" in
  validate|dry-run|setup)
    ;;
  *)
    echo "usage: $0 [validate|dry-run|setup]" >&2
    exit 64
    ;;
esac

cargo run -p deployment_tools -- "$MODE" --config "$CONFIG" --output "$OUTPUT"
