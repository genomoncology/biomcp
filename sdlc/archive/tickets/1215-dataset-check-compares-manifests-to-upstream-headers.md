---
flow: build
priority: 4
deps: [1208, 1211]
---

# 1215: Dataset check compares manifests to upstream headers

## Goal

`biomcp dataset check [<id>]` tells a user whether a downloaded GEO series still matches the provider. It re-reads the series matrix header once per downloaded series, compares `!Series_last_update_date` against the series-level `upstream_last_updated` in the manifest, and prints one row per asset: current, changed upstream, withdrawn, or unknown. It downloads nothing and changes nothing. `dataset download --refresh` stays the only way to fetch new bytes.

```
biomcp dataset check
biomcp dataset check geo:GSE48843
biomcp --json dataset check
```

The motivating consumer is a hackathon team screening public GEO studies of drug-treated AML cell lines. That team keeps a series on disk for weeks and has no way to learn that the submitter revised it.

## Current Facts

- A downloaded asset never expires today and nothing re-reads the provider. Ticket 1208 writes `{asset_id, url, path, bytes, sha256, downloaded_at, snapshot_id, previous}` per asset into `manifest.json` in the series folder, plus a series-level `imports` list and a series-level block `{platforms, upstream_last_updated, status, is_public, matrix_urls, scanned_at}`.
- Ticket 1211 adds `src/sources/geo_matrix.rs` with `series_prefix`, `read_header(url, lines_after_marker)`, and the `BIOMCP_GEO_FTP_BASE` override, and returns `!Series_submission_date`, `!Series_last_update_date`, and `!Series_status` as `submitted`, `last_updated`, and `status`, plus the predicate `is_public`. `read_header` stops at the table marker and reads no further byte.
- GEO publishes no other freshness signal. Measured 2026-09-16 (experiment 203): the GEO download service sends `Content-Length` and `Content-Disposition size=`, with no `ETag` and no `Last-Modified`. The FTP host sends `Last-Modified` only.
- GEO publishes one date for a series and none for an individual file. The series date is therefore the date for every asset of that series, count files and supplementary files included. Ticket 1208 records it once in the `series` block rather than per asset.
- The motivating case has no matrix asset at all. GSE48843 is an RNA-seq series whose matrix table is empty and whose values live in `ncbi:GSE48843_raw_counts_GRCh38.p13_NCBI.tsv.gz`. A user downloads the count file and never the matrix. 654 of 774 measured matrix files have an empty table, 490 of them RNA-Seq (ticket 1207).
- `!Series_status` reads `Public on Sep 11 2019` in the measured GSE100446 header. A comparison against the word `Public` reports every public series as withdrawn, so this command tests `is_public` instead.
- The HTTP cache is the only layer with expiry. `ResolvedCacheConfig` holds `max_size`, `min_disk_free`, and `max_age` (`src/cache/config.rs:56-61`), with a 24-hour default age (`DEFAULT_MAX_AGE_SECS`, `src/cache/config.rs:11`). Dataset files sit outside it.
- `resolve_study_root()` reads `BIOMCP_STUDY_DIR` and falls back to `dirs::data_dir()/biomcp/studies` (`src/sources/cbioportal_study.rs:278`, `:287`). Ticket 1208 adds `BIOMCP_DATA_DIR` and `resolve_dataset_root`.
- The MCP shell allows read-only helper families and rejects everything else with `GENERIC_MCP_REJECTION_MESSAGE` (`src/mcp/shell.rs:326`). Ticket 1208 rejects `dataset download` and `dataset path` by name because they reveal workstation-local paths.
- `biomcp list <entity>` renders through `render` (`src/cli/list/mod.rs:11`) and `render_json` (`:38`), and the unknown-entity message lists every valid entity at `src/cli/list/mod.rs:91`. Ticket 1203 adds `dataset` to both.

## Design

### Shape

`dataset check` with an optional positional dataset ID in either form, `geo:GSE48843` or `GSE48843`. With no ID it checks every series under the dataset root. With an ID it checks that one series and fails when the series has no manifest, naming the download command.

`--limit` caps how many series are checked in one run, 1 to 200, default 50. Series are visited in folder-name order, so a capped run is repeatable, and the output says how many were skipped.

### What it reads

For each series in scope, `dataset check` reads `manifest.json`, then calls `read_header` with `lines_after_marker = 0` on the URLs in the manifest's series-level `matrix_urls`, stopping at the first one that answers. The check is per series and never per asset. A series whose only downloaded asset is an NCBI count file still gets one header read, because `matrix_urls` holds every platform matrix URL the 1207 scan built, whether or not that matrix carried values. A manifest with no `matrix_urls`, which is one written before this field existed, reports `unknown` for the series with a note naming the refresh command. It sends no other request, opens no downloaded file, and writes nothing, not even the manifest.

The `ncbi-gene` folder that ticket 1208 writes has no `series` block and belongs to no series. `dataset check` skips it and says so in the totals.

### Status

The verdict is computed once per series and applies to every asset row of that series, because GEO publishes one date and one status for the series and none for any single file.

| Status | Rule |
| --- | --- |
| `current` | The series has an `upstream_last_updated` and the header returns the same value byte for byte. |
| `changed_upstream` | The series has an `upstream_last_updated` and the header returns a different value. |
| `withdrawn` | The header's `is_public` is `false`, or every URL in `matrix_urls` returns HTTP 404. |
| `unknown` | No header could be read, the manifest has no `matrix_urls`, or the series `upstream_last_updated` is `null`. |

`withdrawn` wins over the date comparison, because a withdrawn series can still carry an old date. `unknown` now means one thing only: the header could not be read, or the manifest is too old to name a matrix URL. It is no longer the answer for a count file or a supplementary file, because the series date covers those. A header read that fails for a transport reason gives `unknown` and a note naming the URL and the reason. One failed series never stops the run.

Dates are compared as strings. BioMCP parses no date and normalizes no status word, which matches how ticket 1211 returns them. The one status rule is `is_public`, the predicate ticket 1211 defines, and the raw status prints verbatim beside it.

### Output

- Markdown prints one table per series under a line naming the series verdict, the manifest's series `upstream_last_updated`, and the value the header returned. Each row is asset ID, role, status, and `downloaded_at`, and every row of a series carries that series' status. A series that is `changed_upstream` or `withdrawn` prints a summary line above the table. A `withdrawn` series prints the raw status verbatim. The output ends with the GEO attribution line from ticket 1204 and the next command `biomcp dataset download <id> --asset <asset-id> --refresh` for the first asset of the first changed series.
- JSON: `{data_as_of, data_as_of_kind, checked, skipped, series: [{id, path, status, manifest_last_updated, upstream_last_updated, upstream_status, upstream_is_public, matrix_url_read, assets: [{asset_id, role, status, downloaded_at}], notes: []}], summary: {current, changed_upstream, withdrawn, unknown}}`. The summary counts series, and a second block counts the asset rows. `data_as_of` is the time this run read the provider, with `data_as_of_kind: "retrieved"`, following the definition ticket 1204 sets. GEO's own dates are `manifest_last_updated` and `upstream_last_updated` and never fill `data_as_of`.
- Exit code is 0 whatever the statuses are. A changed record is a fact, not a failure. A run that could read no manifest at all exits nonzero.
- BioMCP never re-downloads, never rewrites a manifest entry, and never deletes a file. The command names the refresh command and stops there.
- An HTTP 200 whose body is an HTML human-verification page where gzip was expected is a provider error. The note names the URL and the file, and that series reports `unknown`. BioMCP never works around a bot check.

### Surface

- CLI-only, like `dataset download` and `dataset path`, because the output carries workstation-local paths. `is_allowed_mcp_command` (`src/mcp/shell.rs:472`) gains a `DatasetCommand::Check` arm that is rejected with `DATASET_LOCAL_MCP_REJECTION_MESSAGE`, the constant ticket 1208 owns. This ticket writes no wording of its own.
- `biomcp list dataset` gains a row for the command.

### Docs

- The GEO source page states that a header read is one request per platform that stops at the table marker, and that GEO offers no `ETag` or `Last-Modified`.
- `docs/user-guide/cli-reference.md` and `src/cli/list_reference.md` gain the command, and `CHANGELOG.md` gains an entry.
- The docs state that GEO publishes one date per series, that the check is per series, and that `unknown` means the header could not be read, not that anything is wrong with the files.

## Acceptance

Fixture-backed Rust tests, no live network. The fixture server serves matrix headers under `BIOMCP_GEO_FTP_BASE`, and `BIOMCP_DATASET_DIR` points at a temp root with hand-written manifests.

1. A manifest whose series `upstream_last_updated` equals the fixture header reports `current` for the series and for every asset row. A differing value reports `changed_upstream` and names both values.
2. A fixture header whose status is `Withdrawn` reports `is_public: false` and `withdrawn` with the status verbatim, even when the date matches. A fixture header whose status is `Public on Sep 11 2019` reports `is_public: true` and never `withdrawn`.
3. Every matrix URL returning HTTP 404 reports `withdrawn`.
4. A series whose only asset is `ncbi:GSE48843_raw_counts_GRCh38.p13_NCBI.tsv.gz` still sends one header read, taken from the series `matrix_urls`, and reports `current` or `changed_upstream` rather than `unknown`. The fixture matrix for that series has an empty table, which changes nothing, because the check reads the header only.
5. A transport failure on one series reports `unknown` with a note naming the URL, and the other series still report. A manifest with no `matrix_urls` reports `unknown` with the note naming the refresh command and sends no request.
6. The request log shows one header read per series and no other request. A series with two platforms stops at the first URL that answers. No file under the dataset root is opened for reading beyond `manifest.json`, and no file is written. A test compares the whole tree before and after by path, size, and modification time.
7. `dataset check geo:GSE48843` checks one series. An ID with no manifest fails and names the download command.
8. `--limit` caps the run in folder-name order and reports the skipped count. `--limit 0` and `--limit 201` fail at parse time.
9. Exit code is 0 for a run with `changed_upstream` and `withdrawn` rows. A root with no manifest at all exits nonzero.
10. An HTML body served with HTTP 200 for a matrix URL reports `unknown` with a note naming the URL, and writes nothing.
11. The MCP shell rejects `dataset check` with `DATASET_LOCAL_MCP_REJECTION_MESSAGE`, the same constant `dataset download` uses.
12. JSON carries `data_as_of` from an injected clock, `data_as_of_kind: "retrieved"`, and the summary counts. No payload sets `data_as_of` from a GEO date. Markdown ends with the GEO attribution line and the refresh next command.
13. Spec `spec/entity/dataset.md` gains one block for a `current` series and one for a `changed_upstream` series, against the fixture server.

`make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA.

## Out of scope

- Downloading anything, refreshing anything, and writing to any manifest. `dataset download --refresh` is the only way to fetch new bytes.
- Checking any file against the provider byte for byte. The check reads the series header and nothing else. Count files and supplementary files take the series verdict, because GEO publishes no date of their own.
- Hashing files on disk. SHA-256 answers whether the local bytes changed, which is ticket 1208's conflict rule, not this command's question.
- A watch mode, a timer, a background refresh, and any automatic action on a changed record.
- Listing what is on disk. That is the `dataset list` ticket.
- Checking imported studies. An imported study is frozen by design.

## Decisions

Open to Ian's overturn.

1. `dataset check` re-reads headers only. The verdict is per series, because GEO publishes one date and one status for a series and none for a file. The proposal's line that `unknown` is the honest answer for count files is retired: the series date covers them, and `unknown` now means only that the header could not be read.
2. Dates are compared as strings, and the status is tested with ticket 1211's `is_public` predicate. GEO publishes dates in one format, and a parsed date adds a failure mode for no gain. The status is a sentence, so it is never compared to a word.
3. `withdrawn` wins over a date comparison.
4. The exit code is 0 for a changed record. A user scripting on the exit code would get a failure for a normal fact.
5. The command is CLI-only, matching `dataset download` and `dataset path`.

## Review

- Design review: pending
- Code review: pending
