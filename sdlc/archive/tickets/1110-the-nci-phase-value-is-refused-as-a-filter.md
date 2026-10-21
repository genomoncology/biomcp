---
flow: build
priority: 6
---

**Archived 2026-09-03. Merged into ticket 1109, not abandoned.**

The same defect on the NCI side: the tool emits a phase value it then refuses as a filter. Both fail in `normalize_phase` at `normalization.rs:155`, and one fix must satisfy both or the round trip is not fixed. 1109 now states the requirement for both sources and lists both observable outcomes.

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

## Correct behavior

Roman-numeral phase notation is recognized. Same requirement as defect 1, applied to the NCI source.

Write that as a failing test, then fix. Red before green.

The assertion to write: Every phase value emitted from the recorded NCI payload round-trips as a filter.

This behavior is held against both 0.9 and 1.0 alike. It is recorded as case 6 of the clinical-trial conformance cases in the sibling BioData repository. **An attempt cannot read that repository**, so the statement above is this ticket's own authoritative copy, reconciled against the case when the ticket was filed. If it looks wrong, stop and say so rather than implementing something different.

## Boundary

Do not change NCI phase rendering. Defect 1 covers the CTGov multi-phase form of this failure and is filed separately.
