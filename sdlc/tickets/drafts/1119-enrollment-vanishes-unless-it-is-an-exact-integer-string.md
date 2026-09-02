---
flow: build
priority: 5
---

# Enrollment disappears when the provider encodes it as a float

`json_get_string` renders a numeric enrollment of `120` as `"120.0"`, the integer parse fails, and the field silently becomes `None`.

A trial that reports its enrollment shows none. A parse failure and a genuinely absent field are indistinguishable in the output, so nothing indicates that a value was received and dropped.

Verified in `src/transform/trial.rs` on 2026-09-02 against `0.9.0-dev.6`.

## Required behavior

Enrollment survives any numeric encoding the provider uses.

A parse failure is visible rather than silent, because a dropped value and an absent value are different facts.

## Why this is a draft

The conformance case needs a recorded provider payload that does not exist in `testdata/sources/` yet: NCI payloads carrying integer, float, and string enrollment.

ADR 0017 requires fixtures recorded from the provider rather than hand-written, and defect 17 is what happens when that rule is broken — a fixture written to match the struct rather than the provider let a dead field pass its own test for months.

So this ticket waits on a decision Ian owns: who records the missing payloads, and whether both projects share one recorded set. Promote it once that payload exists.
## Done, observably

- Integer, float, and string encodings of the same enrollment all convert to the same number.
- A value the converter cannot read is reported rather than dropped.

## Where correct behavior is written

`repos/biodata/sdlc/planning/clinical-trial-conformance/cases.json`, case 8. Take the assertion from that case, write it as a failing test, then fix. Do not copy the expected behavior into this repository as a second statement of it.

Reported by the BioData lead in `notes/biomcp/feedback/2026-09-02-seventeen-trial-defects-to-fix-in-0-9.md`, defect 8 of seventeen.
## Boundary

Do not change how enrollment is displayed or filtered.
