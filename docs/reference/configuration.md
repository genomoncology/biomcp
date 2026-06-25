# Configuration Reference

This page classifies supported operator runtime configuration separately from
internal fixture overrides and release/install variables.

## Operator API Keys

| Variable | Purpose |
|---|---|
| `ALPHAGENOME_API_KEY` | Enables `get variant <id> predict` |
| `DISGENET_API_KEY` | Enables gene/disease `disgenet` sections |
| `NCBI_API_KEY` | Improves NCBI E-utilities quota for PubMed, PubTator, PMC OA, and ID Converter paths |
| `NCI_API_KEY` | Enables trial operations with `--source nci` |
| `ONCOKB_TOKEN` | Enables the explicit `variant oncokb <id>` helper |
| `OPENFDA_API_KEY` | Improves OpenFDA quota headroom |
| `S2_API_KEY` | Enables authenticated Semantic Scholar quota |
| `UMLS_API_KEY` | Enables optional discover cross-vocabulary enrichment |

## Operator Data and Cache Knobs

| Variable or file | Purpose |
|---|---|
| `BIOMCP_CACHE_DIR` | Overrides the BioMCP cache root |
| `BIOMCP_CACHE_MODE` | Cache behavior; `infinite` is used for local replay/spec cache hits |
| `BIOMCP_CACHE_MAX_AGE` | Optional cache age limit |
| `BIOMCP_CACHE_MAX_SIZE` | Optional cache size limit |
| `BIOMCP_CACHE_MIN_DISK_FREE` | Minimum free disk budget before cache eviction |
| `BIOMCP_STUDY_DIR` | Local cBioPortal-style study dataset root |
| `BIOMCP_DDINTER_DIR` | Local DDInter download bundle root |
| `BIOMCP_EMA_DIR` | Local EMA human-medicines download root |
| `BIOMCP_WHO_DIR` | Local WHO Prequalification download root |
| `BIOMCP_CVX_DIR` | Local CDC CVX/MVX download root |
| `BIOMCP_GTR_DIR` | Local GTR download root |
| `BIOMCP_WHO_IVD_DIR` | Local WHO IVD download root |
| `cache.toml` | Persistent cache defaults under the resolved config root |
| `RUST_LOG` | stderr tracing filter; default CLI behavior is quiet, and `tools/biomcp-ci` sets `error` |

## Test and Fixture Override Seams

`BIOMCP_*_BASE`, `BIOMCP_*_URL`, and fixture process variables are internal
fixture overrides unless this page lists them in an operator section. They let
BioMCP's own verification harness redirect a source to a local server or fixture
file. Do not treat those base-URL overrides as stable operator API.

Known examples include `BIOMCP_PUBTATOR_BASE`, `BIOMCP_EUROPEPMC_BASE`,
`BIOMCP_PUBMED_BASE`, `BIOMCP_S2_BASE`, `BIOMCP_MYCHEM_BASE`,
`BIOMCP_OPENFDA_BASE`, `BIOMCP_VAERS_BASE`, `BIOMCP_GWAS_BASE`,
`BIOMCP_OLS4_BASE`, `BIOMCP_REACTOME_BASE`, `BIOMCP_WIKIPATHWAYS_BASE`,
`BIOMCP_WHO_PQ_URL`, `BIOMCP_WHO_PQ_API_URL`, `BIOMCP_WHO_VACCINES_URL`,
`BIOMCP_WHO_IVD_URL`, `BIOMCP_GTR_TEST_VERSION_URL`,
`BIOMCP_GTR_CONDITION_GENE_URL`, `BIOMCP_CVX_URL`,
`BIOMCP_CVX_TRADENAME_URL`, and `BIOMCP_MVX_URL`.

## Release and Install Variables

| Variable | Purpose |
|---|---|
| `BIOMCP_BIN` | Selects a built binary for spec wrappers and local demos |
| `BIOMCP_GITHUB_REPO` | Installer repository override |
| `BIOMCP_INSTALL_DIR` | Installer destination override |
| `BIOMCP_TAG` | Release/tag helper override |
| `BIOMCP_VERSION` | Installer/version override |
| `BIOMCP_SPEC_CACHE_HIT` | Spec wrapper hint that enables replay-style cache mode when unset by the caller |

## Observability and Degradation

- Human-facing diagnostics and tracing go to stderr, never JSON stdout.
- JSON query responses use `_meta.source_status` where a command has structured
  source availability/auth state to expose.
- `biomcp health --apis-only` reports API/source connectivity and excluded
  key-gated rows; full `biomcp health` also reports local runtime data and cache
  readiness.
- Optional sources degrade by omission, explicit unavailable notes, or
  `_meta.source_status`; they must not fabricate empty-source certainty.
- `SourceUnavailable` means the source is supported but temporarily unavailable.
  It is distinct from unsupported sections or invalid command grammar.
