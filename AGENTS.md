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

Run affected tests during development. Run one Rust build at a time on a build host.

There is intentionally no `make check` convention. `make lint` includes the
quality ratchet, `make test` includes both Rust and Python contract tests, and
`make spec` is the routine mustmatch contract gate. Routine gates use
`--no-default-features` so lint, test, and spec share one small Cargo graph;
that graph does not exercise AlphaGenome. Run `make full-feature-check` for all
shipped features and the AlphaGenome behavior tests, and `make release-gate`
for the release checks. `make landing-gate` runs lint, routine tests and
specifications in order. `make main-gate` runs that standard gate and then
`make full-feature-check`. Routine lint includes test targets. Long, load,
coverage and release-only packaging checks stay outside per-landing gates.

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

Every release works the checklist at `sdlc/release-checklist.md`, in order, with evidence for each item. Ian cuts every release after local checks and the full nonpublishing rehearsal pass; the tagged release workflow must also pass.

## Hygiene

Do not commit secrets, PHI, absolute local paths, planning notes, or local
runtime state. Keep local caches and generated build outputs out of git.

## Landing changes

One coordinator lands one ticket at a time through `pm ticket land`. The configured `ticketsGate` runs `make landing-gate` through the build job on the reviewed merge candidate. Reuse a passing gate while its inputs stay unchanged. After merging, run `make main-gate` through the build job on the actual pushed main revision before another ordinary landing.

A failed or absent check for current main holds ordinary landings. Only a reviewed repair or backout may land during that hold; check the resulting main before resuming. On resume, inspect current main's exact-revision job evidence before landing another ticket.

Everyday builds and tests use build jobs. GitHub Actions run for releases after local checks pass, and retain manual dispatch for explicit checks. The release workflow must qualify its candidate directly; a prior CI run on main does not qualify it. Rehearse the full release workflow without publishing before Ian tags a release.

## Source licensing

State each source's terms plainly in its registry entry and on the licensing page. Label restricted terms as restricted. Users are responsible for their own licenses. BioMCP never buys a license or contacts a provider without Ian's approval.
