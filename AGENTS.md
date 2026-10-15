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

Keep incremental compilation on for the whole edit loop:

```
CARGO_INCREMENTAL=1 cargo nextest run --no-default-features -E 'test(...)'
```

Keep `CARGO_INCREMENTAL` set for the whole loop and unset it for the gates:
toggling the setting recompiles the crate.

A test that takes more than a few seconds is a defect to fix, not a cost to
accept. Run one Rust build at a time on a machine.

There is intentionally no `make check` convention. `make lint` includes the
quality ratchet, `make test` includes both Rust and Python contract tests, and
`make spec` is the routine mustmatch contract gate. Routine gates use
`--no-default-features` so lint, test, and spec share one small Cargo graph;
that graph does not exercise AlphaGenome. Run `make full-feature-check` for all
shipped features and the AlphaGenome behavior tests, and `make release-gate`
for both lanes.

## BioData 1.0 migration verification

Run the repository-owned focused migration checks on M5 at the reviewed revision and exact BioData pin. Record affected behavior checks and inherited findings. Do not dispatch hosted verification for the 1.0 migration. Live-provider smoke remains separate. Keep routine checks focused on the changed behavior.

BioMCP ticket numbers use one global repository sequence. Create every new ticket with `pm ticket new` and keep its allocated number. This checkout declares only active milestone `1.0` targeting `biodata/biomcp-1.0`. Use component `backend` for integration work. Never infer a release from a ticket number or reserve a branch-specific range.

## Releases

A ticket merged for a release carries its changelog bullet in its own landing, so the release gate never finds a ticket without one. Keep the New features, Fixes and Internal headings of the release under way in place even when a section is empty, so no landing adds or moves a heading.

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

A red main is fixed promptly, and it does not block a release by itself: the release bar is the tagged commit, which must have a finished green CI run and pass the release workflow. Ian ruled on this on 2026-09-29.

CI skips the Rust jobs only when every file a push changes is documentation, an `sdlc/` file, or a `notes/` file; the classify job decides by the changed files, never by the commit message, and the repository-contract job still runs the docs and record tests plus the strict documentation build. Never rely on a commit-message prefix to skip CI.

## Source licensing

State each source's terms plainly in its registry entry and on the licensing page. Label restricted terms as restricted. Users are responsible for their own licenses. BioMCP never buys a license or contacts a provider without Ian's approval.

## Landing a ticket into the 1.0 line

The line is a branch, not main, so pm's usual landing commands refuse it. Land
in three steps: fast-forward the line to the reviewed ticket-branch tip with a
direct push (`git push origin <reviewed-tip>:refs/heads/<the 1.0 branch>`),
commit the ticket's build record on the line (record-only landing demands the
record ride the line), then run `pm ticket land NNNN --record-only` from a
checkout sitting ON a local branch of the line at its tip. Review verdicts ride
the ticket's own branch as usual.
