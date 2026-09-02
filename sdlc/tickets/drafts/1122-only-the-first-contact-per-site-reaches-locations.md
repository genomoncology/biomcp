---
flow: build
priority: 4
---

# Locations keep one contact per site while contacts keeps them all

Locations take only the first contact a site lists. The contacts section keeps every one. CTGov routinely lists a primary and a backup, so one payload produces two different contact counts depending on which part of the output a reader looks at.

Verified in `src/transform/trial.rs` on 2026-09-02 against `0.9.0-dev.6`.

## Required behavior

Every contact a site lists is kept.

The two views of the same site agree about how many contacts it has.

## Why this is a draft

The conformance case needs a recorded provider payload that does not exist in `testdata/sources/` yet: a CTGov trial carrying a site that lists two contacts.

ADR 0017 requires fixtures recorded from the provider rather than hand-written, and defect 17 is what happens when that rule is broken. This ticket waits on a decision Ian owns: who records the missing payloads, and whether both projects share one recorded set. Promote it once that payload exists.
## Done, observably

- A site listing two contacts reports two in both views.
- The counts agree for every site on the trial.

## Where correct behavior is written

`repos/biodata/sdlc/planning/clinical-trial-conformance/cases.json`, case 13. Take the assertion from that case, write it as a failing test, then fix. Do not copy the expected behavior into this repository as a second statement of it.

Reported by the BioData lead in `notes/biomcp/feedback/2026-09-02-seventeen-trial-defects-to-fix-in-0-9.md`, defect 13 of seventeen.
## Boundary

Do not change contact ordering or which contact is treated as primary.
