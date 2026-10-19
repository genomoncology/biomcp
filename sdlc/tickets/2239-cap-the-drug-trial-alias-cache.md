---
priority: 2
---

# 2239 — Cap the drug trial-alias cache

Status: OPEN.

Milestone: 0.9.4

## Outcome

- Changes: `src/entities/drug/get.rs` `src/entities/drug/get/tests.rs` `CHANGELOG.md` `sdlc/issues/2026-09-23-large-futures-and-per-call-runtimes.md` and this ticket.

The in-memory drug trial-alias cache in a long-running `biomcp serve` stays bounded.

## Evidence

`sdlc/issues/2026-09-23-large-futures-and-per-call-runtimes.md`, smaller items; deferred by 1243. `src/entities/drug/get.rs` near lines 698-701 holds a plain HashMap with no cap, filled near line 916.

## Keeps

Answers are unchanged. Use the simplest bound that works, such as a fixed entry cap.

## Proof

A unit test inserts past the cap and shows the size stays bounded and recent lookups still hit. Mark the issue finding resolved.

## Coordinator development brief

Bound the existing process-local alias HashMap using a single fixed cap and existing project clear-before-insert policy. Source inspection rg for TRIAL_ALIAS_CACHE and cache.insert identifies unconditional insertion without a cap. A small production insertion helper may first preserve current behavior so a local-map unit test demonstrates growth past the intended cap; then add the bound. No wrapper or dependency is needed. Keep current answers and recent post-rollover lookups, and exercise replacement at capacity.

All builds and tests run through the configured build job. Stage private test/source patches on a published base without pushing WIP; retain patch and per-file hashes with raw evidence, then verify the final published source matches. Push source once ready for review. The developer may resolve the named issue findings as required by the outcome, but does not edit ticket status or send mail.
