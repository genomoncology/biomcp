---
priority: 2
---

# 2237 — Repair garbled GTR trademark characters

Status: COMPLETE.

Milestone: 0.9.4

Reviews: revision 50cf524, accept

Reviews: revision 999a212, accept

Reviews: revision c663ebc, accept

## Outcome

- Changes: `CHANGELOG.md` `sdlc/issues/2026-09-09-findings-from-the-2026-09-01-interface-study-with-no-ticket.md` `sdlc/tickets/2237-repair-garbled-gtr-trademark-characters.md` `src/sources/gtr.rs` `tests/test_public_example_accessions.py`.

Every displayed GTR text field, including test, lab and manufacturer names and method text, shows ™ and ® correctly instead of "â„¢" and "Â®".

## Evidence

Finding 6 of `sdlc/issues/2026-09-09-findings-from-the-2026-09-01-interface-study-with-no-ticket.md`. The GTR CSV reader (`src/sources/gtr.rs` near lines 475-482 and 562) does no encoding repair. Check first whether the upstream file itself is double-encoded.

## Keeps

Apply only the two demonstrated text-decoding repairs to displayed GTR text. Keep other content and row identities unchanged.

## Proof

A fixture row containing the double-encoded forms renders ™ and ® in JSON and Markdown. Mark the issue file resolved.

## Coordinator development brief

Repair only the four GTR display-name columns. Replace the two demonstrated trademark mojibake forms; preserve other fields and rows. Baseline job2237/1792420422680-42961 on239d7adc4 found double encoding in723of224309upstreamrows. Both copied-fixture JSON and Markdown contain the broken trademark and fail the expected Ion AmpliSeq trademark assertion. Raw evidence is retained in experiment779. Run the four-name outside-in regression and unchanged-field comparisons before review.

All builds and tests run through the configured build job. Stage private test/source patches on a published base without pushing WIP; retain patch and per-file hashes with raw evidence, then verify the final published source matches. Push source once ready for review. The developer may resolve the named issue findings as required by the outcome, but does not edit ticket status or send mail.

## Focused proof

Source5d334fff374226f13c5c7ea70ca8a7567ee8ad82 fixes only four name fields. Red1792420579310-52705 failed on garbled trademarks; green1792420629282-59797 passed four CLI regressions, fresh build, formatting, Ruff, boundaries/coupling and all-target Clippy. Read-only published blob comparison matches the tested four-file manifest and patch SHA f556f94df66a5f96528170f5b2d1a102fbe0760293025081d0a9dba62dc91bc8. Experiment779-biomcp-2237-gtr-text retains upstream bytes and raw evidence. Method text corruption is reported to PM as the same discovered class; no claim that it is fixed.

PM decision: 2026-10-19. Extend this same defect class to displayed method text and any other displayed GTR text field with the same corruption. Use the same two replacements and the existing four-file source claim. No separate ticket. The earlier four-name proof remains valid but does not qualify this extension; expand the outside-in regression to demonstrate all affected display fields while preserving unaffected content.

## Complete displayed-text extension proof

Source13308d6de8cae4b6cbee55b60e9029df76c41538 extends the same two replacements across displayed descriptive/list fields and fallback link test type, preserving identifiers and row selection. Red1792421633656-41940 failed the expanded test against the name-only predecessor. Green1792421517340-13561 passed four CLI cases, fresh build, formatting, Ruff, ratchet, zero-coupling and all-target Clippy. All four published file hashes matched the tested manifest and patchSHA256 b5512049b3c0e259adbdfb56482b4442fbced78119132648f89dc9870f814e09. No new live captures were taken.

Fresh source reviewer accepted13308d6de8cae4b6cbee55b60e9029df76c41538. Detail JSON and Markdown were executed; search and fallback callers were inspected through shared parsing, not independently exercised by extra tests. Complete raw evidence remains in experiments/779-biomcp-2237-gtr-text/display-extension alongside the original upstream/name-only proof. This extension supersedes the earlier explicit limit on method text.

Keep: experiments/779-biomcp-2237-gtr-text — Retain cited raw proof until release evidence is explicitly released.
