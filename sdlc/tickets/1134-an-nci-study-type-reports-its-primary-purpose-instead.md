---
flow: build
priority: 8
---

# Every NCI trial reports the wrong study type, because the reader falls through to primary purpose

`from_nci_trial` reads the study type at `src/transform/trial.rs:596`:

```rust
let study_type = json_get_string(trial, &["study_type", "studyType", "primary_purpose"])
```

NCI sends neither `study_type` nor `studyType`. Every NCI trial falls through to the third name and reports its primary purpose as its study type. NCI states the study type separately, as `study_protocol_type`.

In this repository's recorded capture, record `NCT05929768` carries `study_protocol_type` = `"Interventional"` and `primary_purpose` = `"TREATMENT"`. BioMCP reports the study type of that trial as `TREATMENT`.

This is the one field in the group that is worse than empty. The other NCI field-name defects, tickets 1119, 1133, 1135 and 1137, produce absence, and absence at least looks like absence. This one produces a confident wrong answer in a field that looks populated, and a reader has no signal that the value came from a different question. "Interventional" and "TREATMENT" are answers to two different questions, and substituting one for the other is a category error, not a formatting difference.

Measured against `testdata/sources/nci_cts/search_melanoma.json`, which carries 58 field names per record. The BioData lead's live query of `clinicaltrialsapi.cancer.gov/api/v2/trials` on 2026-09-02 reported the same 58 names independently.

## A test keeps this green with an invented key

`src/transform/trial.rs:959` supplies `"study_type": "Interventional"` at the top level and the test asserts on it. NCI does not send that key, so the suite proves the reader can read a shape no provider produces. Replacing that assertion is intended, not accidental.

## Required behavior

An NCI trial reports the study type NCI states.

Primary purpose is a different fact and is never substituted for the study type.

A key list that matches nothing in any recorded capture from that provider is a defect, not a reason to fall through to an unrelated field.

## Done, observably

- Converting a recorded NCI payload reports the study type the payload states, not its primary purpose.
- A payload whose primary purpose differs in wording from its study type does not report the purpose as the type. The recorded capture is exactly that case.
- The assertion is made against a recorded capture, not against an object written by hand.
- No test in the NCI conversion path asserts a study type read from a key name absent from every recorded capture.

## The fixture, honestly

`testdata/sources/nci_cts/search_melanoma.json` carries both fields and proves the substitution. It is classified `pending_verification` in `testdata/sources/capture-receipts.json`, so it has no provider receipt. The one receipted NCI capture was recorded minimized to six fields and carries neither name.

That gap is real and it is not this ticket's job to close. Use the capture that carries both fields, and say in the record which capture the proof rests on. The receipting gap is ticket 1138.

## Where correct behavior is written

`sdlc/planning/clinical-trial-conformance/cases.json` in the BioData repository, case 20, "A study type is not a primary purpose". That file is the shared statement of correct behavior, held against both 0.9 and 1.0 so the two cannot drift.

The behavior is restated above in full, because an attempt runs in a worktree where that path resolves to nothing. ADR 0025's amendment of 2026-09-03 says the restatement is what carries the statement across, and a person reconciled the two when this ticket was filed. If the restatement above looks wrong, stop and say so rather than implementing something different.

## Boundary

Change only the NCI study-type read and its tests.

Do not change the ClinicalTrials.gov study-type path at `src/transform/trial.rs:360-362`. Its name is correct.

Do not change how study type is displayed, filtered or sorted, and do not add a primary-purpose field to the output. Whether BioMCP should also carry primary purpose as a fact of its own is a separate question and not this ticket.

Do not touch interventions (1133), age range (1135), enrollment (1119) or the stop reason (1137).

## History

Found 2026-09-03 by the BioData lead while auditing the conformance cases, verified here independently against this repository's own capture the same day. Split out of ticket 1132, which bundled five behaviors and was superseded for that reason.
