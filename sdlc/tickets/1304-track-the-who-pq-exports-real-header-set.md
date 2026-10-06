# 1304 — Track the WHO PQ export's real header set and degrade honestly

Status: OPEN.

## Outcome

`biomcp who sync` downloads the WHO Prequalification exports, because the
validator accepts the header set the export ships today; when a refresh
does fail, the command reports what actually happened instead of claiming
success or aborting dependent searches. GitHub issue #288.

## Evidence

- Live probes 2026-10-06: all three export endpoints return HTTP 200
  with real CSV from an ordinary network. The medicines exports ship
  title-case headers (`"WHO Reference Number"`, `"Therapeutic Area"`,
  `"Date of Prequalification"`) and the finished-pharma export no longer
  carries a `Basis of Alternative Listing` column at all.
- `src/sources/who_pq.rs` REQUIRED_HEADERS and API_REQUIRED_HEADERS pin
  the old uppercase schema including the dead column, so both medicines
  files fail validation; the vaccines header set still matches.
- Failure rendering collapses to "API request to WHO Prequalification
  failed. Retry the remote source." with no detail.
- On a host with no cached data the sync aborts as Source unavailable and
  the region-less drug search aborts with it instead of degrading the way
  EMA does (issue #288 reproduction). On a host with partial data the
  sync prints two refresh warnings and then claims "synchronized
  successfully" with essentially nothing downloaded — reproduced
  2026-10-06.

## Change detail

1. Re-derive the required header sets from recorded current captures of
   all three exports (`src/sources/who_pq.rs`), record the captures with
   receipts, and validate case-insensitively on the trimmed header text.
2. Make `who sync` report the true outcome: which files refreshed, which
   failed, and exit nonzero when required files are missing after the
   run.
3. Make the region-less drug search degrade like EMA when WHO PQ data is
   absent, instead of aborting.
4. Fold the issue's diagnosis notes into the error path: a validation
   failure names the file and says headers did not match, never a generic
   retry message.

## Keeps

- `--region us` and `--region eu` behavior stays unchanged.
- The stale-after window, size hints, and body caps stay unchanged.

## Proof

- A spec or fixture test replaying the recorded current exports proves the
  three files parse and required columns are found.
- A failed-refresh case proves the summary names the failing file and the
  exit code is nonzero.
- The region-less search with WHO PQ absent returns results with a
  degradation warning, matching the EMA pattern.

## Priority note

P2: blocks every `who sync` and every region-less drug search on fresh
installs; P1 for users who need WHO PQ data.
