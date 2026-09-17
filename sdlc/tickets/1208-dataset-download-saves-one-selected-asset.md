---
flow: build
priority: 3
deps: [1207]
---

# 1208: Dataset download saves one selected asset

## Goal

`biomcp dataset download <id> --asset <asset-id>` downloads one asset that ticket 1207 listed, under a byte limit, into a plain local folder, and records its checksum. `biomcp dataset path <id> --asset <asset-id>` prints the local path or fails without downloading. This is step 2 of the 2026-09-16 research-data decision brief, cut to the smallest useful form.

```
biomcp dataset download geo:GSE48843 --asset ncbi:GSE48843_raw_counts_GRCh38.p13_NCBI.tsv.gz --dry-run
biomcp dataset download geo:GSE48843 --asset ncbi:GSE48843_raw_counts_GRCh38.p13_NCBI.tsv.gz --max-size 1G
biomcp dataset path geo:GSE48843 --asset ncbi:GSE48843_raw_counts_GRCh38.p13_NCBI.tsv.gz
```

## Current Facts

- `biomcp study download <id>` installs cBioPortal bundles into the local study root (`src/sources/cbioportal_download.rs`, `resolve_study_root` at `src/sources/cbioportal_study.rs:278`, override `BIOMCP_STUDY_DIR`). It streams through `ordinary_middleware_client_for_base`, stages the bundle, and renames the staging directory into place (`src/sources/cbioportal_download.rs:528`). This ticket follows that pattern.
- The WHO sources install local files the same way (`src/sources/who_ivd.rs`, `src/sources/who_pq.rs`).
- Ticket 1207 gives each asset a stable ID (`matrix:<GPL>`, `annot:<GPL>`, `ncbi:<name>`, `suppl:<name>`), a URL, and a `snapshot_id`.
- MCP tools are read-only and withhold local paths (https://biomcp.org/reference/mcp-server/). This ticket keeps that boundary.
- Measured 2026-09-16: GSE48843 raw counts are 1.5 MB. A full AML series matrix download of 774 files was 1.1 GB.
- Measured 2026-09-16 (experiment 203): the GEO download service sends `Content-Length` and `Content-Disposition size=`, with no `ETag` or `Last-Modified`. The FTP host sends `Last-Modified` only. A repeat download of GSE48843 raw counts had the same SHA-256. The counts page prints rounded sizes ("1.4 Mb" for 1,493,706 bytes).
- The annotation link on the counts page returned HTTP 200 with a reCAPTCHA HTML page in place of gzip.
- Ticket 1211 returns `!Series_last_update_date` from the matrix header, and ticket 1207 carries it through the asset scan as `upstream_last_updated`. It is the only freshness signal GEO publishes for a series.
- The HTTP cache already has three budgets and no fourth: `ResolvedCacheConfig` holds `max_size`, `max_age`, and `min_disk_free` (`src/cache/config.rs:56-61`). `DiskFreeThreshold` is a percent or a byte count (`src/cache/config.rs:15`), defaults to `Percent(10)` (`DEFAULT_MIN_DISK_FREE`, `src/cache/config.rs:12`), reads `BIOMCP_CACHE_MIN_DISK_FREE` (`src/cache/config.rs:85`), and answers `is_violated(available, total)` (`src/cache/config.rs:28`) and `display()` (`src/cache/config.rs:32`). `inspect_filesystem_space(path)` returns available and total bytes (`src/cache/limits.rs:50`), and `evaluate_cache_limits` uses the pair (`src/cache/limits.rs:76`). Downloaded datasets have no budget at all today.
- A cross-process file lock already exists in the repo. `fs2` is a dependency (`Cargo.toml:82`), the article session store keeps a sibling lock file (`LOCK_FILE`, `src/cli/article/session.rs:18`) and takes it with `lock_exclusive` (`src/cli/article/session.rs:131`) or `try_lock_exclusive` (`:200`). Temp paths carry the process ID (`create_temp_path`, `src/cli/article/session.rs:286`).
- No source sends an HTTP `Range` request today. `grep -rn "Accept-Ranges\|Content-Range" src` matches only `src/sources/cpic.rs:375-376`, which reads a `content-range` header it did not ask for. Resume is new code.
- `resolve_study_root()` (`src/sources/cbioportal_study.rs:278`) reads `BIOMCP_STUDY_DIR` and otherwise falls back to `dirs::data_dir()/biomcp/studies` (`src/sources/cbioportal_study.rs:287`). No `BIOMCP_DATA_DIR` exists.

## Design

### One data root

Add `BIOMCP_DATA_DIR`. When it is set, datasets live under `<BIOMCP_DATA_DIR>/datasets/` and studies under `<BIOMCP_DATA_DIR>/studies/`. When it is not set, the default is `dirs::data_dir()/biomcp/`, which keeps today's study path byte for byte (`resolve_study_root`, `src/sources/cbioportal_study.rs:278`).

`BIOMCP_STUDY_DIR` and `BIOMCP_DATASET_DIR` keep working and take precedence over `BIOMCP_DATA_DIR`, so no existing install moves. `resolve_study_root` gains the `BIOMCP_DATA_DIR` step between its env check and its `dirs::data_dir()` fallback, and a new `resolve_dataset_root` follows the same three steps. The resolution order is named in the docs and pinned by a test.

### Layout

- Root: `<dataset root>/<source>/<accession>/`. It is not the disposable cache.
- A series with one platform writes its files directly in that folder. A multi-platform series writes each platform's files into a `<GPL>/` subfolder, for example `geo/GSE256354/GPL15520/`. Asset IDs are unchanged, because callers treat them as opaque. The manifest holds the path, so no caller builds one.
- `download` resolves the asset by re-running the 1207 scan for that series. It fails when the asset ID is not listed, and names the listed IDs.
- `--dry-run` prints the plan and transfers nothing: asset ID, URL, known size or `unknown`, destination path, and the byte limit.
- A known size comes from `Content-Length` or `Content-Disposition size=`. BioMCP never takes a size from the rounded page text.
- `--max-size` defaults to `2G` and uses the `cache clean --max-size` parser (`src/cli/cache.rs:43-45`), so values look like `500M` or `5G`. A known size over the limit fails before any transfer. An unknown size streams and aborts at the limit.
- `--dry-run` and `--max-size` follow `cache clean`. `--refresh` is new.
- Before writing, BioMCP checks the content type and the first bytes. A gzip asset must start with the gzip magic bytes (`1f 8b`). An HTML body with HTTP 200 is a failure that names the URL, and nothing is written.
- **Free-disk floor.** Before any transfer, `download` calls `inspect_filesystem_space` (`src/cache/limits.rs:50`) on the dataset root and checks the cache's `min_disk_free` threshold from `resolve_cache_config()` (`src/cache/config.rs:82`) with `is_violated` (`src/cache/config.rs:28`). One threshold covers both budgets, so a user tunes `BIOMCP_CACHE_MIN_DISK_FREE` once. The check runs twice: on the free space as it stands, and on the free space minus a known `Content-Length`. Either violation fails before a byte is written, naming the threshold through `display()` (`src/cache/config.rs:32`), the available bytes, and the total. An unknown size checks only the first form, and the stream aborts if the floor is crossed during transfer, leaving the staging file. `--dry-run` reports the check and transfers nothing. Nothing is evicted. Downloaded bytes are inputs to analyses, so BioMCP never deletes them on its own.
- **Manifest lock.** Every manifest write takes an exclusive lock on `manifest.lock` beside `manifest.json` in the series folder, through `fs2::FileExt` as the article session store does (`src/cli/article/session.rs:131`). The holder writes its process ID into the lock file before it takes the lock. A blocked writer waits up to 30 seconds, then fails with `another biomcp process (pid <N>) is writing <path>`, reading the PID from the file. An unreadable or empty lock file gives `pid unknown` in the same message. The lock covers the read, the edit, and the rename, so two concurrent downloads of one series never lose an entry.
- **Resume.** Transfer streams into `<name>.partial` in the destination folder and renames on success. BioMCP never returns a partial file as complete. When a `.partial` file exists and the manifest has no completed entry for that asset, `download` sends `Range: bytes=<size>-` and appends on HTTP 206, checking that `Content-Range` names the same start byte. A server that answers HTTP 200 restarts the transfer and truncates the staging file first. Any other status is an error and the staging file stays. Resume is the default; `--no-resume` deletes the staging file and starts again. The staging file is validated the same way a fresh one is: the gzip magic-byte and content-type check runs on the first bytes of the completed file, not on the resumed chunk. The byte limit counts the whole file, resumed bytes included.
- On success, `manifest.json` in the series folder gains or replaces one entry: `{asset_id, url, path, bytes, sha256, downloaded_at, upstream_last_updated, snapshot_id}`. `upstream_last_updated` is the series `!Series_last_update_date` that the 1207 scan carried forward, or `null` when the asset has no such header, which is the honest answer for NCBI count files and supplementary files. SHA-256 stays the only content change signal, because the servers send no `ETag` and the download service sends no `Last-Modified`.
- `manifest.json` also carries a series-level `imports` list, empty until ticket 1209 appends to it. Each entry is `{study, imported_at, asset_id}`. It records which local studies came from this series, so both directions are visible.
- Repeat rules: BioMCP hashes an existing file with the same asset ID. A file that matches the manifest SHA-256 is reused and reported as already present, with no asset request. With `--refresh`, BioMCP downloads again. Matching SHA-256 keeps the file. A different SHA-256 is a conflict. It keeps the old file as `<name>.<old sha prefix>`, records both, and reports the conflict. Without `--refresh`, a file on disk whose SHA-256 differs from the manifest fails and is never overwritten. An imported study keeps the SHA-256 it recorded. A refresh never changes an imported input silently.
- `path` reads only the manifest. It prints the absolute path alone on stdout, or exits nonzero with `not downloaded` and the download command as a suggestion.
- Both commands are CLI-only. The MCP shell allows helper families and the read-only `study` subcommands, and rejects `study download <id>` (`GENERIC_MCP_REJECTION_MESSAGE`, `src/mcp/shell.rs:326`). `dataset samples` stays allowed. `dataset download` and `dataset path` are rejected by name, with the "reveals workstation-local paths" wording used for `cache path`, and the rejection message lists `dataset samples` as allowed.
- Progress goes to stderr. `--json` returns the manifest entry.
- Every output carries `data_as_of`, set to `upstream_last_updated` when the manifest entry has one and to `downloaded_at` otherwise, with `data_as_of_kind` naming which. Markdown ends with the GEO attribution line from ticket 1204.
- A provider that answers a bot check reports the URL, names the file, and stops. The content check already covers it: an HTTP 200 HTML body where bytes were expected is a failure naming the URL. BioMCP never works around a check.

### Docs

- `docs/reference/configuration.md` gains `BIOMCP_DATA_DIR` and `BIOMCP_DATASET_DIR`, states the resolution order, and states that `BIOMCP_STUDY_DIR` and `BIOMCP_DATASET_DIR` win when set.
- The GEO source page and `biomcp list dataset` show the layout, including the `<GPL>/` subfolder for a multi-platform series.
- The docs name the three cache layers and say which rule each one follows:
  - The HTTP cache under the cache root, with the `max_age` time-to-live, the `max_size` cap, and the `min_disk_free` floor (`src/cache/config.rs:56-61`). `biomcp cache clean` evicts from it.
  - Dataset files under the dataset root. They never expire and nothing evicts them. `dataset download` refuses to start below the same `min_disk_free` floor. A user deletes them by hand.
  - Imported studies under the study root. They are frozen at import time and no refresh changes one under a running analysis.

## Acceptance

1. A fixture server serves a small counts file. `download` writes it, records the right SHA-256, and `path` prints its absolute path.
2. `--dry-run` sends no request for the asset URL. The request log proves it.
3. A known size over `--max-size` fails before transfer. An unknown size stream aborts at the limit and leaves no final file.
4. A killed transfer leaves only `.partial`. The next download resumes it with a `Range` header and succeeds. With `--no-resume` it removes the staging file and starts again.
5. A second download without `--refresh` makes no asset request. With `--refresh` and changed bytes, both versions are on disk and in the manifest.
6. An unlisted asset ID fails and lists the valid IDs.
7. An HTML body served with HTTP 200 for a gzip asset fails, names the URL, and leaves no file.
8. A file on disk edited after download makes the next download fail without overwriting it.
9. A size comes from `Content-Length` in the fixture, not from page text.
10. `path` for an asset not yet downloaded exits nonzero and makes no network request.
11. The MCP shell rejects `dataset download` and `dataset path`.
12. Spec `spec/entity/dataset.md` gains one dry-run block and one download-then-path block against the fixture server.
13. The manifest entry carries `upstream_last_updated` from the 1207 scan, and `null` for an asset whose series header carried no update date.
14. A fixture server that answers the resume request with HTTP 206 and a matching `Content-Range` produces the same SHA-256 as an uninterrupted download. A server that answers HTTP 200 restarts and produces the same SHA-256. A `Content-Range` naming another start byte fails and leaves the staging file.
15. An injected filesystem-space reader below the threshold fails before any request, and the message names the threshold, the available bytes, and the total. A known `Content-Length` that would cross the floor fails the same way. `--dry-run` reports the check and sends nothing.
16. Two processes writing one series manifest serialize. The blocked one is made to time out and its message names the holder's PID. A lock file with no readable PID gives `pid unknown`.
17. A fresh manifest carries `imports: []`. A multi-platform fixture writes each platform's file under its `<GPL>/` folder and records that path.
18. `BIOMCP_DATA_DIR` alone puts datasets in `datasets/` and studies in `studies/`. `BIOMCP_STUDY_DIR` and `BIOMCP_DATASET_DIR` override it. With none set, `resolve_study_root` returns today's path byte for byte.
19. The docs check accepts the `BIOMCP_DATA_DIR` and `BIOMCP_DATASET_DIR` configuration rows, the layout section, and the three cache layers.

## Out of scope

- A content-addressed store, a SQLite catalog, pins, leases, eviction, garbage collection, backup, and restore. The brief lists these. None is built until a user needs it.
- Fetching whole series, all assets at once, SRA reads, or archive extraction.
- Run registration and project state. BotAssembly records runs.
- Re-reading upstream headers to compare dates, and listing what is on disk. Those are the `dataset check` and `dataset list` tickets.

## Decisions

Open to Ian's overturn.

1. Downloads reuse the cache's `min_disk_free` threshold rather than adding a setting of their own. One number governs both budgets.
2. Nothing is evicted from the dataset root automatically.
3. `BIOMCP_DATA_DIR` is added, and `BIOMCP_STUDY_DIR` and `BIOMCP_DATASET_DIR` keep working and take precedence.
4. The manifest lock waits 30 seconds and then fails naming the holder's PID, rather than waiting without bound.
5. Resume is on by default. `--no-resume` is the escape hatch.

## Review

- Design review: pending
- Code review: pending
