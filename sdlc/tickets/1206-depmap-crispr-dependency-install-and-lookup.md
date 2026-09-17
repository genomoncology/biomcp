---
flow: build
priority: 4
deps: [1202]
---

# 1206: DepMap CRISPR dependency install and lookup

## Goal

`biomcp depmap sync` installs the DepMap model table and CRISPR gene effect matrix from the Figshare mirror. `biomcp get gene <symbol> dependency` and `biomcp gene dependency <symbol> [--lineage <text>]` then print the models with the lowest gene effect scores. `biomcp get cell-line <CVCL id> depmap` prints the matching DepMap models and says by name when a line has no CRISPR screen. `biomcp cell-line dependency <CVCL id>` prints the genes with the lowest gene effect scores for that line. Every output names the DepMap release and its date. BioMCP reports the numbers as published and adds no labels or scores of its own.

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
- `is_allowed_mcp_command` (`src/mcp/shell.rs:472`) matches every `Commands` and subcommand variant by name and has no wildcard arm. `Commands::WhoIvd { .. }` is rejected whole (`src/mcp/shell.rs:564`) because the family writes workstation-local state. It is the model for `depmap sync`.
- `GeneGetArgs.sections` is a trailing variadic argument (`src/cli/gene/mod.rs:40-42`). A section cannot take its own flag, so `get gene X dependency --lineage Y` would parse `--lineage` as a section name. Per-gene helpers with flags live under `GeneCommand` (`src/cli/gene/mod.rs:79`), for example `gene pathways <symbol> --limit`.
- The `cell-line` entity and its CVCL lookup do not exist yet. Ticket 1202 adds them.
- Source pages live in `docs/sources/*.md` with a row in `docs/sources/index.md` (`:41` is WHO IVD) and a nav entry in `mkdocs.yml` (`:66`). Terms live in `docs/reference/source-licensing.md` and `docs/reference/sources.json`.
- Survey of 2026-09-17 (workspace experiment 204): the `RRID` column matched all ten AML and leukemia test lines. The join is not one-to-one. `Model.csv` has 134 rows with no RRID, and 4 RRIDs map to two models each (CVCL_0041 maps to ACH-000833 and ACH-001189).
- HL-60 (ACH-000002) and KG-1 (ACH-000386) have no CRISPR screen in 24Q4. `CRISPRInferredModelGrowthRate.csv` (41 KB) lists the screened models, and the gene effect matrix row list gives the same answer.
- 24Q4 was still the newest Figshare release on 2026-09-17, 21 months after its 2024-12-10 date. The depmap.org portal download API still answers scripts with "We're verifying you're a person".
- PRISM drug sensitivity is not in article 27993248. No file name contains PRISM, Drug, or Repurposing. Figshare holds it in separate articles: "PRISM Repurposing 20Q2 Dataset" (20564034) and "PRISM Repurposing 19Q4 Dataset" (9393293).

## Design

### Command shape

The scored review proposed `get gene <symbol> dependency --lineage`. That shape cannot parse because `get gene` sections take no flags. The ticket splits it the way `pathways` is split today:

```
biomcp depmap sync [--article <figshare id>]
biomcp get gene KMT2A dependency
biomcp gene dependency KMT2A --lineage Myeloid --limit 20
biomcp get cell-line CVCL_2119 depmap
biomcp cell-line dependency CVCL_2119 --limit 20
```

### Figshare client changes

- Read `computed_md5` into `FigshareFile.md5` with `#[serde(alias = "computed_md5")]`. Keep `md5` for recorded fixtures.
- Add an optional `group: Option<u64>` to `FigshareArticleSearchRequest`, serialized only when set. Add `search_articles_in_group(group, search_for)`. Existing calls keep sending the same request body.
- Add `download_file_to_path(file, dest, max_bytes)`. It streams to a unique temp file, enforces the byte cap from `content_length` and from the running total, computes MD5 while streaming, and returns an error when the MD5 differs from `file.md5`. It reuses the provider URL policy (`validate_download_url`) and the accepted-status retry of `download_file`.

### Source module `src/sources/depmap.rs`

- Root: `BIOMCP_DEPMAP_DIR`, default `dirs::data_dir()/biomcp/depmap`, same shape as `resolve_who_ivd_root`.
- Release resolution: search group 36075 for `:title: DepMap`. Keep titles that match `^DepMap (?<release_tag>\d{2}Q[1-4]) Public$`. Pick the highest year and quarter. `--article <id>` skips the search and must still match the title pattern. `release_tag` is the captured group, `24Q4` for the article titled `DepMap 24Q4 Public`. It is stored in `manifest.json` and is the only release string any output prints.
- Files: `Model.csv` and `CRISPRGeneEffect.csv` only. Cap each download at 1 GiB. Nothing downloads on a lookup. The files download only on `depmap sync`.
- Index built at sync, in a temp directory that is renamed into place after every file validates:
  - `models.tsv`: `ModelID`, `RRID`, `CellLineName`, `OncotreeLineage`, `OncotreePrimaryDisease`, `OncotreeSubtype`, one row per `Model.csv` row.
  - `gene_effect.f32`: little-endian `f32`, one block of `model_count` values per gene in header order, where `model_count` is the number of models in the installed file, 1,178 in 24Q4. Empty cells become `NaN`.
  - `gene_effect_axes.json`: the gene labels (`SYMBOL (Entrez)`) and model IDs in file order. `model_count` is the length of the model ID list, and every reader takes the block size from it rather than from a constant.
  - `manifest.json`: article id, title, `release_tag`, DOI, license, `published_date`, `model_count`, and for each file its name, id, size, and MD5, plus `indexed_at`.
- The raw `CRISPRGeneEffect.csv` is deleted after the index validates. The install keeps `Model.csv`.
- A gene lookup seeks to one block and reads `model_count` values. It never opens the CSV.
- Gene matching: the symbol resolved by the existing gene lookup, matched exactly against the `SYMBOL` part of the label. No alias matching in the index.

### Surfaces

- `dependency` gene section: the 10 models with the lowest gene effect. Models with `NaN` are left out. Each row prints model ID, cell line name, RRID, lineage, primary disease, and gene effect to three decimals. The header prints `release_tag`, the count of scored models, and the command to see more. The section stays out of `all`, the same as `diagnostics`.
- `gene dependency <symbol> [--lineage <text>] [--limit N] [--offset N]`: the same rows, filtered by case-insensitive exact match on `OncotreeLineage` when `--lineage` is set. Limit is 1 to 50 with a default of 10.
- `depmap` cell-line section: one key of the same name in the `section_outcomes` registry that ticket 1202 builds, holding a list of every `models.tsv` row whose `RRID` equals the CVCL id, usually one and sometimes two. Each entry says whether the model has a row in the gene effect matrix. A model with no row prints `<name> (<ModelID>) has no CRISPR screen in <release_tag>`. An empty list prints `No DepMap model lists <CVCL id> as its RRID in <release_tag>`.
- `cell-line dependency <CVCL id> [--limit N] [--offset N]`: the genes with the lowest gene effect for the matched models, one table per model, `NaN` left out, same limits as `gene dependency`. It lives in the `CellLineCommand` group that ticket 1205 adds, or adds that group if 1205 has not landed. A line with no screen prints the no-screen message and no table. The model axis is already in `gene_effect_axes.json`, so the lookup scans the index once and reads one value per gene block at the model's offset. The index needs no new file.
- Not installed: the section outcome is unavailable with the message `DepMap data is not installed. Run \`biomcp depmap sync\`.` No network call happens.
- Release on every output: every Markdown output prints `DepMap <release_tag>, published <published_date>` from `manifest.json`, and every JSON output carries `release: {tag, title, published_date, doi}`. `title` is the Figshare title, `DepMap 24Q4 Public`, and it is never the release string an output prints.
- `data_as_of` on every output: every JSON payload carries `data_as_of` and `data_as_of_kind: "release"`, with `data_as_of` set to `<release_tag> (<published_date>)`, for example `24Q4 (2024-12-10)` from the newest Figshare release measured on 2026-09-17. The value comes from `manifest.json`, so it is the release the user installed and never a hard-coded string.
- Attribution on every output: every Markdown output ends with `DepMap <data_as_of>, CC BY 4.0. Cite the release DOI <doi>.`, which reads `DepMap 24Q4 (2024-12-10), CC BY 4.0. Cite the release DOI 10.25452/figshare.plus.27993248.v1.` on the measured release. The release and DOI parts come from `manifest.json`, and the release part is the `data_as_of` value.
- Bot checks: `depmap.org/portal/api/download/files` answers scripts with a human verification page, measured 2026-09-16 and again on 2026-09-17. BioMCP never calls it. If any request in this ticket returns HTTP 200 with an HTML body where JSON or CSV bytes were expected, the command reports the URL, names the file, and stops. It never retries through the check, never rewrites the request, and never scrapes the page. `depmap sync` leaves any earlier install untouched when that happens.
- JSON: `dependency` carries `release`, `scored_models`, `total`, and `rows`. `depmap` carries `release` and `models` (a list, possibly empty), each with `screened: bool`. `cell-line dependency` carries `release` and one `{model_id, screened, total, rows}` entry per model.
- MCP arms: `is_allowed_mcp_command` (`src/mcp/shell.rs:472`) is an exhaustive match with no wildcard, so every new variant is classified here or the build fails. `Commands::Depmap` with the `Sync` subcommand is rejected, following `Commands::WhoIvd { .. }` (`src/mcp/shell.rs:564`), because it downloads and writes workstation-local state. `GeneCommand::Dependency` and `CellLineCommand::Dependency` are allowed, because they read the installed index and reveal no local path. The `dependency` gene section and the `depmap` cell line section reach MCP through the typed `get` tool with no arm change.
- Health: one local probe reports installed, the release title, and missing files. It uses the same helper shape as `who_ivd_local_data_outcome`. The data has no stale timer.

### Docs

- `docs/sources/depmap.md`: what BioMCP reads, the four commands, the install size, and the fact that BioMCP reports gene effect values as published.
- Rows in `docs/sources/index.md`, `mkdocs.yml`, `docs/reference/source-licensing.md`, and `docs/reference/sources.json` (tier 1, CC BY 4.0, reviewed 2026-09-16, with a note that DepMap asks for citation of the release DOI).
- The index description says "one block of `model_count` values per gene", with the count read from `gene_effect_axes.json`. It never hard-codes 1,178, which belongs to the 24Q4 file.
- Help lists for gene sections and the `list gene` page gain `dependency`.

## Fixtures

- `testdata/sources/depmap/`: a recorded Figshare search response cut to the three DepMap articles plus one unrelated title, an article response for a fixture article with two files, a `Model.csv` with 6 rows (one without RRID), and a `CRISPRGeneEffect.csv` with 5 models by 4 genes that includes one empty cell. The MD5 values in the article fixture match the fixture files.
- The fixture `Model.csv` gives two rows the same RRID, and one model with an RRID has no row in the fixture matrix.
- A spec fixture script serves these files from a local HTTP server through `BIOMCP_FIGSHARE_BASE` and a download URL on the same host, then runs `depmap sync` into a temp `BIOMCP_DEPMAP_DIR`.

## Acceptance

Focused Rust tests:

1. `computed_md5` in a file response fills `FigshareFile.md5`.
2. A search with a group sends `group` in the body. A search without a group sends today's body byte for byte.
3. Release resolution picks 24Q4 from the recorded search and ignores the unrelated title. An `--article` whose title does not match fails.
4. `download_file_to_path` rejects a body over the cap and a body whose MD5 differs, and leaves no file at the destination in either case.
5. Index build on the fixture CSV round-trips every value, maps the empty cell to `NaN`, and writes the manifest with article id, `release_tag`, `model_count`, and MD5 values. The reader takes the block size from `model_count` in `gene_effect_axes.json`, so the 5-model fixture reads without a code change.
6. A failed validation leaves an earlier install untouched.
7. Gene lookup returns fixture rows in ascending gene effect order, excludes `NaN`, applies `--lineage` case-insensitively, and pages with limit and offset.
8. An unknown symbol returns an empty row list with `total: 0`.
9. The cell-line section returns both models for the shared RRID, an empty list for a CVCL id with no row, and `screened: false` with the no-screen message for the unscreened model.
10. `cell-line dependency` returns rows in ascending gene effect order for a screened model and the no-screen message for the unscreened one.
11. Every Markdown output in these tests contains `release_tag` and the published date, never the Figshare title, and ends with the CC BY 4.0 attribution line naming the release DOI. Every JSON output carries `release.published_date`, `data_as_of` in the form `<release_tag> (<published_date>)`, and `data_as_of_kind: "release"`.
12. With no install, the sections and the helpers report unavailable with the exact message and make no Figshare or DepMap request. The install check runs before the gene symbol resolves, so `get gene KMT2A dependency` with no install makes no request at all. Every other lookup resolves its symbol through the existing gene lookup first, which is a network request and is unchanged by this ticket.
13. `all` does not include `dependency`.
14. An HTML body served with HTTP 200 where the Figshare article JSON or a CSV file was expected fails, names the URL and the file, leaves no install, and leaves an earlier install untouched.
15. The MCP shell rejects `depmap sync` and allows `gene dependency` and `cell-line dependency`.

Executable spec (`spec/entity/depmap.md`): one block runs `depmap sync` against the fixture and checks the manifest release title. One block pins the Markdown table of `get gene <fixture gene> dependency`. One JSON block checks `gene dependency <fixture gene> --lineage <fixture lineage> --json` row count and first model ID. One JSON block checks `get cell-line <fixture shared CVCL> depmap --json` for two model IDs. One Markdown block checks the no-screen message and the release line.

`make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA. No test touches the network.

## Out of scope

- PRISM drug sensitivity. PharmacoDB already serves PRISM through ticket 1205, so no PRISM follow-up ticket is planned.
- `CRISPRGeneDependency.csv`, `ModelCondition.csv`, and the omics expression and mutation files (339 to 507 MB each).
- A per-model dependency list inside the `get cell-line` card. The `cell-line dependency` helper covers it.
- The depmap.org download API, automatic refresh, and any stale timer.
- Any threshold, "essential" label, ranking beyond sort order, or cross-release comparison.
- LINCS, and any change to the team repository that motivated this work.

## Decisions

These follow the 2026-09-17 source survey. Ian can overturn any of them.

- The 21-month release limit is recorded here, in this ticket's Decisions block, and not in an ADR. Survey finding D14 asked for an ADR. The repo writes an ADR under `sdlc/planning/adr/` when a decision reverses a recorded product boundary, which is what ADR 0001 does. This decision sets one source's freshness expectation and travels with the ticket that implements it. Both places are durable and Ian can move it.
- BioMCP reads DepMap from the Figshare mirror only. The depmap.org portal download API sits behind a bot check, so BioMCP does not call it. The newest Figshare release is 24Q4 (2024-12-10), 21 months old on 2026-09-17, and BioMCP names that limit on every output instead of hiding it. When DepMap publishes a newer release to Figshare group 36075, `depmap sync` picks it up with no code change.
- The `depmap` section returns a list because the RRID join is not one-to-one.
- The per-line dependency view fits in this ticket because it reuses the same index.
- PRISM is served through PharmacoDB (ticket 1205).

## Complexity

- Contract score: 2 (new sync command, new gene section, new gene helper with flags, new cell-line section and helper, two Figshare client changes)
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
