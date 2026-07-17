# Diagnostic

Use diagnostic commands when you need source-native diagnostic inventory from
BioMCP's local-runtime diagnostic surface. The entity now merges two local
sources and one opt-in live regulatory overlay:

- NCBI Genetic Testing Registry (GTR) for gene-centric genetic tests
- WHO Prequalified IVD for infectious-disease diagnostics
- OpenFDA device 510(k) and PMA for optional U.S. regulatory status overlays

## Search diagnostic tests

Gene-first GTR search:

```bash
biomcp search diagnostic --gene BRCA1 --limit 5
biomcp search diagnostic --gene EGFR --type Clinical --source gtr --limit 5
```

WHO IVD infectious-disease search:

```bash
biomcp search diagnostic --disease HIV --source who-ivd --limit 5
biomcp search diagnostic --disease tuberculosis --source who-ivd --limit 5
```

Default `--source all` keeps GTR gene workflows valid and adds WHO IVD when the
filters can match it:

```bash
biomcp search diagnostic --disease malaria --source all --limit 5
biomcp search diagnostic --manufacturer InTec --source all --limit 5
```

Diagnostic search is filter-only. At least one of `--gene`, `--disease`,
`--type`, or `--manufacturer` is required. All provided filters are
conjunctive, `--limit` must stay within `1..=50`, and result ordering is
deterministic: normalized test name ascending, then accession ascending after
the source-specific match sets are merged.

`--disease` is a bounded disease phrase filter: it must contain at least three
alphanumeric characters and matches complete words or phrases at boundaries
against GTR condition names and WHO IVD `Pathogen/Disease/Marker`. Short noisy
tokens such as `ma` are rejected. Use `--limit` and `--offset` to page larger
diagnostic result sets.

`--source` accepts `gtr`, `who-ivd`, or `all` (default). GTR remains the
gene-capable source. WHO IVD supports `--disease`, `--type`, and
`--manufacturer`, but explicit `--source who-ivd --gene ...` is invalid and
returns a source-aware recovery hint. On the default `--source all` route,
gene-only searches stay valid because the WHO IVD leg is skipped.

## Cross-entity diagnostic pivots

Gene and disease cards can embed diagnostic-test summaries as opt-in sections:

```bash
biomcp get gene BRCA1 diagnostics
biomcp get disease tuberculosis diagnostics
```

The gene pivot uses the same GTR-backed diagnostic search as
`biomcp search diagnostic --gene <symbol>`. The disease pivot uses the default
multi-source diagnostic route, so GTR and WHO IVD rows can appear together when
both local bundles match the condition. Embedded sections show a compact table
with accession, name, type, manufacturer or lab, public source label, genes,
and conditions.

Disease diagnostic cards show at most 10 rows. When rows exist, the card prints
a `See also:` command for the broader paged search surface, for example
`biomcp search diagnostic --disease tuberculosis --source all --limit 50`.
Continue with `--offset` on `search diagnostic` for later pages.

Diagnostic search rows and embedded gene/disease diagnostic tables cap the
`Genes` and `Conditions` cells at five displayed values and append `+N more`
when additional values are available. Use `biomcp get diagnostic <id> genes`
or `biomcp get diagnostic <id> conditions` when you need the full lists; JSON
search output keeps the full deduped symbol arrays.

`diagnostics` is intentionally opt-in on gene and disease cards. It is not
expanded by `biomcp get gene <symbol> all` or
`biomcp get disease <name_or_id> all`. When local diagnostic data is
unavailable, the parent gene or disease card still succeeds and renders a note
with the sync command to enable the pivot.

## Get a diagnostic record

```bash
biomcp get diagnostic GTR000006692.3
biomcp get diagnostic "ITPW02232- TC40"
```

Default output returns the summary card only. The base card keeps concise
metadata such as source label, test type, manufacturer, and the source-native
summary fields for the resolved record. GTR cards keep laboratory, institution,
country, CLIA number, statuses, and method categories. WHO IVD cards add
target/marker, regulatory version, and prequalification year. The FDA overlay
is opt-in and does not change the summary card unless you explicitly request
`regulatory`.

## Request diagnostic sections

Supported public section tokens are `genes`, `conditions`, `methods`,
`regulatory`, and `all`, but support is resolved per source.

```bash
biomcp get diagnostic GTR000006692.3 genes
biomcp get diagnostic GTR000006692.3 conditions
biomcp get diagnostic GTR000006692.3 methods
biomcp get diagnostic GTR000006692.3 regulatory
biomcp get diagnostic GTR000006692.3 all
biomcp get diagnostic "ITPW02232- TC40" conditions
biomcp get diagnostic "ITPW02232- TC40" regulatory
biomcp get diagnostic "ITPW02232- TC40" all
```

Source-aware section support:

- GTR: `genes`, `conditions`, `methods`, `regulatory`
- WHO IVD: `conditions`, `regulatory`

`all` expands only to the source-native local sections and intentionally
excludes `regulatory` because the FDA overlay is live and optional. `summary`
is always included and is not a separate section token.

Markdown hides unrequested sections entirely. When a requested section is
empty, BioMCP still renders the heading with a truthful empty-state note. JSON
keeps the same distinction: unrequested sections are omitted, while requested
empty sections serialize as `[]`. WHO IVD `genes` and `methods` are not empty
sections; they are unsupported requests and fail before any data fetch.

When you request `regulatory`, BioMCP queries OpenFDA device 510(k) and PMA
records against a bounded set of source-native diagnostic names. Markdown adds
`## Regulatory (FDA Device)` only when the section is requested. JSON adds a
top-level `regulatory` field only when the section is requested. A healthy
no-match records `section_outcomes.regulatory` as `empty`; a temporary OpenFDA
failure records `unavailable` with no provider credit. Markdown reports the
unavailable state in-band rather than presenting it as a healthy no-match.

## JSON metadata

Non-empty `search diagnostic --json` responses include `_meta.next_commands`.
The first follow-up drills the top result with `biomcp get diagnostic <id>`,
and `biomcp list diagnostic` is always present. Search rows also preserve the
source (`gtr` or `who-ivd`) so merged result pages stay truthful.

`get diagnostic --json` also includes `_meta.next_commands` and
`_meta.section_sources`. Follow-up commands are section-aware, so requesting
`genes` suppresses the redundant `genes` follow-up but still suggests the
remaining supported sections. WHO product codes can contain spaces, so
follow-up commands quote them automatically. GTR summary cards now advertise
four visible follow-up section commands because `regulatory` is opt-in but
still supported on that source.

## Local data setup

Diagnostic commands read GTR local data from `BIOMCP_GTR_DIR` and WHO IVD local
data from `BIOMCP_WHO_IVD_DIR` first, then the platform data directory. On
first use, BioMCP auto-downloads the required files into those roots and
refreshes stale data automatically.

Required GTR files:

- `test_version.gz`
- `test_condition_gene.txt`

Required WHO IVD files:

- `who_ivd.csv`

Confirm local diagnostic readiness with full health output:

```bash
biomcp health
```

Force-refresh the local data manually:

```bash
biomcp gtr sync
biomcp who-ivd sync
```

If you need to override the default path:

```bash
export BIOMCP_GTR_DIR="/path/to/gtr"
export BIOMCP_WHO_IVD_DIR="/path/to/who-ivd"
export OPENFDA_API_KEY="..."
biomcp health
```

Manual preseed remains supported for offline or controlled environments. A
complete GTR root must contain both GTR files listed above, and a complete WHO
IVD root must contain `who_ivd.csv`. Full `biomcp health` reports separate
`GTR local data (...)` and `WHO IVD local data (...)` rows so operators can see
which bundle is missing, stale, or explicitly configured.
