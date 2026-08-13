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
`make lint` bootstraps the repository's checksum-verified pinned ShellCheck and
actionlint binaries on supported hosts, then runs the repo lint script and the
quality ratchet. Unsupported hosts receive explicit manual installation
instructions. `make spec` is
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
Use `make lint`, `make test`, and `make spec` as the canonical local gates.
The Python portion of `make test` uses four bounded file-distributed workers;
use `make test PYTEST_WORKERS=1` for a one-worker diagnostic run. There is no
supported `make check` command. `make release-gate` is the single
release-readiness command; it runs the routine gates, the named full-feature
proof, and release-profile specs. The GitHub
Release workflow additionally hard-runs the live contract and release smokes in
`validate` before publishing assets; the contract smoke workflow is manual-only
and does not run on a daily schedule. Use `make test-contracts` to rerun just
the release-critical Python/docs lane.

On Linux, `make test` and `make spec` require Bubblewrap. Each target prepares
dependencies and compiled artifacts first, then runs its assertions in a
network namespace where public DNS and direct public connections are blocked.
Loopback fixtures and Unix sockets remain available. CI's Linux result is
authoritative; other operating systems report that this enforcement is
unsupported. `make verify` remains the explicit network-enabled lane.

Routine gates use `--no-default-features` for one reusable lint/test/spec Cargo
graph and therefore do not exercise AlphaGenome. `make full-feature-check`
lints all targets with all shipped features, runs the AlphaGenome behavior
tests, and builds the all-feature release CLI. `make release-gate` runs the
routine gates plus that full-feature proof.

### Local Pre-Commit Hook

Developers opt in to the repository-owned hook for each checkout; the
repository does not install it automatically. Run
`scripts/install-pre-commit-hook` to install it:

```bash
scripts/install-pre-commit-hook
```

Check whether the current checkout already has that exact handoff without
changing it:

```bash
scripts/install-pre-commit-hook --check
```

The installed `.git/hooks/pre-commit` file is only a thin handoff to the tracked
`scripts/pre-commit` entrypoint. Every commit still runs the credential and
forbidden-artifact scans. A change containing only Markdown in the repository
root, `sdlc/`, `docs/`, `architecture/`, `spec/`, or `skills/` runs strict
documentation checks without Cargo, rustfmt, or Clippy. Any other staged path
runs the full Rust pre-commit checks. The artifact helper
`scripts/pre-commit-reject-march-artifacts.sh` still permits staged deletions
and allows only `.march/code-review-log.md` under `.march/`.
The full path runs `cargo fmt --check` and
`cargo clippy --no-default-features --lib --tests -- -D warnings`.

### Rust maintenance ratchets

Canonical lint pins every tracked Rust source file above 1,000 physical lines
in `tools/rust-source-size-inventory.json`. Files below the threshold may grow
only to 1,000 lines, and new files may not begin above it. When a large file
shrinks, run `tools/update-rust-source-size-inventory` to lower its baseline.
The command refuses growth unless one path is named with `--authorize` plus a
ticket, an exact reason, and a removal condition. Prefer a local extraction to
raising a baseline.

Every `allow(dead_code)` is also checked against
`tools/dead-code-exceptions.json`. Keep the adjacent `dead-code reason:` comment
and the inventory's owner and removal condition precise. Add or broaden an
exception only when a ticket explicitly requires it; generated bindings are
excluded solely at their exact generated path.

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
