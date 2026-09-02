---
flow: build
priority: 3
---

# The conversion path renders English prose where a consumer needs structure

Two functions produce display text inside conversion rather than at rendering. `format_age_range` returns strings like `"18 Years to Any age"`, and `truncate_summary` cuts to two sentences and 500 bytes. Both are called from the conversion path.

A consumer of the converted value therefore receives a rendered sentence and has to parse English back into bounds, and receives a summary already cut with no access to the full text. Both decisions belong to a renderer, which knows what it is rendering for, rather than to conversion, which does not.

This one matters more than its severity suggests, because a second implementation reading the same converted values inherits the prose.

Verified in `src/transform/trial.rs` on 2026-09-02 against `0.9.0-dev.6`.

## Required behavior

Conversion offers the age bounds as structure. A renderer produces any English.

Conversion offers the summary the source supplied. A renderer decides what to shorten and by how much.

Current rendered output does not change; the same sentence a reader sees today is still what they see.

## Done, observably

- A consumer can read the minimum and maximum age as values without parsing a sentence.
- A consumer can reach the untruncated summary.
- The Markdown a reader sees is unchanged.

## Where correct behavior is written

`repos/biodata/sdlc/planning/clinical-trial-conformance/cases.json`, case 2. That file is the shared statement of correct behavior for this defect, held against both 0.9 and 1.0.

Take the assertion from that case, write it as a failing test, then fix. Red before green. Do not copy the expected behavior into this repository as a second statement of it. If the case looks wrong, stop and say so rather than implementing something different.

Reported by the BioData lead in `notes/biomcp/feedback/2026-09-02-seventeen-trial-defects-to-fix-in-0-9.md`, defect 2 of seventeen.
## Boundary

Do not change what a card looks like. Do not change age filtering. One case for this defect may need a recorded payload that does not exist yet; if so, say which and stop rather than hand-writing a fixture.
