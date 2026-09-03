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

## Correct behavior

Conversion emits the age bounds and the full summary. Truncation and English phrasing happen in the renderer, which keeps its current output bytes.

Write that as a failing test, then fix. Red before green.

The assertion to write: The converted value carries an unabridged summary and structured age bounds. Rendered output for the corpus is byte-identical to 0.9 before the fix.

This behavior is held against both 0.9 and 1.0 alike. It is recorded as case 2 of the clinical-trial conformance cases in the sibling BioData repository. **An attempt cannot read that repository**, so the statement above is this ticket's own authoritative copy, reconciled against the case when the ticket was filed. If it looks wrong, stop and say so rather than implementing something different.

## Boundary

Do not change what a card looks like. Do not change age filtering. One case for this defect may need a recorded payload that does not exist yet; if so, say which and stop rather than hand-writing a fixture.
