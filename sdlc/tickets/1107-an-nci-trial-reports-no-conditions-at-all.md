---
flow: build
priority: 8
---

# Every NCI trial reports an empty condition list

`json_get_string_list` reads an array by calling `v.as_str()` on each element and discarding whatever returns `None`:

```rust
serde_json::Value::Array(arr) => {
    return arr
        .iter()
        .filter_map(|v| v.as_str())
```

NCI sends `diseases` as an array of objects. Every element answers `None`, `filter_map` drops all of them, and the function returns an empty vector. No error is raised and nothing marks the list as failed, so an NCI trial reports that it studies no conditions.

A caller asking what an NCI trial is for gets nothing, and cannot tell that from a trial that genuinely lists no conditions.

Verified in `src/transform/trial.rs` on 2026-09-02 against `0.9.0-dev.6`, and by the BioData audit against the recorded payload `testdata/sources/nci_cts/search_melanoma_20260811.json`.

## Required behavior

An NCI trial reports the conditions its payload carries.

An element the converter cannot read is an error. It is never dropped silently, because a silent drop is indistinguishable from an absence in the source.

## Done, observably

- Converting the recorded NCI payload yields a non-empty condition list matching the disease names the payload carries.
- An unreadable element produces an error rather than a shorter list.
- A single condition string carrying a comma inside one name is not split into two conditions.

## Where correct behavior is written

`repos/biodata/sdlc/planning/clinical-trial-conformance/cases.json`, case 4. That file is the shared statement of correct behavior for this defect, held against both 0.9 and 1.0 so the two cannot drift into disagreeing about what correct means.

Take the assertion from that case, write it as a failing test, then fix. Red before green.

**Do not copy the expected behavior into this repository as a second statement of it.** Reference the case. If the case's expected behavior looks wrong, stop and say so rather than implementing something different; that disagreement gets settled in the case file, not in this codebase.

Reported by the BioData lead in `notes/biomcp/feedback/2026-09-02-seventeen-trial-defects-to-fix-in-0-9.md`, defect 4 of seventeen, verified against BioMCP `60f5377e` and re-verified here against `0.9.0-dev.6` on 2026-09-02.
## Amendment, 2026-09-03: interventions are a different bug, and the comma split moves here

Two corrections, both measured against this repository's own recorded capture `testdata/sources/nci_cts/search_melanoma.json`, which carries 58 fields per record.

**Interventions leave this ticket.** The original text said interventions fail through the same `as_str()` call. They do not. There is no top-level `interventions` key in the payload at all, so `json_get_string_list` at `src/transform/trial.rs:619` finds nothing to read and returns an empty vector before reaching `as_str()`. NCI nests them at `arms[].interventions[].name`. That is a field-name defect and it is now ticket 1132, together with four more of the same kind.

Fixing this ticket as written would have repaired conditions, left interventions empty, and written a record claiming both were done.

**The comma split arrives here.** Draft 1118 held defect 7, the scalar branch of `json_get_string_list` splitting `"Lung Cancer, Non-Small Cell"` into two conditions that are not real. No provider sends a scalar condition string, so that branch is unreachable and the defect can never be exercised. 1118 is archived rather than left as a case nothing can prove. Its requirement is carried here, because this ticket rewrites the same function: whatever replaces the array reading must not reintroduce a comma split on a single name.

The `diseases` half of this ticket is unchanged and confirmed. `diseases` is present, it is an array of objects, and every element is dropped.

## The disagreement with case 4 is settled. Proceed.

The first attempt refused on 2026-09-03 at 14:18, and it was right to. This ticket names BioData's case 4 as the authoritative statement and tells you to stop if that statement looks wrong. Case 4 still reads "Interventions are read the same way." The amendment above says the opposite. An attempt cannot pick between two statements of correct behavior, and it must not.

**That disagreement has now been settled between the two leads, and this section records the outcome so no attempt has to settle it.**

The measurement that decided it: `testdata/sources/nci_cts/search_melanoma.json`, this repository's own recorded capture, carries 58 fields per record. `diseases` is present and is an array of objects. There is **no top-level `interventions` key at all**. So conditions and interventions fail for two different reasons, and only one of them is `as_str()` discarding objects.

The settlement:

- **Case 4's conditions half stands unchanged.** It is correct and it is what this ticket implements.
- **Case 4's interventions half is superseded.** Interventions are nested at `arms[].interventions[].name`. That is a field-name defect and it belongs to ticket 1132, which fixes it alongside four more of the same kind.
- BioData has been asked to amend case 4 accordingly. The written record is `notes/biodata/feedback/2026-09-03-five-nci-field-names-and-three-cases-to-change.md`.

**You are authorized to implement case 4's conditions half alone.** Doing so is not a second statement of expected behavior and it is not a conditions-only reinterpretation made by an attempt. It is the settled statement, recorded here by the BioMCP lead on 2026-09-03 before you were dispatched.

If case 4 has already been amended by the time you read it, follow the amended case and this section becomes redundant. If any *other* part of case 4 looks wrong, the original instruction still applies: stop and say so.

## Boundary

Do not change how CTGov conditions are read; that path works. Do not change the shape of the condition list on the output side.

Do not touch interventions, age range, study type, enrollment or `why_stopped`. All five are ticket 1132.
