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

## Contract quick reference

spec/*.md
make lint
make test
make spec
rust-standards
python-standards
cli-design
mustmatch
testing-mindset

## Gates

Run the standard gates directly:

```
make lint
make test
make spec
```

There is intentionally no `make check` convention. `make lint` includes the
quality ratchet, `make test` includes both Rust and Python contract tests, and
`make spec` is the routine mustmatch contract gate.

Routine gates use `--no-default-features` so lint, test, and spec share one
small Cargo graph; that graph does not exercise AlphaGenome. Run `make
full-feature-check` for all shipped features and the AlphaGenome behavior tests.
`make release-gate` includes both lanes.

## Skill rail

Use this hybrid repo rail for dispatched work:

- rust-standards
- python-standards
- cli-design
- mustmatch
- testing-mindset

## Hygiene

Do not commit secrets, PHI, absolute local paths, planning notes, or March
runtime artifacts. Keep `.march/` runtime state, `.march-runtime/`, local caches,
and generated build outputs out of git.
