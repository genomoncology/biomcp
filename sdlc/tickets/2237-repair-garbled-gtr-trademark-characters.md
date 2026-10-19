---
priority: 2
---

# 2237 — Repair garbled GTR trademark characters

Status: OPEN.

Milestone: 0.9.4

## Outcome

- Changes: `src/sources/gtr.rs` `tests/test_public_example_accessions.py` `CHANGELOG.md` `sdlc/issues/2026-09-09-findings-from-the-2026-09-01-interface-study-with-no-ticket.md` and this ticket.

Genetic test, lab and manufacturer names from the GTR bundle show ™ and ® correctly instead of "â„¢" and "Â®".

## Evidence

Finding 6 of `sdlc/issues/2026-09-09-findings-from-the-2026-09-01-interface-study-with-no-ticket.md`. The GTR CSV reader (`src/sources/gtr.rs` near lines 475-482 and 562) does no encoding repair. Check first whether the upstream file itself is double-encoded.

## Keeps

Change only text decoding of GTR fields. Keep every other field and row unchanged.

## Proof

A fixture row containing the double-encoded forms renders ™ and ® in JSON and Markdown. Mark the issue file resolved.

## Coordinator development brief

Repair only the four GTR display-name columns. Replace the two demonstrated trademark mojibake forms; preserve other fields and rows. Baseline job2237/1792420422680-42961 on239d7adc4 found double encoding in723of224309upstreamrows. Both copied-fixture JSON and Markdown contain the broken trademark and fail the expected Ion AmpliSeq trademark assertion. Raw evidence is retained in experiment779. Run the four-name outside-in regression and unchanged-field comparisons before review.

All builds and tests run through the configured build job. Stage private test/source patches on a published base without pushing WIP; retain patch and per-file hashes with raw evidence, then verify the final published source matches. Push source once ready for review. The developer may resolve the named issue findings as required by the outcome, but does not edit ticket status or send mail.

## Focused proof

Source5d334fff374226f13c5c7ea70ca8a7567ee8ad82 fixes only four name fields. Red1792420579310-52705 failed on garbled trademarks; green1792420629282-59797 passed four CLI regressions, fresh build, formatting, Ruff, boundaries/coupling and all-target Clippy. Read-only published blob comparison matches the tested four-file manifest and patch SHA f556f94df66a5f96528170f5b2d1a102fbe0760293025081d0a9dba62dc91bc8. Experiment779-biomcp-2237-gtr-text retains upstream bytes and raw evidence. Method text corruption is reported to PM as the same discovered class; no claim that it is fixed.
