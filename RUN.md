# BioMCP Runbook

## What This Runbook Covers

This is the exact operator guide for the merged-main release binary. For the
shared target, owned artifacts, and promotion contract, see
`architecture/technical/staging-demo.md`.

## Prerequisites

- Rust toolchain with `cargo`
- `cargo-nextest` for repo-local `make test` and `make check`
- `cargo-deny` for the repo-local license and advisory policy checks in `make check`
- `uv` for repo-local pytest and spec flows
- `curl` for `scripts/contract-smoke.sh`

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
./target/release/biomcp get article 22663011 tldr   # requires S2_API_KEY
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

Run the heavier local ticket proofs explicitly:

```bash
make check
make release-gate
make test-contracts
```

The installed pre-commit hook is the fast local gate. It should run
`scripts/pre-commit-reject-march-artifacts.sh` before `cargo fmt --check` and
`cargo clippy --lib --tests -- -D warnings`. The March helper rejects staged
non-deletion `.march/*` paths outside the exhaustive allowlist:
`.march/code-review-log.md` and `.march/validation-profiles.toml`. The hook
does not run `cargo nextest run`, `make check`, `make spec-pr`, or
`make release-gate`, or `make test-contracts`.

Use `make check` for the canonical local gate; it runs the full Rust
lint/test/security/quality-ratchet lane and now includes `make test-contracts`,
so landing-copy, Python, and strict-docs regressions fail the same local gate.
Its `lint` phase runs both `cargo deny check licenses` and
`cargo deny check advisories`, and its `test` phase shells out to
`cargo nextest run`. Use `make release-gate` for the single release-readiness
signal; it runs `make check` followed by `make spec-pr`. Use `make spec-pr` for
the stable PR-blocking spec lane by itself; it runs `pytest-xdist` with
`-n auto --dist loadfile` for the parallel-safe bulk, then runs
`spec/05-drug.md`, `spec/13-study.md`, and
`spec/21-cross-entity-see-also.md` serially because those files share
repo-global local-data fixtures. Use `make spec-smoke` as a targeted local
rerun for the ten stable smoke section IDs in `SPEC_SMOKE_ARGS`; it resolves
those IDs to current mustmatch pytest item IDs at runtime, runs serially with a
120s mustmatch timeout, and is not a required PR gate. Use `make test-contracts`
to rerun just the Python/docs contract lane. Repo-root Ruff still runs through
`bin/lint`, but `pyproject.toml` excludes `architecture/experiments/**` so
scratch experiment scripts do not block the production Python lint gate. Use
`git commit --no-verify` to skip the hook for a one-off commit.

`make test-contracts` runs `cargo build --release --locked`, `uv sync --extra dev`, `pytest tests/ -v --mcp-cmd "./target/release/biomcp serve"`, and `mkdocs build --strict` - the same steps that PR CI `contracts` requires. `make check` now pulls that lane in transitively, while `make test-contracts` remains the direct rerun command when only the Python/docs contract lane needs another pass.

## Smoke Checks

```bash
BIOMCP_BIN=./target/release/biomcp ./scripts/genegpt-demo.sh
BIOMCP_BIN=./target/release/biomcp ./scripts/geneagent-demo.sh
./scripts/contract-smoke.sh --fast
# Optional keyed article proof:
./target/release/biomcp article citations 22663011 --limit 3
```

Use `architecture/technical/staging-demo.md` for the promotion contract and
`scripts/source-contracts.md` for the deeper source probe inventory.

## MCP Contract Verification

```bash
uv run pytest tests/test_mcp_contract.py -v --mcp-cmd "./target/release/biomcp serve"
uv run pytest tests/test_mcp_http_surface.py tests/test_mcp_http_transport.py -v
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8080/readyz
curl http://127.0.0.1:8080/
```

See `docs/reference/mcp-server.md` for the documented MCP surface.

## Spec Suite

```bash
make spec
make spec-smoke
```

`make spec` and `make spec-pr` use `pytest-xdist` with `-n auto --dist loadfile`
for the parallel-safe bulk, then run
`spec/05-drug.md`, `spec/13-study.md`, and `spec/21-cross-entity-see-also.md`
serially because those files share repo-global local-data fixtures. `make
spec-smoke` resolves the ten stable targeted smoke section IDs in
`SPEC_SMOKE_ARGS` to current mustmatch pytest item IDs, then runs them serially
with the longer 120s mustmatch timeout.
Use `spec/README-timings.md` as the current per-heading audit and the source of
truth for which headings stay smoke-only via `SPEC_PR_DESELECT_ARGS`; the
ratchet also checks that scanned `SPEC_SMOKE_ARGS` entries resolve to
collectable mustmatch pytest items.

When running repo-local checks through `uv run`, make sure `target/release` is
ahead of `.venv/bin` on `PATH` or refresh the editable install with
`uv pip install -e .` so `uv run` does not pick a stale `.venv/bin/biomcp`.
