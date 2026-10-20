---
priority: 2
---

# 2239 — Cap the drug trial-alias cache

Status: COMPLETE.

Milestone: 0.9.4

Reviews: revision bcaf048, accept

Reviews: revision bbec55e, accept

Reviews: revision dd01918, accept

Reviews: revision de3d97b, accept

## Outcome

- Changes: `CHANGELOG.md` `sdlc/archive/issues/2026-09-23-large-futures-and-per-call-runtimes.md` `sdlc/tickets/1260-unify-variant-transcript-numbering-in-lookup-and-display.md` `sdlc/tickets/2237-repair-garbled-gtr-trademark-characters.md` `sdlc/tickets/2238-reject-limit-and-offset-in-variant-erepo-caid-mode.md` `sdlc/tickets/2240-use-the-ca-bundle-for-alphagenome.md` `sdlc/tickets/2241-carry-stale-cache-notes-in-get-json.md` `sdlc/tickets/2242-settle-three-suspected-bugs-with-live-runs.md` `sdlc/tickets/2243-fix-the-remaining-test-reliability-leftovers.md` `sdlc/tickets/2244-find-the-single-cpu-gencc-lease-hang.md` `src/entities/drug/get.rs` `src/entities/drug/get/tests.rs` and this ticket. Carried PM ticket and milestone records are unchanged except this ticket own reviewed proof.

The in-memory drug trial-alias cache in a long-running `biomcp serve` stays bounded.

## Evidence

`sdlc/archive/issues/2026-09-23-large-futures-and-per-call-runtimes.md`, smaller items; deferred by 1243. `src/entities/drug/get.rs` near lines 698-701 holds a plain HashMap with no cap, filled near line 916.

## Keeps

Answers are unchanged. Use the simplest bound that works, such as a fixed entry cap.

## Proof

A unit test inserts past the cap and shows the size stays bounded and recent lookups still hit. Mark the issue finding resolved.

## Coordinator development brief

Bound the existing process-local alias HashMap using a single fixed cap and existing project clear-before-insert policy. Source inspection rg for TRIAL_ALIAS_CACHE and cache.insert identifies unconditional insertion without a cap. A small production insertion helper may first preserve current behavior so a local-map unit test demonstrates growth past the intended cap; then add the bound. No wrapper or dependency is needed. Keep current answers and recent post-rollover lookups, and exercise replacement at capacity.

All builds and tests run through the configured build job. Stage private test/source patches on a published base without pushing WIP; retain patch and per-file hashes with raw evidence, then verify the final published source matches. Push source once ready for review. The developer may resolve the named issue findings as required by the outcome, but does not edit ticket status or send mail.

## Focused proof

Source5aa6e7445efae13a2a3e68316e2ba18e723091ab implements one fixed entry cap using the existing clear-at-cap policy. Replacement preserves other entries. Red1792420611032-54653 failed at513entries; green1792420726147-86471 passed nine alias tests, formatting, warning-denied all-target Clippy and quality checks. Published correspondence1792420941080-45022 matched every changed file and the exact patch. The bound counts entries rather than bytes; rollover may cause later provider lookups. Durable build-host experiment783-biomcp-2239-drug-alias-cache-proof retains biomcp-2239-retained-proof.tar.gz, SHA25616c308ddfe668d459fc1693608cb6d4c72f479c3819f7af88c72af078714917f. The original private local experiment490 remains a duplicate until review is finished.

Keep: experiments/490-biomcp-2239-drug-alias-cache-proof — Retain cited raw proof until the release evidence is explicitly released.
Keep: experiments/783-biomcp-2239-drug-alias-cache-proof — Retain cited raw proof until the release evidence is explicitly released.
