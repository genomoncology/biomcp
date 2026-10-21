# AGENTS.md — BioMCP

This repository is public and talks only about itself.

BioMCP is a Rust CLI and MCP server with a Python docs/contract harness. The
Rust runtime lives under `src/`; Python tests, docs checks, and executable
contracts exercise the shipped CLI/MCP surface.

## Packaged agent guide

BioMCP is a biomedical CLI and MCP server. For example, search from the CLI
with `biomcp search articles --gene BRAF`.

Installed packages place agent skills in `share/biomcp/skills/`. Packaged
Markdown documentation is available locally and on the canonical site at
https://biomcp.org/:

| Topic           | Packaged documentation               | Website                             |
| --------------- | ------------------------------------ | ----------------------------------- |
| Blog            | `share/biomcp/docs/blog/`            | https://biomcp.org/blog/            |
| Charts          | `share/biomcp/docs/charts/`          | https://biomcp.org/charts/          |
| Concepts        | `share/biomcp/docs/concepts/`        | https://biomcp.org/concepts/        |
| Getting started | `share/biomcp/docs/getting-started/` | https://biomcp.org/getting-started/ |
| How-to          | `share/biomcp/docs/how-to/`          | https://biomcp.org/how-to/          |
| Reference       | `share/biomcp/docs/reference/`       | https://biomcp.org/reference/       |
| Sources         | `share/biomcp/docs/sources/`         | https://biomcp.org/sources/         |
| User guide      | `share/biomcp/docs/user-guide/`      | https://biomcp.org/user-guide/      |

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

- Unit/static layers run with `make test`: Rust nextest finishes before the Python
  CLI/MCP/docs contract lane starts, even with `make -j`.

## Working

Run affected tests on the configured build machine, leaving `CARGO_INCREMENTAL` unset so the configured compiler cache can run:

```
cargo nextest run --no-default-features -E 'test(...)'
```

The routine build job books 8 GiB disk and 7 GiB memory. The Makefile's `TEST_LANE_WORKERS` sets the shared Cargo, nextest and pytest worker allowance for that booking. Keep the allowance and booking together when measuring a different job shape. Run `make test TEST_LANE_WORKERS=1` for a smaller worker allowance.

Run affected tests during development. Run one Rust build at a time within a lane. Build-job admission schedules independent lanes in parallel.

For focused Python checks in a build job, run `make sync-python-dev` in that job's checkout. This target runs `uv sync --extra dev --no-install-project` in the checkout's own `.venv` and supplies pytest and the Ruff pin declared in the development dependencies. Run affected checks with `.venv/bin/python -m pytest tests/<affected-test>.py` and `.venv/bin/python -m ruff check <changed-python-files>`. Supply the prepared `BIOMCP_BIN` when checks need the CLI. These commands use the same job environment without syncing or triggering an editable Rust package build. Never borrow another checkout's environment. Existing Makefile Python commands use `uv run --no-sync` with that same local environment.

`make lint` prepares this environment before running Python lint. Direct `bin/lint` requires the prepared local environment and checks the declared Ruff version. `tools/bootstrap-lint-tools` supplies ShellCheck and actionlint.

tools/run-offline isolates the network only on Linux, so run any gate that must prove offline behavior on a Linux build host.

There is intentionally no `make check` convention. `make lint` includes the
quality ratchet, `make test` includes both Rust and Python contract tests, and
`make spec` is the routine mustmatch contract gate. Routine gates use
`--no-default-features` so lint, test, and spec share one small Cargo graph;
that graph does not exercise AlphaGenome. Run `make full-feature-check` for all
shipped features and the AlphaGenome behavior tests with the development
profile, including PNG, SVG and terminal output checks. `make release-gate`
requires the same full-feature build and output checks with the optimized
release profile before release-profile specifications. The final release
rehearsal also requires optimized artifacts and their output checks. `make landing-gate` runs lint, routine tests and
specifications in order. Landing lint skips live dependency advisories only
when the four Cargo and Python dependency inputs match a verified preceding
`origin/main`; missing or ambiguous comparison context runs advisories.
Standalone lint, full main checks and release qualification always run live
advisories. Required advisory checks run cargo-deny against RustSec and the
pinned official OSV-Scanner against both Cargo.lock and uv.lock. OSV adds
GitHub Security Advisories and Python package advisories. The advisory-only
bootstrap mode installs the verified upstream binary without a Go toolchain;
installation, parsing, network errors and advisory findings fail the gate.
The policy helper runs through `uv run --no-sync python` to use the repository's
pinned development interpreter without installing the Rust project.
The scanner configuration derives only the reviewed advisory IDs and reasons
from deny.toml; OSV applies each ID to its aliases. No separate exception list
or package-wide override is maintained. The release marker subset relies on the release workflow's full
qualification lint. License checks always run. `make main-gate` runs that standard gate and then
`make repo-audit` and `make full-feature-check`. The audit stage retains the
whole-repository source, capture, coupling and CLI surface checks; routine Python
keeps their focused malformed-input and wrapper checks. Release qualification
also runs this required audit stage. Routine lint includes test targets. Long, load,
coverage and release-only packaging checks stay outside per-landing gates.
Routine test runners warn with the test identity and machine load when a case
exceeds the threshold in their runner configuration; time alone never fails
or terminates a test. Assertions still fail. Exhaustive Rust cases use a
`long_suite_` test name and Python cases use `@pytest.mark.long`. Run these explicitly
with `make test-long` at release candidate; `make release-gate` includes them.
Live-service tests and subprocess helpers retain their existing ignored selections.

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

Land accepted tickets in batches through one lander, as the shared build coordinator skill (`skills/build-coordinator.md`) says. Run `make landing-gate` once on the batch candidate through the configured build job. Reuse that passing gate while its inputs stay unchanged. Do not repeat the gate after merging.

A failed batch gate holds that landing. A reviewed repair or backout must pass the candidate gate before publication. On resume, inspect the candidate's exact-revision gate evidence before landing it.

Run `make main-gate` once on the final candidate before the full nonpublishing release rehearsal until nightly qualification is scheduled. Nightly qualification will own the additional repository audit, all-feature checks, rendered output checks and installed Linux wheel smoke.

Everyday builds and tests use build jobs. GitHub Actions run for releases after local checks pass, and retain manual dispatch for explicit checks. The release workflow must qualify its candidate directly; a prior CI run on main does not qualify it. Rehearse the full release workflow without publishing before Ian tags a release.

## Source licensing

State each source's terms plainly in its registry entry and on the licensing page. Label restricted terms as restricted. Users are responsible for their own licenses. BioMCP never buys a license or contacts a provider without Ian's approval.
