---
flow: build
priority: 5
---

# A condition name containing a comma becomes two conditions that do not exist

The scalar branch of `json_get_string_list` splits on commas; the array branch does not. So `"Lung Cancer, Non-Small Cell"` arriving as a single string becomes two diseases, `"Lung Cancer"` and `"Non-Small Cell"`, neither of which is a real condition name. The same value arriving inside an array survives intact.

One provider value, two readings, and the wrong one invents diseases.

Verified in `src/transform/trial.rs` on 2026-09-02 against `0.9.0-dev.6`.

## Required behavior

A condition name is never split on a comma inside it.

A scalar and an array carrying the same value convert to the same result.

## Why this is a draft

The conformance case needs a recorded provider payload that does not exist in `testdata/sources/` yet: an NCI payload carrying a comma-bearing condition name in scalar form.

ADR 0017 requires fixtures recorded from the provider rather than hand-written, and defect 17 is what happens when that rule is broken — a fixture written to match the struct rather than the provider let a dead field pass its own test for months.

So this ticket waits on a decision Ian owns: who records the missing payloads, and whether both projects share one recorded set. Promote it once that payload exists.
## Done, observably

- `"Lung Cancer, Non-Small Cell"` converts to one condition.
- The scalar and array forms of the same payload value agree.

## Where correct behavior is written

`repos/biodata/sdlc/planning/clinical-trial-conformance/cases.json`, case 7. Take the assertion from that case, write it as a failing test, then fix. Do not copy the expected behavior into this repository as a second statement of it.

Reported by the BioData lead in `notes/biomcp/feedback/2026-09-02-seventeen-trial-defects-to-fix-in-0-9.md`, defect 7 of seventeen.
## Boundary

Do not change how a genuinely multi-valued field is read. Defect 4 covers the array-of-objects failure in the same function and is filed separately as ticket 1107.
