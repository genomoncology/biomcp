---
flow: build
priority: 4
---

# The location postal code is requested, parsed, and then thrown away

`LocationZip` is named in the field set sent to ClinicalTrials.gov and parsed into the source struct. The output type has no field for it, so the value is discarded after being fetched.

The tool pays for the field on every request and no caller can ever see it. A caller asking where a trial recruits gets a location without its postal code, which is the part a patient needs to judge distance.

Verified in `src/sources/clinicaltrials.rs` and `src/entities/trial/mod.rs` on 2026-09-02 against `0.9.0-dev.6`.

## Required behavior

A field this tool fetches over the wire reaches the caller, or it is not fetched.

## Done, observably

- A CTGov trial with a US site reports that site's postal code.
- The value appears in Markdown and in JSON.

## Where correct behavior is written

`repos/biodata/sdlc/planning/clinical-trial-conformance/cases.json`, case 14. That file is the shared statement of correct behavior for this defect, held against both 0.9 and 1.0.

Take the assertion from that case, write it as a failing test, then fix. Red before green. Do not copy the expected behavior into this repository as a second statement of it. If the case looks wrong, stop and say so rather than implementing something different.

Reported by the BioData lead in `notes/biomcp/feedback/2026-09-02-seventeen-trial-defects-to-fix-in-0-9.md`, defect 14 of seventeen.
## Boundary

Do not add other location fields. Do not change which sites are returned or their order.
