---
flow: build
priority: 5
---

# A period inside an abbreviation ends a sentence, and the summary loses its content

Sentence detection treats any `.` followed by whitespace or end-of-string as a boundary. Clinical brief summaries are full of abbreviations that match: `pts.`, `vs.`, `approx.`, `e.g.`, `i.v.`, `Dr.`.

A summary that begins "This study enrolls 40 pts. with relapsed disease..." truncates at `pts.`, discarding the disease, the design and the endpoint. The output looks like a complete short summary rather than a fragment.

Verified in `src/transform/trial.rs` on 2026-09-02 against `0.9.0-dev.6`.

## Required behavior

Truncation never cuts inside an abbreviation.

A truncated summary is recognisable as truncated.

## Done, observably

- A summary containing the listed abbreviations truncates at a real sentence boundary.
- The two-sentence summary of such a trial carries its disease and its design.

## Correct behavior

Conversion does not truncate at all (defect 2). Where the renderer truncates, a period inside a known abbreviation does not end a sentence, and a truncated value is marked truncated.

Write that as a failing test, then fix. Red before green.

The assertion to write: A summary containing each listed abbreviation renders without losing the following clause. The converted value is never truncated.

This behavior is held against both 0.9 and 1.0 alike. It is recorded as case 10 of the clinical-trial conformance cases in the sibling BioData repository. **An attempt cannot read that repository**, so the statement above is this ticket's own authoritative copy, reconciled against the case when the ticket was filed. If it looks wrong, stop and say so rather than implementing something different.

## Boundary

Do not change the sentence count or the byte cap. This ticket is about where a boundary is, not how many are kept.
