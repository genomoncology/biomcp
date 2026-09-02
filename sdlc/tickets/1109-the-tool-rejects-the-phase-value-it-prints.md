---
flow: build
priority: 6
---

# A phase value the tool prints is refused when passed back as a filter

A two-phase CTGov trial renders `phase: "PHASE1/PHASE2"`. Passing that exact string back as a phase filter refuses, because `normalize_enum_key` drops the slash and the result matches no known arm.

The tool rejects its own output. A caller who reads a phase off a trial card and filters on it gets an error, and nothing in the card warns that its own value is not a valid input.

Verified in `src/transform/trial.rs` and `src/entities/trial/search/normalization.rs` on 2026-09-02 against `0.9.0-dev.6`.

## Required behavior

A phase value this tool emits is accepted as a phase filter.

## Done, observably

- The phase string rendered for a multi-phase trial is accepted as a filter and selects that trial.
- Single-phase values keep working exactly as they do now.

## Where correct behavior is written

`repos/biodata/sdlc/planning/clinical-trial-conformance/cases.json`, case 1. That file is the shared statement of correct behavior for this defect, held against both 0.9 and 1.0.

Take the assertion from that case, write it as a failing test, then fix. Red before green. Do not copy the expected behavior into this repository as a second statement of it. If the case looks wrong, stop and say so rather than implementing something different.

Reported by the BioData lead in `notes/biomcp/feedback/2026-09-02-seventeen-trial-defects-to-fix-in-0-9.md`, defect 1 of seventeen.
## Boundary

Do not change how phases render. The NCI Roman-numeral form of the same round-trip failure is defect 6, filed separately; the two may share a fix, and neither ticket requires the other.
