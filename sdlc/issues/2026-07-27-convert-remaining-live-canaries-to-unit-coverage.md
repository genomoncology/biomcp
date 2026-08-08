# Convert the remaining red live canaries to request-plan + captured-response unit tests

Severity: should-fix.

This is live-verify-lane work. `make verify` is deliberately not a
gate rung here (`sdlc/planning/verify-lane.md`), so this cannot fail
a flight and should not outrank gate-lane work when it is triaged.

Carried over from March, where it was raised against ticket 614
on 2026-07-27 and left open. The text
below is as filed.
## Summary

`make verify` is red on `main` on live canaries unrelated to any ticket in
flight. They assert upstream *availability*, which is not BioMCP behaviour, and
they block every ticket that reaches `05-verify`.

Ian's standing direction: a live test may be replaced by unit tests provided
those unit tests fully cover the CLI-to-API-call transition and the parsing of a
locally captured response. Everything on our side of the socket must stay
covered; only "does the provider still serve this today" is dropped.

## Already done under ticket 614

`spec/entity/article-indexing-live.md` is retired. Its PubMed half is now
covered by:

- Tier 2 `citation_plan_sets_required_query_params_and_api_key` (existing) —
  exact method, path and query, with the API key excluded from the recorded shape.
- Tier 3 `parses_a_real_pubmed_citation_capture` (new) — the real
  `efetch.fcgi` response for PMID 22663011, captured 2026-07-27 and committed at
  `testdata/sources/pubmed/efetch_citation_22663011.xml`. It passed first run,
  so the parser does match the live shape.

The hand-written citation XML test was kept alongside it: the real capture
proves fidelity, the synthetic one covers ORCID, collective-name and unicode
edges the capture does not contain.

## STOP — read this before converting anything (2026-07-30)

**Two of the canaries listed below are catching real product defects, not
provider drift.** They were hand-driven against live providers on 2026-07-30
while every routine gate was green on `main` at `ad4c4f96`:

- **`clingen-cspec-live.md` (item 1)** — `gene cspec <GENE> --version <IRI>`
  fails for every version IRI BioMCP's own manifest advertises. The fetch,
  envelope check, capture write, and capture read-back all succeed; the paging
  step fails and the result is misreported as a ClinGen API failure advising
  "retry the remote source." Owned by **ticket 633**.
- **`article-assets-live.md` (item 2)** — `get article <pmid> assets` fails for
  essentially every article because `PubTatorAnnotationInfons.normalized_id` is
  typed `Option<u64>` and PubTator returns a string for Disease annotations.
  This also silently disables PubTator article identity verification entirely.
  Owned by **ticket 632**.

Retiring either canary would have deleted the only detection of a live defect.
Neither may be converted or retired until its owning ticket has landed and the
canary passes; the offline tiers that replace it must be written against a real
capture that exercises the shape which broke, or the replacement inherits the
blind spot.

The other two entries were re-checked the same day:

- **`article-graph-live.md` (item 3)** — passed on manual re-run; the verify-run
  failure was a Semantic Scholar transient. Genuine flake.
- The **Seven-Variant Recall canary** in `variant-articles-live.md` fails only
  because `NCBI_API_KEY` is absent from the machine's environment. It preflights
  out before doing any work. Environment gap, not a defect and not drift.

**General lesson for this issue:** "the provider changed" was the recorded
explanation for both 632 and 633, and it was wrong both times. Reproduce each
remaining canary by hand and confirm the provider is actually at fault before
scheduling any conversion.

## Remaining, in priority order

1. **`spec/entity/clingen-cspec-live.md`** — highest risk. `src/sources/clingen_cspec.rs`
   has **zero** unit tests and there is no `testdata/sources/clingen_cspec/`
   capture. Retiring this canary before adding tiers 2 and 3 would reduce
   coverage, so it must be converted, not deleted. Routine coverage exists only
   through the frozen `spec/entity/clingen-cspec.md` fixture lane.
2. **`spec/entity/article-assets-live.md`** — `ncbi_efetch` has 5 construction and
   7 parsing tests plus a `pmc_wrapped_article.xml` capture. Needs a captured
   JATS/PMC-HTML pair that names a PDF and a workbook so the per-file coverage
   outcomes are asserted offline.
3. **`spec/entity/article-graph-live.md`** — `semantic_scholar` is the best covered
   of the three (20 construction, 10 parsing tests, 9 captures). Most likely to
   retire cheaply once the recommendations path has a captured empty-collection
   response.

4. **`spec/entity/variant-hotspots.md`** — the BRAF V600E structure-context
   request exited nonzero during 614 verification. Carried over from
   `614-live-verify-canary-drift.md`; not yet assessed for tier coverage, so its
   position in this order is provisional.

## Also noted, not blocking

The Europe PMC supplementary half of the retired page is covered structurally
(request plan, plus five zip-parsing tests) but is **not anchored to a real
capture** — the zip fixtures are synthesized. A real supplementary zip is around
250 KB, which is a poor trade for a committed fixture; a smaller open-access
article should be found if anchoring is wanted.

## Why this matters beyond the flakes

A synthesized fixture can only confirm that a parser agrees with itself. Ticket
614 found a feature whose every fixture was hand-written to the shape the code
expected, so four defects survived the whole ticket lifetime with a green gate.
Anchoring each provider to at least one real capture is the cheap defence.
