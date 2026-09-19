---
flow: build
priority: 4
deps: [1208, 1209]
---

# 1216: Dataset list shows what is on disk

## Goal

`biomcp dataset list` tells a user what BioMCP has already downloaded. It reads the manifests under the dataset root and prints, per series, each asset with its bytes and `downloaded_at`, the studies imported from it, and the total on disk. It makes no network request and builds no index over downloaded values.

```
biomcp dataset list
biomcp dataset list geo:GSE48843
biomcp --json dataset list
```

The motivating consumer is a hackathon team screening public GEO studies of drug-treated AML cell lines. That team downloads series over weeks and cannot see what it already has, how much disk it holds, or which local study came from which file.

## Current Facts

- Nothing lists downloaded datasets today. Ticket 1208 writes `manifest.json` per series with one entry per asset (`{asset_id, url, path, bytes, sha256, downloaded_at, snapshot_id, previous}`), a series-level block (`{platforms, upstream_last_updated, status, is_public, matrix_urls, scanned_at}`), and a series-level `imports` list. Ticket 1209 appends `{study, imported_at, asset_id, sha256}` to that list after a study installs.
- A `--refresh` that finds different bytes keeps the old file as `<name>.<old sha prefix>` and records it in the entry's `previous` list as `{path, bytes, sha256, downloaded_at}`, newest first (ticket 1208).
- Ticket 1208 stores `gene_info:<organism>` and `gene_history` once under `<dataset root>/ncbi-gene/` with their own manifest and no `series` block.
- Ticket 1208 adds `BIOMCP_DATA_DIR` and `resolve_dataset_root`, keeping `BIOMCP_DATASET_DIR` and `BIOMCP_STUDY_DIR` as overrides that take precedence. `resolve_study_root()` reads `BIOMCP_STUDY_DIR` and falls back to `dirs::data_dir()/biomcp/studies` (`src/sources/cbioportal_study.rs:278`, `:287`).
- Ticket 1208 writes a multi-platform series into a `<GPL>/` subfolder per platform, and the manifest holds the path, so no caller builds one.
- Manifest writes take the series `manifest.lock` through `fs2::FileExt` (`Cargo.toml:82`), following the article session store (`LOCK_FILE`, `src/cli/article/session.rs:18`, `lock_exclusive` at `:131`).
- A byte-count report has a precedent. `CacheStatsReport` (`src/cli/cache.rs:218`) carries `referenced_blob_bytes` (`src/cli/cache.rs:221`) and renders a Markdown table (`src/cli/cache.rs:245`). `biomcp cache stats` is the command (`src/cli/cache.rs:26`).
- The cache's free-disk floor is `DiskFreeThreshold` (`src/cache/config.rs:15`), which prints itself with `display()` (`src/cache/config.rs:32`), and `inspect_filesystem_space(path)` returns available and total bytes (`src/cache/limits.rs:50`). Ticket 1208 reuses the same threshold for downloads.
- The MCP shell rejects everything outside its allowed families with `GENERIC_MCP_REJECTION_MESSAGE` (`src/mcp/shell.rs:326`). Ticket 1208 rejects `dataset download` and `dataset path` by name because they reveal workstation-local paths.
- `biomcp list <entity>` renders through `render` (`src/cli/list/mod.rs:11`) and `render_json` (`:38`), and the unknown-entity message lists every valid entity at `src/cli/list/mod.rs:91`. Ticket 1203 adds `dataset` to both.
- Measured 2026-09-16: a full AML series matrix download of 774 files was 1.1 GB, and GSE48843 raw counts are 1.5 MB. DepMap's dependency matrix is 429 MB, measured from its public mirror; BioMCP does not integrate DepMap, and ticket 1206 is deferred. A user needs a disk number.

## Design

### Shape

`dataset list` with an optional positional dataset ID in either form, `geo:GSE48843` or `GSE48843`. With no ID it lists every series under the dataset root. With an ID it lists that one series and fails when the series has no manifest, naming the download command. An empty dataset root prints `No datasets downloaded.` and exits 0.

`--limit` caps how many series are listed, 1 to 500, default 100, in folder-name order, and the output says how many were skipped. Totals cover the listed series only, and the output says so.

### What it reads

`dataset list` reads `manifest.json` under each series folder, and the `ncbi-gene/` manifest as one more entry, and nothing else. It sends no request, opens no downloaded file, and reads no value inside one.

Locking: when `manifest.lock` exists beside a manifest, `dataset list` takes it in shared mode for that read, so it never sees a half-written manifest. It writes nothing, not even the lock file's PID line, so it never creates a lock file. A series folder with no `manifest.lock`, which is what a hand-written manifest has, is read without a lock and reported normally. A missing lock file cannot be locked without creating one, and creating one would break the rule that the tree is unchanged. The shared wait is bounded the way ticket 1208 bounds the exclusive one: 30 seconds, then that series reports one note naming the holder's PID read from the lock file, or `pid unknown`, and the rest of the listing still prints.

`bytes` comes from the manifest entry, not from the filesystem. When the file named by `path` is missing, the row is marked `missing` and its bytes are left out of the totals. A file whose size on disk differs from the manifest is marked `size_differs` and the manifest value is still the one reported. BioMCP hashes nothing here; SHA-256 is ticket 1208's conflict rule.

Each `previous` record in an asset entry prints as its own row under the asset, with the state `superseded`, its own path, bytes, SHA-256, and `downloaded_at`. Its bytes count in the per-series and grand totals, because the file is on disk and holds space. A `superseded` row whose file is missing is marked `missing` like any other. The `role` of a `superseded` row is the role of the asset it belongs to.

This is the only indexing BioMCP does over downloaded data. Searching inside the values is a database with its own index format and query language. `study query` and `study filter` already answer per-study questions. Anything larger belongs outside BioMCP.

### Output

- Markdown prints one block per series: the dataset ID and the series folder path, then an asset table of asset ID, role, bytes, `downloaded_at`, and state (`ok`, `missing`, `size_differs`, or `superseded`), then an imported-studies line naming each study and its import time, then a per-series byte total. The output ends with a totals block: series count, asset count, total bytes, and the free space on the dataset root's filesystem against the `min_disk_free` threshold, rendered with the same `display()` the cache uses (`src/cache/config.rs:32`). Bytes print as both an exact count and a human-readable size.
- JSON: `{data_as_of, data_as_of_kind, root, listed, skipped, series: [{id, path, bytes, upstream_last_updated, status, assets: [{asset_id, role, path, bytes, sha256, downloaded_at, state, previous: [{path, bytes, sha256, downloaded_at, state}]}], imports: [{study, imported_at, asset_id, sha256}]}], totals: {series, assets, superseded, bytes, missing}, disk: {available_bytes, total_bytes, min_disk_free}}`. `data_as_of` is the read time with `data_as_of_kind: "retrieved"`, following the definition ticket 1204 sets: when BioMCP read the data. `upstream_last_updated` is GEO's own date, read from the manifest's `series` block, and never fills `data_as_of`. An import's `sha256` says which version of the asset that study read, so a study that ran before a `--refresh` points at its `superseded` row.
- A series whose `manifest.json` cannot be parsed is reported as one note naming the path and the parse error, and the rest of the listing still prints. A run where no manifest parsed exits nonzero.
- Exit code is 0 when at least one manifest was read, whatever the asset states are.
- Nothing is deleted and nothing is suggested for deletion. Downloaded bytes are inputs to analyses, so BioMCP never evicts them and never offers to.

### Surface

- CLI-only, like `dataset download` and `dataset path`, because the output carries workstation-local paths. `is_allowed_mcp_command` (`src/mcp/shell.rs:472`) gains a `DatasetCommand::List` arm that is rejected with `DATASET_LOCAL_MCP_REJECTION_MESSAGE`, the constant ticket 1208 owns. This ticket writes no wording of its own.
- `biomcp list dataset` gains a row for the command.

### Docs

- `biomcp list dataset`, `docs/user-guide/cli-reference.md`, and `src/cli/list_reference.md` gain the command, and `CHANGELOG.md` gains an entry.
- The GEO source page states that the dataset root holds files that never expire, that nothing evicts them, and that `dataset list` is how a user sees the total before deleting anything by hand.
- The docs state that `dataset list` reads manifests only, makes no network request, and builds no index over downloaded values.

## Acceptance

Fixture-backed Rust tests, no live network. `BIOMCP_DATASET_DIR` points at a temp root with hand-written manifests and files.

1. Two series with three assets between them list in folder-name order, with bytes, `downloaded_at`, and a per-series total. The grand total is the sum.
2. A manifest entry whose file is missing is marked `missing` and is left out of the totals, and `totals.missing` counts it. A file whose size differs is marked `size_differs` and reports the manifest bytes.
3. An `imports` list with two entries prints both study names, import times, and SHA-256 values, in JSON and in Markdown. An import whose SHA-256 matches a `previous` row is shown against that row. An empty list prints no import line.
4. A multi-platform series reports each asset at its `<GPL>/` path from the manifest. The `ncbi-gene` manifest lists as its own entry with no series dates.
5. The request log is empty. A test asserts that no file under the dataset root other than each `manifest.json` is opened, and that the tree is unchanged by path, size, and modification time after the run, including that no `manifest.lock` was created.
6. `dataset list geo:GSE48843` lists one series. An ID with no manifest fails and names the download command. An empty root prints `No datasets downloaded.` and exits 0.
7. `--limit` caps the listing, reports the skipped count, and confines the totals to the listed series. `--limit 0` and `--limit 501` fail at parse time.
8. An unparsable manifest gives one note and the other series still list. A root where no manifest parses exits nonzero.
9. The totals block reports free space and the `min_disk_free` threshold from an injected filesystem-space reader, rendered by `display()`.
10. A manifest with a `manifest.lock` held exclusively by another handle makes `dataset list` wait and then read the finished content, never a partial one. A series folder with no `manifest.lock` is read without a lock, and no lock file appears afterwards. A holder that never releases makes that one series report a note naming the PID after the 30-second bound, and the other series still list. Fixtures include one series with a `manifest.lock` and one without.
11. The MCP shell rejects `dataset list` with `DATASET_LOCAL_MCP_REJECTION_MESSAGE`, the same constant `dataset download` uses.
12. JSON carries `data_as_of` from an injected clock and `data_as_of_kind: "retrieved"`, and no payload sets it from a GEO date. Bytes print as an exact count and a human-readable size in Markdown.
13. Spec `spec/entity/dataset.md` gains one block that downloads an asset, imports a study, and then lists the root, showing the asset row and the imported study.
14. An asset entry with one `previous` record prints a `superseded` row with its own path, bytes, SHA-256, and `downloaded_at`, and those bytes count in the per-series and grand totals. A `superseded` row whose file is missing is marked `missing` and left out of the totals.

`make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA.

## Out of scope

- Any network request. The command answers from manifests alone.
- Any index over downloaded values, any search inside a file, and any summary of what a file contains.
- Reading the studies under the study root. The `imports` list in the dataset manifest is the link, and `study list` already lists studies.
- Deleting, evicting, pruning, or suggesting any of those. Nothing leaves the dataset root except by hand.
- Hashing files to verify them. SHA-256 is ticket 1208's conflict rule.
- Comparing anything to the provider. That is the `dataset check` ticket.
- Sorting by size, filtering by age, and any ranking.

## Decisions

Open to Ian's overturn.

1. `dataset list` reads manifests only. BioMCP builds no index over downloaded values.
2. Sizes come from the manifest, not from the filesystem. A mismatch is reported rather than silently corrected.
3. Nothing is evicted from the dataset root automatically, and the command offers no deletion.
4. The command is CLI-only, matching `dataset download` and `dataset path`.
5. Rows print in folder-name order, so a capped run is repeatable.
6. A manifest with no lock file is read without a lock. Creating one would write to a tree this command promises to leave alone, and a hand-written manifest has none.
7. A superseded file prints as its own row and counts in the totals. It holds disk, and a user deciding what to delete needs to see it.

## Review

- Design review: pending
- Code review: pending
