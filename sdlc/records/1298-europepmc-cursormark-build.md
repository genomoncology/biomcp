# 1298 build: Europe PMC cursorMark paging

Ticket: sdlc/tickets/1298-page-europepmc-with-cursormark.md
Branch: tickets/1298-page-europepmc, final sha 2d75971d
Built and driven by the branch lead; reviewed fresh twice.

## Prior evidence

Experiment 439 reproduced the row loss: Europe PMC ignores the page
parameter upstream, so every fetch returned page one and PMID dedup
collapsed searches to 18 of 50 rows while burning the full fetch budget.
Three-page walks under each supported sort (relevance, date desc,
citations desc) returned 75 of 75 unique rows under cursorMark, so sort
stability held.

## Changes landed on the branch

- src/sources/europepmc.rs: search plans take a cursor_mark argument
  (validated non-empty), send cursorMark instead of page, and parse
  nextCursorMark from responses. Single-row lookups (DOI, PMID, PMCID)
  and the retracted rescue pass the start cursor.
- src/entities/article/backends.rs: the europepmc loop carries cursor
  state, serves offset by walking and discarding rows inside fetched
  pages (bounded by the documented 1250-row window; MCP caps offset at
  1000), and terminates on the limit, an absent cursor, a repeated
  cursor, an empty page, or fetched rows reaching the hit count.
- Fixture trio spec/fixtures/{setup,run,cleanup}-europepmc-cursor-fixture.sh
  with four corpora pinning all four termination rules; spec case in
  spec/entity/article.md. Package count 1385 to 1388; the unpaced-origin
  owners list registers the new fixture.
- Legacy shapes: construction tests assert cursorMark; the disease and
  fulltext fixture handlers keyed on page moved to cursorMark.
- The variant-articles corpus canary routes by canonicalizing only the
  paging parameter, so captured bodies, receipts, and the sha256
  provenance chain stay byte-identical while the cursorMark request
  shape is served; the shape test pins the canonicalized routing.

## Proof

- Fixture: 50 rows in exactly 2 requests (no page parameter on the
  wire); hitCount stop at 2 requests; absent-cursor stop at 3 requests;
  offset 25 returns 10 rows starting at the pinned PMID.
- Live API: a limit-50 search returned 50 distinct rows.
- Design review: FIX then folded; sort walks validated live.
- Code review: ACCEPT, no blocking findings; its strengthening finding
  (pin the hitCount stop) is folded as the short corpus.
- CI: green at 2d75971d, all lanes.
- Yellow: lint and spec green; Rust 3918 of 3918 at the prior sha
  (identical Rust; the last commit touched only a Python test); full
  Python suite green there except four tests that require the offline
  sandbox wrapper, which pass through tools/run-offline. The one real
  yellow catch during gating was the corpus canary, fixed on the branch.

## Deferred gaps

- The date and citations --source europepmc CLI arm still takes raw
  limit and offset without the window guard (pre-existing; bounded
  behavior documented).
- nextCursorMark deserialization has no direct Rust unit test; it is
  covered end to end by the fixture.
