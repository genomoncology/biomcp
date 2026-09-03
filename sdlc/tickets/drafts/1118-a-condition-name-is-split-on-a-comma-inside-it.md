---
flow: build
priority: 5
---

# The comma-splitting branch is unreachable, and the requirement belongs on ticket 1107

**Measurement disputes the stated cause. Restated 2026-09-03 and still held. Recommended for retirement, not promotion.**

## What the original ticket said, and why it was wrong

The first version of this ticket said a single condition string is split on commas, so `"Lung Cancer, Non-Small Cell"` becomes two diseases that do not exist.

The scalar branch of `json_get_string_list` at `src/transform/trial.rs:549-556` does split on commas. Nothing can reach it.

`src/transform/trial.rs:576` and `:618` read `["diseases", "conditions"]`. NCI sends `diseases`, always, as an array of objects. It has no `conditions` field at all. The array branch matches first on every record, so the scalar branch never runs for conditions.

Two independent sources agree:

- This repository's own recorded NCI capture, `testdata/sources/nci_cts/search_melanoma.json`. Every record carries `diseases` as a list of objects. No record carries `conditions`.
- The BioData lead's live query of `clinicaltrialsapi.cancer.gov/api/v2/trials` on 2026-09-02. Requesting `include=conditions` returns records containing no such field.

## No other provider reaches it either

`json_get_string_list` has exactly three call sites, all in this file: `:576`, `:618` and `:619`. All three sit inside `from_nci_hit` and `from_nci_trial`. Those two functions have exactly two production callers, `src/entities/trial/search/nci.rs:82` and `src/entities/trial/get.rs:174`, both NCI.

ClinicalTrials.gov conditions are deserialized into typed structs and never pass through this helper.

So the branch is unreachable from every provider BioMCP supports today, not only from NCI. A conformance case for it cannot be fixtured, because no provider can produce the payload it needs.

## Recommendation

Retire this ticket. The behavior it describes cannot occur.

Carry its requirement forward as a boundary on ticket 1107, which rewrites the same function. 1107 fixes the array branch, where `filter_map(|v| v.as_str())` discards NCI's disease objects and returns an empty list. That rewrite must not reintroduce a comma split, and the scalar and array forms of one value must still agree. Stating that on 1107 keeps the requirement mechanical without keeping a ticket for a defect nobody can trigger.

Retiring is Ian's call, and this draft holds until he makes it.

## Where correct behavior is written

`repos/biodata/sdlc/planning/clinical-trial-conformance/cases.json`, case 7. That case carries a `correction_2026_09_02` field recording the measurement. What the case asserts is unchanged, and changing it is BioData's to do after Ian rules.

Reported by the BioData lead in `notes/biomcp/feedback/2026-09-02-seventeen-trial-defects-to-fix-in-0-9.md`, defect 7 of seventeen. Disputed in `notes/biomcp/feedback/2026-09-02-nci-field-names-measured-against-the-live-api.md`.

## Boundary

Do not change how a genuinely multi-valued field is read. Defect 4 covers the array-of-objects failure in the same function and is filed as ticket 1107.
