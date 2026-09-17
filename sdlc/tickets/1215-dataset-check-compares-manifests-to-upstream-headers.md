---
flow: build
priority: 4
deps: [1208, 1211]
---

# 1215: Dataset check compares manifests to upstream headers

## Goal

`biomcp dataset check [<id>]` tells a user whether a downloaded GEO series still matches the provider. It re-reads the series matrix header for each downloaded series, compares `!Series_last_update_date` against the `upstream_last_updated` value in the manifest, and prints one row per asset: current, changed upstream, withdrawn, or unknown. It downloads nothing and changes nothing. `dataset download --refresh` stays the only way to fetch new bytes.

```
biomcp dataset check
biomcp dataset check geo:GSE48843
biomcp --json dataset check
```

The motivating consumer is a hackathon team screening public GEO studies of drug-treated AML cell lines. That team keeps a series on disk for weeks and has no way to learn that the submitter revised it.

## Current Facts

- A downloaded asset never expires today and nothing re-reads the provider. Ticket 1208 writes `{asset_id, url, path, bytes, sha256, downloaded_at, upstream_last_updated, snapshot_id}` per asset into `manifest.json` in the series folder, plus a series-level `imports` list.
- Ticket 1211 adds `src/sources/geo_matrix.rs` with `series_prefix`, `read_header(url, lines_after_marker)`, and the `BIOMCP_GEO_FTP_BASE` override, and returns `!Series_submission_date`, `!Series_last_update_date`, and `!Series_status` as `submitted`, `last_updated`, and `status`. `read_header` stops at the table marker and reads no further byte.
- GEO publishes no other freshness signal. Measured 2026-09-16 (experiment 203): the GEO download service sends `Content-Length` and `Content-Disposition size=`, with no `ETag` and no `Last-Modified`. The FTP host sends `Last-Modified` only.
- NCBI count files and supplementary files carry no date of their own, so ticket 1208 stores `null` for their `upstream_last_updated`.
- The HTTP cache is the only layer with expiry. `ResolvedCacheConfig` holds `max_size`, `min_disk_free`, and `max_age` (`src/cache/config.rs:56-61`), with a 24-hour default age (`DEFAULT_MAX_AGE_SECS`, `src/cache/config.rs:11`). Dataset files sit outside it.
- `resolve_study_root()` reads `BIOMCP_STUDY_DIR` and falls back to `dirs::data_dir()/biomcp/studies` (`src/sources/cbioportal_study.rs:278`, `:287`). Ticket 1208 adds `BIOMCP_DATA_DIR` and `resolve_dataset_root`.
- The MCP shell allows read-only helper families and rejects everything else with `GENERIC_MCP_REJECTION_MESSAGE` (`src/mcp/shell.rs:326`). Ticket 1208 rejects `dataset download` and `dataset path` by name because they reveal workstation-local paths.
- `biomcp list <entity>` renders through `render` (`src/cli/list/mod.rs:11`) and `render_json` (`:38`), and the unknown-entity message lists every valid entity at `src/cli/list/mod.rs:91`. Ticket 1203 adds `dataset` to both.

## Design

### Shape

`dataset check` with an optional positional dataset ID in either form, `geo:GSE48843` or `GSE48843`. With no ID it checks every series under the dataset root. With an ID it checks that one series and fails when the series has no manifest, naming the download command.

`--limit` caps how many series are checked in one run, 1 to 200, default 50. Series are visited in folder-name order, so a capped run is repeatable, and the output says how many were skipped.

### What it reads

For each series in scope, `dataset check` reads `manifest.json`, then calls `read_header` once per platform matrix URL the manifest's matrix assets name, with `lines_after_marker = 0`. It builds no URL for an asset that is not a series matrix. It sends no other request, opens no downloaded file, and writes nothing, not even the manifest.

### Status per asset

| Status | Rule |
| --- | --- |
| `current` | The asset has an `upstream_last_updated` and the header returns the same value byte for byte. |
| `changed_upstream` | The asset has an `upstream_last_updated` and the header returns a different value. |
| `withdrawn` | The header returns a `!Series_status` that is not `Public`, or the matrix file returns HTTP 404. |
| `unknown` | The asset's `upstream_last_updated` is `null`, or the header could not be read. |

`withdrawn` wins over the date comparison, because a withdrawn series can still carry an old date. `unknown` is the honest answer for NCBI count files and supplementary files. A header read that fails for a transport reason gives `unknown` and a note naming the URL and the reason. One failed series never stops the run.

Values are compared as strings. BioMCP parses no date and normalizes no status word, which matches how ticket 1211 returns them.

### Output

- Markdown prints one table per series: asset ID, role, status, `upstream_last_updated` from the manifest, the value the header returned, and `downloaded_at`. A series with any `changed_upstream` or `withdrawn` row prints a summary line above the table naming the counts. Every `withdrawn` row prints the status verbatim. The output ends with the GEO attribution line from ticket 1204 and the next command `biomcp dataset download <id> --asset <asset-id> --refresh` for the first changed row.
- JSON: `{data_as_of, data_as_of_kind, checked, skipped, series: [{id, path, assets: [{asset_id, role, status, manifest_last_updated, upstream_last_updated, upstream_status, downloaded_at}], notes: []}], summary: {current, changed_upstream, withdrawn, unknown}}`. `data_as_of` is the retrieval time with `data_as_of_kind: "retrieved"`, because the command reports what the provider says right now.
- Exit code is 0 whatever the statuses are. A changed record is a fact, not a failure. A run that could read no manifest at all exits nonzero.
- BioMCP never re-downloads, never rewrites a manifest entry, and never deletes a file. The command names the refresh command and stops there.
- An HTTP 200 whose body is an HTML human-verification page where gzip was expected is a provider error. The note names the URL and the file, and that series reports `unknown`. BioMCP never works around a bot check.

### Surface

- CLI-only, like `dataset download` and `dataset path`, because the output carries workstation-local paths. The MCP shell rejects `dataset check` by name with the "reveals workstation-local paths" wording, and its rejection message lists `dataset samples` as allowed.
- `biomcp list dataset` gains a row for the command.

### Docs

- The GEO source page states that a header read is one request per platform that stops at the table marker, and that GEO offers no `ETag` or `Last-Modified`.
- `docs/user-guide/cli-reference.md` and `src/cli/list_reference.md` gain the command, and `CHANGELOG.md` gains an entry.
- The docs state that `unknown` means the provider publishes no date for that asset, not that anything is wrong.

## Acceptance

Fixture-backed Rust tests, no live network. The fixture server serves matrix headers under `BIOMCP_GEO_FTP_BASE`, and `BIOMCP_DATASET_DIR` points at a temp root with hand-written manifests.

1. A manifest whose `upstream_last_updated` equals the fixture header reports `current`. A differing value reports `changed_upstream` and names both values.
2. A fixture header whose `!Series_status` is not `Public` reports `withdrawn` with the status verbatim, even when the date matches.
3. A matrix URL that returns HTTP 404 reports `withdrawn`.
4. An asset with a `null` `upstream_last_updated` reports `unknown` and triggers no request for that asset.
5. A transport failure on one series reports `unknown` with a note naming the URL, and the other series still report.
6. The request log shows one header read per matrix asset and no other request. No file under the dataset root is opened for reading beyond `manifest.json`, and no file is written. A test compares the whole tree before and after by path, size, and modification time.
7. `dataset check geo:GSE48843` checks one series. An ID with no manifest fails and names the download command.
8. `--limit` caps the run in folder-name order and reports the skipped count. `--limit 0` and `--limit 201` fail at parse time.
9. Exit code is 0 for a run with `changed_upstream` and `withdrawn` rows. A root with no manifest at all exits nonzero.
10. An HTML body served with HTTP 200 for a matrix URL reports `unknown` with a note naming the URL, and writes nothing.
11. The MCP shell rejects `dataset check`.
12. JSON carries `data_as_of`, `data_as_of_kind: "retrieved"`, and the summary counts. Markdown ends with the GEO attribution line and the refresh next command.
13. Spec `spec/entity/dataset.md` gains one block for a `current` series and one for a `changed_upstream` series, against the fixture server.

`make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA.

## Out of scope

- Downloading anything, refreshing anything, and writing to any manifest. `dataset download --refresh` is the only way to fetch new bytes.
- Checking NCBI count files and supplementary files against the provider. They carry no date, so they report `unknown`.
- Hashing files on disk. SHA-256 answers whether the local bytes changed, which is ticket 1208's conflict rule, not this command's question.
- A watch mode, a timer, a background refresh, and any automatic action on a changed record.
- Listing what is on disk. That is the `dataset list` ticket.
- Checking imported studies. An imported study is frozen by design.

## Decisions

Open to Ian's overturn.

1. `dataset check` re-reads headers only and reports `unknown` where a provider publishes no date.
2. Dates and statuses are compared as strings. GEO publishes them in one format, and a parsed date adds a failure mode for no gain.
3. `withdrawn` wins over a date comparison.
4. The exit code is 0 for a changed record. A user scripting on the exit code would get a failure for a normal fact.
5. The command is CLI-only, matching `dataset download` and `dataset path`.

## Review

- Design review: pending
- Code review: pending
