**Archived 2026-09-03. Superseded, not abandoned.**

This ticket described enrollment vanishing because a float renders as `120.0` and the integer parse fails. The parse is never reached. The reader asks for `enrollment`, `enrollment_target` and `target_enrollment` at `src/transform/trial.rs:609`, and NCI sends none of the three. It sends `minimum_target_accrual_number`. The field has always been absent for a simpler reason than the one written here.

Absorbed into ticket 1132, which fixes the field name along with four others of the same kind and adds the check that catches the class.

BioData is asked to restate the matching conformance case against the measured field name.

---
---
flow: build
priority: 5
---

# NCI enrollment is always absent, because the reader asks for names NCI never sends

**Measurement disputes the stated cause. Restated 2026-09-03 and still held. Do not promote.**

## What the original ticket said, and why it was wrong

The first version of this ticket said `json_get_string` renders a numeric enrollment of `120` as `"120.0"`, the integer parse fails, and the field silently becomes `None`.

The code does behave that way. NCI never reaches it.

`src/transform/trial.rs:607-611` reads `["enrollment", "enrollment_target", "target_enrollment"]`. NCI sends none of those three. It sends `minimum_target_accrual_number`, which appears nowhere in this repository's source.

So NCI enrollment is not lost on a float encoding. It is absent for every NCI trial, and has been for the life of the code.

Two independent sources agree, and neither is the other's copy:

- This repository's own recorded NCI capture, `testdata/sources/nci_cts/search_melanoma.json`. Every record carries 58 fields. `minimum_target_accrual_number` is present and holds a JSON integer. `enrollment`, `enrollment_target` and `target_enrollment` are absent from every record.
- The BioData lead's live query of `clinicaltrialsapi.cancer.gov/api/v2/trials` on 2026-09-02, at sizes 1 and 50, reporting the same 58 fields.

The float-parse branch is unreachable from any provider today. `json_get_string` and `json_get_string_list` have call sites only in `from_nci_hit` and `from_nci_trial`, and those two functions are called only from `src/entities/trial/search/nci.rs:82` and `src/entities/trial/get.rs:174`. ClinicalTrials.gov enrollment goes through typed deserialization at `src/transform/trial.rs:375-377` instead.

The two existing unit tests at `src/transform/trial.rs:930` and `:963` supply `target_enrollment` and `enrollment_target` and assert `Some(120)` and `Some(420)`. Both keys are invented. This is the same failure as defect 17: a test written against the reader rather than against the provider, passing while the field is dead.

## What the restated defect is

A field name the provider does not send. Same class as defect 17, and the third instance found.

The fix reads `minimum_target_accrual_number`, and the two unit tests are rewritten against a recorded NCI capture rather than a hand-written object.

## Required behavior

NCI enrollment converts from the field NCI actually sends.

Enrollment survives any numeric encoding the provider uses.

A value the converter receives and cannot read is reported rather than silently dropped.

## Done, observably

- An NCI trial carrying `minimum_target_accrual_number` converts to that number.
- The assertion comes from a recorded NCI capture, not from an object written by hand.
- A value the converter cannot read is reported rather than dropped.

## Why this is still a draft

Ian ruled on 2026-09-03 that both disputed defects are marked as measurement disputes and neither is promoted while the conformance case still asserts the old cause.

Case 8 in `repos/biodata/sdlc/planning/clinical-trial-conformance/cases.json` still says `120, 120.0, and "120" all convert to 120`. That assertion cannot be satisfied by an NCI payload, because NCI does not encode enrollment three ways and does not use that field name. The case carries a `correction_2026_09_02` field recording the measurement. Changing what the case asserts is a change to stated correct behavior, and that is BioData's to make after Ian rules.

Promote this once case 8 asserts the field-name defect.

## Where correct behavior is written

`repos/biodata/sdlc/planning/clinical-trial-conformance/cases.json`, case 8. Take the assertion from that case, write it as a failing test, then fix. Do not copy the expected behavior into this repository as a second statement of it.

Reported by the BioData lead in `notes/biomcp/feedback/2026-09-02-seventeen-trial-defects-to-fix-in-0-9.md`, defect 8 of seventeen. Disputed in `notes/biomcp/feedback/2026-09-02-nci-field-names-measured-against-the-live-api.md`.

## Boundary

Do not change how enrollment is displayed or filtered.

Do not remove the numeric-encoding handling in `json_get_string`. It is unreachable from NCI today and stays correct for a provider that does send a number.
