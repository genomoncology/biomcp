---
flow: build
priority: 3
---

# The same age notation is parsed for filtering and kept as unparsed text for output

`normalize_age` carries the provider's age string through to output untouched, while `parse_age_years` parses the same notation to support filtering. One notation, two readings, and only one of them is checked.

A notation the filter path understands and the output path does not, or the reverse, produces a trial that filters correctly and displays wrongly, with nothing to catch it.

Verified in `src/transform/trial.rs` on 2026-09-02 against `0.9.0-dev.6`.

## Required behavior

Age is parsed once, and the parsed value is what both filtering and output use.

The source text stays available beside the parsed value, so a caller can see what the provider actually said.

## Done, observably

- Trials with bounds in years, in months, and with `N/A` all convert to the same structure.
- Filtering and output agree about the bounds of the same trial.
- The provider's original text is still reachable.

## Correct behavior

One parse. The value carries the number, the unit, and the original text, per ADR 0019.

Write that as a failing test, then fix. Red before green.

The assertion to write: Output and filtering read age through the same parse. A trial's minimum age compares equal whether reached from the output or from the filter path.

This behavior is held against both 0.9 and 1.0 alike. It is recorded as case 3 of the clinical-trial conformance cases in the sibling BioData repository. **An attempt cannot read that repository**, so the statement above is this ticket's own authoritative copy, reconciled against the case when the ticket was filed. If it looks wrong, stop and say so rather than implementing something different.

## Boundary

Do not change which trials an age filter selects. Do not change how an age range is displayed to a reader.
