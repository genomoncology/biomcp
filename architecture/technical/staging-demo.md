# BioMCP Staging and Demo Contract

## Shared Target

The shared merged-main target is `./target/release/biomcp`.

BioMCP staging/demo validation is local-binary based, not a long-lived shared
server. After merge, promotion and smoke work should run against the merged
release binary built from the repo root.

The PyPI package (`biomcp-cli`) and the install script are release/distribution
paths. They are not the shared target for merged-main promotion because they do
not guarantee the exact post-merge source state.

## Runtime Modes

- CLI mode: `./target/release/biomcp <command> [args]`
- MCP stdio mode: `./target/release/biomcp serve`
- Equivalent MCP alias: `biomcp mcp`
- Streamable HTTP mode: `./target/release/biomcp serve-http --host 127.0.0.1 --port 8080`
- Shared-network Streamable HTTP variant: `./target/release/biomcp serve-http --host 0.0.0.0 --port 8080`

`serve` is the canonical operator spelling because current client examples and
contract tests use it directly.

## Owned Artifacts and State

- Owned artifact: `./target/release/biomcp`
- Shared runtime does not require a seeded database, index, or generated demo
  corpus for baseline validation
- Baseline validation hits live upstream APIs
- Optional local replay aid: `BIOMCP_CACHE_MODE=infinite`
- Streamable HTTP owned endpoints: `POST/GET /mcp`, `GET /health`, `GET /readyz`, and `GET /` on the chosen bind address

Cache state is process-local and optional. It is not a shared artifact that
future tickets should treat as part of promotion.

## Canonical Smoke and Proof Contract

CLI/demo smoke:

```bash
BIOMCP_BIN=./target/release/biomcp ./scripts/genegpt-demo.sh
BIOMCP_BIN=./target/release/biomcp ./scripts/geneagent-demo.sh
```

Source contract smoke:

```bash
./scripts/contract-smoke.sh --fast
./scripts/contract-smoke.sh
```

MCP stdio proof:

```bash
uv sync --extra dev --no-install-project
uv run --no-sync pytest tests/test_mcp_contract.py -v --mcp-cmd "./target/release/biomcp serve"
```

Streamable HTTP proof:

1. Start `./target/release/biomcp serve-http --host 127.0.0.1 --port 8080`
2. Confirm `GET /health` returns `{"status":"ok"}`
3. Confirm `GET /readyz` returns `{"status":"ok"}`
4. Confirm `GET /` returns the BioMCP identity document
5. Confirm one MCP initialize request succeeds against `POST/GET /mcp`

Automated MCP contract coverage includes stdio plus the Streamable HTTP suites
in `tests/test_mcp_http_surface.py` and `tests/test_mcp_http_transport.py`.

## Promotion Steps

1. Refresh merged `main`
2. Run `cargo build --release --locked`
3. Run `BIOMCP_BIN=./target/release/biomcp ./scripts/genegpt-demo.sh`
4. Run `BIOMCP_BIN=./target/release/biomcp ./scripts/geneagent-demo.sh`
5. Run `./scripts/contract-smoke.sh --fast`
6. Run `./target/release/biomcp article citations 22663011 --limit 3` (with `S2_API_KEY` for authenticated quota, or without it to exercise the shared-pool path)
7. Run `uv sync --extra dev --no-install-project`, then `uv run --no-sync pytest tests/test_mcp_contract.py -v --mcp-cmd "./target/release/biomcp serve"` when work touches MCP-facing behavior, docs, or startup expectations

## Credentials and Environment Variables

Baseline smoke does not require credentials.

Optional runtime keys:

- `NCBI_API_KEY`
- `S2_API_KEY`
- `OPENFDA_API_KEY`
- `NCI_API_KEY`
- `ONCOKB_TOKEN`
- `ALPHAGENOME_API_KEY`

The canonical OncoKB production token name is `ONCOKB_TOKEN` across code,
docs, and scripts. `BIOMCP_CACHE_MODE=infinite` is a local replay aid, not a
shared deployment contract.

## Known Constraints

- Rate limiting is process-local
- Some sources are optional or key-gated
- OncoKB demo responses can be limited or empty for expected cases
- PubTator paging constraints still apply
- Broader technical context lives in `architecture/technical/overview.md`
