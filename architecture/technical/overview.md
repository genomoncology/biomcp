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
  `discover`, read-only `skill` lookup/list/render behavior, and MCP-safe `study`
  subcommands (`list`, `download --list`, `top-mutated`, `query`, `filter`,
  `cohort`, `survival`, `compare`, `co-occurrence`) are allowed.
  Operator-local or mutating commands such as `cache`, `update`, `serve`,
  `serve-http`, and `skill install` stay blocked over MCP. The same transport
  boundary removes the typed article `full_text_path` field before MCP JSON is
  serialized and uses that field to replace `Saved to:` paths in readable MCP
  text with an availability indicator. Source/status/provenance remain intact;
  direct CLI rendering and saved-file behavior remain unchanged. This policy is
  shared by stdio and Streamable HTTP rather than implemented as a path regex.
  See `src/mcp/shell.rs`, `tests/rmcp_client_contract.rs`, and
  `tests/test_skill_prompt_contract.py` for the canonical boundary.
- **HTTP mode:** `biomcp serve-http --host 0.0.0.0 --port 8080` starts the
  Streamable HTTP server. Remote MCP traffic uses `/mcp`, and lightweight
  probes live at `/health`, `/readyz`, and `/`. This is the canonical scaling
  answer when rate limiting needs to be shared across concurrent agent workers,
  since rate limiting is otherwise process-local.

The binary is also distributed as `biomcp-cli` on PyPI (a thin Python wrapper
that ships the platform-specific Rust binary). Python is packaging only;
no Python logic is involved in query processing.

## Public Surface and Crate Boundary

BioMCP's supported product surface is the `biomcp` CLI/MCP runtime. The Rust
crate exists so Cargo can share implementation between the binaries; its module
exports are internal implementation details, unstable, have no semver guarantee,
and are not for downstream import. Do not treat `biomcp_cli::...` modules as a
supported library API.

## Human Output Integrity

Renderer-owned human text passes through the shared sequence-aware sanitizer in
`src/render/human.rs` at CLI and MCP emission boundaries. It removes terminal
control sequences and invisible bidi controls while preserving ordinary Unicode;
inline chart labels and diagnostics additionally convert layout controls to safe
separators. Typed entity values remain unchanged.

Pretty JSON stays semantically faithful: `src/render/json.rs` is the sole
production pretty writer and lexically escapes raw DEL/C1 and bidi controls after
serde serialization. Parsing the output reproduces the original keys and values.

## Build and Packaging

```
cargo build --release --locked   # Rust binary
uv tool install biomcp-cli       # existing PyPI package
curl ... install.sh | bash       # binary installer (resolves latest release)
```

- **Edition:** Rust 2024
- **Current version:** see `Cargo.toml`; `scripts/check-version-sync.sh` keeps
  committed `Cargo.toml`, `Cargo.lock`, `pyproject.toml`, root `uv.lock`,
  `manifest.json`, both `server.json` version fields, `CITATION.cff`, and the
  Homebrew formula aligned.
- **Package name:** `biomcp-cli` on PyPI; binary name is `biomcp`
- **Release state:** v0.8.25 is the latest published release. The manual
  workflow is disabled until ticket 0957 installs the public-artifact gate.
- **Metadata changes:** Commit synchronized metadata and changelog updates;
  package versions are never stamped from tags.
- **Generated AlphaGenome client:** Normal builds do not run or require
  `protoc`; they include the committed generated Rust source directly. The
  explicit `scripts/regenerate-alphagenome-proto` maintainer command requires
  pinned `protoc` 28.3, applies the reviewed dead-code annotation, validates a
  temporary candidate, and atomically replaces only that generated file. CI
  uses `scripts/regenerate-alphagenome-proto --check` so drift is reported
  without modifying the checkout.

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
Keyword-bearing queries keep that default source set, and Semantic Scholar
remains an optional compatible search leg on that path. Semantic Scholar and
LitSense2 are available through explicit `--source semanticscholar` and
`--source litsense2`. Strict Europe PMC-only filters such as `--open-access` and
`--type` disable the federated planner and route to Europe PMC only.
`--source pubtator` and `--source semanticscholar` with strict Europe PMC-only
filters are rejected at the front door. `--source` accepts
`all|pubtator|europepmc|pubmed|semanticscholar|litsense2` in v1.

Article filters remain raw as the shared contract for planning, ranking,
rendering, JSON metadata, and session loop-breaker state. At the provider
boundary, direct and compatible federated PubMed ESearch cleans bounded
question-format filler words from unfielded gene, disease, drug, and keyword
clauses. PubTator3, Europe PMC, and Semantic Scholar receive their existing
query inputs on the default route; explicit LitSense2 searches keep their
LitSense2 query input.

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
feature, and MCP inline SVG output. Every dynamic title, label, category, and
legend is sanitized before it enters the chart backend. Completed terminal chart
output is not scrubbed because the backend's own ANSI styling is trusted and
intentional. `--cols` and `--rows` size terminal
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
| `S2_API_KEY` | Semantic Scholar article enrichment/navigation | Optional authenticated Semantic Scholar requests at 1 req/sec per BioMCP process; shared-pool requests run at 1 req/2sec without the key |
| `OPENFDA_API_KEY` | OpenFDA | Higher rate limits |
| `NCI_API_KEY` | NCI CTS trial search (`--source nci`) | Required for NCI source |
| `DISGENET_API_KEY` | DisGeNET scored gene/disease associations | Required for `get gene <symbol> disgenet` and `get disease <name_or_id> disgenet`; DisGeNET sections are unavailable without the key |
| `ONCOKB_TOKEN` | OncoKB production variant helper | Required for `variant oncokb <id>`; the helper is unavailable without the token |
| `UMLS_API_KEY` | UMLS discover clinical crosswalk enrichment | Optional; `discover` still returns OLS4 results without the key but omits UMLS crosswalks |
| `ALPHAGENOME_API_KEY` | AlphaGenome variant effect prediction | Required for AlphaGenome |

For demo and offline workflows: `BIOMCP_CACHE_MODE=infinite` enables infinite
cache mode, replaying prior responses without hitting upstream APIs.

## Rate Limiting

Rate limiting is process-local. Multiple concurrent CLI invocations or MCP
server workers do NOT share a limiter. For deployments with many concurrent
agent workers, run a single shared `biomcp serve-http` endpoint so all workers
share one limiter budget and one Streamable HTTP `/mcp` surface.

Semantic Scholar has an additional process-boundary contract for benchmark and
agent harnesses: `S2_API_KEY` is read from the `biomcp` subprocess environment,
so a parent runner that strips env vars must provide an explicit allowlist before
BioMCP can authenticate. The target contract and follow-up work are documented
in [Semantic Scholar runtime contract](semantic-scholar-runtime-contract.md).

## Release Pipeline

v0.8.25 is the latest published release. Package versions are committed metadata, not values stamped from tags. `scripts/check-version-sync.sh` uses
`Cargo.toml` as the canonical comparison value and keeps committed `Cargo.toml`, `Cargo.lock`, `pyproject.toml`, root `uv.lock`, `manifest.json`, both `server.json` version fields, `CITATION.cff`, and the Homebrew formula version synchronized.

The manual release workflow is disabled until ticket 0957 installs the
public-artifact gate. It is a read-only manual guard and intentionally creates
no release, registry update, image, wheel, tap update, documentation deployment,
or public asset. Existing installation documentation continues to describe the
already published v0.8.25 channels; `install.sh` resolves the latest release
with platform assets rather than the latest merge to `main`.

CI (`.github/workflows/ci.yml`) runs five parallel jobs: `check`
(`cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`),
`version-sync` (`bash scripts/check-version-sync.sh`),
`climb-hygiene` (`bash scripts/check-no-climb-tracked.sh`),
`contracts` (`cargo build --release --locked`,
`uv sync --extra dev --no-install-project`,
`uv run --no-sync pytest tests/ -v`,
`uv run --no-sync mkdocs build --strict`), and `spec-stable`
(release build, spec-cache metadata/restore, then `make spec-pr`). The
`version-sync` checkout fetches full tag history so its pre-1.0 changelog
boundary check is reliable. Routine release proof uses `make release-gate`,
which composes `make lint`, `make test`, and `make spec`; opt-in live
confidence uses `make verify` (`make release-live-smoke` aliases it).
`spec-stable` restores `.cache/biomcp-specs/`, exports
`BIOMCP_SPEC_CACHE_HIT=1` only on cache hits, and relies on
`tools/biomcp-ci` to flip warm-cache replay on for the canary docs.

Python/docs/spec gate lanes intentionally use `uv sync --extra dev --no-install-project`
followed by `uv run --no-sync ...`. They install Python tooling only and do not
install or rebuild the maturin-backed current project before pytest or mkdocs.

## Verification Approach

Ticket 373 added the request-contract testing reset: routine validation now proves BioMCP-owned CLI intent, source request plans, fixture response/status mapping, entity orchestration, and renderer/envelope contracts without depending on public upstream availability. The target boundaries, profile split, and migration invariants are documented in [Request-contract test architecture target](request-contract-test-architecture.md).

BioMCP has six distinct verification and operator-inspection surfaces.

### 1. CI and Repo Gates

- The canonical local gates are `make lint`, `make test`, and `make spec`.
  In the current `Makefile`, `make lint` runs the repo lint script plus the
  quality ratchet; the lint script runs `cargo deny check licenses` plus
  `cargo deny check advisories`, and still rejects deprecated install strings
  in `README.md` and `docs/`.
- Repo-local `make test` runs `cargo nextest run` plus the Python/docs contract
  lane against `target/spec/biomcp`. Routine `make spec` shares that selected
  binary; the CI `check` job still uses the raw `cargo fmt --check`,
  `cargo clippy -- -D warnings`, and `cargo test` sequence directly.
- CI in `.github/workflows/ci.yml` runs the broader repo baseline in parallel:
  `check`, `version-sync`, `climb-hygiene`, `contracts`, and `spec-stable`.
- Docs-site validation and Python contract tests now run under `make test`;
  CI still keeps that lane in the separate `contracts` job for parallelism.
- `make release-gate` is the single local routine release-blocking signal; it
  runs `make lint`, `make test`, and `make spec` directly. Both
  executable-contract consumers are explicitly routed to
  `target/release/biomcp`. Live public-upstream
  confidence is opt-in through `make verify` (`make release-live-smoke` aliases it).
- The grounding implementation surfaces for this split are `Makefile`,
  `.github/workflows/ci.yml`, and `.github/workflows/contracts.yml`.

#### March Validation Profiles

The standard repo gates are `make lint`, `make test`, and `make spec`.
March validation profiles are retired; the repository no longer keeps tracked
profile files under `.march/` for routine proof routing.

The exhaustive tracked and staged `.march/*` allowlist is
`.march/code-review-log.md`. `.march/` remains ignored by `.gitignore`;
allowlisted tracked files are rare explicit index exceptions, not ignore-rule
negations. The Python cleanup contract rejects every other tracked `.march/*`
path, and the pre-commit helper rejects staged non-deletion `.march/*` paths
outside the same allowlist.

Live public-upstream proof moved to the explicit opt-in `make verify` operator
lane, with `make release-live-smoke` kept as an alias.

### 2. Spec Suite (`spec/`)

BDD executable documentation written as `mustmatch` spec files. The suite
exercises CLI output at the command level using stable structural markers
(headers, table columns, query echoes) rather than brittle upstream data
values.

Routine local validation runs `make spec` for the routine executable-spec gate.
`make spec-contracts` remains available as a legacy deterministic subset. Live public-upstream confidence is explicit and
opt-in through `make verify` (with `make release-live-smoke` as a compatibility
alias), which uses `tools/biomcp-ci` for discover/OLS4, disease, article
source-status, variant-normalization, phenotype, protein, pathway, and other
live smoke commands.

PR CI runs `make spec-pr` via the `spec-stable` job in
`.github/workflows/ci.yml`. That job builds the release binary first, reads
`Cargo.toml` via Python `tomllib` to emit a `biomcp-version` plus a
workflow-local `spec-cache-schema-version`, restores `.cache/biomcp-specs/`
with the key
`spec-http-${runner.os}-${biomcp-version}-${spec-cache-schema-version}`, and
exports `BIOMCP_SPEC_CACHE_HIT=1` only when the restore hit is warm. The bash
blocks themselves then call `tools/biomcp-ci`, which keeps cache/XDG state
under `.cache/biomcp-specs/`, defaults `RUST_LOG=error`, unsets optional auth
keys, and flips `BIOMCP_CACHE_MODE=infinite` only for those warm CI replays.

Run locally with `make spec` for the offline routine executable-spec gate,
`make spec-contracts` for the legacy deterministic subset, `make verify` for
opt-in live public-upstream confidence, or `make spec-pr` for
the same offline corpus with the longer PR timeout.

Repo-local `make spec` and `make spec-pr` use `scripts/run-specs.sh` over
explicit `SPEC_ROUTINE_PATHS`: local or fixture-backed Markdown specs such as
`spec/entity/article.md`, `spec/entity/study.md`, `spec/entity/variant.md`, and
`spec/surface/mcp.md` run through `mustmatch test` with `--lang bash`; the lone Python
entry is `tests/surface/test_parallel_isolation_contract.py`, a static canary
that keeps disease/discover isolation from regressing. Python/static surface
contracts live under `tests/surface/`; all others run in the `make test`
contract lane, not in the Markdown spec runner. Live-upstream specs leave
routine canaries entirely for `make verify`: phenotype/Monarch, protein/UniProt
and ComplexPortal, disease/discover OLS4 paths, pathway Reactome/WikiPathways/KEGG,
plus gene, drug, diagnostic, trial, PGx, VAERS, and CLI/discover surfaces that
still exercise public APIs. There is no serialized live rerun leg in `make spec`
or `make spec-pr`; public OLS4/pathway confidence belongs to `make verify`.

Use `spec/README-timings.md` as the current validation-lane audit/reference for
the offline deterministic routine lane, opt-in live verify lane, active files,
wrapper/cache contract, and measured warm-cache expectations.

Important: repo-local Python/spec commands should use
`uv sync --extra dev --no-install-project` followed by `uv run --no-sync ...`.
Keep `target/release` ahead of `.venv/bin` and pass `BIOMCP_BIN` when running
CLI specs manually so the release binary, not an editable install, is tested.

### 3. `biomcp health`

`biomcp health` is a curated operator inspection surface, not a full source
inventory ledger.

- The command is grounded in `src/cli/health/`, with the stable entry point in `src/cli/health/mod.rs`.
- It shows per-source connectivity for readiness-significant sources.
- Key-gated sources appear as `excluded` rows when the required environment
  variable is absent.
- `--apis-only` omits the EMA local-data row, the WHO Prequalification
  local-data row, the CDC CVX/MVX local-data row, the GTR local-data row, the
  WHO IVD local-data row, the cache-writability row, and the cache-limits row
  because none of these are upstream API checks.
- Both JSON and Markdown summaries use explicit `healthy`, `warning`, `excluded`,
  and `error` counts. These categories reconcile with the report invariant
  `healthy + warning + excluded + error == total`.
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
- Semantic Scholar authentication depends on `S2_API_KEY` reaching the `biomcp`
  process environment; parent runners with stripped environments need an explicit
  env allowlist before BioMCP can use the key
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
