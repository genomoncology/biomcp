---
flow: build
priority: 5
hold: Retired 2026-09-03. The behavior cannot occur and the requirement lives on ticket 1107. Archived.
---

**Retired 2026-09-03. Archived, not held.**

The comma-splitting branch of `json_get_string_list` cannot execute. The reader takes `["diseases", "conditions"]` and NCI always sends `diseases` as an array, which matches the array branch first. No provider can produce the scalar payload, so no fixture can prove the correction and no failing test can be written.

Its requirement moved to ticket 1107, which rewrote the same function: whatever replaces the array reading must not reintroduce a comma split on a single condition name. 1107 landed on 2026-09-03 carrying that acceptance line.

This was briefly parked in `drafts/` instead of here, to avoid the factory recording an archived ticket as done. That was the wrong trade — a permanent draft is litter in the live queue, and the archive is what ADR 0016 provides for a ticket retired without landing. The mislabel is filed as feedback instead.

---
**Retired 2026-09-03. Not done, and not abandoned either.**

The comma-splitting branch of `json_get_string_list` is unreachable. No provider BioMCP supports sends a condition as a scalar string, so the branch this ticket describes can never run and the defect can never be exercised. A ticket nothing can prove does not belong on the board.

Its requirement moved to ticket 1107, which rewrites the same function. 1107 now states it in full: a condition name is never split on a comma, and the scalar and array forms of one value agree.

BioData retired the matching conformance case, case 7, on 2026-09-03 for the same reason. The `retired` block of `cases.json` carries the wording.

**Why this is a held draft and not an archived ticket.** `sdlc/project/tasks` reports every file under `sdlc/tickets/archive/` as `done`, so archiving a retired ticket tells the board the work happened. It did not. Ticket 1081 is the same mistake already on the record: archived when 1123 took over its work, and counted as done ever since. A retired or superseded ticket therefore stays in `sdlc/tickets/drafts/` with a `hold:` line saying why. The convention is written down in `sdlc/planning/notes/retiring-a-ticket.md`.

**Ian's standing ruling applies.** Retirement was decided here on 2026-09-03 rather than sent up, under the decide-by-default rule. It is cheap to overturn: move the file back to `sdlc/tickets/` and delete the `hold:` line.

---
# The comma-splitting branch is unreachable, and the requirement belongs on ticket 1107

**Measurement disputed the stated cause. Retired 2026-09-03. The sections below are kept as the evidence for that.**

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

## The retirement

This ticket is retired. The behavior it describes cannot occur.

Carry its requirement forward as a boundary on ticket 1107, which rewrites the same function. 1107 fixes the array branch, where `filter_map(|v| v.as_str())` discards NCI's disease objects and returns an empty list. That rewrite must not reintroduce a comma split, and the scalar and array forms of one value must still agree. Stating that on 1107 keeps the requirement mechanical without keeping a ticket for a defect nobody can trigger.

Decided on 2026-09-03 by the BioMCP and BioData leads under Ian's decide-by-default ruling, rather than sent to him as an open question. BioData retired case 7 the same day.

## Where correct behavior is written

`repos/biodata/sdlc/planning/clinical-trial-conformance/cases.json`, case 7. That case carries a `correction_2026_09_02` field recording the measurement. Case 7 was retired on 2026-09-03. `cases.json` no longer carries it as a case; the `retired` block carries the reason.

Reported by the BioData lead in `notes/biomcp/feedback/2026-09-02-seventeen-trial-defects-to-fix-in-0-9.md`, defect 7 of seventeen. Disputed in `notes/biomcp/feedback/2026-09-02-nci-field-names-measured-against-the-live-api.md`.

## Boundary

Do not change how a genuinely multi-valued field is read. Defect 4 covers the array-of-objects failure in the same function and is filed as ticket 1107.
