---
flow: build
priority: 6
---

# The phase value emitted for an NCI trial is refused when passed back as a filter

An NCI payload carries a phase of `"III"`. That value is emitted and then matches no arm when supplied as a phase filter.

Same shape as the multi-phase CTGov case: the tool prints a value it will not accept. Roman numerals rather than a slash.

Verified in `src/transform/trial.rs` and `src/entities/trial/search/normalization.rs` on 2026-09-02 against `0.9.0-dev.6`, and by the BioData audit against `testdata/sources/nci_cts/search_melanoma_20260811.json`.

## Required behavior

A phase value this tool emits for an NCI trial is accepted as a phase filter.

The same filter token means the same thing whichever source answered.

## Done, observably

- The phase string rendered for an NCI trial is accepted as a filter and selects that trial.
- A phase filter selects the equivalent trials from both sources.

## Where correct behavior is written

`repos/biodata/sdlc/planning/clinical-trial-conformance/cases.json`, case 6. That file is the shared statement of correct behavior for this defect, held against both 0.9 and 1.0.

Take the assertion from that case, write it as a failing test, then fix. Red before green. Do not copy the expected behavior into this repository as a second statement of it. If the case looks wrong, stop and say so rather than implementing something different.

Reported by the BioData lead in `notes/biomcp/feedback/2026-09-02-seventeen-trial-defects-to-fix-in-0-9.md`, defect 6 of seventeen.
## Boundary

Do not change NCI phase rendering. Defect 1 covers the CTGov multi-phase form of this failure and is filed separately.
