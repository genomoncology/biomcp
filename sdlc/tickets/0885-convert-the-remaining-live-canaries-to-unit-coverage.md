---
flow: build
priority: 8
---
# Convert the remaining live canaries to unit coverage

## Done when

No canary asserts upstream availability. Each one that did is either a
deterministic unit test against a captured response, or is gone with its
removal argued in the body. `make verify` failing then means BioMCP
changed, not that a provider had a bad afternoon.

## Why here, why now

Five further issues are folded in below rather than filed separately.
They are five sightings of one defect: a canary asserted that a public
service was up, the service was not, and the report reads as though
BioMCP broke. Fixing them one provider at a time cannot converge,
because the next provider outage files the sixth report.

This is the same program as tickets 0666 and 0671-0684 — moving live
assertions onto captured contracts — which is why it shares their
priority band.

## The finding

Raised under March and carried over when BioMCP moved to the sdlc
factory. Reproduced in full below; `severity` is March's word, and
this ticket's priority is the one that counts.

<!-- from 2026-07-27-convert-remaining-live-canaries-to-unit-coverage.md -->

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

## Inventory rebuilt 2026-08-09 during review triage

The lists below are July's and have gone stale; the executable
registry is the truth. Verified against `scripts/run-specs.sh`
(`SPEC_LIVE_PATHS`) and the records on this date:

- Both owning tickets named in the STOP block have LANDED: 0632
  (PubTator normalized_id) and 0633 (CSpec version paging). The
  conversion holds they imposed are released — but per the STOP
  block's lesson, re-run each canary by hand and see it pass before
  converting it.
- Item 1 (`clingen-cspec-live.md`) is OBSOLETE: it is no longer in
  `SPEC_LIVE_PATHS` and `testdata/sources/clingen_cspec/` now exists.
  Nothing to do.
- The live canaries actually in the registry today and owned by THIS
  ticket: `article-assets-live.md`, `article-graph-live.md`,
  `variant-myvariant-live.md`, `variant-articles-live.md`. The mixed
  entity and surface files also listed there belong to the conversion
  program (0666, 0669, 0671-0677, 0673), not here.
- `spec/entity/ddinter-live.md` exists on disk but is NOT in the
  registry — classify it during the work (stale file, or coverage
  that moved) and record the answer.

Derive the working inventory from `SPEC_LIVE_PATHS` at flight time,
not from the historical lists below, which stay for their reasoning
and lessons.

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

<!-- from 605-seven-variant-recall-canary-live-provider-drift.md -->

## Summary

`make verify` passed ticket 605's new G5 v2 identity canary at 7/7, but the older Seven-Variant Recall Canary failed every recall/coverage threshold. All seven legacy variants were reported incomplete.

## Detail

The failing verify run reported `reference_recall_at_least_9_of_12: false`, `variant_coverage_at_least_6_of_7: false`, and `route_specific_pmids_present_for_expected_variants: false`. `mlh1_family_pmids_present` remained true. The incomplete list was APC p.E1317Q, APC p.Q2322R, ATM p.C2464R, BRCA1 p.M1783I, MLH1 p.G67E, MSH2 p.L341P, and PTEN p.D326N.

The new authoritative-identity canary passed directly and inside the same live lane, so this is not evidence that ticket 605's RefSeq identity path is broken. It is an unresolved live-provider/recall failure in the pre-existing release canary. The current aggregate output does not expose enough per-route source status to identify the failing provider cheaply.

## Suggested action

Destination: **verify-group / experiment-harness**. Re-run the legacy canary in a credentialed release environment. If it remains red, capture each variant's route-level `source_status`, matched aliases, and terminal fields in the canary artifact, then repair the failing provider route rather than weakening PMID or coverage thresholds. Keep release promotion blocked until the existing canary is green or the provider failure is concretely triaged.

<!-- from 606-live-verify-monarch-and-citation-canaries-unavailable.md -->

## Summary

`make verify` could not complete on 2026-07-22 because the live Monarch phenotype canary returned an HTTP middleware failure and the live related-paper citation canary exited nonzero. These are outside ticket 606's provider-query behavior.

## Detail

The ticket's real-provider strict-query canary passed independently with the release binary after the failure. The full verify command nevertheless exited 2 on these unrelated live assertions, leaving an operator unable to distinguish ticket evidence from transient upstream availability.

## Suggested action

Investigate the two provider failures and either repair the live integrations or classify expected upstream unavailability through the existing `verify-group`/operator policy. Add a `verify-group` test or harness check that preserves red failures for unexpected response shapes while reporting known transient availability explicitly.

<!-- from 612-live-verify-unrelated-diagnostics.md -->

## Summary

`make verify` cannot provide a green aggregate result because unrelated live diagnostics fail or time out, even though ticket 612's two CAR assertions pass directly with the release binary.

## Detail

On 2026-07-24, the live suite failed article recommendation/citation assertions, the G5 v2 readiness diagnostic timed out, and the discover-code labels diagnostic lacked expected SNOMEDCT and ICD10CM source rows. `spec/entity/clingen-car-live.md` passed both assertions directly with `BIOMCP_BIN=target/release/biomcp mustmatch test spec/entity/clingen-car-live.md --lang bash -v`.

## Suggested action

Investigate each failing upstream diagnostic and either repair its runtime behavior or make the verify harness report independently attributable source failures. Improved-test destination: verify-group.

## Audit update — 2026-07-24

- The discover label failure no longer reproduces on current main: a no-cache direct
  command returned both `SNOMEDCT` and `ICD10CM` with no errors. Issue 601 is closed with
  the exact commit/binary proof.
- An independent direct G5 v2 run again reported 7/7 resolved identities, exact routes,
  route-tied aliases, source status, and terminal state. Its prior timeout is not a
  current identity-contract failure, although honest live incompleteness remains visible.
- The legacy seven-variant recall gate is independently still red and remains release
  blocking under issue 605.
- Recommendation/citation diagnostics require one current attributed rerun before their
  disposition is changed.

Ticket 623 owns the final grouped reconciliation and must not weaken any threshold or
convert a required provider outage into healthy emptiness.

<!-- from 614-live-verify-canary-drift.md -->

## Summary

The live verify matrix is not fully green despite the ticket's ClinGen LDH probe
passing. Four unrelated public-upstream canaries failed during verification.

## Detail

`make verify` and the post-repair shared live runner failed these existing checks:

- `spec/entity/article-assets-live.md`: PMID 20516115 no longer reports both named
  supplements in an acceptable provider-labelled outcome.
- `spec/entity/article-graph-live.md`: Semantic Scholar recommendation requests
  for PMID 23450558 and PMID 22663011 exited nonzero.
- `spec/entity/clingen-cspec-live.md`: the specified ATM CSpec resource no longer
  returned the required capture-provenance fields.
- `spec/entity/variant-hotspots.md`: the BRAF V600E structure-context request
  exited nonzero.

The new `spec/entity/clingen-ldh-live.md` passed in the same runner after its
omission was repaired, so these failures are not caused by the LDH change.

## Suggested action

Investigate each current provider response, then either repair the client or
retarget the live probe to a still-supported stable resource. Preserve the
live lane for genuinely mutable upstream behavior; add deterministic fixture
or request-contract coverage under `test`/`spec` when a client-side regression
is found.

## Superseded — 2026-07-27

Ian's standing direction resolves the "repair or retarget" question above: a
live canary may be replaced by unit tests provided those tests fully cover the
CLI-to-API-call transition and the parsing of a locally captured response.
Only "does the provider still serve this today" is dropped, and that was never
BioMCP behaviour.

The conversion plan and per-canary priority order live in
`2026-07-27-convert-remaining-live-canaries-to-unit-coverage.md`. That issue
owns the remaining work; this one records the original observation.

`spec/entity/article-indexing-live.md` is already retired with offline
replacement (ticket 614, merged `89cf6f11`). Three remain:
`clingen-cspec-live.md` (must be **converted, not deleted** — zero unit tests
today), `article-assets-live.md`, `article-graph-live.md`.

`spec/entity/variant-hotspots.md` is listed above but is **not** covered by the
conversion issue and still needs its own look.

<!-- from 636-make-semantic-scholar-canary-credential-routing-deterministic.md -->

The live article-graph page cannot reliably prove its stated `S2_API_KEY` dependency: during verification, its unauthenticated Semantic Scholar detail and recommendations requests both returned HTTP 200 and the entire page passed 11/11 with `S2_API_KEY` unset.

The live provider receipts were `GET /graph/v1/paper/PMID:23450558?...` → 200 and `GET /recommendations/v1/papers/forpaper/97ae9501d5f7f5ddc2d38ea98abdca2dc4939d42?...` → 200 for both authenticated and unauthenticated requests. The ticket's historical 429 is therefore a transient provider state, not a reproducible contract.

Suggested action: add a deterministic `test` or `spec` fixture that captures the `x-api-key` header and verifies the live-canary command uses the raw release binary (key present), while `tools/biomcp-ci` deliberately omits it. Keep the live page as a provider smoke test rather than treating an intermittent anonymous 429 as a required red proof.
