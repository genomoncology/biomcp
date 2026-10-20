---
flow: build
priority: 3
hold: Retired 2026-09-06. Record 1171 removed the NCI detail parser and clone target. Archived.
---

# The NCI trial reader clones the eligibility object per field

**Retired 2026-09-06. Archived, not held.** Record 1171 replaced the raw NCI
detail conversion with BioData-backed projection, so the named clone path no
longer exists.

## Goal

Reading an NCI trial's structured eligibility borrows the provider object instead of copying it once per field.

At commit `b2e05326`, after the BioData trial-reference integration, `from_nci_trial` in `src/transform/trial.rs` still resolves `structured_eligibility` to a borrowed JSON map, then reads three fields from it. Each read wraps the map in a fresh owned value:

```rust
json_get_string(&serde_json::Value::Object(value.clone()), &["min_age"])
json_get_string(&serde_json::Value::Object(value.clone()), &["max_age"])
json_get_string(&serde_json::Value::Object(value.clone()), &["sex"])
```

`json_get_string` takes `&serde_json::Value`, so each call deep-copies the whole structured-eligibility map to hand it a reference it could have had for free. Every full NCI trial detail conversion through `from_nci_trial` pays this three times; NCI search hits use a different conversion path.

Commit `eddcac9e` introduced the pattern while adding the `sex` read and the `as_object()` guard. The object check is worth keeping: record 1136 established that NCI sends `eligibility` as an object where the code once expected a string. A non-object value is ignored for structured eligibility and the trial conversion continues; that behavior does not require the clone.

## Done, observably

- Reading NCI structured eligibility performs no deep copy of the provider object. The output strings may still allocate as they do today.
- The implementation uses a borrowed-map accessor or equivalent direct borrowed access; `from_nci_trial` does not construct `Value::Object(value.clone())` for these reads.
- A non-object or null outer `eligibility`, and a non-object or null `eligibility.structured`, still yields no structured eligibility fields while trial conversion succeeds. Malformed primary age strings retain their current retained-unparsed behavior.
- Minimum age, maximum age, and sex parse exactly as they do today, including the retained-unparsed handling of an NCI no-limit maximum.

## Boundary

This ticket changes how the NCI reader accesses fields it already reads. It does not add or remove fields, change age parsing or the age-range string, change the sex mapping, touch the ClinicalTrials.gov reader, or alter the BioData-backed `TrialReference` and fallible CTGov conversion landed immediately before it.
