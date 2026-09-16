---
flow: build
priority: 4
deps: [1202]
---

# 1206: DepMap CRISPR dependency install and lookup

## Goal

`biomcp depmap sync` installs the DepMap model table and CRISPR gene effect matrix from the Figshare mirror. `biomcp get gene <symbol> dependency` and `biomcp gene dependency <symbol> [--lineage <text>]` then print the models with the lowest gene effect scores. `biomcp get cell-line <CVCL id> depmap` prints the matching DepMap model. BioMCP reports the numbers as published and adds no labels or scores of its own.

The motivating consumer is a hackathon team screening public GEO studies of drug-treated AML cell lines. The same lookup serves any agent asking which cancer models depend on a gene.

## Current Facts

- No DepMap code exists. `grep -rni depmap src docs` finds nothing, and `notes/biomcp-ideas-20260916.md` (idea 5, score 62) records the same. Ian ruled on 2026-09-16 that DepMap terms do not block this work because BioMCP is open-source and non-commercial.
- Measured 2026-09-16: `depmap.org/portal/api/download/files` returns a bot verification page to scripts. The Figshare API does not. `GET https://api.figshare.com/v2/articles/27993248` returns "DepMap 24Q4 Public", license CC BY 4.0, DOI `10.25452/figshare.plus.27993248.v1`, group 36075, and 73 files (30.83 GB).
- Needed files in that article: `Model.csv` (645,696 bytes, 2,105 rows, file 51065297) and `CRISPRGeneEffect.csv` (428,678,699 bytes, file 51064667). `CRISPRGeneDependency.csv` is 421,115,594 bytes and `ModelCondition.csv` is 219,100 bytes. `CRISPRGeneEffect.csv` holds 1,178 models by 17,916 genes. Its header starts with an empty cell, then `A1BG (1),A1CF (29974),...`. Each row starts with a `ModelID`.
- `Model.csv` has the columns `ModelID`, `CellLineName`, `StrippedCellLineName`, `OncotreeLineage`, `OncotreePrimaryDisease`, `OncotreeSubtype`, and `RRID` among 47. Example values: `ModelID` ACH-000362, `RRID` CVCL_2119, `CellLineName` MOLM-13, `OncotreeLineage` Myeloid, `OncotreePrimaryDisease` Acute Myeloid Leukemia. Sixty-one rows mention acute myeloid.
- Release discovery: `POST /v2/articles/search` with `{"group":36075,"search_for":":title: DepMap"}` returns 27993248 (24Q4, 2024-12-10), 25880521 (24Q2), and 24667905 (23Q4). The same search with no group returns no DepMap releases. 24Q4 is the newest release on Figshare as of 2026-09-16.
- The Figshare client exists. `FigshareClient` (`src/sources/figshare.rs:65`) has `article` (`:239`), `search_articles` (`:267`), and `download_file` (`:295`). `download_file` buffers the body in memory up to `MAX_FIGSHARE_FILE_BYTES` (`:17`), which equals `DEFAULT_MAX_BODY_BYTES`, 8 MiB (`src/sources/mod.rs:338`). A 429 MB file cannot pass through it.
- `FigshareArticleSearchRequest` (`src/sources/figshare.rs:109`) sends only `search_for` and `page_size`. It has no `group` field.
- `FigshareFileResponse` (`src/sources/figshare.rs:89`) reads a `md5` field. The live API returns `computed_md5` and `supplied_md5` and no `md5`, so `FigshareFile.md5` is always `None` today.
- The streaming install pattern is `CBioPortalDownloadClient::download_study_archive_to_path` (`src/sources/cbioportal_download.rs:97`). It checks `content_length` against a byte cap, writes to a unique temp path (`:294`), and installs only after validation (`:501`).
- The local source pattern is WHO IVD. It has a `BIOMCP_WHO_IVD_DIR` override with a `dirs::data_dir()/biomcp/who-ivd` default (`resolve_who_ivd_root`, `src/sources/who_ivd.rs:403`), a required file list (`:17`), a missing file helper (`:395`), a `who-ivd sync` command (`src/cli/system/mod.rs:73`, `src/cli/commands.rs:142`, `src/cli/system/dispatch.rs:363`), and a health probe (`src/cli/health/local.rs:192`).
- Gene sections are fixed names in `src/entities/gene.rs:244-260`, listed in `GENE_OUTCOME_KEYS` (`:261`) and `GENE_SECTION_NAMES` (`:291`). `parse_sections` (`:1129`) expands `all` to a fixed list that already leaves out `diagnostics`, `disgenet`, and `funding` (`:1168`).
- `GeneGetArgs.sections` is a trailing variadic argument (`src/cli/gene/mod.rs:40-42`). A section cannot take its own flag, so `get gene X dependency --lineage Y` would parse `--lineage` as a section name. Per-gene helpers with flags live under `GeneCommand` (`src/cli/gene/mod.rs:79`), for example `gene pathways <symbol> --limit`.
- The `cell-line` entity and its CVCL lookup do not exist yet. Ticket 1202 adds them.
- Source pages live in `docs/sources/*.md` with a row in `docs/sources/index.md` (`:41` is WHO IVD) and a nav entry in `mkdocs.yml` (`:66`). Terms live in `docs/reference/source-licensing.md` and `docs/reference/sources.json`.
- PRISM drug sensitivity is not in article 27993248. No file name contains PRISM, Drug, or Repurposing. Figshare holds it in separate articles: "PRISM Repurposing 20Q2 Dataset" (20564034) and "PRISM Repurposing 19Q4 Dataset" (9393293).

## Design

### Command shape

The scored review proposed `get gene <symbol> dependency --lineage`. That shape cannot parse because `get gene` sections take no flags. The ticket splits it the way `pathways` is split today:

```
biomcp depmap sync [--article <figshare id>]
biomcp get gene KMT2A dependency
biomcp gene dependency KMT2A --lineage Myeloid --limit 20
biomcp get cell-line CVCL_2119 depmap
```

### Figshare client changes

- Read `computed_md5` into `FigshareFile.md5` with `#[serde(alias = "computed_md5")]`. Keep `md5` for recorded fixtures.
- Add an optional `group: Option<u64>` to `FigshareArticleSearchRequest`, serialized only when set. Add `search_articles_in_group(group, search_for)`. Existing calls keep sending the same request body.
- Add `download_file_to_path(file, dest, max_bytes)`. It streams to a unique temp file, enforces the byte cap from `content_length` and from the running total, computes MD5 while streaming, and returns an error when the MD5 differs from `file.md5`. It reuses the provider URL policy (`validate_download_url`) and the accepted-status retry of `download_file`.

### Source module `src/sources/depmap.rs`

- Root: `BIOMCP_DEPMAP_DIR`, default `dirs::data_dir()/biomcp/depmap`, same shape as `resolve_who_ivd_root`.
- Release resolution: search group 36075 for `:title: DepMap`. Keep titles that match `^DepMap (\d{2})Q([1-4]) Public$`. Pick the highest year and quarter. `--article <id>` skips the search and must still match the title pattern.
- Files: `Model.csv` and `CRISPRGeneEffect.csv` only. Cap each download at 1 GiB. Nothing downloads on a lookup. The files download only on `depmap sync`.
- Index built at sync, in a temp directory that is renamed into place after every file validates:
  - `models.tsv`: `ModelID`, `RRID`, `CellLineName`, `OncotreeLineage`, `OncotreePrimaryDisease`, `OncotreeSubtype`, one row per `Model.csv` row.
  - `gene_effect.f32`: little-endian `f32`, one block of 1,178 values per gene in header order. Empty cells become `NaN`.
  - `gene_effect_axes.json`: the gene labels (`SYMBOL (Entrez)`) and model IDs in file order.
  - `manifest.json`: article id, title, DOI, license, `published_date`, and for each file its name, id, size, and MD5, plus `indexed_at`.
- The raw `CRISPRGeneEffect.csv` is deleted after the index validates. The install keeps `Model.csv`.
- A gene lookup seeks to one block and reads 1,178 values. It never opens the CSV.
- Gene matching: the symbol resolved by the existing gene lookup, matched exactly against the `SYMBOL` part of the label. No alias matching in the index.

### Surfaces

- `dependency` gene section: the 10 models with the lowest gene effect. Models with `NaN` are left out. Each row prints model ID, cell line name, RRID, lineage, primary disease, and gene effect to three decimals. The header prints the release title, the count of scored models, and the command to see more. The section stays out of `all`, the same as `diagnostics`.
- `gene dependency <symbol> [--lineage <text>] [--limit N] [--offset N]`: the same rows, filtered by case-insensitive exact match on `OncotreeLineage` when `--lineage` is set. Limit is 1 to 50 with a default of 10.
- `depmap` cell-line section: the `models.tsv` row whose `RRID` equals the CVCL id, plus whether the model has a row in the gene effect matrix. No per-model score list in this ticket.
- Not installed: the section outcome is unavailable with the message `DepMap data is not installed. Run \`biomcp depmap sync\`.` No network call happens.
- JSON: `dependency` carries `release`, `scored_models`, `total`, and `rows`. `depmap` carries `release` and `model` (or `null`).
- Health: one local probe reports installed, the release title, and missing files. It uses the same helper shape as `who_ivd_local_data_outcome`. The data has no stale timer.

### Docs

- `docs/sources/depmap.md`: what BioMCP reads, the four commands, the install size, and the fact that BioMCP reports gene effect values as published.
- Rows in `docs/sources/index.md`, `mkdocs.yml`, `docs/reference/source-licensing.md`, and `docs/reference/sources.json` (tier 1, CC BY 4.0, reviewed 2026-09-16, with a note that DepMap asks for citation of the release DOI).
- Help lists for gene sections and the `list gene` page gain `dependency`.

## Fixtures

- `testdata/sources/depmap/`: a recorded Figshare search response cut to the three DepMap articles plus one unrelated title, an article response for a fixture article with two files, a `Model.csv` with 6 rows (one without RRID), and a `CRISPRGeneEffect.csv` with 5 models by 4 genes that includes one empty cell. The MD5 values in the article fixture match the fixture files.
- A spec fixture script serves these files from a local HTTP server through `BIOMCP_FIGSHARE_BASE` and a download URL on the same host, then runs `depmap sync` into a temp `BIOMCP_DEPMAP_DIR`.

## Acceptance

Focused Rust tests:

1. `computed_md5` in a file response fills `FigshareFile.md5`.
2. A search with a group sends `group` in the body. A search without a group sends today's body byte for byte.
3. Release resolution picks 24Q4 from the recorded search and ignores the unrelated title. An `--article` whose title does not match fails.
4. `download_file_to_path` rejects a body over the cap and a body whose MD5 differs, and leaves no file at the destination in either case.
5. Index build on the fixture CSV round-trips every value, maps the empty cell to `NaN`, and writes the manifest with article id and MD5 values.
6. A failed validation leaves an earlier install untouched.
7. Gene lookup returns fixture rows in ascending gene effect order, excludes `NaN`, applies `--lineage` case-insensitively, and pages with limit and offset.
8. An unknown symbol returns an empty row list with `total: 0`.
9. The cell-line section finds the model by RRID and returns `null` for a CVCL id with no row.
10. With no install, both sections report unavailable with the exact message and make no request.
11. `all` does not include `dependency`.

Executable spec (`spec/entity/depmap.md`): one block runs `depmap sync` against the fixture and checks the manifest release title. One block pins the Markdown table of `get gene <fixture gene> dependency`. One JSON block checks `gene dependency <fixture gene> --lineage <fixture lineage> --json` row count and first model ID. One JSON block checks `get cell-line <fixture CVCL> depmap --json` model ID.

`make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA. No test touches the network.

## Out of scope

- PRISM drug sensitivity. It lives in separate Figshare articles (20564034, 9393293) and becomes a follow-up ticket.
- `CRISPRGeneDependency.csv`, `ModelCondition.csv`, and the omics expression and mutation files (339 to 507 MB each).
- A per-model list of top dependencies on the cell-line card.
- The depmap.org download API, automatic refresh, and any stale timer.
- Any threshold, "essential" label, ranking beyond sort order, or cross-release comparison.
- LINCS, and any change to the team repository that motivated this work.

## Complexity

- Contract score: 2 (new sync command, new gene section, new gene helper with flags, new cell-line section, two Figshare client changes)
- State and timing score: 2 (large streamed download, atomic install, binary index)
- Reach score: 2 (Figshare client, new source, gene entity and CLI, cell-line entity, health, docs)
- Proof score: 1 (recorded fixtures, round-trip index test, spec against a local server)
- Cost of error score: 1 (a bad index gives wrong numbers until the next sync)
- Total: 8
- Final level: 3
- Selected model: build tier, high reasoning

## Review

- Design review: pending
- Code review: pending
