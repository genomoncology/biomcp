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
- Series matrix files live at `https://ftp.ncbi.nlm.nih.gov/geo/series/<prefix>/<GSE>/matrix/`. The prefix replaces the last three digits with `nnn`: `GSE100446` gives `GSE100nnn`, and both `GSE982` and `GSE14` give `GSEnnn`. A single-platform series publishes `<GSE>_series_matrix.txt.gz`. A multi-platform series publishes one `<GSE>-GPL<n>_series_matrix.txt.gz` per platform. Measured 2026-09-16 on 585 AML series (774 files): listing the folder found every multi-platform file that a single-name download missed.
- The `gds` ESummary record carries `ftplink`, the series FTP folder. The live GSE982 record read `ftplink: "ftp://ftp.ncbi.nlm.nih.gov/geo/series/GSEnnn/GSE982/"` on 2026-09-17, which is the `series_prefix` rule spelled out by the provider.
- `!Series_status` is never the bare word `Public`. The measured GSE100446 header reads `!Series_status "Public on Sep 11 2019"`. The submission and update lines read `"Jun 25 2017"` and `"Nov 04 2019"`.
- The file is gzip text. Header lines start with `!Series_` or `!Sample_`. A `!Sample_` line holds one tab-separated, quoted column per sample. Characteristics appear as repeated `!Sample_characteristics_ch1` lines with `key: value` cells. The value table starts at `!series_matrix_table_begin`. Sequencing series have an empty table.
- Measured 2026-09-16 on 774 real matrix files (experiment 203): 33 files have header lines over 64 KiB. The longest line is 1,132,383 bytes. The largest header is 8,364,887 bytes (GSE256354-GPL15520). The largest compressed file is 210 MB. The long lines are mostly `!Sample_data_processing`, `extract_protocol`, `growth_protocol`, and `treatment_protocol`.
- 7 of those files have bare carriage returns inside quoted header cells. Splitting on them makes 4 files show a false column-count mismatch. Splitting on line feeds only gives 0 mismatches.
- 5,912 sample IDs appear in more than one series file. A SuperSeries repeats the samples of its SubSeries. GSE982 is a SubSeries of GSE995.

## Design

- The platform list comes from the `esummary` `gpl` field. One platform maps to `<GSE>_series_matrix.txt.gz`. Several platforms map to one `<GSE>-GPL<n>_series_matrix.txt.gz` each, in `gpl` order, capped at 10 files. The reader never parses the FTP listing.
- New `src/sources/geo_matrix.rs` builds URLs from base `https://ftp.ncbi.nlm.nih.gov/geo` with override `BIOMCP_GEO_FTP_BASE`. `series_prefix` returns `GSEnnn` for any series number under 1000. The rule stays computed rather than read from the ESummary `ftplink` field, because the reader must work from an accession alone. A test pins `series_prefix("GSE982")` against the `ftplink` value in the ESummary fixture, so a provider change to the folder rule fails the build.
- `read_header(url, lines_after_marker)` uses the unbuffered client. It feeds chunks through a streaming gzip decoder and a line reader. The line reader splits on line feed only. A bare carriage return stays inside its cell. It reads `lines_after_marker` lines after `!series_matrix_table_begin`, default 0, then drops the response. It never reads a byte past that.
- Caps: 8 MiB compressed and 16 MiB expanded per file, and 4 MiB per line. A cap hit before the marker returns `BodyLimit` naming the file. A file that ends without the marker is a provider error. HTTP 404 for one platform becomes a per-platform "matrix file not found" note, and the other platforms still render.
- Three `!Series_` lines are also returned as named fields, because later tickets read them: `!Series_submission_date` as `submitted`, `!Series_last_update_date` as `last_updated`, and `!Series_status` as `status`. The values are the raw cell text, trimmed of surrounding quotes and nothing else. BioMCP parses no date and normalizes no status word. A line the file does not carry gives `null`. The three fields stay in `series_lines` as well, so the raw view loses nothing. `read_header` returns them to every caller, which is how tickets 1204, 1207, 1208, and 1215 get a freshness signal. GEO's download service sends no `ETag` and no `Last-Modified`, so the header is the only date the provider publishes for a series.
- One predicate sits beside the three fields, because the status value is a sentence and no caller may compare it to a word. `status_is_public(raw)` is true when the trimmed raw value starts with `Public`. `read_header` returns it as a fourth field, `is_public`, next to `status`. It is `null` when the file carried no `!Series_status` line. Every caller that needs a yes or no reads `is_public` and prints `status` verbatim. Tickets 1204, 1207, and 1215 do exactly that. The predicate lives in `geo_matrix.rs` and is defined once.
- Parsing keeps every `!Series_` line as `{key, values}` in file order, with repeated keys preserved. It keeps every `!Sample_` line as a key with one unquoted cell per sample. Characteristics cells stay raw strings such as `cell line: MOLM-13`. The parser does not split, merge, rename, or label them. A column count that differs from the `!Sample_geo_accession` count is a provider error.
- A SuperSeries and its SubSeries list the same samples. The section renders each file as published. Any tally across series counts distinct GSM IDs. The `links` section from ticket 1204 shows the relation.
- JSON: `series: {platforms: [{platform, file, submitted, last_updated, status, is_public, series_lines: [...], samples: [{accession, fields: [{key, value}]}]}]}`. Markdown prints the submitted, last updated, and status values first, then the series lines as a key and value list, then one block per sample with its raw lines in file order.
- The section carries no `data_as_of` of its own. It rides on the card's top-level `data_as_of`, which ticket 1204 defines as the retrieval time with `data_as_of_kind: "retrieved"`. `submitted`, `last_updated`, and `status` are the provider's own dates and travel as their own fields, never as `data_as_of`.
- `series` is opt-in. `all` never includes it, because it reads a file.
- When `series` runs, it fills the card's `submitted`, `last_updated`, `status`, and `is_public` fields that ticket 1204 defines, from the first platform file that parsed. A card rendered without `series` keeps 1204's `unknown` values and its note. An `is_public` of `false` renders 1204's warning line with the raw `status` verbatim.
- Docs: add the section to the GEO source page and `biomcp list dataset`, stating that it reads only the header and stops at the table marker. Add the matrix host to the GEO row in `docs/reference/data-sources.md`.

## Acceptance

Fixtures under `testdata/sources/geo/`: two trimmed gzip matrix files for one two-platform series and one single-platform file. Each has a table marker followed by rows. One file has an empty table. One file has a bare carriage return inside a quoted characteristics cell. One file carries `!Series_status "Public on Jan 01 2020"` and one carries a status that is another word, such as `"Withdrawn"`.

1. `series_prefix` returns `GSE100nnn`, `GSEnnn`, and `GSEnnn` for `GSE100446`, `GSE982`, and `GSE14`, and file names follow the one-platform and multi-platform rules. `series_prefix("GSE982")` matches the folder in the ESummary fixture's `ftplink`.
2. With the default, a test reader panics if asked for bytes past the marker line, and the parse succeeds. With `lines_after_marker = 2`, it panics past the second line.
3. A header under the cap succeeds when the whole file exceeds the cap. The test injects a small cap. A header line longer than 64 KiB and under 4 MiB parses.
4. A cap hit before the marker returns `BodyLimit`. A missing marker and a column count mismatch return provider errors.
5. The fixture with an embedded carriage return parses with the right column count, and the cell keeps the carriage return byte for byte.
6. Characteristics cells come back byte-identical to the fixture, with repeated keys in order.
7. A two-platform series renders both platforms. A 404 on one keeps the other and adds the note.
8. `get dataset <GSE> all` makes no matrix request.
9. `read_header` returns `submitted`, `last_updated`, and `status` byte for byte from the fixture, and they also appear in `series_lines`. A fixture with no `!Series_status` line returns `null` for `status` and for `is_public`, and still parses. The fixture whose status is `"Public on Jan 01 2020"` returns `is_public: true` with the raw value unchanged, and the fixture whose status is another word returns `is_public: false`. A unit test pins `status_is_public` on both raw values and on a leading-space variant.
10. Spec `spec/entity/dataset.md` gains one JSON block for `series` pinning a raw characteristics cell, the `last_updated` value, and `is_public`, with `BIOMCP_GEO_FTP_BASE` pointed at the fixture server. The same block shows the card's top-level `data_as_of` with `data_as_of_kind: "retrieved"`, and no `data_as_of` inside `series`.

## Out of scope

- The value table itself, supplementary files, and download.
- Typed characteristics, labeling, and grouping.
- Parsing, normalizing, or comparing the three date and status values. Tickets 1204, 1208, and 1215 use them. The one exception is `status_is_public`, which tests a prefix and changes no value.

## Decisions

Open to Ian's overturn: caps are 8 MiB compressed, 16 MiB expanded, 4 MiB per line, and 10 platform files. The line cap was 64 KiB until experiment 203 found 33 of 774 real files with longer lines. The 16 MiB expanded cap covers the largest measured header of 8,364,887 bytes.

The three header fields are returned as raw text. BioMCP stores and compares them as strings, because GEO publishes them in one format and a parsed date adds a failure mode for no gain.

`is_public` is a prefix test and not a parsed status. `!Series_status` reads `Public on Sep 11 2019`, so a byte comparison against `Public` fails on every public series. The predicate lives here so that 1204, 1207, and 1215 share one rule instead of writing three.

## Review

- Design review: pending
- Code review: pending
