#!/usr/bin/env bash
set -euo pipefail

# Run from an exact-revision build job with Rust and nextest available.
# This entry point provisions its pinned protoc and lint tools. It does not
# schedule jobs, fetch a moving branch or publish anything.
cd "$(dirname "$0")/.."

tool_dir="$(tools/bootstrap-lint-tools --protoc)"
export PATH="$tool_dir:$PATH"

make prepare-test
make lint BIOMCP_ADVISORY_SCOPE=always
make stress
make test-long
tools/with-build-identity cargo build --release --locked
tools/with-build-identity cargo nextest run --release --lib rmcp_stdio_recovers_from_tool_panic
scripts/regenerate-alphagenome-proto --check
make repo-audit
make full-feature-check
make installed-wheel-smoke
make feature-coverage
