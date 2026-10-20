---
flow: build
priority: 4
deps: [1208]
---

# 1217: Dataset download resumes an interrupted transfer

## Goal

`biomcp dataset download` continues an interrupted transfer instead of starting again. When a staging file is on disk and the manifest has no completed entry for that asset, the next download asks the server for the missing bytes with a range request and appends them. A server that ignores the range restarts the transfer. `--no-resume` starts again on purpose.

```
biomcp dataset download geo:GSE256354 --asset matrix:GPL15520
biomcp dataset download geo:GSE256354 --asset matrix:GPL15520 --no-resume
```

The motivating consumer is a hackathon team screening public GEO studies of drug-treated AML cell lines. A dropped connection on a 210 MB matrix file costs that team the whole transfer today.

## Current Facts

- Ticket 1208 streams each asset into `<name>.partial` in the destination folder and renames it on success. A failed or aborted transfer leaves the staging file and writes no manifest entry. The next download in 1208 truncates that file and transfers again from byte zero. This ticket replaces that one rule and changes nothing else about the command.
- Ticket 1208 validates a completed file by its content type and its first bytes, and a gzip asset must start with `1f 8b`. It counts the whole file against `--max-size` and against the free-disk floor.
- No source sends an HTTP `Range` request today. `grep -rn "Accept-Ranges\|Content-Range" src` matches only `src/sources/cpic.rs:375-376`, inside `parse_content_range_total` (`src/sources/cpic.rs:373`), which reads a `content-range` header it did not ask for. Resume is new code.
- The streaming download pattern is `CBioPortalDownloadClient::download_study_archive_to_path` (`src/sources/cbioportal_download.rs:97`). It builds its client with `ordinary_middleware_client_for_base` (`src/sources/ordinary_url_policy.rs:199`, used at `src/sources/cbioportal_download.rs:282`), checks `content_length` against a byte cap (`:125`), creates the destination file (`:137`), and reads chunks with an idle timeout (`:141`). Nothing in that path opens a file for append.
- Temp paths already carry the process ID (`create_temp_path`, `src/cli/article/session.rs:285`), which is how two processes avoid one staging name.
- Measured 2026-09-16 (experiment 203): the GEO download service sends `Content-Length` and `Content-Disposition size=`, and no `ETag` and no `Last-Modified`. The largest measured series matrix file is 210 MB. DepMap's dependency matrix is 429 MB.
- With no validator to send, a conditional resume is impossible. The server can offer no proof that the bytes already on disk came from the file it is serving now.

## Design

- **When resume runs.** A `<name>.partial` file exists, the manifest has no completed entry for that asset, and `--no-resume` was not given. Otherwise the transfer starts at byte zero and truncates any staging file first.
- **The request.** `download` sends `Range: bytes=<staged size>-` and opens the staging file for append.
  - HTTP 206 with a `Content-Range` whose first byte equals the staged size: append the body.
  - HTTP 206 with a `Content-Range` naming another start byte: fail, name both byte offsets, and leave the staging file untouched. BioMCP never writes bytes at an offset the server did not confirm.
  - HTTP 200: the server ignored the range. Truncate the staging file and write the whole body.
  - HTTP 416: the staged size is at or past the end of the file. Truncate the staging file and start again.
  - Any other status is an error and the staging file stays.
- **Validation is unchanged.** The gzip magic-byte and content-type check runs on the first bytes of the completed file, never on the resumed chunk, because a resumed chunk starts mid-file. `--max-size` and the free-disk floor count the whole file, resumed bytes included. The free-disk check runs on the bytes still to come.
- **A stale staging file.** GEO sends no `ETag` and no `Last-Modified`, so a resume cannot be made conditional and BioMCP cannot prove the staged bytes match the file the server holds now. The completed file's SHA-256 is recorded either way, so a resumed file that mixes two versions is visible as a SHA-256 that differs from a later download. The docs say plainly that resume trusts the staged bytes, and `--no-resume` is the way to discard them.
- `--no-resume` deletes the staging file and starts the transfer again. It needs no `--refresh`, because a staging file means no completed entry exists and `--refresh` governs completed files.
- `--dry-run` reports whether a staging file exists, its size, and whether the next run would resume or restart. It transfers nothing.
- The output names the resumed byte count. `--json` returns the manifest entry, unchanged in shape.
- The MCP surface is unchanged. `dataset download` stays rejected with `DATASET_LOCAL_MCP_REJECTION_MESSAGE`, the constant ticket 1208 owns.

### Docs

- The GEO source page and `docs/user-guide/cli-reference.md` state that a download resumes by default, that `--no-resume` discards the staged bytes, and that GEO publishes no validator, so resume trusts what is on disk.
- `CHANGELOG.md` gains an entry.

## Acceptance

Fixture-backed Rust tests, no live network. The fixture server answers range requests under the download base.

1. A killed transfer leaves `<name>.partial` and no manifest entry. The next download sends `Range: bytes=<staged size>-`, the server answers HTTP 206 with a matching `Content-Range`, and the final file's SHA-256 equals an uninterrupted download's.
2. A server that answers HTTP 200 truncates the staging file and produces the same SHA-256.
3. A `Content-Range` naming another start byte fails, names both offsets, and leaves the staging file byte for byte as it was.
4. HTTP 416 truncates and restarts, and produces the same SHA-256.
5. `--no-resume` deletes the staging file, sends no `Range` header, and produces the same SHA-256.
6. The gzip magic-byte check runs on the completed file. A resumed transfer whose completed bytes are HTML fails and leaves no final file.
7. `--max-size` counts staged bytes plus incoming bytes. A resume that would cross the limit aborts and leaves no final file.
8. The free-disk check runs on the bytes still to come, from an injected filesystem-space reader, and its message names the threshold, the available bytes, and the total.
9. `--dry-run` with a staging file present reports the staged size and the word resume, and sends no request for the asset URL.
10. Spec `spec/entity/dataset.md` gains one block that interrupts a download and resumes it against the fixture server.

`make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA.

## Out of scope

- Resuming a `study download` archive or a `depmap sync` file. Those install through their own clients.
- A conditional resume with `If-Range`, an `ETag`, or a `Last-Modified` check. GEO publishes none of them.
- Parallel or chunked downloads, a progress bar, and any retry policy beyond what ticket 1208 already has.
- Resuming a completed file. A completed entry is governed by `--refresh` (ticket 1208).

## Decisions

Open to Ian's overturn.

1. Resume is on by default. `--no-resume` is the escape hatch.
2. Resume is split out of ticket 1208 because it is independent new code with its own HTTP semantics and no other ticket needs it. Ticket 1208 already carries eight features and 19 acceptance items.
3. A resume trusts the staged bytes. GEO offers no validator, so the alternative is to discard every staging file, which is what this ticket exists to avoid. The recorded SHA-256 makes a mixed file visible.
4. A `Content-Range` that names another start byte fails rather than restarting. A server that answers a different offset is not behaving as asked, and a silent restart would hide it.

## Review

- Design review: pending
- Code review: pending
