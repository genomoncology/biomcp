# Show clinicians the stale-cache age and the true gene source

From the 2026-09-27 review file (Product, bullets 2-5). Two things a
clinician actually sees are wrong or missing today; two smaller
wording faults ride along. Ian's order puts this before check
hardening.

## Problem

1. **Stale-cache age is invisible on the clinician-facing paths.**
   `note_stale_cache_serve` (`src/sources/mod.rs:199-223`) reads the
   age header and writes one `warn!` line to stderr, then strips the
   header. MCP and JSON consumers never see it. Ticket 1242's
   acceptance said each listed path shows the failure. The function
   has no test.
2. **MyDisease and DisGeNET genes are labeled Open Targets.**
   `assign_top_genes` (`src/entities/disease/enrichment.rs:576-609`)
   recognizes only Monarch and CIViC as fallback sources, so genes
   seeded from MyDisease or DisGeNET get the Open Targets label on
   the default card and on the requested-sections path. The fix
   needs the real source labels, not a generic "other".
3. **Search-all dropped-filter note pastes raw upstream error text**
   (`src/entities/search_all/dispatch.rs:421`). Say plainly which
   source failed, in our words, with the upstream text available as
   detail.
4. **DDInter calls an HTML download reply an unreadable bundle**
   (`src/sources/ddinter.rs:471`). Filed 2026-09-26, dropped. Say
   what happened: the endpoint returned HTML where the bundle was
   expected.

## Fix

1. Carry the stale-cache age into the output notes on both
   clinician-facing paths: the MCP markdown card's notes area and
   the JSON body's notes/meta field, using the same honest wording
   the log line has ("older than the provider's freshness window",
   with the age). Add a unit test for `note_stale_cache_serve` (or
   its successor) covering both the present-header and absent-header
   cases, and one test per path proving the age reaches the rendered
   output. Keep stripping the header from what crosses the wire.
2. Recognize MyDisease and DisGeNET in the fallback-source
   detection with their real display names, on both paths (default
   card and requested sections). Tests must construct associations
   sourced from each and assert the heading and provenance label.
   Follow the data to the seed sites if the source strings differ
   from the display names; do not invent labels the repo does not
   already use for these sources.
3. Replace the pasted upstream text with a plain sentence naming
   the source; keep the upstream detail in a log line or a debug
   field. Test it with an upstream error fixture.
4. Fix the DDInter wording and pin it with a test using an
   HTML-content-type reply.

If any part must be deferred (for example entity-page plumbing the
age through a layer that has no notes field yet), open an issue file
naming the gap and reference it from the record; do not let it
silently drop.

## Review

- Design review: n/a (Ian's order named the items and the shape)
- Code review: REJECT once (three P0s: label double-match, unearned
  fixture assertion, MCP path bypassing the stale-notes scope),
  fixed and re-reviewed ACCEPT 2026-09-27
- Verification: yellow gate at 5754b910 — lint, test, spec, stress
  OK, zero failed lines, after eight cycles; see the record

## Acceptance

- A stale serve shows its age in an MCP card and a JSON body (tests
  prove both), and the log line keeps its meaning.
- MyDisease- and DisGeNET-sourced top-gene lists carry their real
  source labels on both paths; Open Targets is credited only for
  lists it produced.
- The search-all note and the DDInter error say what happened in
  plain words; both pinned by tests.
- Every deferral has an issue file.
- Yellow gate green.
