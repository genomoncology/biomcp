---
flow: build
priority: 5
---

# A result total the provider did not supply is reported as an exact count

When CTGov omits the total, `unwrap_or(0)` reports zero. On the NCI branch the code requests a single row, so `results.len()` reports one. Both are presented as exact counts.

A caller is told there are no matching trials, or exactly one, when the truth is that the provider did not say. `TrialCount::Unknown` already exists in this codebase and nothing uses it.

Verified in `src/sources/ctgov.rs` and `src/entities/trial/search/mod.rs` on 2026-09-02 against `0.9.0-dev.6`.

## Required behavior

A total the provider did not supply is reported as unknown, not as a number.

## Why this is a draft

The conformance case needs a recorded provider payload that does not exist in `testdata/sources/` yet: a CTGov response and an NCI response that omit the total.

ADR 0017 requires fixtures recorded from the provider rather than hand-written, and defect 17 is what happens when that rule is broken. This ticket waits on a decision Ian owns: who records the missing payloads, and whether both projects share one recorded set. Promote it once that payload exists.
## Done, observably

- A response omitting the total reports an unknown total rather than 0 or 1.
- A response carrying a total still reports that number.
- The distinction survives into JSON.

## Where correct behavior is written

`repos/biodata/sdlc/planning/clinical-trial-conformance/cases.json`, case 9. Take the assertion from that case, write it as a failing test, then fix. Do not copy the expected behavior into this repository as a second statement of it.

Reported by the BioData lead in `notes/biomcp/feedback/2026-09-02-seventeen-trial-defects-to-fix-in-0-9.md`, defect 9 of seventeen.
## Boundary

Do not change paging or how many rows a search requests.
