# Europe PMC ignores the page parameter; BioMCP's Europe PMC searches return one page forever

Status: closed.

Resolution: filed as ticket 1298 (page Europe PMC with cursorMark); root cause verified 2026-10-05 in the recorded repro experiment.

Priority: P1. Every Europe PMC search and every federated article search that includes Europe PMC returns a fraction of its rows and burns its page-fetch budget today, against a live upstream change.

Filed 2026-10-05 from the recorded repro experiment (`~/workspace/experiments/439-reproduce-1293-p2-and-1291-switch/`).

## What happens

Direct calls to Europe PMC's REST search (no BioMCP involved) with `query=...&pageSize=25&page=1`, `page=2`, `page=3` return identical result lists: the same PMID sequence, byte-identical row sets, for two different queries. Adding `cursorMark=*&page=2` still returns page 1's rows, and the response carries a working `nextCursorMark`. Europe PMC's supported pagination is cursorMark-based; the `page` parameter is silently ignored.

`search_europepmc_page` pages with `page` (`src/sources/europepmc.rs:234`). Every fetch returns page 1; the dedup set collapses the results. Measured on main a877443f: `search article --source europepmc "glioblastoma treatment resistance" --limit 50 -j` returns **18 rows of 50** after all 50 MAX_PAGE_FETCHES (~30 s), with a misleading "article search is deep (>20 page fetches)" warning. The defect predates ticket 1293: QA 0003 recorded identical rows and order on both binaries.

## Change

Page Europe PMC with `cursorMark`: first request `cursorMark=*`, follow `nextCursorMark`. Keep the dedup guard as a safety net and keep the deep-page warning keyed to real page count. Pin the behavior with a recorded fixture that serves distinct pages by cursorMark.

## Proof

A spec case with a recorded fixture returning different rows per cursorMark, showing `--limit 50` returns 50 distinct rows; and a regression showing the current `page=` form is no longer sent.
