# BioMCP Runbook

## What This Runbook Covers

This is the exact operator guide for the merged-main release binary. For the
shared target, owned artifacts, and promotion contract, see
`architecture/technical/staging-demo.md`.

## Prerequisites

- Rust toolchain with `cargo`
- `cargo-nextest` for repo-local `make test`
- `cargo-deny` for the repo-local license and advisory policy checks in `make lint`
- `uv` for repo-local pytest and spec flows
- `curl` for `scripts/contract-smoke.sh`
- `bubblewrap` for public-network isolation in routine `make test` and `make spec`

Install the Rust helper tools with:

```bash
cargo install cargo-nextest --locked
cargo install cargo-deny --locked
```

## Build The Shared Target

```bash
cargo build --release --locked
```

The shared target path is `./target/release/biomcp`.

## Run: CLI Mode

```bash
./target/release/biomcp health --apis-only
./target/release/biomcp get gene BRAF
./target/release/biomcp get article 22663011 tldr   # anonymous works; S2_API_KEY raises quota
```

Use `docs/user-guide/cli-reference.md` for the full command grammar and entity
surface.

## Run: MCP Stdio Mode

```bash
./target/release/biomcp serve
```

Minimal client configuration:

```json
{
  "mcpServers": {
    "biomcp": {
      "command": "./target/release/biomcp",
      "args": ["serve"]
    }
  }
}
```

`serve` is the canonical operator spelling and is equivalent to `biomcp mcp`.

## Run: Streamable HTTP Mode

```bash
./target/release/biomcp serve-http --host 127.0.0.1 --port 8080
```

This serves MCP over Streamable HTTP at `/mcp`. Use `--host 0.0.0.0` only when
the endpoint must be reachable from other machines or containers on the network.

Owned routes:

- `POST/GET /mcp`
- `GET /health`
- `GET /readyz`
- `GET /`

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `BIOMCP_CACHE_MODE` | Set `infinite` to replay cached responses locally |
| `NCBI_API_KEY` | Higher rate limits for PubTator3, PubMed/efetch, PMC OA, and NCBI helpers |
| `S2_API_KEY` | Optional Semantic Scholar TLDR, citation graph, and recommendations |
| `OPENFDA_API_KEY` | Higher OpenFDA rate limits |
| `NCI_API_KEY` | Required for NCI CTS trial queries |
| `ONCOKB_TOKEN` | Canonical OncoKB production token |
| `ALPHAGENOME_API_KEY` | Required for AlphaGenome variant prediction |

## Pre-Merge Checks

Normal builds do not run or require `protoc`. The AlphaGenome client consumes
committed generated Rust bytes. When its protobuf inputs deliberately change,
a maintainer installs pinned `protoc` 28.3, runs
`scripts/regenerate-alphagenome-proto`, and reviews the one generated-file
diff. `scripts/regenerate-alphagenome-proto --check` is the non-writing CI
proof.

Run the heavier local ticket proofs explicitly:

```bash
make lint               # repo lint plus quality ratchet
make test               # Rust nextest plus Python/docs contract lane
make spec               # offline deterministic routine spec gate
make release-gate       # routine gates + full-feature proof + release specs
make verify             # opt-in live public-upstream confidence
make test-contracts     # rerun just Python/docs contract lane
```

The installed pre-commit hook is the fast local gate. It should run
`scripts/pre-commit-reject-march-artifacts.sh` before `cargo fmt --check` and
`cargo clippy --no-default-features --lib --tests -- -D warnings`. The March helper rejects staged
non-deletion `.march/*` paths outside the exhaustive allowlist:
`.march/code-review-log.md`. The hook does not run `cargo nextest run`,
`make lint`, `make test`, `make spec`,
`make spec-pr`, `make release-gate`, or `make test-contracts`.

Use `make lint`, `make test`, and `make spec` for the canonical local gates.
Routine gates use `--no-default-features`, so Clippy, nextest, and spec
preparation reuse one small Cargo graph and do not exercise AlphaGenome. Run
`make full-feature-check` to lint all targets with all shipped features, run
the AlphaGenome behavior tests, and build the all-feature release CLI. The
release gate includes this full-feature proof after the routine lint and test
lanes and before release-profile specs.
`make lint` runs the repo lint script, `cargo deny check licenses`,
`cargo deny check advisories`, and the quality ratchet. `make test` runs
`cargo nextest run` plus the Python/docs contract lane, so landing-copy,
Python, and strict-docs regressions fail the same local test gate. Python
contracts use four bounded file-distributed workers; set `PYTEST_WORKERS=1`
for a one-worker diagnostic run. Use
`make release-gate` for the single release-readiness signal; it runs routine
lint and test, the named full-feature proof, then specs against the all-feature
release binary. There is no supported `make check` command. Use
`make verify` only as an explicit opt-in live public-upstream confidence lane;
`make release-live-smoke` is a compatibility alias for that operator lane.
`make spec-pr` remains available for the offline executable-spec corpus by
itself; it runs explicit local/fixture-backed `SPEC_ROUTINE_PATHS` through
`scripts/run-specs.sh` with `mustmatch test` and `--lang bash` plus the longer timeout.
`make spec` runs the same offline path set with the shorter local timeout and
should pass with external network blocked.

On Linux, `make test` and `make spec` enforce that claim with Bubblewrap. They
finish dependency setup and compilation before entering a network namespace,
then prove public DNS and direct public TCP are unavailable while loopback and
Unix sockets still work. A missing or unusable Bubblewrap installation fails
closed. Non-Linux machines cannot provide the authoritative isolation result;
use the canonical Linux CI job. `make verify` intentionally stays outside this
boundary because it is the opt-in live-upstream lane.

The executable docs do not hand-roll env setup inside bash blocks anymore.
`scripts/run-specs.sh` owns one explicit artifact-preparation phase, fixture
standup, binary-runner routing, and the standalone mustmatch PATH guard.
`scripts/prepare-spec-artifacts.py` builds the feature-off CLI and MCP helper
once for routine specs, captures Cargo tree and package metadata once, and
passes stable paths to every page. Live verification additionally prepares a
distinct feature-on CLI and compiles Rust test executables once with
`cargo test --no-run`; pages execute those files directly. No spec page or
fixture helper may invoke a build-inducing Cargo command. `tools/biomcp-ci`
remains the command wrapper:
it resolves the repo root from
its own path, points `BIOMCP_CACHE_DIR` and `XDG_*` under
`.cache/biomcp-specs/`, defaults `RUST_LOG=error`, unsets optional auth keys,
and only forces `BIOMCP_CACHE_MODE=infinite` when CI restored a warm cache and
exported `BIOMCP_SPEC_CACHE_HIT=1`. Cold runs leave `BIOMCP_CACHE_MODE`
untouched so the shared cache can refill naturally. Use `make test-contracts`
to rerun just the Python/docs contract lane. Repo-root Ruff still runs through
`bin/lint`, but `pyproject.toml` excludes `architecture/experiments/**` so
scratch experiment scripts do not block the production Python lint gate. Use
`git commit --no-verify` to skip the hook for a one-off commit.

`make test-contracts` builds the selected contract profile, then runs `uv sync --extra dev --no-install-project`, the complete Python corpus with four bounded file-distributed workers, and `uv run --no-sync mkdocs build --strict` with its absolute binary path in `BIOMCP_BIN`. `make spec` delegates compilation only to its preparation phase and uses stable copies under `.cache/spec-artifacts/`; `make release-gate` passes its already-built release CLI into that phase. The `--no-install-project`/`--no-sync` split is intentional: Python/docs lanes install only Python dev tooling and exercise the selected binary instead of rebuilding the maturin package into `.venv`. CI calls this same target with its release binary instead of maintaining a second Python/docs command sequence. `make test-contracts` remains the direct rerun command when only the Python/docs contract lane needs another pass.

## Smoke Checks

```bash
BIOMCP_BIN=./target/release/biomcp ./scripts/genegpt-demo.sh
BIOMCP_BIN=./target/release/biomcp ./scripts/geneagent-demo.sh
./scripts/contract-smoke.sh --fast
# Optional keyed article proof:
./target/release/biomcp article citations 22663011 --limit 3
```

Use `architecture/technical/staging-demo.md` for the promotion contract and
`scripts/source-contracts.md` for the deeper source probe inventory. The GitHub
Release workflow hard-runs `make spec`, `scripts/contract-smoke.sh`, and
`scripts/release-smoke.sh` in its `validate` job before publishing assets; the
contract smoke workflow is manual-only and no longer runs on a daily schedule.

## MCP Contract Verification

The protocol-level stdio and Streamable HTTP assertions live in `tests/rmcp_client_contract.rs`; route-only HTTP checks live in `tests/test_mcp_http_surface.py`.

```bash
cargo nextest run --test rmcp_client_contract
uv run --no-sync pytest tests/test_mcp_http_surface.py -v
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8080/readyz
curl http://127.0.0.1:8080/
```

See `docs/reference/mcp-server.md` for the documented MCP surface.

## Spec Suite

```bash
make spec               # offline deterministic routine spec gate
make spec-contracts
make verify             # opt-in live public-upstream confidence
make release-live-smoke # compatibility alias for make verify
make spec-pr
```

`make spec` is the offline deterministic routine spec gate. `make
spec-contracts` is a deterministic legacy subset kept for profile compatibility;
`make release-gate` runs the routine gates, full-feature proof, and release-profile `make spec` gate. `make verify` is the explicit opt-in live lane for
discover/OLS4, disease, article source-status, variant-normalization,
phenotype, protein, pathway, and the other public-upstream specs through
`tools/biomcp-ci`; `make release-live-smoke` delegates to `make verify` for old
operator muscle memory.

`make spec` and `make spec-pr` both run explicit `SPEC_ROUTINE_PATHS`: the
Markdown subset (`spec/entity/article.md`, `spec/entity/study.md`,
`spec/entity/variant.md`, and `spec/surface/mcp.md`) through `mustmatch test`,
plus the lone `tests/surface/test_parallel_isolation_contract.py` pytest canary
that guards disease/discover isolation. `make spec-contracts` stays a Markdown
subset for profile compatibility.
Live-upstream specs such as `spec/entity/phenotype.md`, `spec/entity/protein.md`,
`spec/entity/disease.md`, `spec/surface/discover.md`, `spec/entity/pathway.md`,
and `spec/surface/cli.md` run only in `make verify`. Every bash block in those
lanes should call `tools/biomcp-ci`, which owns release-binary resolution,
repo-owned cache roots, optional-key stripping, and warm-cache replay on CI
cache hits; `scripts/run-specs.sh` invokes the Markdown files with the
standalone `mustmatch test` binary.

The path arrays in `scripts/run-specs.sh` are the operator inventory. The
matching `SPEC_ROUTINE_PATHS`, `SPEC_STATIC_PATHS`, and `SPEC_LIVE_PATHS` lists
in `Makefile` are checked by the contract suite. Add a page to both inventories
and declare any new Rust executable in `scripts/prepare-spec-artifacts.py`;
pages consume only the exported prepared path.

Use `spec/README-timings.md` as the current validation-lane audit/reference for
the offline deterministic routine lane, the opt-in live verify lane, the active
canary corpus, the wrapper/cache contract, and warm-cache expectations.

When running repo-local Python/docs/spec checks through `uv`, use
`uv sync --extra dev --no-install-project` followed by `uv run --no-sync ...`.
Keep `target/release` ahead of `.venv/bin` on `PATH` and pass
`BIOMCP_BIN=./target/release/biomcp` when invoking executable specs manually.
Do not use `uv run --extra dev ...` for Python-only gate lanes: that asks uv to
install the maturin-backed current project and can redundantly rebuild the Rust
CLI before pytest or mkdocs starts.
