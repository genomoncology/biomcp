# BioMCP Technical Overview

## System Shape

BioMCP is a single Rust binary (`biomcp`) with three operating modes:

- **CLI mode:** Standard command-line invocation. Each command is a blocking
  async call that prints markdown to stdout and exits.
- **MCP server mode:** `biomcp serve` starts a JSON-RPC MCP server over stdio.
  The advertised MCP tool is `biomcp`, and `src/mcp/shell.rs` enforces a
  read-only allowlist rather than mirroring the full CLI: `search`, `get`,
  helper families (`gene`, `variant`, `drug`, `disease`, `article`,
  `pathway`, `protein`), `list`, `version`, `health`, `batch`, `enrich`,
  `suggest`, `discover`, read-only `skill` lookup/list/render behavior, and MCP-safe `study`
  subcommands (`list`, `download --list`, `top-mutated`, `query`, `filter`,
  `cohort`, `survival`, `compare`, `co-occurrence`) are allowed.
  Operator-local or mutating commands such as `cache`, `update`, `serve`,
  `serve-http`, and `skill install` stay blocked over MCP.
  See `src/mcp/shell.rs` and `spec/15-mcp-runtime.md` for the canonical
  boundary.
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
- **Current version:** see `Cargo.toml` (`scripts/check-version-sync.sh` keeps
  `Cargo.toml`, `Cargo.lock`, `pyproject.toml`, `manifest.json`, and
  `CITATION.cff` aligned)
- **Package name:** `biomcp-cli` on PyPI; binary name is `biomcp`
- **PyPI publishing:** GitHub Actions trusted publisher (no token needed)
- **Release checklist:** Bump `Cargo.toml`, `Cargo.lock`, `pyproject.toml`,
  `manifest.json`, and `CITATION.cff`, update `CHANGELOG.md`, verify version
  sync, then cut a GitHub release tag — the release workflow builds and
  publishes

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

## Article Federation and Front-Door Validation

`search article --source all` plans PubTator3 plus Europe PMC plus PubMed.
Keyword-bearing queries also add LitSense2, and Semantic Scholar remains an
optional compatible search leg on that path. Strict Europe PMC-only filters
such as `--open-access` and `--type` disable the federated planner and route
to Europe PMC only.
`--source pubtator` with strict Europe PMC-only filters is rejected at the
front door. `--source` remains
`all|pubtator|europepmc|pubmed|litsense2` in v1; the CLI does not expose a
user-facing `--source semanticscholar` mode.

Article filters remain raw as the shared contract for planning, ranking,
rendering, JSON metadata, and session loop-breaker state. At the provider
boundary, direct and compatible federated PubMed ESearch cleans bounded
question-format filler words from unfielded gene, disease, drug, and keyword
clauses. PubTator3, Europe PMC, LitSense2, and Semantic Scholar receive their
existing query inputs.

After fetch, article results deduplicate across PMID, PMCID, and DOI where
possible, then re-rank locally. Before local ranking, the PMID-eligible
deduplicated pool caps each federated source's contribution after
deduplication and before ranking. Default: 40% of `--limit` on federated pools
with at least three surviving primary sources. Rows count against their
primary source after deduplication. `--max-per-source 0` uses the default cap,
and setting it equal to `--limit` disables capping. The capped pool then
re-ranks locally with an effective relevance mode:

- `lexical` preserves the calibrated PubMed rescue plus lexical directness
  comparator byte-for-byte;
- `semantic` sorts the LitSense2-derived semantic signal descending and falls
  back to the lexical comparator; and
- `hybrid` scores each row as
  `0.4*semantic + 0.3*lexical + 0.2*citations + 0.1*position` by default
  using the same LitSense2-derived semantic signal, with `semantic=0` when
  LitSense2 did not match, plus CLI weight overrides for experimentation.

Keyword-bearing article queries default to `hybrid`, while entity-only article
queries default to `lexical`. The local ranking pipeline still has four
explicit responsibilities:

1. **Lexical preparation:** build ranking concepts from structured filters plus
   decomposed keyword terms, then normalize query-side and document-side text
   symmetrically.
2. **Per-source provenance:** preserve `matched_sources` together with
   source-local backend position through merge and dedup so backend-local rank
   survives federation.
3. **Pre-ranking source balancing:** cap one source before local ranking can
   flood the visible pool, but only after deduplication decides the primary
   source for each row.
4. **Mode-aware scoring:** keep the existing lexical comparator as a stable
   fallback while exposing the LitSense2-derived semantic signal, citation
   support, and average source-local position as explicit ranking signals.

The architectural invariants for the shipped contract are:

- merge order must never act as an implicit source priority;
- compound-name normalization must stay symmetric between anchor creation and
  result normalization;
- multi-concept keywords must not collapse into one exact-phrase anchor for
  ranking; and
- calibrated PubMed rescue still applies inside lexical fallback paths, but it
  is one signal inside the explicit ranking contract rather than an invisible
  source preference.

The validation boundary is also part of the architecture contract:

- `search article` rejects missing filters, invalid date values, inverted date
  ranges, and unsupported `--type` values before backend calls.
- `get article` accepts PMID, PMCID, and DOI only and rejects unsupported
  identifiers such as publisher PIIs with a clean `InvalidArgument`.
- Semantic Scholar helper commands accept PMID, PMCID, DOI, arXiv, and
  Semantic Scholar paper IDs and reject other identifiers before calling the
  backend.

## Chart Rendering

Chart rendering belongs to the local study analytics surface, not the generic
entity lookup path. The architecture has two related chart surfaces that share
the same chart vocabulary but serve different purposes.

- `biomcp chart` serves embedded markdown chart docs through
  `src/cli/chart.rs`, `docs/charts/`, and `RustEmbed`.
- `biomcp chart` documents the chart surface, but does not render charts.
- `biomcp study ... --chart` is the rendering path, with `ChartArgs` defined
  in `src/cli/types.rs` and output generation implemented in
  `src/render/chart.rs`.

The rendering entrypoints are `study query`, `study co-occurrence`,
`study compare`, and `study survival`. Across those commands, BioMCP supports
`bar`, `stacked-bar`, `pie`, `waterfall`, `heatmap`, `histogram`, `density`,
`box`, `violin`, `ridgeline`, `scatter`, and `survival`, with the command and
data-shape matrix enforced in code:

| Command | Valid chart types |
|---------|-------------------|
| `study query --type mutations` | `bar`, `pie`, `waterfall` |
| `study query --type cna` | `bar`, `pie` |
| `study query --type expression` | `histogram`, `density` |
| `study co-occurrence` | `bar`, `pie`, `heatmap` |
| `study compare --type expression` | `box`, `violin`, `ridgeline`, `scatter` |
| `study compare --type mutations` | `bar`, `stacked-bar` |
| `study survival` | `bar`, `survival` |

The renderer targets terminal, SVG file, PNG file behind the `charts-png`
feature, and MCP inline SVG output. `--cols` and `--rows` size terminal
output. `--width` and `--height` size SVG, PNG, and MCP inline SVG output.
`--scale` is PNG-only. `--title`, `--theme`, and `--palette` style rendered
charts. Heatmaps reject `--palette` because `study co-occurrence --chart
heatmap` uses a fixed continuous colormap.

MCP chart responses are handled by `rewrite_mcp_chart_args()`, which turns a
charted study request into a text pass plus an SVG pass. In that rewrite
boundary, `--terminal` is stripped, `--output` / `-o` are rejected, and
`--cols` / `--rows` and `--scale` are rejected for the SVG pass. The SVG pass
preserves chart selection, sizing, and styling flags and injects inline-SVG
output for MCP clients; MCP does not return terminal or file output.

For the user-facing chart reference and examples, see `docs/charts/index.md`.
That guide covers workflows and examples in detail; this overview documents
where the chart docs, study rendering path, and MCP response rewrite fit
together.

## API Keys

Most commands work without credentials. Optional keys improve rate limits or
unlock additional data:

| Key | Source | Effect |
|-----|--------|--------|
| `NCBI_API_KEY` | PubTator3, PMC OA, NCBI ID converter | Higher rate limits |
| `S2_API_KEY` | Semantic Scholar article enrichment/navigation | Optional authenticated Semantic Scholar requests at 1 req/sec; shared-pool requests run at 1 req/2sec without the key |
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

The semver tag is the canonical release/version authority. PR CI enforces
version parity before release via the `version-sync` job and
`scripts/check-version-sync.sh`. The release workflow builds binaries,
publishes PyPI wheels, and deploys docs from the tagged source, while
`install.sh` resolves the latest release with platform assets, not the latest
merge to `main`. The existing `### Post-tag public proof` block is the live
verification step for tag-to-binary and tag-to-docs parity.
`workflow_dispatch` can replay a specified tag, but only as an explicit-tag
rebuild path, not a second source of release truth.

1. Update version in `Cargo.toml`, `Cargo.lock`, `pyproject.toml`,
   `manifest.json`, `CITATION.cff`, and `CHANGELOG.md`
2. Commit and push to `main`
3. Cut a GitHub release with a semver tag
4. GitHub Actions validates and publishes:
   - CI (`.github/workflows/ci.yml`) runs five parallel jobs: `check`
     (`cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`),
     `version-sync` (`bash scripts/check-version-sync.sh`),
     `climb-hygiene` (`bash scripts/check-no-climb-tracked.sh`),
     `contracts` (`cargo build --release --locked`, `uv sync --extra dev`,
     `uv run pytest tests/ -v --mcp-cmd "./target/release/biomcp serve"`,
     `uv run mkdocs build --strict`), and `spec-stable`
     (`cargo build --release --locked`, then `make spec-pr`).
   - Volatile live-network headings run separately in `.github/workflows/spec-smoke.yml`,
     which runs the full `make spec` suite on a schedule and by manual dispatch.
     The local `make spec-smoke` target is a targeted operator rerun lane and
     is not wired into CI in this ticket.
   - Release validation runs the Rust checks again, then
     `uv run pytest tests/ -v --mcp-cmd "biomcp serve"` and
     `uv run mkdocs build --strict`.
   - Release build jobs package cross-platform binaries, publish PyPI wheels,
     and deploy docs.
5. `install.sh` resolves the latest tagged release with downloadable assets

### Post-tag public proof

After the new tag is published, hand these commands to the verify/devops pass
so release-visible version identity and docs parity are checked against the
live surfaces:

```bash
tag="${BIOMCP_TAG:?set BIOMCP_TAG to the published release tag, e.g. v0.8.22}"
version="${tag#v}"
tmpdir="$(mktemp -d)" && BIOMCP_INSTALL_DIR="$tmpdir" BIOMCP_VERSION="$tag" bash install.sh >/tmp/biomcp-install.log && "$tmpdir/biomcp" version | head -n 1
bioasq_page="$(mktemp)" && curl -fsSL -A 'Mozilla/5.0' https://biomcp.org/reference/bioasq-benchmark/ >"$bioasq_page" && rg -q 'hf-public-pre2026' "$bioasq_page" && rg -q 'Phase A\+' "$bioasq_page" && rg -q 'Phase B' "$bioasq_page"
api_keys_page="$(mktemp)" && curl -fsSL -A 'Mozilla/5.0' https://biomcp.org/getting-started/api-keys/ >"$api_keys_page" && rg -q 'shared Semantic Scholar pool at 1 req/2sec' "$api_keys_page" && rg -q 'authenticated quota at 1 req/sec' "$api_keys_page"
drug_page="$(mktemp)" && curl -fsSL -A 'Mozilla/5.0' https://biomcp.org/user-guide/drug/ >"$drug_page" && rg -q 'trastuzumab regulatory --region who' "$drug_page" && rg -q 'WHO Prequalification local data setup' "$drug_page" && rg -q 'available \(default path\)' "$drug_page"
```

Expected markers:

- published tag matches `$tag`
- installed binary starts with `biomcp $version`
- BioASQ route returns all shipped benchmark page markers
- live API Keys docs show both shared-pool and authenticated Semantic Scholar
  guidance
- live Drug docs show the WHO `--region` workflow and WHO local-data setup copy
  together with the local-data path marker

Known issue: `uv sync --extra dev` may rewrite the editable root package
version in `uv.lock` during a release cut. Verify whether the lockfile
version bump should ship with the release commit.

## Verification Approach

BioMCP has six distinct verification and operator-inspection surfaces.

### 1. CI and Repo Gates

- `make check` is the required local ticket gate. In the current `Makefile`,
  that means `lint`, `test`, and `check-quality-ratchet`, and the `lint`
  stage rejects deprecated install strings in `README.md` and `docs/`.
- Repo-local `test` now maps to `cargo nextest run`; the CI `check` job still
  uses `cargo test` directly.
- CI in `.github/workflows/ci.yml` runs the broader repo baseline in parallel:
  `check`, `version-sync`, `climb-hygiene`, `contracts`, and `spec-stable`.
- Docs-site validation and Python contract tests do not run under `make check`;
  they live in `make test-contracts` and the CI `contracts` job.
- The grounding implementation surfaces for this split are `Makefile`,
  `.github/workflows/ci.yml`, and `.github/workflows/contracts.yml`.

#### March Validation Profiles

`.march/validation-profiles.toml` is the source of record for BioMCP's March
validation tiers. The shared build flow currently maps `kickoff` to
`preflight`, leaves `01-design` and `02-design-review` without a validation
profile, runs `focused` for `03-code` and `04-code-review`, and runs
`full-blocking` for `05-verify`.

The exhaustive tracked and staged `.march/*` allowlist is
`.march/code-review-log.md` and `.march/validation-profiles.toml`.
`.march/` remains ignored by `.gitignore`; allowlisted tracked files are rare
explicit index exceptions, not ignore-rule negations. The Python cleanup
contract rejects every other tracked `.march/*` path, and the pre-commit helper
rejects staged non-deletion `.march/*` paths outside the same allowlist.

| Profile | Command | Current build-flow use |
|---|---|---|
| `preflight` | `cargo check --all-targets` | `kickoff` |
| `baseline` | `cargo check --all-targets` | declared, not assigned |
| `focused` | `cargo test --lib && cargo clippy --lib --tests -- -D warnings` | `03-code`, `04-code-review` |
| `full-blocking` | `make check && make spec-pr` | `05-verify` |
| `full-contracts` | `make check && make spec-pr && make test-contracts` | declared, not assigned |

`full-blocking` deliberately uses `make check && make spec-pr`, not full
`make spec`, because `SPEC_PR_DESELECT_ARGS` is the stable PR-blocking spec
set. `full-contracts` is declared for tickets that need the contracts lane, but
the shared build flow does not assign it today.

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

Run locally with `make spec`. Use `make spec-smoke` as the serial targeted
local rerun for the ten stable smoke section IDs in `SPEC_SMOKE_ARGS`.

Repo-local `make spec` and `make spec-pr` use `pytest-xdist` with
`-n auto --dist loadfile` for the parallel-safe bulk, then run
`spec/05-drug.md`, `spec/13-study.md`, and `spec/21-cross-entity-see-also.md`
serially because those files share repo-global local-data fixtures. `make
spec-smoke` does not use xdist; it resolves stable targeted smoke section IDs
to current mustmatch pytest item IDs at runtime, then runs those items serially
with a 120s mustmatch timeout.
Use `spec/README-timings.md` as the current audit record for the PR lane and as
the smoke-only section inventory for `SPEC_PR_DESELECT_ARGS`; the ratchet
checks that scanned `SPEC_SMOKE_ARGS` entries resolve to collectable mustmatch
pytest items.

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
- `--apis-only` omits the EMA local-data row, the WHO Prequalification
  local-data row, the CDC CVX/MVX local-data row, the GTR local-data row, the
  WHO IVD local-data row, the cache-writability row, and the cache-limits row
  because none of these are upstream API checks.
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

### 6. Remote HTTP Demo Artifact (`examples/streamable-http/streamable_http_client.py`)

Release verification for the Streamable HTTP surface also includes the
standalone Streamable HTTP demo client
(`examples/streamable-http/streamable_http_client.py`). Run `biomcp serve-http`, then execute:

```bash
uv run --script examples/streamable-http/streamable_http_client.py
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
- Semantic Scholar participates in article search fan-out only on the
  compatible `search article --source all` path
- Semantic Scholar always owns TLDR, citations, references, and
  recommendations
- Federated totals are approximate
- Some sources (OncoKB production, NCI CTS, AlphaGenome) require API keys
- OncoKB demo endpoint has a known no-hit response for some variants — this
  is expected behavior, not a bug
- PubTator coerces small `size` parameters — use fixed internal page sizes
  (25) to avoid offset drift in pagination
- ClinicalTrials.gov mutation discovery cannot rely on `EligibilityCriteria`
  alone; search mutation-related title, summary, and keyword fields too

## Operator Notes

Runtime operator docs now live in `architecture/technical/staging-demo.md` and
`RUN.md`. Use those documents for the shared target, promotion contract, and
exact release-binary run/smoke commands, then use `scripts/` for the source
probe inventory and demo helpers.
