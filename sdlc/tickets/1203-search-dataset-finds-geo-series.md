---
flow: build
priority: 3
deps: []
---

# 1203: Search dataset finds NCBI GEO series

## Goal

`biomcp search dataset` finds public NCBI GEO series (GSE) by free text, organism, and series type. Each row gives a namespaced dataset ID (`geo:GSE164073`), the GSE accession, title, organism, series type, every platform (GPL), sample count, PubMed IDs, and summary. A hackathon team screening public GEO studies of drug-treated AML cell lines motivated it. That team had to write its own E-utilities loop to answer "which GEO series match this disease and treatment".

## Current Facts

- Nothing under `src/`, `docs/`, or `spec/` mentions GEO or the `gds` database today.
- `src/sources/ncbi_efetch.rs` wraps only `efetch.fcgi` (`full_text_xml_plan`, `src/sources/ncbi_efetch.rs:60`). It reads `BIOMCP_PUBMED_BASE` (`:12`) and `NCBI_API_KEY` through `crate::sources::ncbi_api_key()` (`:26`).
- `PubMedClient` already runs ESearch and ESummary, but `db` is fixed to `pubmed`. `esearch_plan` hard-codes it at `src/sources/pubmed.rs:302`, and `esummary_plan` hard-codes it at `src/sources/pubmed.rs:401`. `send` (`src/sources/pubmed.rs:158`) applies the auth-aware cache mode and the bounded body read. No `elink` exists.
- `crate::sources::ncbi_api_key()` (`src/sources/mod.rs:447`) trims `NCBI_API_KEY` and drops blanks.
- The rate limiter matches policies by URL prefix (`resolve_key_and_interval`, `src/sources/rate_limit.rs:194`). The `pubmed-eutils` policy covers `BIOMCP_PUBMED_BASE` or `https://eutils.ncbi.nlm.nih.gov/entrez/eutils` (`src/sources/rate_limit.rs:114`). It spaces requests 334 ms apart without a key and 100 ms apart with one (`pubmed_eutils_min_interval`, `src/sources/rate_limit.rs:257`). Any request under that base shares that budget with no new policy.
- `SourceProvider` defines `PUBMED` and `NCBI_EFETCH` (`src/error.rs:52`, `src/error.rs:53`) and parses their names at `src/error.rs:201`.
- The `study` entity means a cBioPortal DataHub bundle with local install (`src/cli/commands.rs:78`, `ENTITY_FLAGS` entry `("study", false, false)` at `src/cli/list/catalog.rs:53`). The scored review proposes the name `dataset` for GEO so `study` keeps one meaning.
- `SearchEntity` (`src/cli/commands.rs:269`) lists each searchable entity with an `after_help` example block. `search author` is the smallest recent precedent (`src/cli/commands.rs:284`, `src/cli/author/search.rs:26`).
- The typed MCP search tool covers eight entities (`typed_search_schema`, `src/mcp/shell.rs:284`). `disease` and `drug` are not among them and stay reachable through the raw tool.
- `biomcp list <entity>` routing lives in `src/cli/list/mod.rs:17` and `:73`, and the unknown-entity message at `src/cli/list/mod.rs:91` lists every valid entity.
- Source guides live in `docs/sources/*.md`, one page per provider, with an index table (`docs/sources/index.md`) and a `Sources:` nav block in `mkdocs.yml:43`. The source checklist is `architecture/technical/source-integration.md:630`.
- Source fixtures live under `testdata/sources/<source>/`, and source tests split into `construction.rs` (plans only) and `parsing.rs` (committed bytes only), as in `src/sources/pubmed/tests/`.

## Design

### Generic E-utilities helper

New module `src/sources/ncbi_eutils.rs`, declared in `src/sources/mod.rs`:

```rust
pub(crate) fn esearch_plan(db: &str, term: &str, retstart: usize, retmax: usize,
    extra: &[(&str, String)], api_key: Option<&str>) -> Result<RequestPlan, BioMcpError>;
pub(crate) fn esummary_plan(db: &str, ids: &[String], api_key: Option<&str>)
    -> Result<Option<RequestPlan>, BioMcpError>;
```

Both set `retmode=json` and add `api_key` only when the trimmed key is nonblank. They keep the validation `PubMedClient` applies today: a nonblank term up to 4096 bytes, `retmax` from 1 to 10000, and nonblank IDs. `db` must be one of `pubmed` or `gds`. Any other value is an `InvalidArgument`. That list grows only when a caller needs it.

`PubMedClient::esearch_plan` and `PubMedClient::esummary_plan` call the helper with `db = "pubmed"`. The date parameters pass through `extra`. The existing PubMed construction tests stay unchanged and must still pass, which proves the PubMed request bytes did not move. The efetch callers stay as they are.

### GEO source

New module `src/sources/geo.rs` with `GeoClient`. It uses `shared_client()`, `env_base(NCBI_EUTILS_BASE, "BIOMCP_PUBMED_BASE")`, and `ncbi_api_key()`, the same way `NcbiEfetchClient::new` does. Sharing the base gives GEO the `pubmed-eutils` rate-limit policy and the auth-aware cache mode for free. It adds `SourceProvider::NCBI_GEO` (`"NCBI GEO"`, parse names `ncbi-geo` and `NCBI GEO`) for error context.

`GeoClient::search(term, offset, limit)` runs one ESearch on `db=gds` and then one ESummary for the returned UIDs. A zero-hit ESearch makes no ESummary request.

The decoder reads these ESummary JSON members per UID: `accession`, `entrytype`, `title`, `summary`, `taxon`, `gdstype`, `gpl`, `gse`, `n_samples`, `pubmedids`, `suppfile`, `geo2r`, `pdat`. The recorded fixture decides exact shapes. The names above follow the E-utilities `gds` document summary.

- A `GSE` record maps straight to a row.
- A `GDS` record maps to its parent series `GSE<gse>`. If that GSE already appears in the result, the decoder drops the GDS record. If the GDS names more than one parent, it yields one row per parent. The row keeps the GDS accession in `curated_datasets`.
- The decoder drops `GPL` and `GSM` records.
- `gpl` and `gse` hold semicolon-separated numbers. The decoder prefixes each one (`GPL570`) and keeps the upstream order.
- `taxon` and `gdstype` hold semicolon-separated lists. They become arrays in upstream order.
- `suppfile` holds a comma-separated list of supplementary file types (`CEL`, `TXT`, `BW`). It becomes `supplementary_types`, an array in upstream order. A blank value becomes an empty array.
- `geo2r` (`yes` or `no`) becomes the boolean `geo2r`. Measured 2026-09-16 on 585 AML series: 104 had `yes` and a series matrix table, 200 had `yes` and an empty table, and 3 had `no` with a table. The flag therefore means GEO offers an analysis view, often from NCBI-computed RNA-seq counts. It does not mean the series matrix carries values. The source page states this, and ticket 1207 reports where values live.
- `pdat` holds the date the series went public, for example `2004/01/30` on the live GSE982 record read 2026-09-17. It becomes `published`, an ISO date string, on every row. It is free in a response the command already makes. It is not an update date, so it never fills `data_as_of`, and a missing or unparsable value becomes `null`.
- A missing required member (`accession`, `entrytype`, `title`) is an `Api` error naming the UID.

### Query

`src/entities/dataset/` builds the ESearch term from the flags, joined with `AND`:

| Flag | Term |
| --- | --- |
| `--disease <text>` | `(<text>)`. A second free-text term that reads better in examples. It is not a disease-ontology filter. |
| `--keyword <text>` (`-k`) | `(<text>)` |
| `--organism <text>` | `"<text>"[ORGN]` |
| `--type <text>` | `"<text>"[GTYP]` |
| always | `gse[ETYP]` |

At least one of `--disease` or `--keyword` is required. Otherwise the command fails with `InvalidArgument` and the message names both flags. Values pass through verbatim after trimming. BioMCP adds no synonym or organism alias table. Embedded double quotes are rejected. `--limit` defaults to 10 and accepts 1 to 50. `--offset` defaults to 0 and maps to `retstart`.

Change from the review: the review's shape has no `--type`. This ticket adds it because the scope asks for it and GEO indexes the series type as `[GTYP]`. The review's `--organism human` example passes through unchanged. The recorded fixture shows whether Entrez matches a common name.

`gse[ETYP]` keeps GDS, GPL, and GSM hits out of the result. The GDS mapping stays as a guard. A user term can carry its own field tags, and the ESummary contract allows GDS records.

### CLI and output

`SearchEntity::Dataset(dataset::DatasetSearchArgs)` with help `Search NCBI GEO series by disease, keyword, organism, or type` and this example block:

```text
EXAMPLES:
  biomcp search dataset --disease "acute myeloid leukemia" --organism "Homo sapiens" -k DMSO
  biomcp search dataset -k venetoclax --type "expression profiling by high throughput sequencing"

See also: biomcp list dataset
```

JSON:

```json
{
  "query": "<the ESearch term sent>",
  "total": 42,
  "offset": 0,
  "returned": 2,
  "results": [
    {
      "id": "geo:GSE00000",
      "source": "geo",
      "accession": "GSE00000",
      "title": "...",
      "organisms": ["Homo sapiens"],
      "series_types": ["Expression profiling by array"],
      "platforms": ["GPL570"],
      "sample_count": 12,
      "pmids": ["12345678"],
      "curated_datasets": [],
      "supplementary_types": ["CEL", "TXT"],
      "geo2r": true,
      "published": "2004-01-30",
      "summary": "..."
    }
  ]
}
```

`_meta.next_commands` includes `biomcp get article <pmid>` for the first PMID of the first row that has one. Ticket 1204 adds `get dataset`; until then the output never suggests it.

Identity: the `id` field is always `geo:<GSE>`. Commands that take a dataset ID accept both `geo:GSE164073` and bare `GSE164073`.

Markdown prints `# Datasets: <query>`, then `Found <total> series`, then one block per row. Each block has the accession and title as a heading, then organism, type, platforms, samples, published, and PMIDs lines, then the summary. The summary is cut at 400 characters with an ellipsis, and JSON keeps it whole.

Registration:
- `list dataset` prints the search usage and filter notes.
- `ENTITY_FLAGS` gains `("dataset", true, false)`.
- The unknown-entity list gains `dataset`.
- `src/cli/health/catalog.rs` gains an `NCBI GEO` probe: `esearch.fcgi?db=gds&retmode=json&retmax=1&term=gse[ETYP]`.
- The command stays raw-MCP only, and the typed search schema does not change.

### Docs

- New `docs/sources/ncbi-geo.md` in the pattern of `docs/sources/pubmed.md`. It covers what BioMCP exposes, the filters, the GDS-to-GSE rule, the shared `NCBI_API_KEY` budget, and the fact that sample files and expression matrices are not read.
- Add a row to `docs/sources/index.md` and a nav entry to `mkdocs.yml`.
- Add an NCBI GEO row to `docs/reference/data-sources.md` and `docs/reference/source-licensing.md` (NCBI tier 1; submitters keep rights to their data).
- Update `docs/user-guide/cli-reference.md` and add a `CHANGELOG.md` entry.

## Fixtures

Record under `testdata/sources/geo/`, trimmed to the members the decoder reads:
- `esearch_aml.json`: two UIDs and a `count` above two.
- `esummary_mixed.json`: one multi-platform GSE with two GPLs and two PMIDs, one GSE with no PMIDs, one GDS whose parent is the first GSE, one GDS whose parent is absent from the result, and one GPL record.
- `esearch_empty.json`: zero hits.

Tests (no network):

1. Helper construction: `esearch_plan` and `esummary_plan` with `db=gds` set `db`, `retmode`, `term`, `retstart`, `retmax`, `id`, and `api_key` exactly. A blank key adds no `api_key`, and an unknown `db` is rejected.
2. The existing `src/sources/pubmed/tests/construction.rs` cases pass unchanged.
3. Term builder: each flag alone and all four together produce the exact term. A term with neither `--disease` nor `--keyword` fails, and so does a value with an embedded quote.
4. Decoder on `esummary_mixed.json`: GSE rows carry prefixed platforms in order, the sample count, PMIDs, `supplementary_types`, `geo2r`, and `published` from `pdat`. A row with no `pdat` carries `published: null`. The GDS with a present parent is dropped and shows in that parent's `curated_datasets`. The GDS with an absent parent becomes a `GSE<gse>` row. The GPL record is dropped. A record missing `accession` yields an `Api` error naming its UID.
5. Search against a local fixture server under `BIOMCP_PUBMED_BASE`: the request log shows one ESearch and one ESummary with `db=gds`. The empty ESearch makes no ESummary request.
6. Rate limit: a GEO URL under the default base resolves to the `pubmed-eutils` policy key.
7. CLI parse: `--limit 0` and `--limit 51` fail. The help text and example block match the Design.

Executable spec `spec/entity/dataset.md`, backed by a fixture setup script in `spec/fixtures/` that serves the recorded files and exports `BIOMCP_PUBMED_BASE`:
- one JSON block: `search dataset --disease "acute myeloid leukemia" --json` gives `returned`, the first accession, and its `platforms` array.
- one Markdown block pins the `# Datasets:` heading and one `Platforms:` line.
- one error block covers a missing `--disease` and `--keyword`.

## Acceptance

`search dataset` exists with the exact help and examples. `list dataset` works. The seven test groups and three spec blocks pass. `make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA. PubMed request construction is unchanged. The new source page builds in the docs check.

## Boundaries

No `get dataset`, no sample list, no matrix or SOFT file read, no `elink` pivot from articles or cell lines, and no `search all` leg. Those belong to later tickets. No typed MCP tool change. No ranking, scoring, treatment labeling, or organism alias table. No change to the efetch callers or the rate-limit policy table. LINCS and other drug-response sources stay out.

## Complexity

- Contract score: 1 (one new search command with four filters and a fixed JSON shape)
- State and timing score: 0 (two bounded requests under an existing budget)
- Reach score: 2 (new source, helper refactor under PubMed, entity, CLI, list, health, docs)
- Proof score: 1 (request construction, decoder fixture, request log, spec)
- Cost of error score: 1 (the PubMed refactor touches the main article path)
- Total: 5
- Minimum level floor: none
- Final level: 3
- Reasons: new source and entity plus a shared-helper refactor under the article search path
- Selected model: level 3 implementer

## Review

- Design review: pending
- Code review: pending
