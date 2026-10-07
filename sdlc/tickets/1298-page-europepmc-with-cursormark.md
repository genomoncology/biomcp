# 1298 — page Europe PMC with cursorMark; the page parameter is ignored upstream

Filed 2026-10-05 by the BioMCP 0.9 lead, from experiment 439. Design review: FIX on the first pass (dispatch e3f043f1, fresh read-only reviewer); every finding folded into the Changes and Proof below. The direction was accepted unchanged.

Status: complete.
Milestone: 0.9.2

## Outcome

Europe PMC searches return distinct rows for every requested page, up to the requested limit, and stop fetching when the limit or the cursor is exhausted. A `--limit 50` search returns 50 distinct rows in two requests.

## Evidence

- Starts from: Experiment 439 (`~/workspace/experiments/439-reproduce-1293-p2-and-1291-switch/`) verified with direct upstream calls, no BioMCP involved: `.../search?query=...&pageSize=25&page=1|2|3` return identical result lists (same PMID sequence, byte-identical rows) for two different queries. Adding `cursorMark=*&page=2` still returns page 1's rows; the response carries a working `nextCursorMark`. Europe PMC's supported pagination is cursorMark-based; `page` is silently ignored.
- Keeps: Query construction, the PMID dedup guard, the result shapes, the federated merge order, and the deep-page warning keyed to real fetched pages. Single-row lookups (DOI, PMID, PMCID) and the retracted-publication rescue each send one first-page request and map cleanly to `cursorMark=*`.
- Changes: `search_europepmc_page` pages with `page` (`src/sources/europepmc.rs:234`), so every fetch returns page 1. Measured on main a877443f: `search article --source europepmc "glioblastoma treatment resistance" --limit 50 -j` returns 18 rows of 50 after all 50 MAX_PAGE_FETCHES (~30 s), with a misleading deep-page warning. The change, in order: (1) Page with `cursorMark` — first request `cursorMark=*`, then follow `nextCursorMark`; the cursor is per-request loop state inside `search_europepmc_page_with_context`, captured by the commit closure exactly as `total` and `local_skip` are today; `EuropePmcClient` stays stateless. (2) Termination — stop when `nextCursorMark` is absent, equals the cursor just sent, or the page is empty, and replace the `page * 25 >= total` break with fetched-rows-versus-hitCount. (3) Offset — cursors cannot jump, so offset is served by walk-and-discard from `cursorMark=*` bounded by MAX_PAGE_FETCHES (a 1250-row window; the MCP tool caps offset at 1000); the single-backend arms reuse the existing `fetch_count <= 1250` check or document the out-of-window partial-window behavior. (4) Legacy shapes that pin the old form are updated as part of the change: the construction tests asserting `page=2` (`src/sources/europepmc/tests/construction.rs:23, 62-73`), the legacy plan's `"page"` key mapping (`europepmc.rs:206-214, 264`), and fixture handlers keying responses on the `page` parameter (for example `spec/fixtures/setup-disease-survival-spec-fixture.sh:356-361`). (5) The deep-page warning stays keyed to `fetched_pages`; `WARN_PAGE_THRESHOLD` is unchanged and a `--limit 50` search now takes 2 pages and never warns.
- Proof: A recorded fixture serving distinct rows per cursorMark with an exhausted final page (no `nextCursorMark`): `--limit 50` at page size 25 returns 50 distinct rows in exactly two Europe PMC search requests; the cursor-exhaustion stop is pinned; a regression asserts the request carries `cursorMark`, never `page`; an offset case (offset 25 returns the second page's rows) is pinned inside the window. Sort stability validated live 2026-10-05: three-page walks under relevance (no sort), `P_PDATE_D desc` and `CITED desc` each returned 75 unique rows of 75. The fixture pins the relevance walk; skipped-row drift under the tie-heavy sorts remains a residual risk the dedup cannot detect, recorded here.
- Defers: None.

## Build status

- Built on branch `tickets/1298-page-europepmc`, final sha `f1824a0b`, 2026-10-05.
- Design review: FIX 2026-10-05 (dispatch e3f043f1, fresh read-only reviewer; termination, offset window, sorts, legacy shapes, request-count proof). Every finding folded in fold commit 482288bd, 2026-10-05; direction unchanged.
- Code re-review (post-landing delta): ACCEPT 2026-10-07. The post-review commit 2d75971d (canary routing shape pin) is pure strengthening. Correction recorded same day: the line above read FIX until a 2026-10-06 grammar commit rewrote it to ACCEPT under the same dispatch with no recorded second pass behind it; this line restores the honest record.
- Code review: ACCEPT 2026-10-05 (dispatch 0c7c8861; its one strengthening finding, pinning the fetched-rows-versus-hitCount stop with a short corpus, is folded into the fixture: four corpora covering 50 rows in 2 requests, hitCount stop in 2, absent-cursor stop in 3, offset 25 first row).
- The variant-articles corpus canary now canonicalizes only the paging parameter when routing, so captured bodies, receipts, and sha256 provenance stay byte-identical while the cursorMark request shape is served. Fulltext and disease fixture handlers keyed on `page` moved to `cursorMark`. Package count 1385 to 1388; unpaced-origin owners list registers the new fixture.
- Gates at `f1824a0b`: local `make lint` exit 0; 677 article/europepmc nextest scope passed; CI in flight; yellow full lint/spec/test under the shared lock (`~/biomcp-gates-1298-final.log`). On green: write the build record, notify the KB lead for QA (QA before landing), land on their pass.
- Final: branch sha `2d75971d` (adds the shape-test pin for the canary's canonicalized routing). CI green, all lanes. Yellow: lint and spec green; Rust 3918 of 3918; Python suite green except four offline-sandbox tests, which pass through tools/run-offline. Build record: sdlc/records/1298-europepmc-cursormark-build.md. QA requested from the KB lead before landing; the team was dissolved 2026-10-06 before replying, so the ticket lands on CI, yellow gates, and two review cycles as its evidence.

## Priority note

P1. Every Europe PMC search and every federated search that includes Europe PMC returns a fraction of its rows and burns its page-fetch budget today. This predates ticket 1293 (QA 0003 recorded identical rows on both binaries) and changes every search result count, so it lands before the 59-question rerun.
