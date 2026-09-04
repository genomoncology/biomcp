---
flow: build
priority: 8
---

# No fixture may carry a key the provider does not send

Ticket 1095 fixed one instance of this. Three structs in `src/sources/clinicaltrials.rs` carried `rename_all = "camelCase"` while the provider names the key `type`, so `intervention_type`, `arm_type` and `reference_type` never deserialized. The reason nothing caught it for months is that the test fixture was hand-written to match the struct rather than the provider. It supplied `armGroupType`, a key ClinicalTrials.gov does not send, so the suite passed while three fields were dead.

`armGroupType` no longer appears anywhere in the tree. The instance is gone. The class is not.

Nothing today stops the next hand-written fixture from inventing a key. A fixture is written to make a test pass, and a fixture written from the struct always makes the test pass, whether or not the struct is right. The failure is silent, it survives review, and it is only found by someone reading the provider's own response beside the code.

## Required behavior

A fixture cannot attest to a key the provider does not send, and this is checked mechanically rather than remembered.

Every key in a source fixture is either present in a recorded capture from that provider endpoint, or is declared as an exception with a stated reason. A key that is neither fails the check and names the fixture, the key, and the endpoint it claims to come from.

The check runs in the gate ladder, so it fails a build rather than producing a report someone has to read.

## Done, observably

- A fixture that introduces a key absent from every recorded capture for its endpoint fails the check, and the message names the fixture, the key and the endpoint.
- Reintroducing `armGroupType` into the arms fixture in `src/transform/trial.rs` fails the check. A test pins that.
- The current tree passes with no new exceptions. If it does not, each failure is either a real defect of the same class or an exception with a written reason, and the ticket says which for every one.
- An authored fixture, one that cannot be recorded because the payload would carry patient-bearing content, is declared as such and passes without being exempted from the key rule. BioData reports that cases 12 and 13 will be authored for exactly this reason, so this path is exercised, not hypothetical.
- The check runs from the gate ladder and fails the build.

## Where this comes from

This guard exists to hold case 17 of the clinical-trial conformance cases, recorded in the sibling BioData repository. **An attempt cannot read that repository**, so the behavior is restated here and this copy is authoritative for this ticket.

Correct behavior: All three read the key 'type'. Every fixture is recorded from the provider, never written to match the parser.

The assertion: Converting a recorded CTGov payload yields non-empty intervention type, arm type, and reference type. No fixture contains a key absent from the pinned provider contract, checked mechanically.

If that looks wrong, stop and say so rather than implementing something different.

## Boundary

`testdata/sources/capture-receipts.json` already classifies fixtures by provenance and is the natural neighbor for this. Extend it or sit beside it; do not build a second, competing record of where fixtures came from.

Do not change any fixture's contents to make the check pass. A fixture that fails is either a defect to file or an exception to justify.

Do not change the deserialization behavior fixed by 1095.

## The capture this check needs, recorded 2026-09-03

Held as a draft for two hours this morning on the belief that no recorded capture carried the arm and reference sections, and that another project had to supply one. Both halves were wrong, and the correction is worth reading before working this ticket.

The gap was self-inflicted. Every ClinicalTrials.gov capture in `testdata/sources/ctgov/` was recorded with a restricted `fields=` list, and not one of those lists ever asked for the arms module. The provider was never withholding anything. We were never asking.

ClinicalTrials.gov v2 is a public API with no key and no authentication. One request with no field restriction returns everything.

`testdata/sources/ctgov/get_nct02576665_full_20260903.json` is that request, recorded from `https://clinicaltrials.gov/api/v2/studies/NCT02576665?format=json` with no minimization. It carries all three keys this check exists to attest:

- `protocolSection.armsInterventionsModule.armGroups[0].type` = `EXPERIMENTAL`
- `protocolSection.armsInterventionsModule.interventions[0].type` = `BIOLOGICAL`
- `protocolSection.referencesModule.references[0].type` = `DERIVED`

And it does not carry `armGroupType` anywhere, which is the invented key that started this.

The receipt is in `testdata/sources/capture-receipts.json`, classified `real_and_receipted`. The record carries no central contact, no location contact, and no telephone or electronic address. The only named person is the overall official, a principal investigator published with name, affiliation and role, which is the same class of information as a paper byline.

So this check is provable today against a capture in this repository. Nothing about the ticket's requirement changes.

The related check running the other direction — no key list in the code may name a field absent from every recorded capture — is ticket 1138, and it is **not** independent of this one. Both checks read the same provenance record. This ticket lands first and builds it; 1138 depends on this ticket and extends what it leaves. Build the record so a second reader can use it, and say in the record what 1138 will need from it.
## The first real instance, and it is not an exception. Added 2026-09-03

The first attempt refused, correctly, on a contradiction this ticket created. The checker it built found `secondaryOutcomes` in `src/transform/trial/tests.rs` and no recorded capture under `testdata/sources` contains that key. The ticket forbids changing fixture bytes, requires the tree to pass, and requires this ticket to name every pre-existing failure. It named none, so the attempt had no authority to resolve the one it found. That is a defect in this ticket, not in the work.

**`secondaryOutcomes` is a real ClinicalTrials.gov v2 key. It is not an invented one and it must not be declared an exception.**

It sits at `protocolSection.outcomesModule.secondaryOutcomes`, a sibling of `primaryOutcomes`. It is absent from `get_nct02576665_full_20260903.json` because NCT02576665 states no secondary outcomes. A trial without one omits the key. A capture of one trial cannot prove a provider never sends a field.

The evidence is a recorded, receipted, unrestricted ClinicalTrials.gov v2 response for **NCT00791778** held in the sibling BioData repository. **An attempt cannot read that repository**, so the finding is restated here and this copy is authoritative for this ticket: that payload carries both `protocolSection.outcomesModule.primaryOutcomes` and `protocolSection.outcomesModule.secondaryOutcomes`. ClinicalTrials.gov v2 needs no key and no authentication, so the same request can be made from this repository.

### What this changes

Record an unrestricted capture of `https://clinicaltrials.gov/api/v2/studies/NCT00791778?format=json` alongside the NCT02576665 capture, with a receipt, and `secondaryOutcomes` becomes attested by a recorded capture like every other key. No exception is written and no fixture byte changes.

Treat that as the pattern rather than a one-off. When the checker names a key, the first question is whether the provider sends it and this repository never captured a record that has one. Only a key the provider genuinely does not send is a defect or an exception.

### The rule this ticket states is too strong, and this is the correction

"Present in a recorded capture" is the right evidence for admitting a key. It is not evidence for rejecting one, because a capture covers the trials it captured and nothing else. A key the checker cannot attest is a prompt to look, not a verdict.

So the check still fails the build on an unattested key, and the resolution order is: record a capture that carries it, or file it as a defect, or declare an exception with a written reason. Only the last two need this ticket's permission, and both are now granted for any key the attempt finds, provided the attempt states which it chose and why. This ticket no longer requires that it enumerate failures in advance, because it could not have known them.

## The first attempt refused, correctly, and here is what it found

Attempt 1 refused in `03-code` on 2026-09-03 at 23:47. The reason: `src/transform/trial/tests.rs` carries the key `secondaryOutcomes`, and no file under `testdata/sources` attested it. The attempt would not weaken the rule and would not edit fixture bytes, so it stopped. That is the right call and the check behaved as designed on its first real test.

The finding was a capture gap, not a defect. `secondaryOutcomes` is a genuine ClinicalTrials.gov key. Five of five melanoma studies sampled from the live API on 2026-09-03 carry it. The capture recorded earlier that day, `get_nct02576665_full_20260903.json`, is a trial that has no secondary outcomes, so its `outcomesModule` holds only `primaryOutcomes`.

**One trial cannot attest a provider's optional keys.** The attested set is the union across captures, never one capture's key set.

`testdata/sources/ctgov/get_nct06131398_full_20260903.json` is now recorded to close that gap. Full record, 13 modules, `secondaryOutcomes` present, receipted, no redaction needed because the record carries no email address and no telephone number.

## The nine keys still unattested, and what is known about each

A scan on 2026-09-03 compared every key in `src/transform/trial/tests.rs` against the union of all `testdata/sources/ctgov` captures. Nine keys remain unattested. They are not one problem and must not be resolved as one.

**Keys that are NCI-shaped, not ClinicalTrials.gov.** `diseases` and `phaseCode`. The test file holds fixtures for both providers. NCI does send these. The scan called them dead only because it compared them against the wrong endpoint's captures. This is direct evidence for a requirement the ticket already carries: the check is endpoint-aware, and a fixture declares which endpoint it claims to come from. A check that unions all captures regardless of provider gives wrong answers in both directions.

**Keys where the fixture and the provider disagree on the name.** `startDate`, `completionDate` and `status`. ClinicalTrials.gov sends `startDateStruct`, `completionDateStruct` and `overallStatus` inside `statusModule`, confirmed in both full captures. If these fixture keys claim to be ClinicalTrials.gov, they are the defect class this ticket exists to catch, and they are the fourth, fifth and sixth instances. Determine which endpoint each fixture claims before ruling.

**Keys that need a capture this repository does not have.** `contacts`, `email` and `phone`. These are real ClinicalTrials.gov location-contact keys. Neither recorded capture carries them, because neither trial publishes site contacts.

Recording one is deliberately not done, and this is a decision rather than an oversight. A capture that attests these keys carries working email addresses and telephone numbers for named people, and this repository is public. Do not record one to make the check pass. Declare these three as exceptions with that reason written down. The reason is durable and belongs in the provenance record beside the key.

**The key that is the seventh instance.** `centralContacts`. Ticket 1138 records that ClinicalTrials.gov never sends it: across 9,230 locations in 1,600 live studies the location key set is exactly `city`, `contacts`, `country`, `facility`, `geoPoint`, `state`, `status` and `zip`. If a fixture supplies `centralContacts` it is attesting a key the provider does not send, which is precisely this ticket's defect. Do not except it.

## What this changes about the work

Nothing in the requirement. The rule stands as written and none of the above weakens it.

Two things are now settled that the first attempt had to discover: the attested set is a union across captures, and the check must know which endpoint a fixture claims. Both were implicit before and both are now stated.

The ruling on `contacts`, `email` and `phone` is the one judgment the ticket should not make on its own. It is made here: they are exceptions, the reason is that attesting them would publish real people's contact details from a public repository, and that reason is written into the provenance record rather than left as a bare exemption.

## Amendment, 2026-09-03: attest against the provider's schema, not a union of samples

The nine unattested keys listed above were worked out by unioning key names across recorded captures. That method has now given two wrong answers in one evening, and it should not be what this check rests on.

ClinicalTrials.gov publishes its own field schema at `https://clinicaltrials.gov/api/v2/studies/metadata`. Public, no key, 278 documented fields with their nesting and types. It is recorded as `testdata/sources/ctgov/field_metadata_20260903.json` with a receipt.

It answers the nine directly. `secondaryOutcomes`, `contacts`, `email`, `phone`, `startDateStruct`, `overallStatus` and `centralContacts` are all documented. `armGroupType`, the invented key that started this class, is absent from the schema entirely.

That last line is the point. A union of samples proves a key is present and never proves a key is absent, which is the question this check asks. The schema answers the question that was actually asked.

**Three consequences for the work.**

The three keys ruled exceptions above — `contacts`, `email`, `phone` — need no exception. The schema documents all three, so they are attested without recording anybody's telephone number. The reasoning behind that ruling still stands wherever no schema exists: do not record a capture carrying real people's contact details in order to satisfy a check.

`startDate`, `completionDate` and `status` are still candidate defects. The schema documents `startDateStruct`, `completionDateStruct` and `overallStatus`. The bare forms are absent. Rule on each once the fixture's claimed endpoint is known.

Attestation is by path, not by bare name. Ticket 1138's correction of the same date carries the argument in full and it applies identically here.

**Where the schema does not exist.** NCI publishes no equivalent, so NCI keeps the capture path with `get_nci_2023_04529_full_20260903.json` as its evidence. The provenance record says which source attests each endpoint. Two mechanisms, one record.
