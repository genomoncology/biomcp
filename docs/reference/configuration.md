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
| `BIOMCP_CACHE_MAX_AGE` | Optional cache age limit, as a positive integer number of seconds |
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

Cache runtime precedence is environment, then `cache.toml`, then built-in default.
`BIOMCP_CACHE_MAX_AGE` overrides `[cache].max_age_secs`; both values are
positive integer seconds.

## Internal and Measurement Controls

| Variable | Purpose |
|---|---|
| `BIOMCP_GENE_GET_STRATEGY` | Internal gene retrieval strategy control used for measurement and rollout comparisons |
| `BIOMCP_GENE_OPTIONAL_TIMEOUT_MS` | Internal timeout, in milliseconds, for optional gene enrichment branches |
| `BIOMCP_GENE_TIMING_PATH` | Internal measurement output path; when set, gene retrieval may write a local JSON timing report to that caller-provided path |

## Test and Fixture Override Seams

`BIOMCP_*_BASE`, `BIOMCP_*_URL`, and fixture process variables are internal
fixture overrides unless this page lists them in an operator section. They let
BioMCP's own verification harness redirect a source to a local server or fixture
file. Do not treat those base-URL overrides as stable operator API.

Known examples include `BIOMCP_ALPHAGENOME_BASE`,
`BIOMCP_CANCERHOTSPOTS_BASE`, `BIOMCP_CBIOPORTAL_BASE`,
`BIOMCP_CBIOPORTAL_DATAHUB_BASE`, `BIOMCP_CHEMBL_BASE`, `BIOMCP_CIVIC_BASE`,
`BIOMCP_CLINGEN_BASE`, `BIOMCP_CLINGEN_EREPO_BASE`, `BIOMCP_COMPLEXPORTAL_BASE`, `BIOMCP_CPIC_BASE`,
`BIOMCP_CTGOV_BASE`, `BIOMCP_CTGOV_CDN_BASE`, `BIOMCP_DGIDB_BASE`, `BIOMCP_DISGENET_BASE`,
`BIOMCP_EMA_REPORT_BASE`, `BIOMCP_ENRICHR_BASE`, `BIOMCP_EUROPEPMC_BASE`,
`BIOMCP_FIGSHARE_BASE`, `BIOMCP_GNOMAD_BASE`, `BIOMCP_GPROFILER_BASE`,
`BIOMCP_GTEX_BASE`, `BIOMCP_GWAS_BASE`, `BIOMCP_HPA_BASE`,
`BIOMCP_HPO_BASE`, `BIOMCP_INTERPRO_BASE`, `BIOMCP_KEGG_BASE`,
`BIOMCP_LITSENSE2_BASE`, `BIOMCP_MEDLINEPLUS_BASE`, `BIOMCP_MONARCH_BASE`,
`BIOMCP_MUTALYZER_BASE_URL`, `BIOMCP_MYCHEM_BASE`, `BIOMCP_MYDISEASE_BASE`,
`BIOMCP_MYGENE_BASE`, `BIOMCP_MYVARIANT_BASE`, `BIOMCP_NCBI_IDCONV_BASE`,
`BIOMCP_NCI_CTS_BASE`, `BIOMCP_NIH_REPORTER_BASE`, `BIOMCP_OLS4_BASE`,
`BIOMCP_ONCOKB_BASE`, `BIOMCP_OPENFDA_BASE`, `BIOMCP_OPENTARGETS_BASE`,
`BIOMCP_PHARMGKB_BASE`, `BIOMCP_PMC_HTML_BASE`, `BIOMCP_PMC_OA_BASE`,
`BIOMCP_PUBMED_BASE`, `BIOMCP_PUBTATOR_BASE`, `BIOMCP_QUICKGO_BASE`,
`BIOMCP_REACTOME_BASE`, `BIOMCP_S2_BASE`, `BIOMCP_SEER_BASE`,
`BIOMCP_STRING_BASE`, `BIOMCP_UMLS_BASE`, `BIOMCP_UNIPROT_BASE`,
`BIOMCP_VAERS_BASE`, `BIOMCP_VARIANTVALIDATOR_BASE_URL`,
`BIOMCP_WIKIPATHWAYS_BASE`, `BIOMCP_WHO_PQ_URL`,
`BIOMCP_WHO_PQ_API_URL`, `BIOMCP_WHO_VACCINES_URL`, `BIOMCP_WHO_IVD_URL`,
`BIOMCP_GTR_TEST_VERSION_URL`, `BIOMCP_GTR_CONDITION_GENE_URL`,
`BIOMCP_CVX_URL`, `BIOMCP_CVX_TRADENAME_URL`, and `BIOMCP_MVX_URL`.
Internal cBioPortal fixture/source-selection seams include
`BIOMCP_CBIOPORTAL_STUDY`, `BIOMCP_CBIOPORTAL_SAMPLE_LIST`, and
`BIOMCP_CBIOPORTAL_MUTATION_PROFILE`.

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

| Variable | Purpose |
|---|---|
| `BIOMCP_DISABLE_KEGG` | Operator-supported degradation control that disables KEGG pathway calls when KEGG should be avoided |

- Human-facing diagnostics and tracing go to stderr, never JSON stdout.
- JSON query responses use `_meta.source_status` where a command has structured
  source availability/auth state to expose.
- `biomcp health --apis-only` reports API/source connectivity and excluded
  key-gated rows; full `biomcp health` also reports local runtime data and cache
  readiness.
- Optional entity lookups expose typed outcomes in JSON/MCP. `inapplicable`
  means a required input was absent and no provider was contacted; `empty` is a
  successful zero-result query; `unavailable` means retrieval produced no usable
  result; and `degraded` preserves partial evidence. `_meta.section_sources`
  repeats requested section outcomes and credits only providers that returned
  usable evidence. Outcome-only helpers such as `variant structure` may expose
  the same state model on their documented status surface.
- `SourceUnavailable` means the source is supported but temporarily unavailable.
  It is distinct from unsupported sections or invalid command grammar.
