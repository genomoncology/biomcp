#!/usr/bin/env bash
set -euo pipefail

# Run from an exact-revision build job. The scheduler prepares the same pinned
# Rust, nextest and protoc toolchain as release qualification. This entry point
# does not schedule jobs, fetch a moving branch or publish anything.
cd "$(dirname "$0")/.."

make prepare-test
make stress
make test-long
tools/with-build-identity cargo build --release --locked
tools/with-build-identity cargo nextest run --release --test rmcp_client_contract rmcp_stdio_recovers_from_tool_panic
scripts/regenerate-alphagenome-proto --check
make feature-coverage
