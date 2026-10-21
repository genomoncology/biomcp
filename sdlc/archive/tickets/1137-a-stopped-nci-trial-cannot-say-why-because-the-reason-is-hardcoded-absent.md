---
flow: build
priority: 6
---

**Archived 2026-09-03. Re-merged into ticket 1132, not abandoned.**

This defect is real and still reproduces. It moved because all five NCI key-name defects live in one thirty-line block of `from_nci_trial`, share one recorded fixture, and are each a one-line change to a key list. Five separate flows would cost five design stages, five reviews and five verifications for one commit's worth of work, and each would rebase onto the last.

Ticket 1132 carries this defect's row in its table, its required behavior, and its assertion.

---
# A stopped NCI trial can never say why, because the reason is hardcoded absent

`from_nci_trial` builds its result at `src/transform/trial.rs:626` with the stop reason set to nothing at all:

```rust
why_stopped: None,
```

There is no lookup to get wrong. The field is unconditionally absent on the NCI path, and NCI does publish the reason. It sends `why_study_stopped`, which appears nowhere in this repository's source.

The distinction the field carries is the one a reader needs. A trial that stopped because the drug harmed patients and a trial that stopped because the sponsor could not recruit are different evidence about that drug, and the status word alone reads the same for both. On NCI trials BioMCP has never carried it.

Measured against this repository's own recorded NCI capture, `testdata/sources/nci_cts/search_melanoma.json`. Every record carries 58 field names and `why_study_stopped` is one of them. The BioData lead's live query of `clinicaltrialsapi.cancer.gov/api/v2/trials` on 2026-09-02 reported the same 58 names independently. A search of the source tree for `why_study_stopped` returns nothing.

## The relationship to ticket 1097

Ticket 1097 is the ClinicalTrials.gov half and the display half of the same user-visible gap. It cites `NCT03515785`, whose CTGov payload carries `whyStopped`, and CTGov's reason is already read at `src/transform/trial.rs:349-354`. 1097 is about a stopped trial's reason being shown wherever its status is shown.

This ticket is only about the NCI conversion. Its job is to make the value exist on the NCI path so that whatever 1097 does with it has something to show. The two do not overlap in the code they touch and neither waits on the other.

## Required behavior

An NCI trial that states why it stopped carries that reason.

A field is absent because the source lacks it, never because the converter never looked.

## Done, observably

- Converting an NCI payload that carries a stop reason yields that reason.
- Converting an NCI payload whose stop reason is null or missing yields an absent reason, and that absence is not indistinguishable from the hardcoded one.
- The assertion is made against a recorded capture, not against an object written by hand, for whichever of the two cases a recorded capture can prove.

## The fixture, honestly

Read this before planning the proof. `testdata/sources/nci_cts/search_melanoma.json` proves the key exists in an NCI trial record, and in that record its value is `null`, because the trial has not stopped. So the repository can prove today that the key is part of the record shape and that a null reads as absent. It cannot yet prove that a stated reason converts, because no recorded NCI capture holds a stopped trial.

Do not invent a value to close that gap. A hand-written payload asserting a reason the provider never sent is the exact failure that kept ticket 1095's defect alive for months, and it is the failure ticket 1138 exists to prevent. Either record a capture of a stopped NCI trial, or prove the half that the recorded evidence supports and say plainly in the record which half is proven and which is not.

The 58-field capture is also classified `pending_verification` in `testdata/sources/capture-receipts.json` and has no provider receipt. That gap is ticket 1138.

## Where correct behavior is written

`sdlc/planning/clinical-trial-conformance/cases.json` in the BioData repository, case 22, "A stop reason the provider states is carried". That file is the shared statement of correct behavior, held against both 0.9 and 1.0 so the two cannot drift.

The behavior is restated above in full, because an attempt runs in a worktree where that path resolves to nothing. ADR 0025's amendment of 2026-09-03 says the restatement is what carries the statement across, and a person reconciled the two when this ticket was filed. If the restatement above looks wrong, stop and say so rather than implementing something different.

## Boundary

Change only the NCI conversion so the stop reason reaches the caller, plus its tests.

Do not change how the stop reason is displayed, and do not change which statuses cause it to be shown. Both are ticket 1097.

Do not change the ClinicalTrials.gov stop-reason path at `src/transform/trial.rs:349-354`. It is correct.

Do not touch interventions (1133), age range (1135), study type (1134), enrollment (1119) or eligibility text (1136).

## History

Found 2026-09-03 by the BioData lead while auditing the conformance cases, verified here independently against this repository's own capture and source the same day. Split out of ticket 1132, which bundled five behaviors and was superseded for that reason.
