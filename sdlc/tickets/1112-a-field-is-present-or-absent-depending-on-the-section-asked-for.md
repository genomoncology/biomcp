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

## Correct behavior

The request asks for every field the conversion populates. A field absent from the converted value means the source did not supply it.

Write that as a failing test, then fix. Red before green.

The assertion to write: The same trial converted from the base request and from the full request carries the same intervention fields.

This behavior is held against both 0.9 and 1.0 alike. It is recorded as case 16 of the clinical-trial conformance cases in the sibling BioData repository. **An attempt cannot read that repository**, so the statement above is this ticket's own authoritative copy, reconciled against the case when the ticket was filed. If it looks wrong, stop and say so rather than implementing something different.

## Boundary

Do not add fields to the base request that nothing reads. Ticket 1095 fixed the separate defect where these keys never deserialized at all; this ticket is about which paths request them.
