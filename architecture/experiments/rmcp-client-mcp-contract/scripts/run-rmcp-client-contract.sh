#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"

cargo test --no-run --test rmcp_client_contract
cargo nextest run --test rmcp_client_contract
