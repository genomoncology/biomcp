---
flow: build
priority: 8
---

# Every NCI trial reports no interventions, because the reader looks for a top-level key NCI does not have

`from_nci_trial` reads interventions at `src/transform/trial.rs:619`:

```rust
let interventions = json_get_string_list(trial, &["interventions"], 25);
```

NCI has no top-level `interventions` key. It nests them inside each study arm, at `arms[].interventions[].name`. `json_get_string_list` finds nothing to read and returns an empty vector, without an error and without anything marking the list as failed.

So an NCI trial reports that it tests no drug at all. A caller asking what a trial is testing gets an empty list, and cannot tell that from a registry that lists none.

Measured against this repository's own recorded NCI capture, `testdata/sources/nci_cts/search_melanoma.json`. Every record carries 58 field names and `interventions` is not among them. In record `NCT05929768` the arms carry the interventions instead: two arms, the first holding 59 intervention objects, the first of those named `Pembrolizumab`. The BioData lead's live query of `clinicaltrialsapi.cancer.gov/api/v2/trials` on 2026-09-02 reported the same 58 names independently.

## An earlier description blamed the wrong cause

Ticket 1107 originally said interventions fail through the same `as_str()` call that loses NCI conditions. They do not. Conditions fail after the key is found, because `diseases` is an array of objects and every element is discarded. Interventions fail before that, because the key is never found. Two causes, two fixes, and fixing 1107 as first written would have left interventions empty while a record claimed both were repaired.

## A test keeps this green with an invented key

`src/transform/trial.rs:935` supplies `"interventions": ["Drug X"]` at the top level and `:943` asserts the list contains it. `:965` does the same with `"Osimertinib"`. Neither payload is one NCI sends, so the suite proves the reader can read a shape no provider produces. Replacing those assertions is intended, not accidental.

## Required behavior

An NCI trial reports the intervention names its payload carries.

A key list that matches nothing in any recorded capture from that provider is a defect, not an empty result.

## Done, observably

- Converting a recorded NCI payload whose arms carry intervention names yields a non-empty intervention list holding those names.
- The assertion is made against a recorded capture, not against an object written by hand.
- A payload element the converter cannot read is reported rather than silently dropped, so absence and failure are distinguishable.
- No test in the NCI conversion path asserts interventions read from a top-level key absent from every recorded capture.

## The fixture, honestly

`testdata/sources/nci_cts/search_melanoma.json` carries the nested arms and proves the shape. It is classified `pending_verification` in `testdata/sources/capture-receipts.json`, so it has no provider receipt. The one receipted NCI capture, `search_melanoma_20260811.json`, was recorded with the response minimized to six fields and carries no arms at all.

That gap is real and it is not this ticket's job to close. Use the capture that carries the shape, and say in the record which capture the proof rests on. The receipting gap is the subject of its own ticket, filed the same day, on checking the reader's key lists against recorded captures.

## Where correct behavior is written

`sdlc/planning/clinical-trial-conformance/cases.json` in the BioData repository, case 18, "Interventions are read from where the provider nests them". That file is the shared statement of correct behavior, held against both 0.9 and 1.0 so the two cannot drift.

The behavior is restated above in full, because an attempt runs in a worktree where that path resolves to nothing. ADR 0025's amendment of 2026-09-03 says the restatement is what carries the statement across, and a person reconciled the two when this ticket was filed. If the restatement above looks wrong, stop and say so rather than implementing something different.

Case 18 supersedes the interventions clause of case 4. Case 4's own `correct` text still reads "Interventions are read the same way", and BioData has been asked to drop that sentence. Follow case 18 for interventions.

## Boundary

Change only the NCI intervention read and its tests.

Do not change how conditions are read. `diseases` is present and fails for a different reason, which is ticket 1107.

Do not change the ClinicalTrials.gov intervention path at `src/transform/trial.rs:400-420`. Its names are correct and it already reads the arms module.

Do not change how interventions are displayed or how `intervention_details` is populated. This ticket is about the values reaching the caller.

Do not touch age range, study type, enrollment or the stop reason. Each is its own ticket.

## History

Found 2026-09-03 by the BioData lead while auditing the conformance cases, verified here independently against this repository's own capture the same day. Split out of ticket 1132, which bundled five behaviors and was superseded for that reason.
