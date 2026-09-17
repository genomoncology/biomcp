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

## Design

- Root: `<data-root>/biomcp/datasets/<source>/<accession>/`, with override `BIOMCP_DATASET_DIR`. The data root is the one `study` uses. It is not the disposable cache.
- `download` resolves the asset by re-running the 1207 scan for that series. It fails when the asset ID is not listed, and names the listed IDs.
- `--dry-run` prints the plan and transfers nothing: asset ID, URL, known size or `unknown`, destination path, and the byte limit.
- A known size comes from `Content-Length` or `Content-Disposition size=`. BioMCP never takes a size from the rounded page text.
- `--max-size` defaults to `2G` and uses the `cache clean --max-size` parser (`src/cli/cache.rs:43-45`), so values look like `500M` or `5G`. A known size over the limit fails before any transfer. An unknown size streams and aborts at the limit.
- `--dry-run` and `--max-size` follow `cache clean`. `--refresh` is new.
- Before writing, BioMCP checks the content type and the first bytes. A gzip asset must start with the gzip magic bytes (`1f 8b`). An HTML body with HTTP 200 is a failure that names the URL, and nothing is written.
- Transfer streams into `<name>.partial` and renames on success. An interrupted transfer leaves only the partial file. The next download deletes it and restarts. BioMCP never returns a partial file as complete. No resume.
- On success, `manifest.json` in the series folder gains or replaces one entry: `{asset_id, url, path, bytes, sha256, downloaded_at, snapshot_id}`. SHA-256 is the only change signal, because the servers send no `ETag` and the download service sends no `Last-Modified`.
- Repeat rules: BioMCP hashes an existing file with the same asset ID. A file that matches the manifest SHA-256 is reused and reported as already present, with no asset request. With `--refresh`, BioMCP downloads again. Matching SHA-256 keeps the file. A different SHA-256 is a conflict. It keeps the old file as `<name>.<old sha prefix>`, records both, and reports the conflict. Without `--refresh`, a file on disk whose SHA-256 differs from the manifest fails and is never overwritten. An imported study keeps the SHA-256 it recorded. A refresh never changes an imported input silently.
- `path` reads only the manifest. It prints the absolute path alone on stdout, or exits nonzero with `not downloaded` and the download command as a suggestion.
- Both commands are CLI-only. The MCP shell allows helper families and the read-only `study` subcommands, and rejects `study download <id>` (`GENERIC_MCP_REJECTION_MESSAGE`, `src/mcp/shell.rs:326`). `dataset samples` stays allowed. `dataset download` and `dataset path` are rejected by name, with the "reveals workstation-local paths" wording used for `cache path`, and the rejection message lists `dataset samples` as allowed.
- Progress goes to stderr. `--json` returns the manifest entry.

## Acceptance

1. A fixture server serves a small counts file. `download` writes it, records the right SHA-256, and `path` prints its absolute path.
2. `--dry-run` sends no request for the asset URL. The request log proves it.
3. A known size over `--max-size` fails before transfer. An unknown size stream aborts at the limit and leaves no final file.
4. A killed transfer leaves only `.partial`. The next download removes it and succeeds.
5. A second download without `--refresh` makes no asset request. With `--refresh` and changed bytes, both versions are on disk and in the manifest.
6. An unlisted asset ID fails and lists the valid IDs.
7. An HTML body served with HTTP 200 for a gzip asset fails, names the URL, and leaves no file.
8. A file on disk edited after download makes the next download fail without overwriting it.
9. A size comes from `Content-Length` in the fixture, not from page text.
10. `path` for an asset not yet downloaded exits nonzero and makes no network request.
11. The MCP shell rejects `dataset download` and `dataset path`.
12. Spec `spec/entity/dataset.md` gains one dry-run block and one download-then-path block against the fixture server.

## Out of scope

- A content-addressed store, a SQLite catalog, pins, leases, eviction, garbage collection, backup, and restore. The brief lists these. None is built until a user needs it.
- Fetching whole series, all assets at once, SRA reads, or archive extraction.
- Run registration and project state. BotAssembly records runs.
