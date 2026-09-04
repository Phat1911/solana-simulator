#!/usr/bin/env bash
# Milestone 24: verify public Devnet deployment metadata against live RPC.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

OUTPUT="${OUTPUT:-deployments/devnet.json}"

cargo run -p deployment_tools -- smoke --output "$OUTPUT"
