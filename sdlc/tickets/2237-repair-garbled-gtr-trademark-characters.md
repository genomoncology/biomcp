---
priority: 2
---

# 2237 — Repair garbled GTR trademark characters

Status: OPEN.

Milestone: 0.9.4

## Outcome

Genetic test, lab and manufacturer names from the GTR bundle show ™ and ® correctly instead of "â„¢" and "Â®".

## Evidence

Finding 6 of `sdlc/issues/2026-09-09-findings-from-the-2026-09-01-interface-study-with-no-ticket.md`. The GTR CSV reader (`src/sources/gtr.rs` near lines 475-482 and 562) does no encoding repair. Check first whether the upstream file itself is double-encoded.

## Keeps

Change only text decoding of GTR fields. Keep every other field and row unchanged.

## Proof

A fixture row containing the double-encoded forms renders ™ and ® in JSON and Markdown. Mark the issue file resolved.
