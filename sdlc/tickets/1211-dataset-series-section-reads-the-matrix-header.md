---
flow: build
priority: 3
deps: [1204]
---

# 1211: Dataset series section reads the matrix header

## Goal

`biomcp get dataset <id> series` streams each series matrix file of a GEO series, stops at the value table, and prints the raw series and sample header lines. The reader is shared: ticket 1207 calls it to peek two lines into the table.

```
biomcp get dataset GSE982 series
biomcp --json get dataset geo:GSE995 series
```

## Current Facts

- The shared middleware reads the whole response body into memory before it returns (`ResponseBodyLimitMiddleware`, `src/sources/mod.rs:778`, buffer loop at `src/sources/mod.rs:833`). The default cap is 8 MiB (`DEFAULT_MAX_BODY_BYTES`, `src/sources/mod.rs:338`). A reader that stops at a marker cannot use that client. `ordinary_middleware_client_for_base` (`src/sources/ordinary_url_policy.rs:199`) builds a client that keeps the provider URL policy and adds no body buffering. The DataHub download client uses it (`src/sources/cbioportal_download.rs:277`) and reads chunks one at a time (`:141`).
- `flate2` is already a dependency (`Cargo.toml:94`). GTR decodes gzip with a cap on expanded bytes (`parse_test_version_records_with_limits`, `src/sources/gtr.rs:444`) and reads `ftp.ncbi.nlm.nih.gov` over HTTPS with an env override (`src/sources/gtr.rs:14-19`).
- Series matrix files live at `https://ftp.ncbi.nlm.nih.gov/geo/series/<prefix>/<GSE>/matrix/`. The prefix replaces the last three digits with `nnn`: `GSE100446` gives `GSE100nnn`, and both `GSE982` and `GSE14` give `GSEnnn`. A single-platform series publishes `<GSE>_series_matrix.txt.gz`. A multi-platform series publishes one `<GSE>-GPL<n>_series_matrix.txt.gz` per platform. Measured 2026-09-16 on 588 AML series: listing the folder found every multi-platform file that a single-name download missed.
- The file is gzip text. Header lines start with `!Series_` or `!Sample_`. A `!Sample_` line holds one tab-separated, quoted column per sample. Characteristics appear as repeated `!Sample_characteristics_ch1` lines with `key: value` cells. The value table starts at `!series_matrix_table_begin`. Sequencing series have an empty table.

## Design

- The platform list comes from the `esummary` `gpl` field. One platform maps to `<GSE>_series_matrix.txt.gz`. Several platforms map to one `<GSE>-GPL<n>_series_matrix.txt.gz` each, in `gpl` order, capped at 10 files. The reader never parses the FTP listing.
- New `src/sources/geo_matrix.rs` builds URLs from base `https://ftp.ncbi.nlm.nih.gov/geo` with override `BIOMCP_GEO_FTP_BASE`. `series_prefix` returns `GSEnnn` for any series number under 1000.
- `read_header(url, lines_after_marker)` uses the unbuffered client. It feeds chunks through a streaming gzip decoder and a line reader. It reads `lines_after_marker` lines after `!series_matrix_table_begin`, default 0, then drops the response. It never reads a byte past that.
- Caps: 8 MiB compressed and 16 MiB expanded per file, and 64 KiB per line. A cap hit before the marker returns `BodyLimit` naming the file. A file that ends without the marker is a provider error. HTTP 404 for one platform becomes a per-platform "matrix file not found" note, and the other platforms still render.
- Parsing keeps every `!Series_` line as `{key, values}` in file order, with repeated keys preserved. It keeps every `!Sample_` line as a key with one unquoted cell per sample. Characteristics cells stay raw strings such as `cell line: MOLM-13`. The parser does not split, merge, rename, or label them. A column count that differs from the `!Sample_geo_accession` count is a provider error.
- JSON: `series: {platforms: [{platform, file, series_lines: [...], samples: [{accession, fields: [{key, value}]}]}]}`. Markdown prints the series lines as a key and value list, then one block per sample with its raw lines in file order.
- `series` is opt-in. `all` never includes it, because it reads a file.
- Docs: add the section to the GEO source page and `biomcp list dataset`, stating that it reads only the header and stops at the table marker. Add the matrix host to the GEO row in `docs/reference/data-sources.md`.

## Acceptance

Fixtures under `testdata/sources/geo/`: two trimmed gzip matrix files for one two-platform series and one single-platform file. Each has a table marker followed by rows. One file has an empty table.

1. `series_prefix` returns `GSE100nnn`, `GSEnnn`, and `GSEnnn` for `GSE100446`, `GSE982`, and `GSE14`, and file names follow the one-platform and multi-platform rules.
2. With the default, a test reader panics if asked for bytes past the marker line, and the parse succeeds. With `lines_after_marker = 2`, it panics past the second line.
3. A header under the cap succeeds when the whole file exceeds the cap. The test injects a small cap.
4. A cap hit before the marker returns `BodyLimit`. A missing marker and a column count mismatch return provider errors.
5. Characteristics cells come back byte-identical to the fixture, with repeated keys in order.
6. A two-platform series renders both platforms. A 404 on one keeps the other and adds the note.
7. `get dataset <GSE> all` makes no matrix request.
8. Spec `spec/entity/dataset.md` gains one JSON block for `series` pinning a raw characteristics cell, with `BIOMCP_GEO_FTP_BASE` pointed at the fixture server.

## Out of scope

- The value table itself, supplementary files, and download.
- Typed characteristics, labeling, and grouping.

## Decisions

Open to Ian's overturn: caps are 8 MiB compressed, 16 MiB expanded, 64 KiB per line, and 10 platform files.
