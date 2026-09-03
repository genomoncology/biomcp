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

## Boundary

Do not change how CTGov conditions are read; that path works. Do not change the shape of the condition list on the output side.

Do not touch interventions, age range, study type, enrollment or `why_stopped`. All five are ticket 1132.
