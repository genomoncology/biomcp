---
flow: build
priority: 5
---

# No fixture may carry a key the provider does not send

Ticket 1095 fixed one instance of this. Three structs in `src/sources/clinicaltrials.rs` carried `rename_all = "camelCase"` while the provider names the key `type`, so `intervention_type`, `arm_type` and `reference_type` never deserialized. The reason nothing caught it for months is that the test fixture was hand-written to match the struct rather than the provider. It supplied `armGroupType`, a key ClinicalTrials.gov does not send, so the suite passed while three fields were dead.

`armGroupType` no longer appears anywhere in the tree. The instance is gone. The class is not.

Nothing today stops the next hand-written fixture from inventing a key. A fixture is written to make a test pass, and a fixture written from the struct always makes the test pass, whether or not the struct is right. The failure is silent, it survives review, and it is only found by someone reading the provider's own response beside the code.

## Required behavior

A fixture cannot attest to a key the provider does not send, and this is checked mechanically rather than remembered.

Every key in a source fixture is either present in a recorded capture from that provider endpoint, or is declared as an exception with a stated reason. A key that is neither fails the check and names the fixture, the key, and the endpoint it claims to come from.

The check runs in the gate ladder, so it fails a build rather than producing a report someone has to read.

## Done, observably

- A fixture that introduces a key absent from every recorded capture for its endpoint fails the check, and the message names the fixture, the key and the endpoint.
- Reintroducing `armGroupType` into the arms fixture in `src/transform/trial.rs` fails the check. A test pins that.
- The current tree passes with no new exceptions. If it does not, each failure is either a real defect of the same class or an exception with a written reason, and the ticket says which for every one.
- An authored fixture, one that cannot be recorded because the payload would carry patient-bearing content, is declared as such and passes without being exempted from the key rule. BioData reports that cases 12 and 13 will be authored for exactly this reason, so this path is exercised, not hypothetical.
- The check runs from the gate ladder and fails the build.

## Where this comes from

The BioData lead, in `notes/biomcp/feedback/2026-09-02-two-fixture-details-before-the-payloads-land.md`, second of two items. Case 17 in `sdlc/planning/clinical-trial-conformance/cases.json` in that repository carries the assertion. Take it from there rather than restating the expected behavior here.

## Boundary

`testdata/sources/capture-receipts.json` already classifies fixtures by provenance and is the natural neighbor for this. Extend it or sit beside it; do not build a second, competing record of where fixtures came from.

Do not change any fixture's contents to make the check pass. A fixture that fails is either a defect to file or an exception to justify.

Do not change the deserialization behavior fixed by 1095.
