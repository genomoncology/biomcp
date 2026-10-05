# 1298 — page Europe PMC with cursorMark; the page parameter is ignored upstream

Filed 2026-10-05 by the BioMCP 0.9 lead, from experiment 439.

Status: OPEN.

## Outcome

Europe PMC searches return distinct rows for every requested page, up to the requested limit, and stop fetching when the limit is reached. A `--limit 50` search returns 50 distinct rows.

## Evidence

- Starts from: Experiment 439 (`~/workspace/experiments/439-reproduce-1293-p2-and-1291-switch/`) verified with direct upstream calls, no BioMCP involved: `.../search?query=...&pageSize=25&page=1|2|3` return identical result lists (same PMID sequence, byte-identical rows) for two different queries. Adding `cursorMark=*&page=2` still returns page 1's rows; the response carries a working `nextCursorMark`. Europe PMC's supported pagination is cursorMark-based; `page` is silently ignored.
- Keeps: Query construction, the dedup guard, and the result shapes stay. Federated merges keep their order.
- Changes: `search_europepmc_page` pages with `page` (`src/sources/europepmc.rs:234`), so every fetch returns page 1. Measured on main a877443f: `search article --source europepmc "glioblastoma treatment resistance" --limit 50 -j` returns 18 rows of 50 after all 50 MAX_PAGE_FETCHES (~30 s), with a misleading deep-page warning. Page with `cursorMark` instead: first request `cursorMark=*`, follow `nextCursorMark`. Keep dedup as a safety net and keep the deep-page warning keyed to real page count.
- Proof: A recorded fixture serving distinct rows per cursorMark, as a spec case showing `--limit 50` returns 50 distinct rows; a regression asserting the request carries `cursorMark`, not `page`.
- Defers: None.

## Priority note

P1. Every Europe PMC search and every federated search that includes Europe PMC returns a fraction of its rows and burns its page-fetch budget today. This predates ticket 1293 (QA 0003 recorded identical rows on both binaries) and changes every search result count, so it lands before the 59-question rerun.
