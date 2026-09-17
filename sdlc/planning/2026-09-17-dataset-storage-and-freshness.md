# Change proposal: dataset storage, freshness, and source dates

Written 2026-09-17. It covers tickets 1204 to 1214, which add a `dataset` entity, local imports, and cell line sources. Evidence comes from two measured spikes: a GEO contract proof over 774 series matrix files, and a cell line source survey over 11 sources and 10 cell lines. Nothing here changes what the commands compute. It changes what BioMCP records, where it writes, and what it prints.

## Why

The tickets get a file onto disk and turn it into a study. They leave four questions unanswered.

- A downloaded file never expires. Nothing tells a user that the provider changed the record.
- Downloaded datasets have no disk budget, while the HTTP cache has three.
- A user cannot list what is already on disk, or see which studies came from which dataset.
- Each source publishes a different version signal, and no output prints one.

## Proposal

### 1. Record the provider's own dates

GEO series matrix headers carry `!Series_submission_date`, `!Series_last_update_date`, and `!Series_status`. Ticket 1211 already streams that header and discards those lines.

- 1211 keeps the three lines and returns them as `submitted`, `last_updated`, and `status`.
- 1204 prints them on the dataset card. A status that is not public renders as a warning line.
- 1208 stores `upstream_last_updated` in the manifest entry at download time.
- 1209 copies it into `import.json` and the study description.

A header read costs one request and stops at the table marker. It is the cheapest true freshness signal GEO offers, because the download service sends no `ETag` and no `Last-Modified`.

### 2. Add a freshness check

New command `biomcp dataset check [<id>]`. It re-reads the matrix header for each downloaded series, compares `last_updated` against the manifest, and prints one row per asset: current, changed upstream, withdrawn, or unknown. It downloads nothing and changes nothing. `--refresh` on `dataset download` stays the only way to fetch new bytes.

`unknown` is the honest answer for NCBI count files and supplementary files, because those carry no date of their own.

### 3. List what is on disk

New command `biomcp dataset list`. It reads the manifests under the dataset root and prints series, assets, bytes, `downloaded_at`, and the studies imported from each asset. It makes no network request.

This is the only indexing BioMCP should do over downloaded data. Searching inside the values is a database with its own index format and query language. `study query` and `study filter` already answer per-study questions. Anything larger belongs outside BioMCP.

### 4. Link a study back to its dataset

`import.json` already records the dataset, the asset, and the SHA-256. The dataset manifest gains an `imports` list with the study name and the import time. Both directions are then visible, and `dataset list` can show them.

### 5. One data root

Today `BIOMCP_DATASET_DIR` and `BIOMCP_STUDY_DIR` are separate settings, so nothing shows that a study came from a dataset.

- Add `BIOMCP_DATA_DIR` with `datasets/` and `studies/` under it.
- `BIOMCP_STUDY_DIR` and `BIOMCP_DATASET_DIR` keep working and win when set, so existing installs are unaffected.
- A multi-platform series writes each platform's files into a `<GPL>/` subfolder. Asset IDs are unchanged, because callers treat them as opaque.

### 6. A disk budget for downloads

The HTTP cache has a size cap, an age cap, and a free-disk floor. Downloads have none, and one GEO search filled 1.1 GB.

- `dataset download` refuses to start when free disk would drop below the cache's `min_disk_free` threshold, and names the threshold.
- `dataset list` prints the total on disk.
- Nothing is evicted automatically. Downloaded bytes are inputs to analyses, so BioMCP never deletes them on its own.

### 7. Print the release on every source output

Each source publishes a different signal, measured 2026-09-17: Cellosaurus 56.0 (2026-06-25), DepMap 24Q4 (2024-12-10), HPA files (2025-11-05), ChEMBL_37 (2026-05-01), and PharmacoDB with no version at all.

Every card, section, and JSON payload from these sources carries a `data_as_of` field holding the release name, the release date, or the retrieval time when the provider publishes neither. Tickets 1202, 1205, 1206, 1213, and 1214 each state which one applies.

### 8. Print the license on every source output

Terms differ, and one is not a plain open license: Cellosaurus CC BY 4.0, DepMap CC BY 4.0, HPA CC BY (the repo says CC BY-SA 4.0 and the survey read CC BY 4.0, so 1213 asks for a recheck), ChEMBL CC BY-SA 3.0, and PharmacoDB CC BY-NC 4.0.

Every source output carries a short attribution line naming the license. A non-commercial source says so in that line, because a user cannot tell from the data.

### 9. One writer per manifest

A loop over many series will run downloads at the same time. Each manifest write takes a lock file in the series folder. A blocked writer waits, then fails with the holder's process ID. Without this, two downloads corrupt one record.

### 10. Resume a large download

The largest matrix file measured is 210 MB, and DepMap's dependency matrix is 429 MB. A dropped connection today means starting again.

`dataset download` writes to a staging file, records the bytes received, and resumes with a range request when the server supports it. A server that ignores the range restarts the transfer. A partial file is never returned, which is already the rule.

### 11. Say that analysis works offline

Once assets are on disk, `study import`, `study query`, `study filter`, `study compare`, and `study score` make no network request. The docs state this, and `dataset path` is the way to reach files without a network.

### 12. Fail plainly on a bot check

The DepMap portal and the NCBI annotation link both return a human verification page with a success code. Every new source states what it does when that happens: report the URL, name the file, and stop. BioMCP never works around a check.

## What this does not change

- No command computes anything new.
- Asset IDs, product kinds, and the sample map stay as the tickets define them.
- Nothing is deleted automatically, and no import changes under a running analysis.

## Ticket edits

| Ticket | Edit |
|---|---|
| 1204 | Print `submitted`, `last_updated`, `status`, and `data_as_of` on the card |
| 1207 | Carry `data_as_of` and the attribution line in assets and products output |
| 1208 | Store `upstream_last_updated`; add the disk floor, the manifest lock, resume, and the `imports` list |
| 1209 | Copy `upstream_last_updated` into `import.json` and the study description |
| 1211 | Keep the three header lines and return them |
| 1202, 1205, 1206, 1213, 1214 | Add `data_as_of` and the license line; state the bot-check behavior |
| New | `dataset check` (ticket 1215) and `dataset list` (ticket 1216) |
| Docs | One data root, the platform subfolder, offline analysis, and the three cache layers |

## Decisions

Open to Ian's overturn.

1. Nothing is evicted from the dataset root automatically.
2. `dataset list` reads manifests only. BioMCP builds no index over downloaded values.
3. `BIOMCP_DATA_DIR` is added, and the two older settings keep working and take precedence.
4. `dataset check` re-reads headers only, and reports `unknown` where a provider publishes no date.
