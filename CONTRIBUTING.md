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

`make test` uses `cargo nextest run`. `make spec` and `make spec-pr` use
`pytest-xdist` for the parallel-safe bulk with `-n auto --dist loadfile`, while
`spec/05-drug.md`, `spec/13-study.md`, and
`spec/21-cross-entity-see-also.md` stay serial because they share repo-global
local-data fixtures. `make spec-smoke` resolves the ten stable targeted smoke
section IDs in `SPEC_SMOKE_ARGS` to current mustmatch pytest item IDs, then runs
them serially with a 120s mustmatch timeout.

### Local Pre-Commit Hook

Developers who opt in to the repo-local pre-commit hook should install it at
`$(git rev-parse --git-path hooks/pre-commit)`. The hook is local Git state;
the repo does not install it automatically.

Use this shape so `scripts/pre-commit-reject-march-artifacts.sh` runs before
`cargo fmt --check` and `cargo clippy --lib --tests -- -D warnings`:

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
cargo clippy --lib --tests -- -D warnings
HOOK
chmod +x "$hook_path"
```

The helper allows only `.march/code-review-log.md` and
`.march/validation-profiles.toml` under `.march/`, and it permits staged
deletions so cleanup commits can remove old March artifacts from tracking.

### Timing Method

Measured on beelink on 2026-04-13 with `/usr/bin/time -p` using warm-cache
steady-state runs. Each command was run once untimed to warm build artifacts and
the shared `.cache` directory, then once with timing enabled.

| Command | Before | After |
|---|---|---|
| `make test` | `54.48s` | `12.32s` |
| `make spec-pr` | `356.22s` | `135.55s` |
| `make check` | `55.15s` | `19.66s` |

The first serial `make spec-pr` warm pass hit two 60s timeouts in
`spec/18-source-labels.md`; the baseline above records the subsequent warmed
steady-state rerun.
