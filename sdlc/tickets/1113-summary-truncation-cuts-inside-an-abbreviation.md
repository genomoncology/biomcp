---
flow: build
priority: 4
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

## Where correct behavior is written

`repos/biodata/sdlc/planning/clinical-trial-conformance/cases.json`, case 10. That file is the shared statement of correct behavior for this defect, held against both 0.9 and 1.0.

Take the assertion from that case, write it as a failing test, then fix. Red before green. Do not copy the expected behavior into this repository as a second statement of it. If the case looks wrong, stop and say so rather than implementing something different.

Reported by the BioData lead in `notes/biomcp/feedback/2026-09-02-seventeen-trial-defects-to-fix-in-0-9.md`, defect 10 of seventeen.
## Boundary

Do not change the sentence count or the byte cap. This ticket is about where a boundary is, not how many are kept.
