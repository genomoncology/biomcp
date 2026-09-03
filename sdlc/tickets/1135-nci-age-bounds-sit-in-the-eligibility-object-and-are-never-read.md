---
flow: build
priority: 7
---

# Every NCI trial reports no age range, because the reader looks at the top level and NCI nests it

`from_nci_trial` reads the age bounds at `src/transform/trial.rs:598-601`:

```rust
let age_range = format_age_range(
    json_get_string(trial, &["minimum_age", "minimumAge", "min_age"]).as_deref(),
    json_get_string(trial, &["maximum_age", "maximumAge", "max_age"]).as_deref(),
);
```

None of those six names appears at the top level of an NCI trial record. NCI nests the bounds inside its eligibility object, at `eligibility.structured.min_age` and `eligibility.structured.max_age`. Both lookups return nothing and the age range is empty for every NCI trial.

A clinician screening a patient asks whether they are old enough to enrol. BioMCP answers nothing on every NCI trial, and an empty answer reads the same as a registry that set no bounds.

Measured against this repository's own recorded NCI capture, `testdata/sources/nci_cts/search_melanoma.json`. Every record carries 58 field names and none of the six is among them. Record `NCT05929768` carries `eligibility.structured.min_age` = `"18 Years"` and `max_age` = `"999 Years"`. The structured object also carries `min_age_number`, `min_age_unit`, `min_age_in_years` and the maximum equivalents. The BioData lead's live query of `clinicaltrialsapi.cancer.gov/api/v2/trials` on 2026-09-02 reported the same 58 names independently.

One thing to notice while reading the payload: `"999 Years"` is a sentinel for no upper bound, not a real ceiling. Reporting an upper age limit of 999 years would be a new wrong answer replacing an absent one. How to represent that is for the design stage to settle; the ticket only says the reported range must be true of the trial.

## A test keeps this green with an invented key

`src/transform/trial.rs:960` supplies `"minimum_age": "18 Years"` at the top level and the test asserts on the rendered range. NCI does not send that key at that place, so the suite proves the reader can read a shape no provider produces. Replacing that assertion is intended, not accidental.

## Required behavior

An NCI trial reports the age bounds its payload carries.

A bound the provider states as unlimited is reported as unlimited, not as a number.

A key list that matches nothing in any recorded capture from that provider is a defect, not an empty result.

## Done, observably

- Converting a recorded NCI payload carrying structured eligibility age bounds yields the lower bound the payload states.
- A trial whose upper bound is the provider's no-limit sentinel is not reported as having a numeric upper age limit.
- The assertion is made against a recorded capture, not against an object written by hand.
- No test in the NCI conversion path asserts an age bound read from a key name absent from every recorded capture.

## The fixture, honestly

`testdata/sources/nci_cts/search_melanoma.json` carries the structured eligibility object and proves the shape. It is classified `pending_verification` in `testdata/sources/capture-receipts.json`, so it has no provider receipt. The one receipted NCI capture was recorded minimized to six fields and carries no eligibility object.

That gap is real and it is not this ticket's job to close. Use the capture that carries the shape, and say in the record which capture the proof rests on. The receipting gap has its own ticket, filed the same day.

## Where correct behavior is written

`sdlc/planning/clinical-trial-conformance/cases.json` in the BioData repository, case 19, "Age bounds are read from the structured eligibility object". That file is the shared statement of correct behavior, held against both 0.9 and 1.0 so the two cannot drift.

The behavior is restated above in full, because an attempt runs in a worktree where that path resolves to nothing. ADR 0025's amendment of 2026-09-03 says the restatement is what carries the statement across, and a person reconciled the two when this ticket was filed. If the restatement above looks wrong, stop and say so rather than implementing something different.

## Boundary

Change only the NCI age read and its tests.

Do not change `format_age_range`. That it renders English rather than structure is ticket 1115, and it stays as it is here. This ticket is about whether the bounds reach it at all.

Do not change the ClinicalTrials.gov age path at `src/transform/trial.rs:368`. Its names are correct.

Do not change age filtering, and do not change how the age range is displayed.

Do not touch interventions, study type, enrollment, eligibility text or the stop reason. Each is its own ticket. In particular, do not start reading eligibility criteria text here; that is the eligibility-text ticket filed the same day.

## History

Found 2026-09-03 by the BioData lead while auditing the conformance cases, verified here independently against this repository's own capture the same day. Split out of ticket 1132, which bundled five behaviors and was superseded for that reason.
