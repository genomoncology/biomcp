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

## Where correct behavior is written

`repos/biodata/sdlc/planning/clinical-trial-conformance/cases.json`, case 3. That file is the shared statement of correct behavior for this defect, held against both 0.9 and 1.0.

Take the assertion from that case, write it as a failing test, then fix. Red before green. Do not copy the expected behavior into this repository as a second statement of it. If the case looks wrong, stop and say so rather than implementing something different.

Reported by the BioData lead in `notes/biomcp/feedback/2026-09-02-seventeen-trial-defects-to-fix-in-0-9.md`, defect 3 of seventeen.
## Boundary

Do not change which trials an age filter selects. Do not change how an age range is displayed to a reader.
