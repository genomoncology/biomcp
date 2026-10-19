---
priority: 1
---

# 2279 — Refresh observed replay response metadata

Status: OPEN.

Milestone: 0.9.4

## Outcome

Land the verified partial response metadata independently of the larger release-admission parent. Current observed media types have exact request provenance; original recording dates remain intact. Correct dbSNP request provenance and explicitly retain KEGG as unverified.

Changes: `testdata/replay/manifest.json`, `sdlc/tickets/2258-admit-the-0-9-4-release-candidate.md`, `CHANGELOG.md`.

## Evidence

Parent 2258 source 2986c2ac6fb2da0df419242f2018387429e160e4 contains the collected manifest delta and the coordinator-owned collection brief, on stable main 1c4753a453569d17dd66a13eb3d312ab8a5af4d4. Collection 2258/1792447971557-46385 observed 40 replies from 47 entries; four were stopped after S2 rate limiting, three KEGG entries were never called. Metadata proof 2258/1792448128443-87011 passed source parity, provenance checks and 29 controlled tests. Inventory remains deliberately partial and failed. All raw evidence remains private in M5 experiment 2096-biomcp-2258-live-collection.

## Keeps

Adopt existing verified metadata; no new provider calls. Audit exact request identity. Inherited observed Content-Types cover 24 entries, but MyDisease observations 18 and 19 omitted the declared/client fields query. Exclude those two updates here; 2278 owns exact-query recapture, their Content-Types, request declarations and provenance. Preserve the remaining exact-request observations, the corrected dbSNP endpoint and original 2026-10-15 capture provenance, and all three exact KEGG restriction notes. Do not change fixture payload bytes, checker/runtime behavior, denied/rate-limited rows or other lane fields. No invented historical metadata, no broad/licensed payload publication. Add a truthful Internal changelog citation.

## Proof

The resulting metadata fields match retained exact-request observations; no credential, payload or original-date change. Compare old/new manifest source hashes, validate dbSNP bytes/provenance and exact KEGG unverified notes, and run affected existing checker/provenance tests through pm job. Reuse the original collection and known limitations. No new full live run or gate. Record ready/blocked/disabled counts without treating them as a qualification pass. This ticket closes only partial metadata delivery; 2258 remains open for complete final qualification.
