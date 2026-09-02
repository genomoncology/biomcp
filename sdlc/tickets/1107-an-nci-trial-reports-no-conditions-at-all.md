---
flow: build
priority: 8
---

# Every NCI trial reports an empty condition list and an empty intervention list

`json_get_string_list` reads an array by calling `v.as_str()` on each element and discarding whatever returns `None`:

```rust
serde_json::Value::Array(arr) => {
    return arr
        .iter()
        .filter_map(|v| v.as_str())
```

NCI sends `diseases` as an array of objects. Every element answers `None`, `filter_map` drops all of them, and the function returns an empty vector. No error is raised and nothing marks the list as failed, so an NCI trial reports that it studies no conditions. Interventions are read through the same call and fail the same way.

A caller asking what an NCI trial is for gets nothing, and cannot tell that from a trial that genuinely lists no conditions.

Verified in `src/transform/trial.rs` on 2026-09-02 against `0.9.0-dev.6`, and by the BioData audit against the recorded payload `testdata/sources/nci_cts/search_melanoma_20260811.json`.

## Required behavior

An NCI trial reports the conditions and the interventions its payload carries.

An element the converter cannot read is an error. It is never dropped silently, because a silent drop is indistinguishable from an absence in the source.

## Done, observably

- Converting the recorded NCI payload yields a non-empty condition list matching the disease names the payload carries.
- Interventions come through the same way.
- An unreadable element produces an error rather than a shorter list.

## Where correct behavior is written

`repos/biodata/sdlc/planning/clinical-trial-conformance/cases.json`, case 4. That file is the shared statement of correct behavior for this defect, held against both 0.9 and 1.0 so the two cannot drift into disagreeing about what correct means.

Take the assertion from that case, write it as a failing test, then fix. Red before green.

**Do not copy the expected behavior into this repository as a second statement of it.** Reference the case. If the case's expected behavior looks wrong, stop and say so rather than implementing something different; that disagreement gets settled in the case file, not in this codebase.

Reported by the BioData lead in `notes/biomcp/feedback/2026-09-02-seventeen-trial-defects-to-fix-in-0-9.md`, defect 4 of seventeen, verified against BioMCP `60f5377e` and re-verified here against `0.9.0-dev.6` on 2026-09-02.
## Boundary

Do not change how CTGov conditions are read; that path works. Do not change the shape of the condition list on the output side. The comma-splitting difference between the scalar and array branches of the same function is defect 7 and is filed separately; fix only the array-of-objects reading here.
