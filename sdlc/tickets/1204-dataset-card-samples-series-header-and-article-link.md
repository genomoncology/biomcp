---
flow: build
priority: 3
deps: [1203]
---

# 1204: Dataset card, samples, series header, and article link

## Goal

An agent lists the samples of one GEO series, reads the raw series and sample header lines of that series, and finds the GEO series linked to a PubMed article. Three commands deliver this:

```
biomcp get dataset GSE982
biomcp dataset samples GSE982 --limit 25
biomcp get dataset GSE982 series
biomcp article datasets 12345678
```

The motivating consumer is a hackathon team screening public GEO studies of drug-treated AML cell lines. The team needed the sample list and the per-sample characteristics lines before it downloaded any expression data.

## Current Facts

- Ticket 1203 adds the `dataset` entity with `search dataset` and `get dataset <GSE>` from E-utilities `esummary` with `db=gds`. This ticket adds sections and one pivot on top of it. The entity name `dataset` avoids `study`, which already names the cBioPortal DataHub command (`src/cli/commands.rs:78`).
- No code calls `elink` today (`grep -rn elink src` finds nothing). The PubMed client wraps only `esearch` and `esummary` for PubMed (`src/sources/pubmed.rs:338`, `src/sources/pubmed.rs:410`) against `BIOMCP_PUBMED_BASE` (`src/sources/pubmed.rs:12-14`). The generic E-utilities helper that 1203 depends on supplies `elink`.
- The shared middleware reads the whole response body into memory before it returns (`ResponseBodyLimitMiddleware`, `src/sources/mod.rs:778`, buffer loop at `src/sources/mod.rs:833`). The default cap is 8 MiB (`DEFAULT_MAX_BODY_BYTES`, `src/sources/mod.rs:338`). A reader that stops at a marker cannot use that client. `ordinary_middleware_client_for_base` (`src/sources/ordinary_url_policy.rs:199`) builds a client that keeps the provider URL policy and adds no body buffering. The DataHub download client uses it (`src/sources/cbioportal_download.rs:277`) and reads chunks one at a time (`:140`). The matrix reader follows that pattern.
- `flate2` is already a dependency (`Cargo.toml:94`). GTR decodes gzip with a cap on expanded bytes (`parse_test_version_records_with_limits`, `src/sources/gtr.rs:444`). GTR also reads `ftp.ncbi.nlm.nih.gov` over HTTPS with an env override per URL (`src/sources/gtr.rs:14-19`).
- Entities parse named sections with a local `parse_sections` (`src/entities/pathway.rs:121`). Article helpers live in `ArticleCommand` (`src/cli/article/mod.rs:308`). `article entities <pmid>` (`:328`) is the model for a PMID-keyed pivot with `--limit`.
- The source registry already has an `ncbi-e-utilities` row (`docs/reference/sources.json:979`). Source pages live under `docs/sources/`, and 1203 adds the GEO page.

GEO facts the design relies on (to confirm against the recorded fixtures):

- `esummary` with `db=gds` returns, per series, `accession`, `entrytype`, `gpl` (platform numbers separated by `;`), `n_samples`, and a `samples` array of `{accession, title}`.
- Series matrix files live at `https://ftp.ncbi.nlm.nih.gov/geo/series/<prefix>/<GSE>/matrix/`. The prefix replaces the last three digits with `nnn`: `GSE100446` gives `GSE100nnn`, and both `GSE982` and `GSE14` give `GSEnnn`. A single-platform series publishes `<GSE>_series_matrix.txt.gz`. A multi-platform series publishes one `<GSE>-GPL<n>_series_matrix.txt.gz` per platform.
- The file is gzip text. Header lines start with `!Series_` or `!Sample_`. A `!Sample_` line holds one tab-separated, quoted column per sample. Characteristics appear as repeated `!Sample_characteristics_ch1` lines with `key: value` cells. The expression table starts at `!series_matrix_table_begin`. Sequencing series have an empty table.
- `elink` with `dbfrom=pubmed&db=gds` returns GDS UIDs of mixed record types (series, platforms, samples, curated DataSets).

## Design

### `get dataset <id>` card

`get dataset <id>` accepts `geo:GSE…` or bare `GSE…` and renders the 1203 row for that series from one `esummary` call: ID, title, organisms, series types, platforms, sample count, PMIDs, supplementary types, `geo2r`, and summary. Sections `publications` (the PMIDs with `get article` next commands) and `links` (the GEO page, BioProject, and SuperSeries/SubSeries relations from `esummary`) read the same response.

`all` means the bounded metadata sections that make no file request: the card, `publications`, and `links`. It never includes `series` or 1207's `products` and `assets`.

### `dataset samples <id>` helper

`biomcp dataset samples <id> --limit 25 --offset 0` renders the `samples` array from the same `esummary` call. It is a helper, like `article entities`, because series can have hundreds of samples and need paging. `--limit` accepts 1 to 500. It makes no extra request and reads no file. Output is a table of sample accession and title in provider order, plus `n_samples`. JSON adds `samples: [{accession, title}]`. Every sample accession is printed as the bare `GSM` string.

### `series` section

`get dataset <GSE> series` reads the matrix header. It is opt-in. The default card never reads the file, and `all` does not include it.

- The platform list comes from the `esummary` `gpl` field. One platform maps to `<GSE>_series_matrix.txt.gz`. Several platforms map to one `<GSE>-GPL<n>_series_matrix.txt.gz` each, in `gpl` order, capped at 10 files. This avoids parsing the FTP directory listing.
- A new `src/sources/geo_matrix.rs` builds the URL from base `https://ftp.ncbi.nlm.nih.gov/geo` with override `BIOMCP_GEO_FTP_BASE`. `series_prefix("GSE100446") == "GSE100nnn"`, and `series_prefix` returns `GSEnnn` for any series number under 1000.
- The reader uses an unbuffered client. It feeds response chunks through a streaming gzip decoder and a line reader, and it stops reading and drops the response at the first `!series_matrix_table_begin` line. It never reads a byte past the marker line.
- Caps: 8 MiB compressed bytes and 16 MiB expanded bytes per file, and 64 KiB per line. A cap hit before the marker returns a `BodyLimit` error naming the file. A file that ends without the marker is a provider error. HTTP 404 for one platform file becomes a per-platform "matrix file not found" note, and the other platforms still render.
- Parsing keeps every `!Series_` line as `{key, values}` in file order, with repeated keys preserved. It keeps every `!Sample_` line as a key with one cell per sample column, unquoted. Characteristics cells stay raw strings such as `cell line: MOLM-13`. The parser does not split, merge, rename, or label them. Any column count that differs from the `!Sample_geo_accession` column count is a provider error.
- JSON: `series: {platforms: [{platform, file, series_lines: [...], samples: [{accession, fields: [{key, value}]}]}]}`. Markdown prints the series lines as a key and value list, then one block per sample with its raw lines in file order.

### `article datasets <pmid>` pivot

`ArticleCommand::Datasets { pmid, limit }` with `--limit` 1 to 50, default 10.

- The command calls `elink` with `dbfrom=pubmed&db=gds&id=<pmid>`, then calls `esummary` with `db=gds` on the linked UIDs, then keeps rows whose `entrytype` is `GSE`. It renders the same compact row that `search dataset` renders (accession, title, organism, platforms, sample count).
- No links gives an empty result with the note `No GEO series linked to PMID <pmid>.` and exit 0.
- Each row carries the next command `biomcp dataset samples geo:<GSE>`. The empty result suggests `biomcp search dataset -k <pmid>` only if 1203 ships `-k`.
- Help text: `Find GEO series linked to one PubMed article`, with two examples and `See also: biomcp list article`.

### Docs

- Add the card, `publications`, `links`, `series`, the `dataset samples` helper, and the `article datasets` row to the GEO source page that 1203 adds under `docs/sources/`. State that `series` reads only the header of the matrix file and stops at the table marker.
- Add the matrix file host to the GEO row in `docs/reference/data-sources.md`. Update `sources.json` and `source-licensing.md` only if 1203 did not already cover GEO FTP.
- Add the sections, the helper, and the pivot to `biomcp list dataset` and `biomcp list article`.

## Acceptance

Fixtures under `testdata/sources/geo/`, all small and recorded:

- `esummary` JSON for one single-platform series with three samples and for one two-platform series.
- Two gzip matrix header files for the two-platform series, trimmed to a few samples. Each has a table marker followed by rows the reader must not reach. One file has an empty table. One single-platform file is also needed.
- `elink` JSON for one PMID that links to a series, a platform, and a sample, plus one PMID with no links.

Rust tests, all offline:

1. `series_prefix` returns `GSE100nnn`, `GSEnnn`, and `GSEnnn` for `GSE100446`, `GSE982`, and `GSE14`, and the file names follow the one-platform and multi-platform rules.
2. `dataset samples` renders the fixture samples in provider order, pages with `--offset`, and sends no matrix request. `get dataset GSE982` and `get dataset geo:GSE982` render the same card.
3. The reader stops at the marker. A test reader panics if it is asked for bytes past the marker line, and the parse still succeeds.
4. A header under the cap succeeds when the whole file exceeds the cap. The test injects a small cap.
5. A header that exceeds the compressed or expanded cap before the marker returns `BodyLimit`. A missing marker returns a provider error. A column count mismatch returns a provider error.
6. Characteristics cells come back byte-identical to the fixture strings, with repeated keys kept in order.
7. A two-platform series renders both platforms. A 404 on one platform keeps the other and adds the note.
8. `article datasets` keeps only `GSE` rows from the mixed `elink` fixture and returns the empty note for the unlinked PMID.
9. `get dataset <GSE>` and `get dataset <GSE> all` send no matrix request.

Spec (`spec/entity/dataset.md`, served by a local fixture server like `spec/fixtures/setup-provider-contract-spec-fixture.sh`, with `BIOMCP_PUBMED_BASE` and `BIOMCP_GEO_FTP_BASE` pointed at it): one block for `get dataset <GSE>`, one for `dataset samples <GSE>`, one JSON block for `series` pinning a raw characteristics cell, and one block for `article datasets <pmid>`. The request log shows no matrix request for `samples`.

`make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA.

## Out of scope

- Expression values, the matrix table, supplementary files, full file download, and probe to gene mapping. Ticket 1207 lists where values live.
- Treatment and control labeling, parsing characteristics into typed fields, and any grouping or scoring of samples.
- The FTP directory listing, SOFT and MINiML formats, and GSM or GPL records as their own entities.
- `cell-line datasets` and LINCS.
- A new MCP tool. The generic `get` tool and the raw article helper path reach the new surfaces.

## Decisions

These are open to Ian's overturn:

- The section is named `series` because it prints the series matrix header. `all` includes it.
- Platform files come from the `esummary` `gpl` field, so the reader never parses the FTP listing.
- Caps are 8 MiB compressed, 16 MiB expanded, 64 KiB per line, and 10 platform files.
