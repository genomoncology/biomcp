---
flow: build
priority: 5
---

**Archived 2026-09-03. Re-merged into ticket 1132, not abandoned.**

NCI enrollment is absent because the reader asks for `enrollment`, `enrollment_target` and `target_enrollment` and NCI sends `minimum_target_accrual_number`. That is one of five key-name defects in the same thirty-line block, and 1132 fixes all five in one pass against one fixture.

---
# NCI enrollment is always absent, because the reader asks for three names NCI never sends

`from_nci_trial` reads enrollment at `src/transform/trial.rs:607-611`:

```rust
let enrollment = json_get_string(
    trial,
    &["enrollment", "enrollment_target", "target_enrollment"],
)
.and_then(|s| s.parse::<i32>().ok());
```

NCI sends none of those three names. It sends `minimum_target_accrual_number`, and that string appears nowhere in this repository's source.

So NCI enrollment is not lost sometimes. It is absent for every NCI trial, and has been for the life of the code. A caller asking how large an NCI trial is gets nothing, and cannot tell that from a registry that declined to say.

Measured against this repository's own recorded NCI capture, `testdata/sources/nci_cts/search_melanoma.json`. Every record carries 58 fields. `minimum_target_accrual_number` is present and holds the JSON integer `2400`. `enrollment`, `enrollment_target` and `target_enrollment` are absent from every record. The BioData lead's live query of `clinicaltrialsapi.cancer.gov/api/v2/trials` on 2026-09-02, at sizes 1 and 50, reported the same 58 field names independently.

## An earlier description of this defect was wrong, and the correction matters

The first version of this ticket said `json_get_string` renders a numeric enrollment of `120` as `"120.0"`, the integer parse fails, and the field silently becomes `None`.

The code does behave that way. NCI never reaches it. The key is never found, so the parse never runs. The captured value is a plain integer, not a float, which makes the old story wrong twice over.

The float-rendering branch is not a defect this ticket fixes and not one it removes. It stays correct for a provider that does send a number under a name the reader looks for.

## Two tests keep this green with invented keys

- `src/transform/trial.rs:930` supplies `"target_enrollment": "120"` and asserts `Some(120)`.
- `src/transform/trial.rs:963` supplies `"enrollment_target": "420"` and asserts `Some(420)`.

Both keys are invented. The tests prove the reader can read a payload nobody sends, so the suite stays green while the field is dead in production. That is the same failure as ticket 1095's fixture, in a second place, and it is why replacing these assertions is meant rather than accidental.

## Required behavior

NCI enrollment carries the number the provider sends.

A key list that matches nothing in any recorded capture from that provider is a defect, not an empty result.

## Done, observably

- Converting the recorded NCI capture yields the enrollment figure the payload carries.
- The assertion is made against a recorded capture, not against an object written by hand.
- A value the converter receives and cannot read is reported rather than silently dropped, so absence and failure are distinguishable.
- No test in the NCI conversion path asserts an enrollment value read from a key name absent from every recorded capture. The two tests named above are replaced, and replacing them is intended.

## Where correct behavior is written

`sdlc/planning/clinical-trial-conformance/cases.json` in the BioData repository, case 8, restated on 2026-09-03 as a field-name mismatch with the outcome `corrected-upstream`. That file is the shared statement of correct behavior for this defect, held against both 0.9 and 1.0.

The behavior is restated above in full, because an attempt runs in a worktree where that path resolves to nothing. ADR 0025's amendment of 2026-09-03 says the restatement is what carries the statement across, and a person reconciled the two when this ticket was filed. If the restatement above looks wrong, stop and say so rather than implementing something different.

## Boundary

Change only the NCI enrollment read and its tests.

Do not change how enrollment is displayed, filtered or sorted.

Do not change the ClinicalTrials.gov enrollment path at `src/transform/trial.rs:375-377`. It deserializes through typed structs and its names are correct.

Do not remove the numeric-encoding handling in `json_get_string`.

Do not touch interventions (1133), age range (1135), study type (1134) or the stop reason (1137).

## History

Filed 2026-09-02 as one of seventeen from the BioData clinical-trial audit, on a cause that measurement disproved. Held as a draft 2026-09-03, briefly folded into a bundled ticket, then restored here as one behavior on one ticket. Restated and promoted 2026-09-03 by the BioMCP lead under Ian's decide-by-default ruling.
