---
flow: build
priority: 7
---

# A phase value the tool prints is refused when passed back as a filter, on both sources

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

## Absorbs ticket 1110, 2026-09-03

Ticket 1110 held the same defect on the NCI side and is archived. Both fail in `normalize_phase` at `src/entities/trial/search/normalization.rs:155`, both are the tool refusing a value it emitted, and a single fix has to satisfy both or it has not fixed the round trip.

- **ClinicalTrials.gov** emits `PHASE1/PHASE2` for a two-phase trial. Fed back as a filter, `normalize_enum_key` drops the slash and the call refuses.
- **NCI** emits `III`. Roman numerals match no arm of the same matcher.

Both forms are confirmed present in this repository's recorded captures: the NCI payload `testdata/sources/nci_cts/search_melanoma.json` carries `phase: "III"`.

## Required behavior, restated to cover both

Every phase value this tool emits is accepted as a phase filter, whichever source produced it, and selects the trials that value describes.

## Done, observably

- A two-phase ClinicalTrials.gov value round-trips: emit it, pass it back as `--phase`, and the call succeeds.
- An NCI Roman-numeral phase value round-trips the same way.
- The same filter token means the same thing whichever source answered.
- A genuinely invalid phase value is still refused, with an error naming what is accepted.
