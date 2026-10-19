---
priority: 1
---

# 2279 — Refresh observed replay response metadata

Status: COMPLETE.

Milestone: 0.9.4

Depends on: 2277, 2278

Reviews: revision 570b42d, accept

Reviews: revision 8bfe1d2, accept

Reviews: revision a42e4ae, accept

## Outcome

Land the verified partial response metadata independently of the larger release-admission parent. Current observed media types have exact request provenance; original recording dates remain intact. Correct dbSNP request provenance and explicitly retain KEGG as unverified.

Changes: `testdata/replay/manifest.json`, `sdlc/tickets/2258-admit-the-0-9-4-release-candidate.md`, `sdlc/tickets/2277-keep-disease-pages-in-replay-storage.md`, `sdlc/tickets/2278-align-recorded-replies-with-their-requests.md`, `src/entities/disease/resolution/paging_tests.rs`, `tests/test_article_disease_filter.py`, `testdata/disease-resolution/melanoma-page-one.json`, `testdata/disease-resolution/melanoma-page-two.json`, `testdata/replay/responses/mydisease/*`, `testdata/replay/responses/ctgov/*`, `testdata/replay/responses/pubmed/pubmed-esummary-fcgi-9019b354c39b.bin`, `tests/golden/cases/get-trial-nct02576665-references.json`, `tests/golden/cases/get-trial-nct06131398-references.json`, `CHANGELOG.md`.

## Evidence

Parent 2258 source 2986c2ac6fb2da0df419242f2018387429e160e4 contains the collected manifest delta and the coordinator-owned collection brief, on stable main 1c4753a453569d17dd66a13eb3d312ab8a5af4d4. Collection 2258/1792447971557-46385 observed 40 replies from 47 entries; four were stopped after S2 rate limiting, three KEGG entries were never called. Metadata proof 2258/1792448128443-87011 passed source parity, provenance checks and 29 controlled tests. Inventory remains deliberately partial and failed. All raw evidence remains private in M5 experiment 2096-biomcp-2258-live-collection.

## Keeps

Adopt existing verified metadata; no new provider calls. Audit exact request identity. Inherited observed Content-Types cover 24 entries, but MyDisease observations 18 and 19 omitted the declared/client fields query. Restore both to their original header/observation fields; only observation 18 had an inherited update, so this removes one and retains twenty-three exact-request observations. 2278 owns exact-query recapture, their Content-Types, request declarations and provenance. Preserve the remaining exact-request observations, the corrected dbSNP endpoint and original 2026-10-15 capture provenance, and all three exact KEGG restriction notes. Do not change fixture payload bytes, checker/runtime behavior, denied/rate-limited rows or other lane fields. No invented historical metadata, no broad/licensed payload publication. Add a truthful Internal changelog citation.

## Proof

The resulting metadata fields match retained exact-request observations; no credential, payload or original-date change. Compare old/new manifest source hashes, validate dbSNP bytes/provenance and exact KEGG unverified notes, and run affected existing checker/provenance tests through pm job. Reuse the original collection and known limitations. No new full live run or gate. Record ready/blocked/disabled counts without treating them as a qualification pass. This ticket closes only partial metadata delivery; 2258 remains open for complete final qualification.

## Verified delivery proof

Developer ca31c638dfe2ea3d9544247259dccbe7e4879a17 adopts the verified parent delta, removes unsupported observation 18 metadata and leaves observation 19 original fields unchanged. The inherited assumption of twenty-two remaining refreshes was wrong: only one of the twenty-four additions belonged to these rows, so the verified result is twenty-three. Every remaining added observation matches its canonical request and retained raw response. No unrelated observation was deleted to force a count.

Focused M5 job 2279/1792448692729-6312 passed exact private patch/file checksum parity, canonical method/URL/body/header audit, raw observation hashes/sizes, original recording date/provenance checks, three dbSNP byte hashes and endpoint, all three exact KEGG unverified notes, whitespace and twenty-nine existing checker controls. No provider calls, body, runtime or checker changes. Nonlive inventory deliberately fails: total 47, ready 35, blocked 9, disabled 3, checked 0, passed 0, drift 0. It is not release qualification. Retain raw helper/log/report under 2096-biomcp-2258-live-collection/2279-metadata-delivery with OWNER2279 and Keep.

Keep: experiments/2096-biomcp-2258-live-collection/2279-metadata-delivery — retain exact-request metadata provenance and controlled partial-inventory proof.

## Reviewed-input integration

Integration revision dd3fb06a2e27ef4d7ce6a8c2c401d2037ea0bc50 merges the accepted 2277 and 2278 branches to resolve their adjacent manifest fields with this metadata delivery. The expanded claim names those inherited changes, not new product scope. Both canonical query-page paths retain their observed metadata; the refreshed PubMed recording retains full capture provenance and response-metadata observations. Recursive semantic union against all three accepted inputs passed, and every nonmanifest consumer, capture, golden, deletion and changelog hash matches its reviewed owner.

M5 integrity job 2279/1792449785625-35154 completed these checks and offline inventory before a report-only nested-key error. Extraction job 2279/1792449822089-38447 passed using retained outputs, without repeating checks or provider calls. Private integration patch SHA256 is 100d6e04eb7ab8a0689b2bd1222f45aa55626907cb7f83a65f509cd2886cfea8. Inventory remains partial: 47 total, 38 ready, six blocked, three disabled, zero checked. Existing owner-specific focused tests remain valid; combined landing gate remains required.
