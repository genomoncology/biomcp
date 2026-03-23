# BioMCP Technical Overview

## System Shape

BioMCP is a single Rust binary (`biomcp`) with two operating modes:

- **CLI mode:** Standard command-line invocation. Each command is a blocking
  async call that prints markdown to stdout and exits.
- **MCP server mode:** `biomcp serve` starts a JSON-RPC MCP server over stdio.
  Agents connect through the MCP protocol and call tools that mirror the CLI
  command surface.
- **HTTP mode:** `biomcp serve-http --host 0.0.0.0 --port 8080` starts the
  Streamable HTTP server. Remote MCP traffic uses `/mcp`, and lightweight
  probes live at `/health`, `/readyz`, and `/`. This is the canonical scaling
  answer when rate limiting needs to be shared across concurrent agent workers,
  since rate limiting is otherwise process-local.

The binary is also distributed as `biomcp-cli` on PyPI (a thin Python wrapper
that ships the platform-specific Rust binary). Python is packaging only;
no Python logic is involved in query processing.

## Build and Packaging

```
cargo build --release --locked   # Rust binary
uv build / uv publish            # PyPI wheel (biomcp-cli)
curl ... install.sh | bash       # binary installer (resolves latest release)
```

- **Edition:** Rust 2024
- **Current version:** 0.8.17 (as of 2026-03-23)
- **Package name:** `biomcp-cli` on PyPI; binary name is `biomcp`
- **PyPI publishing:** GitHub Actions trusted publisher (no token needed)
- **Release checklist:** Bump `Cargo.toml` and `pyproject.toml`, update
  `CHANGELOG.md`, verify version sync, then cut a GitHub release tag — the
  release workflow builds and publishes

## Source Integration Patterns

BioMCP integrates with 15+ upstream APIs. Integration patterns:

| Pattern | Examples |
|---------|---------|
| REST JSON | UniProt, ChEMBL, InterPro, ClinicalTrials.gov, cBioPortal, OncoKB, OpenFDA |
| GraphQL | gnomAD, OpenTargets, CIViC, DGIdb |
| Custom REST JSON | MyGene.info, MyVariant.info, MyChem.info, PubMed/PubTator3, Reactome, g:Profiler |
| Flat-file / XML REST | KEGG (plain-text flat-file / TSV-like responses), HPA (XML) |

All queries are read-only. BioMCP never writes to upstream systems.
Shared HTTP-client reuse is preferred but not universal: source modules may
reuse the shared middleware client or use a source-specific request path when
timeout, retry, caching, request-construction, or transport needs differ.
These transport differences are architectural, not implementation accidents.

Federated queries (e.g., `search all`, unified article search) fan out in
parallel across sources and merge results. Federated totals are approximate
due to cross-source deduplication — `total=None` is the correct design for
federated counts.

See also: [Source integration architecture](source-integration.md) for the
detailed contract for adding a new upstream source or deepening an existing
integration.

## API Keys

Most commands work without credentials. Optional keys improve rate limits or
unlock additional data:

| Key | Source | Effect |
|-----|--------|--------|
| `NCBI_API_KEY` | PubTator3, PMC OA, NCBI ID converter | Higher rate limits |
| `S2_API_KEY` | Semantic Scholar article enrichment/navigation | Optional TLDR, citation graph, and recommendation helpers at 1 req/sec |
| `OPENFDA_API_KEY` | OpenFDA | Higher rate limits |
| `NCI_API_KEY` | NCI CTS trial search (`--source nci`) | Required for NCI source |
| `ONCOKB_TOKEN` | OncoKB production API | Full clinical data (demo available without) |
| `ALPHAGENOME_API_KEY` | AlphaGenome variant effect prediction | Required for AlphaGenome |

For demo and offline workflows: `BIOMCP_CACHE_MODE=infinite` enables infinite
cache mode, replaying prior responses without hitting upstream APIs.

## Rate Limiting

Rate limiting is process-local. Multiple concurrent CLI invocations or MCP
server workers do NOT share a limiter. For deployments with many concurrent
agent workers, run a single shared `biomcp serve-http` endpoint so all workers
share one limiter budget and one Streamable HTTP `/mcp` surface.

## Release Pipeline

1. Update version in `Cargo.toml`, `pyproject.toml`, and `CHANGELOG.md`
2. Commit and push to `main`
3. Cut a GitHub release with a semver tag
4. GitHub Actions validates and publishes:
   - CI (`.github/workflows/ci.yml`) runs five parallel jobs: `check` (`cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`), `version-sync` (`bash scripts/check-version-sync.sh`), `climb-hygiene` (`bash scripts/check-no-climb-tracked.sh`), and `contracts` (`cargo build --release --locked`, `uv sync --extra dev`, `uv run pytest tests/ -v --mcp-cmd "./target/release/biomcp serve"`, `uv run mkdocs build --strict`), and `spec-stable` (`cargo build --release --locked`, then `make spec-pr`).
   - Volatile live-network headings run separately in `.github/workflows/spec-smoke.yml`,
     which runs the full `make spec` suite on a schedule and by manual dispatch.
   - Release validation runs the Rust checks again, then
     `uv run pytest tests/ -v --mcp-cmd "biomcp serve"` and
     `uv run mkdocs build --strict`.
   - release validation runs `pytest tests/` and `mkdocs build --strict`.
   - Release build jobs package cross-platform binaries, publish PyPI wheels,
     and deploy docs.
5. `install.sh` resolves the latest tagged release with downloadable assets

Known issue: `uv sync --extra dev` may rewrite the editable root package
version in `uv.lock` during a release cut. Verify whether the lockfile
version bump should ship with the release commit.

## Verification Approach

BioMCP has six distinct verification and operator-inspection surfaces.

### 1. CI and Repo Gates

- `make check` is the required local ticket gate. In the current `Makefile`,
  that means `lint` plus `test`.
- CI in `.github/workflows/ci.yml` runs the broader repo baseline in parallel:
  `check`, `version-sync`, `climb-hygiene`, `contracts`, and `spec-stable`.
- Docs-site validation and Python contract tests do not run under `make check`;
  they live in `make test-contracts` and the CI `contracts` job.
- The grounding implementation surfaces for this split are `Makefile`,
  `.github/workflows/ci.yml`, and `.github/workflows/contracts.yml`.

### 2. Spec Suite (`spec/`)

BDD executable documentation written as `mustmatch` spec files. The suite
exercises CLI output at the command level using stable structural markers
(headers, table columns, query echoes) rather than brittle upstream data
values.

PR CI runs `make spec-pr` via the `spec-stable` job in
`.github/workflows/ci.yml`. That job builds the release binary first, then
relies on the Makefile's `target/release`-first `PATH` handling so specs do
not accidentally execute a stale `.venv/bin/biomcp`. Volatile live-network
headings run in the separate `Spec smoke (volatile live-network)` workflow
instead.

PR CI now runs `make spec-pr` via the `spec-stable` job in `.github/workflows/ci.yml`.
Volatile live-network headings run separately in `.github/workflows/spec-smoke.yml`.

Run locally with `make spec`.

Important: `uv run` may execute a stale `.venv/bin/biomcp`. Either refresh
with `uv pip install -e .` or ensure `target/release` is ahead of `.venv/bin`
when running CLI specs.

### 3. `biomcp health`

`biomcp health` is a curated operator inspection surface, not a full source
inventory ledger.

- The command is grounded in `src/cli/health.rs`.
- It shows per-source connectivity for readiness-significant sources.
- Key-gated sources appear as `excluded` rows when the required environment
  variable is absent.
- `--apis-only` omits the cache-writability row.
- Partial upstream failures remain visible in the rendered report.
- Current CLI behavior is report-first: the command exits `0` when the report
  renders, even if some upstream rows are failing.

### 4. Contract Smoke Checks (`scripts/contract-smoke.sh`)

`scripts/contract-smoke.sh` is an optional live probe runner for a selected set
of stable public endpoints, not a universal ledger for every integrated source.

- Many covered sources use happy / edge / invalid trios.
- Coverage is selective and operationally curated.
- Secret-gated, volatile, or otherwise unsuitable providers may be skipped or
  reduced.
- The grounding implementation surfaces are `scripts/contract-smoke.sh`,
  `scripts/README.md`, and `.github/workflows/contracts.yml`.

Contract smoke checks run in `.github/workflows/contracts.yml`.

Run: `./scripts/contract-smoke.sh` from the repo root.

### 5. Demo Scripts (`scripts/genegpt-demo.sh`, `scripts/geneagent-demo.sh`)

End-to-end demo flows that reproduce paper-style GeneGPT and GeneAgent
workflows. These scripts:
- Run live against the default binary
- Assert on JSON field presence (not exact values)
- Compute a scoring metric (evidence score for GeneGPT, drug count for GeneAgent)
- Exit non-zero on any assertion failure

These are the canonical smoke checks for a working release.

### 6. Remote HTTP Demo Artifact (`demo/streamable_http_client.py`)

Release verification for the Streamable HTTP surface also includes the
standalone Streamable HTTP demo client
(`demo/streamable_http_client.py`). Run `biomcp serve-http`, then execute:

```bash
uv run --script demo/streamable_http_client.py
```

The demo initializes against `/mcp` and prints `Command:` framing before a
three-step discovery -> evidence -> melanoma trials workflow through the remote
`biomcp` tool:

- `biomcp search all --gene BRAF --disease melanoma --counts-only`
- `biomcp get variant "BRAF V600E" clinvar`
- `biomcp search trial -c melanoma --mutation "BRAF V600E" --limit 5`

Expected structural output includes the connection line and `Command:` markers
so the remote run remains readable in screenshots and recorded demos without
replacing the real BioMCP markdown output.

## Known Constraints

- Rate limiting is process-local (see above)
- Semantic Scholar article helpers are explicitly limited to 1 request/sec per process and are not part of article search fan-out
- Federated totals are approximate
- Some sources (OncoKB production, NCI CTS, AlphaGenome) require API keys
- OncoKB demo endpoint has a known no-hit response for some variants — this
  is expected behavior, not a bug
- PubTator coerces small `size` parameters — use fixed internal page sizes
  (25) to avoid offset drift in pagination
- ClinicalTrials.gov mutation discovery cannot rely on `EligibilityCriteria`
  alone; search mutation-related title, summary, and keyword fields too

## Operator Notes

Runtime operator docs now live in `design/technical/staging-demo.md` and
`RUN.md`. Use those documents for the shared target, promotion contract, and
exact release-binary run/smoke commands, then use `scripts/` for the source
probe inventory and demo helpers.
