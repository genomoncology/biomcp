# AGENTS.md — BioMCP

BioMCP is a Rust CLI and MCP server with a Python docs/contract harness. The
Rust runtime lives under `src/`; Python tests, docs checks, and executable
contracts exercise the shipped CLI/MCP surface.

## Packaged agent guide

BioMCP is a biomedical CLI and MCP server. For example, search from the CLI
with `biomcp search articles --gene BRAF`.

Installed packages place agent skills in `share/biomcp/skills/`. Packaged
Markdown documentation is available locally and on the canonical site at
https://biomcp.org/:

| Topic | Packaged documentation | Website |
| --- | --- | --- |
| Blog | `share/biomcp/docs/blog/` | https://biomcp.org/blog/ |
| Charts | `share/biomcp/docs/charts/` | https://biomcp.org/charts/ |
| Concepts | `share/biomcp/docs/concepts/` | https://biomcp.org/concepts/ |
| Getting started | `share/biomcp/docs/getting-started/` | https://biomcp.org/getting-started/ |
| How-to | `share/biomcp/docs/how-to/` | https://biomcp.org/how-to/ |
| Reference | `share/biomcp/docs/reference/` | https://biomcp.org/reference/ |
| Sources | `share/biomcp/docs/sources/` | https://biomcp.org/sources/ |
| User guide | `share/biomcp/docs/user-guide/` | https://biomcp.org/user-guide/ |

## Repository contract shape

- Behavioral contracts are mustmatch executable docs at:

  ```
  spec/*.md
  ```

  Run them with `make spec`. The spec gates use the standalone `mustmatch test`
  binary through `scripts/run-specs.sh`. Cargo artifact preparation and fixture
  standup belong in the runner and `scripts/prepare-spec-artifacts.py`, not in
  ad hoc docs. Specification pages execute prepared paths and must not invoke
  `cargo run`, `cargo build`, `cargo rustc`, `cargo test`, `cargo tree`, or
  `cargo metadata`.
- Unit/static layers run with `make test`: Rust nextest first, then the Python
  CLI/MCP/docs contract lane.

## Working

While you work, run only the tests for what changed, with incremental
compilation on for the whole edit loop:

```
CARGO_INCREMENTAL=1 cargo nextest run --no-default-features -E 'test(...)'
```

Keep `CARGO_INCREMENTAL` set for the whole loop and unset it for the gates:
toggling the setting recompiles the crate.

Before landing, run `make lint`, `make test` and `make spec` once on the final
branch state, and rerun a gate only when the code changed since that run; CI
runs them again. A test that takes more than a few seconds is a defect to fix,
not a cost to accept. Check live behavior with one pass of the built binary
over the names under test. Run one Rust build at a time on a machine.

There is intentionally no `make check` convention. `make lint` includes the
quality ratchet, `make test` includes both Rust and Python contract tests, and
`make spec` is the routine mustmatch contract gate. Routine gates use
`--no-default-features` so lint, test, and spec share one small Cargo graph;
that graph does not exercise AlphaGenome. Run `make full-feature-check` for all
shipped features and the AlphaGenome behavior tests, and `make release-gate`
for both lanes.

## Releases

A ticket merged for a release carries its changelog bullet in its own landing, so the release gate never finds a ticket without one.

`docs/reference/release-process.md` is the release runbook of record. A
published release builds five platform archives, publishes PyPI wheels through
the protected `pypi` environment, updates the Homebrew tap, and publishes
`ghcr.io/genomoncology/biomcp` for `linux/amd64` and `linux/arm64` with the
`<version>` and `latest` tags; `latest` moves only when the tag is the
repository's latest release and both platform smokes pass. A manual
`workflow_dispatch` with
`container_only: true` republishes only the container image for an already
published tag. Keep container publication in `.github/workflows/release.yml`;
`spec/surface/docker-image.md` and `tests/test_release_workflow_provenance.py`
enforce it.

Every release works the checklist at `sdlc/release-checklist.md`, in order, with evidence for each item. Ian cuts every release; the release bar is the tagged commit's green run.

## Hygiene

Do not commit secrets, PHI, absolute local paths, planning notes, or local
runtime state. Keep local caches and generated build outputs out of git.

## CI and merging

Push every change to a `ticket/NNNN-slug` branch first. CI runs on ticket branches; merge to main only after that branch's CI run has finished green. A started run does not count. No direct commits to main. A red main is fixed promptly, and it does not block a release by itself: the release bar is the tagged commit, which must have a finished green CI run and pass the release workflow. Ian ruled on both points on 2026-09-29.

CI skips the Rust jobs only when every file a push changes is documentation, an `sdlc/` file, or a `notes/` file; the classify job decides by the changed files, never by the commit message, and the repository-contract job still runs the docs and record tests plus the strict documentation build. Never rely on a commit-message prefix to skip CI.

## Source licensing

State each source's terms plainly in its registry entry and on the licensing page. Label restricted terms as restricted. Users are responsible for their own licenses. BioMCP never buys a license or contacts a provider without Ian's approval.
