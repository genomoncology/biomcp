# Contributing to BioMCP

BioMCP does not accept outside pull requests.

We do welcome:

- GitHub Issues for bugs, regressions, and reproducible problems
- GitHub Discussions for feature ideas, usage questions, and documentation requests

This policy keeps release provenance, supply-chain control, and copyright review
for AI-assisted code with the core maintainers. We still want problem reports
and product feedback, and the team will fix confirmed issues in the main repo.

When you open an issue or discussion, include:

- the BioMCP version
- the command you ran
- the relevant output or error text
- any source or API context needed to reproduce the problem

## Repo-Local Test Setup

Install `cargo-nextest` before running repo-local Rust verification:

```bash
cargo install cargo-nextest --locked
```

Normal builds do not run or require `protoc`; they compile the committed
AlphaGenome generated Rust source. Maintainers changing the protobuf inputs use
pinned `protoc` 28.3 and run `scripts/regenerate-alphagenome-proto`, while CI
runs `scripts/regenerate-alphagenome-proto --check` without editing the tree.

`make test` uses `cargo nextest run` plus the Python/docs contract lane.
`make lint` runs the repo lint script and the quality ratchet. `make spec` is
the offline deterministic routine executable-spec gate. `make spec-contracts`
is a deterministic legacy subset kept for profile compatibility. `make verify`
is the explicit opt-in live public-upstream confidence lane; CPIC `web_anon`
auth/permission failures and NIH Reporter funding-source/table unavailability
are reported there as operator-pending by `tools/biomcp-verify-live`, while
unexpected shapes and unclassified failures remain product-red. `make
release-live-smoke` remains a compatibility alias. `make spec-pr` remains
available for the same offline `SPEC_ROUTINE_PATHS` as `make spec`, through
`scripts/run-specs.sh`: routine Markdown specs use `mustmatch test --lang bash`,
with `tests/surface/test_parallel_isolation_contract.py` as the only routine
pytest canary. Other static Python surface contracts live under `tests/surface/`
and run through `make test`. The executable docs themselves call
`tools/biomcp-ci`, which owns release-binary resolution, the repo-owned
`.cache/biomcp-specs/` cache/XDG roots, optional-key stripping, and warm-hit
`BIOMCP_CACHE_MODE=infinite` replay when CI sets `BIOMCP_SPEC_CACHE_HIT=1`.
Use `make lint`, `make test`, and `make spec` as the canonical local gates;
there is no supported `make check` command. `make release-gate` is the single
release-readiness command; it runs the routine gates, the named full-feature
proof, and release-profile specs. The GitHub
Release workflow additionally hard-runs the live contract and release smokes in
`validate` before publishing assets; the contract smoke workflow is manual-only
and does not run on a daily schedule. Use `make test-contracts` to rerun just
the release-critical Python/docs lane.

Routine gates use `--no-default-features` for one reusable lint/test/spec Cargo
graph and therefore do not exercise AlphaGenome. `make full-feature-check`
lints all targets with all shipped features, runs the AlphaGenome behavior
tests, and builds the all-feature release CLI. `make release-gate` runs the
routine gates plus that full-feature proof.

### Local Pre-Commit Hook

Developers who opt in to the repo-local pre-commit hook should install it at
`$(git rev-parse --git-path hooks/pre-commit)`. The hook is local Git state;
the repo does not install it automatically.

Use this shape so `scripts/pre-commit-reject-march-artifacts.sh` runs before
`cargo fmt --check` and
`cargo clippy --no-default-features --lib --tests -- -D warnings`:

```bash
hook_path="$(git rev-parse --git-path hooks/pre-commit)"
mkdir -p "$(dirname "$hook_path")"
cat >"$hook_path" <<'HOOK'
#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

scripts/pre-commit-reject-march-artifacts.sh
cargo fmt --check
cargo clippy --no-default-features --lib --tests -- -D warnings
HOOK
chmod +x "$hook_path"
```

The helper allows only `.march/code-review-log.md` under `.march/`, and it
permits staged deletions so cleanup commits can remove old March artifacts from
tracking.

### Timing Method

Measured on beelink on 2026-04-23 with `/usr/bin/time -p` using warm-cache
steady-state runs. Each command was run once untimed to warm build artifacts and
the repo-owned spec cache under `.cache/biomcp-specs/`, then once with timing
enabled. The `make spec-pr` row was refreshed on 2026-04-24 after the spec-v2
canary cutover. `make release-gate` composes the routine gates, the named
full-feature proof, and release-profile specs, so its
warm timing tracks the current sum of those warmed routine component lanes.

| Command | Observed warm-cache | Notes |
|---|---|---|
| `make lint` | refresh pending | includes the quality ratchet |
| `make test` | refresh pending | Rust nextest plus Python/docs contract lane |
| `make spec-contracts` | `337.47s` | Markdown-only deterministic subset, including local MCP proof (2026-06-17; ticket 427) |
| `make spec` / `make spec-pr` | refresh pending | offline deterministic routine spec lane plus the parallel-isolation pytest canary |
| `make verify` | `operator-run` | opt-in live public-upstream smoke; not part of routine gates |
| `make release-gate` | refresh pending | lint + test + spec routine gate |
