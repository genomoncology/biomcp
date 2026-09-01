---
flow: build
priority: 8
---

# Three ClinicalTrials.gov type fields never deserialize, and the test fixture certifies the bug

Every arm on every trial card prints an empty type. Verified against the repository build `biomcp 0.9.0-dev.6` (`./target/debug/biomcp`) on 2026-09-01:

```
$ biomcp get trial NCT04185038 arms
| Arm | Type | Interventions | Description |
|---|---|---|---|
| ARM A (Tumor Cavity Infusion) - [CLOSED TO ENROLLMENT] | - | Biological: ... |
| ARM B (Ventricular System Infusion) - [CLOSED TO ENROLLMENT] | - | Biological: ... |
| ARM C (DIPG) - [CLOSED TO ENROLLMENT] | - | Biological: ... |
| Arm D (Non-pontine DMG) | - | Biological: ... |
| Arm E | - | Biological: ... |
```

Five arms, five empty types. The upstream record carries a type for each.

`CtGovIntervention.intervention_type`, `CtGovArmGroup.arm_group_type` and `CtGovReference.reference_type` are permanently absent. The three structs carry `#[serde(rename_all = "camelCase")]` (`src/sources/clinicaltrials.rs:463`, `:477`, `:578`), so serde looks for `interventionType`, `armGroupType` and `referenceType`. The pinned ClinicalTrials.gov API v2 contract names all three keys `type`:

- `/protocolSection/armsInterventionsModule/interventions/type`
- `/protocolSection/armsInterventionsModule/armGroups/type`
- `/protocolSection/referencesModule/references/type`

No field-level rename appears anywhere in that file. A caller asking what kind of intervention a trial studies, or whether an arm is experimental or a comparator, gets nothing, and no error says why.

The suite does not catch it because the fixture was built from the struct rather than from the provider. `src/transform/trial.rs:783` supplies `"armGroupType": "EXPERIMENTAL"`. That string is the only occurrence of `armGroupType` in the source tree. `interventionType` and `referenceType` appear nowhere at all, in source or fixture. So the test passes while the field is dead against the real API.

The recorded captures cannot settle it either. `CTGOV_GET_FIELDS_BASE` (`src/sources/clinicaltrials.rs:18-33`) requests `InterventionName` and `InterventionOtherName` and never requests `InterventionType`, so the provider never returned it on that path. The field is requested only for the `arms` section (`:87`, `:90`), while `from_ctgov_study` populates intervention detail unconditionally. The same field is therefore absent or present depending on which section was asked for, with nothing telling the caller which they got.

This is silent data loss in production, not a latent risk.

Full finding, with the reasoning and the evidence list: `notes/biomcp/feedback/2026-09-01-three-trial-fields-never-deserialize.md` in the workspace notes.

## Required behavior

An arm's type, an intervention's type and a reference's type appear in output when the provider supplies them.

A test that claims to pin a provider field is written against what the provider sends, not against what the struct declares. A fixture whose key does not exist in the provider contract does not count as coverage.

A field the caller can see in one section and not in another is a reportable condition, not a silent difference.

## Done, observably

- `get trial NCT04185038 arms` prints a type for each arm.
- Intervention type and reference type appear in output, in markdown and in JSON, wherever the provider supplies them.
- The suite fails if any of the three fields stops deserializing.
- No test in the trial transform path asserts against a key absent from the pinned provider contract.

## Boundary

This ticket changes user-visible output, which is intended and is the point of filing it as a named change rather than a quiet patch. It does not redesign the trial card, does not add sections, and does not change which trials any search returns. Whether other structs in `src/sources/clinicaltrials.rs` were tested the same way is worth checking while here; fixing an unrelated struct's separate defect is not in scope and belongs in its own ticket.
