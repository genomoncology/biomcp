---
flow: build
priority: 9
hold: Superseded 2026-09-03. It carried five behaviors on one ticket. Each is now its own ticket. Kept as a held draft so the board does not read it as done.
---


# The NCI reader asks for five field names NCI never sends, so five fields are always empty or wrong

`from_nci_trial` reads most of its fields through `json_get_string`, which walks a list of candidate key names and returns the first one present. Five of those lists name no key the provider actually sends. Each one fails silently, because a missing key and an empty value are the same answer.

Measured against this repository's own recorded capture, `testdata/sources/nci_cts/search_melanoma.json`. Every record carries 58 fields. Here is what the code asks for and what the payload holds:

| What | Code asks for | Payload actually carries | Result today |
| --- | --- | --- | --- |
| Interventions | `interventions` at the top level, `src/transform/trial.rs:662` | `arms[].interventions[].name` | always empty |
| Age range | `minimum_age`, `minimumAge`, `min_age` at the top level, `:642`–`:643` | `eligibility.structured.min_age` = `"18 Years"`, `max_age` = `"999 Years"` | always empty |
| Study type | `study_type`, `studyType`, `primary_purpose`, `:639` | `study_protocol_type` = `"Interventional"` | falls through to `primary_purpose` and reports `"TREATMENT"` |
| Enrollment | `enrollment`, `enrollment_target`, `target_enrollment`, `:652` | `minimum_target_accrual_number` | always absent |
| Why stopped | nothing; hardcoded `None` at `:669` | `why_study_stopped` | always absent |

None of these is a parsing bug. In every case the reader asks a question the payload cannot answer and takes silence for an answer.

## Why this survived

Two unit tests in the same file keep the enrollment path green by feeding it key names no provider sends:

- `src/transform/trial/tests.rs:280` supplies `"target_enrollment": "120"` and asserts `Some(120)`.
- `src/transform/trial/tests.rs:315` supplies `"enrollment_target": "420"` and asserts `Some(420)`.

Both keys are invented. The test proves the reader can read a payload nobody sends, so the suite stays green while the field is dead in production. That is defect 17's failure exactly, in a second place: a test written against the code's own shape rather than the provider's.

## Required behavior

Every field above carries the value the provider sends.

A key list that matches nothing in a recorded capture is a defect, not an empty result.

## Done, observably

- Converting the recorded NCI capture yields a non-empty intervention list, taken from the arms.
- The same conversion yields an age range of 18 years and up, a study type of `Interventional`, and an enrollment figure.
- `why_stopped` carries `why_study_stopped` when the provider sends one.
- No test asserts a value read from a key name absent from every recorded capture.

## The check that keeps this from coming back

This is the fifth and sixth instance of one class found in a single pass, so fix the class as well as the instances.

Lift each candidate key list into a named constant. Add one test that, for every recorded capture under `testdata/sources/nci_cts/`, takes the union of keys present across all records and fails when a named key list has no member in that union.

`["enrollment", "enrollment_target", "target_enrollment"]` fails that test today. So does `["interventions"]`. A list that legitimately names an optional field the capture happens not to carry is declared as an exception with a written reason, and the reason names the field.

One test file and one refactor. No new tooling, no new dependency, and it runs from the existing gate ladder.

## Boundary

Change only the NCI conversion path and its tests. Do not change the ClinicalTrials.gov reader; its key names are correct.

Do not change how conditions are read. `diseases` is present and the failure there has a different cause, filed as ticket 1107.

Do not change the display of any of these fields. This ticket is about the values reaching the caller, not how they are shown. `format_age_range` rendering English is filed as ticket 1115 and stays as it is here.

Absorbs draft 1118 and draft 1119, both archived on 2026-09-03. 1119 described the enrollment failure as a float-parsing bug; the parse is never reached, because the key is never found. 1118's requirement moved to 1107.

Found by the BioData lead on 2026-09-03 while auditing the conformance cases, and verified here independently against `testdata/sources/nci_cts/search_melanoma.json` on 2026-09-03. BioData is asked to carry these as conformance cases so both versions are held to them.

## Re-merged 2026-09-03, and why the split was undone

This ticket was split into 1133, 1134, 1135, 1137 and 1119 earlier today, one per field. All five are archived and the work is back here.

The split was a reasonable call and the reason for reversing it is throughput, not correctness. All five defects live in one thirty-line block of `from_nci_trial`, they share one recorded fixture, and each is a one-line change to a key list. Five tickets means five design stages, five design reviews, five code reviews and five verifications for one commit's worth of work, and each of the last four rebases onto the one before it because they edit the same function.

The acceptance test carries five assertions against one payload. That is one test file, not a bundle of unrelated work, and the claim underneath them is single: **the NCI reader reads the payload where NCI puts it.**

Ticket 1136 stays separate. Its defect is a type mismatch rather than a key name — the reader calls `as_str` on an object — and it needs the eligibility structure understood rather than a key renamed.

Ticket 1138 also stays separate. It is the guard that stops this class returning, not a fix, and it must land **after** this ticket. Its check fails on `["interventions"]` and on the enrollment list, so landing it first would turn main red and block every ticket in the channel.

All line citations above were refreshed on 2026-09-03 after ticket 1107 rewrote this file. The five defects were re-verified against `testdata/sources/nci_cts/search_melanoma.json` at that time.
