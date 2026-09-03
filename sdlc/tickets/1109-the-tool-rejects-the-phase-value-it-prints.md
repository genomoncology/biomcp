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

## Correct behavior

Phase is a list of phase values, not a joined string. Every phase value any implementation emits is accepted as a filter input for the same trial.

Write that as a failing test, then fix. Red before green.

The assertion to write: For every trial in the corpus, each emitted phase value round-trips: filtering on it returns that trial.

This behavior is held against both 0.9 and 1.0 alike. It is recorded as case 1 of the clinical-trial conformance cases in the sibling BioData repository. **An attempt cannot read that repository**, so the statement above is this ticket's own authoritative copy, reconciled against the case when the ticket was filed. If it looks wrong, stop and say so rather than implementing something different.

## Boundary

Do not change how phases render. The NCI Roman-numeral form of the same round-trip failure is defect 6, filed separately; the two may share a fix, and neither ticket requires the other.
