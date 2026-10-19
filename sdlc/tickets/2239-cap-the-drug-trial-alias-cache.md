---
priority: 2
---

# 2239 — Cap the drug trial-alias cache

Status: OPEN.

Milestone: 0.9.4

## Outcome

The in-memory drug trial-alias cache in a long-running `biomcp serve` stays bounded.

## Evidence

`sdlc/issues/2026-09-23-large-futures-and-per-call-runtimes.md`, smaller items; deferred by 1243. `src/entities/drug/get.rs` near lines 698-701 holds a plain HashMap with no cap, filled near line 916.

## Keeps

Answers are unchanged. Use the simplest bound that works, such as a fixed entry cap.

## Proof

A unit test inserts past the cap and shows the size stays bounded and recent lookups still hit. Mark the issue finding resolved.
