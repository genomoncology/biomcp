---
flow: build
priority: 5
---

# Intervention type and description are null unless the arms section was requested

The base field list omits intervention type and description, while the arms section requests both. The converter populates them unconditionally from whatever it was given.

So the same field on the same trial is present or absent depending on which section the caller asked for, and nothing tells the caller which they got. A consumer that reads the field once and caches it records an absence that is an artifact of the request rather than a fact about the trial.

Verified in `src/sources/clinicaltrials.rs` on 2026-09-02 against `0.9.0-dev.6`.

## Required behavior

A field is absent because the source lacks it, never because of which section the caller requested.

Where a field genuinely is not fetched on a path, the output says so rather than rendering it as null.

## Done, observably

- Intervention type and description carry the same values for the same trial whichever section was requested.
- A caller can distinguish "the source has no value" from "this path did not fetch it".

## Where correct behavior is written

`repos/biodata/sdlc/planning/clinical-trial-conformance/cases.json`, case 16. That file is the shared statement of correct behavior for this defect, held against both 0.9 and 1.0.

Take the assertion from that case, write it as a failing test, then fix. Red before green. Do not copy the expected behavior into this repository as a second statement of it. If the case looks wrong, stop and say so rather than implementing something different.

Reported by the BioData lead in `notes/biomcp/feedback/2026-09-02-seventeen-trial-defects-to-fix-in-0-9.md`, defect 16 of seventeen.
## Boundary

Do not add fields to the base request that nothing reads. Ticket 1095 fixed the separate defect where these keys never deserialized at all; this ticket is about which paths request them.
