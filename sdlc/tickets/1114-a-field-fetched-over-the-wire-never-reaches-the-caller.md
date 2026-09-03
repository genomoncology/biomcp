---
flow: build
priority: 5
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

## Correct behavior

A field the request asks for is either exposed to the caller or not requested. Zip is exposed.

Write that as a failing test, then fix. Red before green.

The assertion to write: Every field named in the request field list is reachable from the converted value, checked mechanically against the field list.

This behavior is held against both 0.9 and 1.0 alike. It is recorded as case 14 of the clinical-trial conformance cases in the sibling BioData repository. **An attempt cannot read that repository**, so the statement above is this ticket's own authoritative copy, reconciled against the case when the ticket was filed. If it looks wrong, stop and say so rather than implementing something different.

## Boundary

Do not add other location fields. Do not change which sites are returned or their order.
