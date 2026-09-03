---
flow: build
priority: 5
---

# No key list in the code may name a field no provider sends

Ticket 1126 checks that a fixture cannot attest to a key the provider does not send. This is the same rule pointed the other way: the code cannot read a key the provider does not send. Both failures are silent, and between them they have produced six dead fields in this repository that nobody noticed for months.

## The instances, so the size of the class is on the record

One in ClinicalTrials.gov, fixed by ticket 1095. Three structs sought `interventionType`, `armGroupType` and `referenceType` while the provider names all three `type`. Three fields were permanently empty.

Five in the NCI reader, all found in one pass on 2026-09-03 and each filed as its own ticket. `interventions` (1133) sought at the top level while NCI nests it in the arms. `minimum_age` and its variants (1135) sought at the top level while NCI nests them in structured eligibility. `study_type` (1134) sought and absent, so every trial falls through and reports its primary purpose instead. `enrollment`, `enrollment_target` and `target_enrollment` (1119) all sought, all absent, while NCI sends `minimum_target_accrual_number`. And a stop reason (1137) hardcoded absent while NCI sends `why_study_stopped`.

Six instances of one class. None of them raised an error, because a key that is missing and a value that is empty are the same answer to `json_get_string`. Every one was found by a person reading a provider's own response beside the code, which is not a check.

Worse, the suite actively defended four of them. Tests at `src/transform/trial.rs:930`, `:935`, `:959`, `:960`, `:963` and `:965` supply `target_enrollment`, `enrollment_target`, a top-level `interventions`, a top-level `minimum_age` and a top-level `study_type`. Every one of those keys is invented. A test written against the reader's own shape passes forever while the field is dead in production.

## Required behavior

A key name the conversion code reads is either present in a recorded capture from that provider endpoint, or is declared as an exception with a stated reason.

Where the code offers several alternative names for one value, at least one of them is present in a recorded capture, or the whole group is a declared exception. A group where every name is dead is the failure this check exists to catch.

A key name that is neither present nor declared fails the check, and the message names the key, the group it belongs to, and the endpoint it claims to come from.

The check runs from the gate ladder, so it fails a build rather than producing a report someone has to read.

## What makes this harder than it looks, and must be handled

Absence in a capture does not always mean the provider does not send it. Two ways that bites, both already true in this repository.

**A capture recorded with a restricted field list proves nothing about the fields it did not request.** Every ClinicalTrials.gov capture under `testdata/sources/ctgov/` except one was recorded with a `fields=` list, and none of those lists ever asked for the arms module. A naive union of keys would call correct field names dead. The receipts already record each capture's request URL, which carries the list that was asked for, so the evidence needed to tell "the provider did not send it" from "we never asked" is on disk.

**A capture can be minimized after recording.** The only receipted NCI capture, `testdata/sources/nci_cts/search_melanoma_20260811.json`, holds six fields. The capture that carries all 58, `search_melanoma.json`, is classified `pending_verification` and has no receipt. So for NCI the repository today has evidence without provenance and provenance without evidence, and a check that trusts only receipts would pass everything while a check that trusts only content would rest on an unreceipted file. Resolving that is part of this ticket.

Do not resolve either by loosening the rule until the current tree passes. A check that passes because it was weakened is worth less than no check.

## Done, observably

- Adding a key name to a conversion key list that no recorded capture for its endpoint supports fails the check, and the message names the key, its group and the endpoint.
- Each of the six instances above fails the check when reintroduced. Tests pin that, so the check is proven against the defects it was built for rather than against invented examples.
- For NCI, the check rests on a capture that both carries the field names and has a provider receipt. Recording one is in scope.
- For ClinicalTrials.gov, a correct field name that no capture requested does not fail the check, and a field name that a capture requested and the provider did not return does fail it.
- Every exception carries a written reason naming the field and why no capture can attest it.
- The current tree passes with the exceptions it needs and no others. For each failure found on the way, the ticket's record says whether it was a real defect of this class or an exception, and why.
- The check runs from the gate ladder and fails the build.

## Where correct behavior is written

`sdlc/planning/clinical-trial-conformance/cases.json` in the BioData repository, case 17, "The wire key the provider sends is the key the parser reads". That file is the shared statement of correct behavior, held against both 0.9 and 1.0 so the two cannot drift. Case 17's own assertion covers the fixture direction; this ticket covers the code direction of the same rule, and both are needed for the case to hold.

The behavior is restated above in full, because an attempt runs in a worktree where that path resolves to nothing. ADR 0025's amendment of 2026-09-03 says the restatement is what carries the statement across, and a person reconciled the two when this ticket was filed.

## Boundary

This is a sibling of ticket 1126, not a part of it. 1126 guards fixtures; this guards the code. They will want the same provenance record and should share it rather than each building one.

`testdata/sources/capture-receipts.json` already classifies fixtures by provenance and carries each capture's request URL. Extend it or sit beside it. Do not build a second, competing record of where captures came from.

Do not fix any of the six instances here. Ticket 1095 landed the ClinicalTrials.gov one; the five NCI ones are tickets 1119, 1133, 1134, 1135 and 1137. Each has its own record. This ticket makes them fail; the others make them pass. If the check is landed first and the tree is red, that is the expected order, and the check may be landed with the known-failing groups declared as exceptions that name their owning ticket.

Do not change any converter's behavior, and do not change a capture's contents to make the check pass.

A new Rust test runs under the existing `make test` lane and needs no path opening. If the design finds that the check cannot run from the ladder without touching `sdlc/scripts/`, stop and say so rather than working around it; the ticket needs an `opens:` line and the gate reads that from origin/main.

## History

Proposed by the BioMCP lead on 2026-09-03 after the NCI field-name audit found five instances in one pass, and approved as a ticket in its own right rather than as an amendment to 1126. The class check that ticket 1132 carried in prose lives here instead; 1132 was superseded for bundling.
