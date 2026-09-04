---
flow: build
priority: 8
deps: ["1132", "1126"]
---

# No key list in the code may name a field no provider sends

Ticket 1126 checks that a fixture cannot attest to a key the provider does not send. This is the same rule pointed the other way: the code cannot read a key the provider does not send. Both failures are silent, and between them they have produced six dead fields in this repository that nobody noticed for months.

## The instances, so the size of the class is on the record

One in ClinicalTrials.gov, fixed by ticket 1095. Three structs sought `interventionType`, `armGroupType` and `referenceType` while the provider names all three `type`. Three fields were permanently empty.

Five in the NCI reader, all found in one pass on 2026-09-03 and each filed as its own ticket. `interventions` (1133) sought at the top level while NCI nests it in the arms. `minimum_age` and its variants (1135) sought at the top level while NCI nests them in structured eligibility. `study_type` (1134) sought and absent, so every trial falls through and reports its primary purpose instead. `enrollment`, `enrollment_target` and `target_enrollment` (1119) all sought, all absent, while NCI sends `minimum_target_accrual_number`. And a stop reason (1137) hardcoded absent while NCI sends `why_study_stopped`.

A seventh, found 2026-09-03 while an agent measured the live provider to check a different ticket's premise. `CtGovLocation` at `src/sources/clinicaltrials.rs`, in `CtGovLocation` carries `central_contacts`, and ClinicalTrials.gov never sends it. Across 9,230 locations in 1,600 live studies the location key set is exactly `city`, `contacts`, `country`, `facility`, `geoPoint`, `state`, `status` and `zip`. Both `extract_locations` at `src/transform/trial.rs`, in `extract_locations` and `extract_contacts` at `:197` chain onto that always-empty field. It is the quietest instance yet: it produces no wrong value, only a contact that could never appear, so nothing looks missing at all. This one has no separate ticket, because removing an always-empty field changes no observable behavior and there is no failing test to write. The check this ticket builds is the only thing that can catch it, which is exactly the argument for the check.

Seven instances of one class. None of them raised an error, because a key that is missing and a value that is empty are the same answer to `json_get_string`. Every one was found by a person reading a provider's own response beside the code, which is not a check.

Worse, the suite actively defended four of them. Tests at `src/transform/trial.rs`, `:935`, `:959`, `:960`, `:963` and `:965` supply `target_enrollment`, `enrollment_target`, a top-level `interventions`, a top-level `minimum_age` and a top-level `study_type`. Every one of those keys is invented. A test written against the reader's own shape passes forever while the field is dead in production.

## Required behavior

A key name the conversion code reads is either present in a recorded capture from that provider endpoint, or is declared as an exception with a stated reason.

Where the code offers several alternative names for one value, at least one of them is present in a recorded capture, or the whole group is a declared exception. A group where every name is dead is the failure this check exists to catch.

A key name that is neither present nor declared fails the check, and the message names the key, the group it belongs to, and the endpoint it claims to come from.

The check runs from the gate ladder, so it fails a build rather than producing a report someone has to read.

## What makes this harder than it looks, and must be handled

Absence in a capture does not always mean the provider does not send it. Two ways that bites, both already true in this repository.

**A capture recorded with a restricted field list proves nothing about the fields it did not request.** Every ClinicalTrials.gov capture under `testdata/sources/ctgov/` except one was recorded with a `fields=` list, and none of those lists ever asked for the arms module. A naive union of keys would call correct field names dead. The receipts already record each capture's request URL, which carries the list that was asked for, so the evidence needed to tell "the provider did not send it" from "we never asked" is on disk.

**A capture can be minimized after recording.** This was a real gap for NCI and it is now closed. `search_melanoma_20260811.json` had a receipt and six fields; `search_melanoma.json` had all 58 fields and no receipt. The repository held evidence without provenance and provenance without evidence. Recording a capture that has both is no longer part of this ticket, because one was recorded on 2026-09-03. Read the section below.

Do not resolve either by loosening the rule until the current tree passes. A check that passes because it was weakened is worth less than no check.

## Done, observably

- Adding a key name to a conversion key list that no recorded capture for its endpoint supports fails the check, and the message names the key, its group and the endpoint.
- Each of the seven instances above fails the check when reintroduced, `central_contacts` included. Tests pin that, so the check is proven against the defects it was built for rather than against invented examples.
- For NCI, the check rests on `testdata/sources/nci_cts/get_nci_2023_04529_full_20260903.json`, which carries the field names and has a provider receipt. Recording it is done and is not in scope.
- For ClinicalTrials.gov, a correct field name that no capture requested does not fail the check, and a field name that a capture requested and the provider did not return does fail it.
- Every exception carries a written reason naming the field and why no capture can attest it.
- The current tree passes with the exceptions it needs and no others. For each failure found on the way, the ticket's record says whether it was a real defect of this class or an exception, and why.
- The check runs from the gate ladder and fails the build.

## Where correct behavior is written

`sdlc/planning/clinical-trial-conformance/cases.json` in the BioData repository, case 17, "The wire key the provider sends is the key the parser reads". That file is the shared statement of correct behavior, held against both 0.9 and 1.0 so the two cannot drift. Case 17's own assertion covers the fixture direction; this ticket covers the code direction of the same rule, and both are needed for the case to hold.

The behavior is restated above in full, because an attempt runs in a worktree where that path resolves to nothing. ADR 0025's amendment of 2026-09-03 says the restatement is what carries the statement across, and a person reconciled the two when this ticket was filed.

## Boundary

This is a sibling of ticket 1126, not a part of it. 1126 guards fixtures; this guards the code. Both need the same provenance record, so 1126 lands first and builds it. This ticket extends what 1126 left. Do not build a second, competing record of where captures came from, and do not rebuild the one 1126 delivered.

`testdata/sources/capture-receipts.json` already classifies fixtures by provenance and carries each capture's request URL. It is the record both checks read.

Do not fix any of the six instances here. Ticket 1095 landed the ClinicalTrials.gov one; the five NCI ones are tickets 1119, 1133, 1134, 1135 and 1137. Each has its own record. This ticket makes them fail; the others make them pass. If the check is landed first and the tree is red, that is the expected order, and the check may be landed with the known-failing groups declared as exceptions that name their owning ticket.

Do not change any converter's behavior, and do not change a capture's contents to make the check pass.

A new Rust test runs under the existing `make test` lane and needs no path opening. If the design finds that the check cannot run from the ladder without touching `sdlc/scripts/`, stop and say so rather than working around it; the ticket needs an `opens:` line and the gate reads that from origin/main.

## History

Proposed by the BioMCP lead on 2026-09-03 after the NCI field-name audit found five instances in one pass, and approved as a ticket in its own right rather than as an amendment to 1126. The class check that ticket 1132 carried in prose lives here instead; 1132 was superseded for bundling.

## This ticket must land after 1132

Its check fails on `main` today. `["interventions"]` names no key the NCI payload carries, and `["enrollment", "enrollment_target", "target_enrollment"]` names three. Both are real defects and both are fixed by ticket 1132.

Landing this guard before those fixes would turn `main` red and block every ticket in the channel, because the green-main gate runs before any ticket starts. That is the opposite of what a guard is for.

Land 1132 first. Then this check passes on the corrected code and holds the class shut.

## The NCI capture this check needs, recorded 2026-09-03

`testdata/sources/nci_cts/get_nci_2023_04529_full_20260903.json`, recorded from `https://clinicaltrialsapi.cancer.gov/api/v2/trials?size=1&current_trial_status=Active` with no field restriction. It carries all 58 top-level keys the provider sends. The receipt is in `testdata/sources/capture-receipts.json`, classified `real_and_receipted`.

It attests every field name the five NCI defects turn on. `arms[].interventions` is present. `eligibility.structured.min_age` is present and reads `"18 Years"`. `study_protocol_type`, `minimum_target_accrual_number` and `why_study_stopped` are all present. None of `interventions`, `study_type`, `enrollment`, `enrollment_target`, `target_enrollment` or `minimum_age` appears at the top level.

Two reductions were made and neither removes a key name. The sites array was cut from 1261 entries to the first 3, each carrying every site key the provider sends. Within those 3 sites the seven person-bearing and organization-contact values read `REDACTED`. The unredacted response carried 326 distinct email addresses and 1028 distinct telephone numbers, and this repository is public. The receipt states both reductions.

The check reads key names, so the redaction costs it nothing. If the design finds it needs an attested *value* from one of the seven redacted fields, stop and say so rather than recording an unredacted capture.

## Correction, 2026-09-03: the seventh instance is described wrongly, and the check as specified cannot catch it

Two claims above are wrong and one of them undermines the check this ticket builds. Both were found by reading the provider's own published schema rather than by sampling captures.

**ClinicalTrials.gov publishes its field schema.** `https://clinicaltrials.gov/api/v2/studies/metadata` is public, needs no key, and documents 278 fields with their nesting and types. It settles attestation exactly, where sampled captures only ever sample.

**`centralContacts` is documented, so "the provider never sends it" is false.** The schema places it at `protocolSection.contactsLocationsModule.centralContacts`, type `Contact[]`, a sibling of `locations`. It is well populated: all 20 recruiting melanoma trials sampled on 2026-09-03 publish one.

The defect is a nesting error, not a nonexistent field. `CtGovContactsLocationsModule.central_contacts` is correct and `src/transform/trial.rs` already reads it correctly when building the contacts section. `CtGovLocation.central_contacts` is the dead one, because `Location` has no such member in the schema. It is consulted twice as a fallback, in `extract_locations` and in `extract_contacts`, and neither fallback can ever fire.

The observable cost stays as this ticket described it. Central contacts do reach the caller through the module-level read. Nothing is missing from the output. That part was right.

**The check as specified would pass this defect.** The requirement above says a key name the code reads is "present in a recorded capture from that provider endpoint." `centralContacts` is present in captures, at module level. A check that treats a capture as a flat set of key names finds it, marks the location field attested, and moves on. The ticket names this instance as one of the seven the check must catch, and as specified it catches six.

**A key name is attested at its path, not as a bare name.** `contactsLocationsModule.centralContacts` is attested. `contactsLocationsModule.locations[].centralContacts` is not. The same word is real in one place and invented in another, and only the path tells them apart. Build the check on paths.

`armGroupType`, the invented key that started this whole class, is absent from the schema entirely. So the schema separates a real key at the wrong path from a key that does not exist at all, and both are defects worth different messages.

## The decision this correction carries

Read the provider's published schema where one exists, and fall back to recorded captures where none does.

The capture-union approach has now produced two wrong answers in one evening. It called `secondaryOutcomes` dead, and that is a documented field our one capture happened not to carry, which refused ticket 1126's first attempt. It would have called `centralContacts` attested at a path where it does not exist. A sample proves presence and never proves absence, which is the exact question this check asks.

ClinicalTrials.gov has a schema. NCI does not publish an equivalent, so NCI keeps the capture path and keeps `get_nci_2023_04529_full_20260903.json` as its evidence. Say in the provenance record which source attests each endpoint.

Cost of this decision. The schema is a network dependency the gate ladder must not acquire, so the schema is recorded into `testdata/sources/` like any other capture, with a receipt, and refreshed deliberately. A field the provider adds after the recording reads as unattested until someone re-records. That is the same staleness captures already have, and the schema is one file instead of a growing set of sampled trials.
