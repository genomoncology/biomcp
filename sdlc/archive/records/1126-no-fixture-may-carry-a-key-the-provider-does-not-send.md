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

Every key in a provider-shaped source fixture is attested to the level the
provider evidence can support, or is declared as an exception with a stated
reason. ClinicalTrials.gov uses its recorded field schema and is checked
recursively at full paths. NCI publishes no equivalent schema, so its receipted
unrestricted capture supports top-level-key checking only; nested NCI paths are
an explicit limitation, not claimed proof. An unattested CTGov path or NCI
top-level key fails the check and names the fixture, checked path, and endpoint.

The check runs in the gate ladder, so it fails a build rather than producing a report someone has to read.

## Done, observably

- A CTGov fixture that introduces an unattested key or puts an attested key at
  an unattested full path fails the check. An NCI fixture with an unattested
  top-level key fails. Each message names the fixture, checked path, and
  endpoint; nested NCI paths are not claimed by this ticket.
- Reintroducing `armGroupType` into the arms fixture in
  `src/transform/trial/tests.rs` fails the check. A test pins that.
- Moving documented `centralContacts` from
  `protocolSection.contactsLocationsModule.centralContacts` to
  `protocolSection.contactsLocationsModule.locations[].centralContacts` also
  fails. This pins path-aware behavior that a flat key-name check cannot satisfy.
- A new provider-shaped trial fixture that is not declared in the fixture-key
  inventory fails closed and names the undeclared fixture. Registration is not
  an opt-in escape hatch.
- The current tree passes with only the narrow compatibility exceptions
  enumerated in the final ruling below. Any additional failure is either a real
  defect of the same class or requires a separately authorized exception with a
  written reason.
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

**Historical finding, superseded by the schema amendment below.** One trial
cannot attest all optional keys. The temporary union-of-captures approach closed
this instance, but it is not the final attestation rule for ClinicalTrials.gov.

`testdata/sources/ctgov/get_nct06131398_full_20260903.json` is now recorded to close that gap. Full record, 13 modules, `secondaryOutcomes` present, receipted, no redaction needed because the record carries no email address and no telephone number.

## The nine keys still unattested, and what is known about each

A scan on 2026-09-03 compared every key in `src/transform/trial/tests.rs` against the union of all `testdata/sources/ctgov` captures. Nine keys remain unattested. They are not one problem and must not be resolved as one.

**Keys that are NCI-shaped, not ClinicalTrials.gov.** `diseases` is an NCI key;
`phaseCode` is a legacy compatibility alias and is not an NCI wire key. The test
file holds fixtures for both providers. The scan compared both against the wrong
endpoint and therefore could not rule on either. This is direct evidence for a
requirement the ticket already carries: the check is endpoint-aware, and a
fixture declares which endpoint it claims to come from. A check that unions all
captures regardless of provider gives wrong answers in both directions.

**Historical unresolved findings, settled below.** The first scan could not tell
whether `startDate`, `completionDate`, `status`, `contacts`, `email`, and `phone`
were invalid because it had neither endpoint assignments nor schema evidence.
The schema amendment and final ruling below settle them; this paragraph carries
no implementation instruction.

**Historical finding, corrected by the schema amendment below.**
`centralContacts` is real at module level and invalid inside a location. Its full
path, not its bare name, determines whether it is attested.

## What this changes about the work

Nothing in the requirement. The rule stands as written and none of the above weakens it.

**Historical rulings, superseded by the schema amendment below.** The first
attempt established that fixtures must be endpoint-aware, but its
union-of-captures attestation rule was incomplete. Its proposed exceptions for
`contacts`, `email`, and `phone` are also withdrawn: the recorded CTGov schema
attests those paths without publishing anyone's contact values.

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

## Final implementation ruling, 2026-09-04

This section resolves the remaining ambiguity and governs over contradictory
historical findings above.

### Covered fixture boundary and fail-closed discovery

The existing `testdata/sources/capture-receipts.json` remains the one provenance
and fixture-key inventory. Extend the existing receipt checker; do not add a
parallel registry.

For this ticket, the mechanically complete provider-fixture boundary is the
trial records actually fed into the conversion layer, not transport envelopes:

- every declared trial-record selector inside JSON documents under
  `testdata/sources/clinicaltrials/` and `testdata/sources/nci_cts/`; and
- every inline `json!` provider object passed into `from_ctgov_study`,
  `from_nci_trial`, or `from_nci_hit` in `src/transform/trial/tests.rs` and
  `src/transform/trial/tests/*.rs`.

Each on-disk contract names a JSON pointer or array selector for the trial
record it checks. For example, the `studies`, `nextPageToken`, and `totalCount`
members of a ClinicalTrials.gov search response are transport-envelope fields,
not study fields in the provider's field schema; the contract selects each
`studies[]` record and does not pretend the study schema attests the envelope.
Likewise, an NCI search contract selects each `data[]` trial record. Receipt
classification and whole-file inventory remain enforced separately by the
existing audit.

The checker must discover the inline boundary from converter uses and compare
it with declarations in the inventory. For the two named on-disk directories,
every file that is consumed as a trial fixture must have at least one declared
record selector; a newly added consumed file or inline provider-shaped fixture
without an endpoint/selector declaration fails closed. Tests pin a declared
fixture, an undeclared inline fixture, and an undeclared consumed file. It is
insufficient to walk only selectors somebody chose to register. Other entities,
transport-envelope keys outside a selector, and arbitrary `json!` unit values
outside this boundary are excluded.

ClinicalTrials.gov records are checked recursively by full path against the
recorded provider schema. NCI has no schema, and one unrestricted captured trial
cannot prove that an optional nested path is invalid. Therefore this ticket
checks every selected NCI trial's **top-level keys** against the top-level union
from the receipted unrestricted capture. Nested NCI objects are explicitly out
of scope until stronger receipted evidence exists; do not manufacture dozens of
exceptions for optional nested fields in `search_melanoma.json`. This limitation
still catches the NCI-shaped aliases that motivated the shared machinery and is
recorded in both the inventory and completion record so it cannot be mistaken
for recursive proof.

An `authored` fixture is still checked against provider paths; that
classification explains why values were written rather than recorded and is
not an exemption from key attestation.

### Existing compatibility inputs are explicit exceptions

Three current inline tests deliberately exercise legacy compatibility aliases
rather than claiming to reproduce an NCI payload. Keep their bytes unchanged
and record narrow selector-and-path exceptions in the shared inventory:

- `from_nci_trial_maps_supported_alias_fields`: top-level `nctId`,
  `briefTitle`, `overallStatus`, `phaseCode`, `leadSponsor`, `startDate`,
  `completionDate`, and `briefSummary`;
- `trial_sections_maps_supported_nci_fields`: top-level `phase_code`; and
- `trial_status_normalization_variants`: top-level `nctId`, `briefTitle`,
  `status`, and `overallStatus` for both inline objects.

The written reason is: these are synthetic unit inputs that pin accepted legacy
aliases and do not attest the NCI wire contract. Ticket 1138 owns the code-side
decision about dead key aliases; these fixture exceptions neither attest those
aliases nor authorize new ones. No other exception is pre-authorized. Report
the exact exceptions in the completion record, so 1138 can remove any that its
code cleanup makes obsolete.

### Acceptance additions

Focused tests must prove all four directions: a schema-attested correct path
passes; `armGroupType` fails as an unknown key; documented `centralContacts`
fails at the location path; and an undeclared inline provider fixture fails
closed. Diagnostics name the fixture, full path, and endpoint. The repository
passes with only the compatibility exceptions enumerated above.

## Implementation findings that now belong to the contract

The first reviewed implementation exposed three ways a superficially passing
checker could leave the class open. They are requirements, not incidental code
review notes.

- Fixture discovery must either understand comments and the supported Rust
  `json!` construction forms or fail closed with an actionable diagnostic.
  Delimiter-looking text inside `//` or block comments must not hide a later
  converter fixture. A dynamic or otherwise unsupported fixture-file reference
  must be rejected as unsupported; it must not disappear from the consumed-file
  inventory. This ticket does not authorize adding a Rust parser dependency.
- Every on-disk selector must resolve to one or more JSON objects representing
  trial records. A selector that resolves to a scalar, list container, or no
  records fails. Selecting a scalar and therefore checking zero paths is not a
  successful audit.
- The compatibility exception set is closed. The checker enforces the exact
  selector/path entries above and the prescribed legacy-alias rationale; a new
  exception, an altered reason that loses that rationale, or an unused exception
  fails the gate. A passing count of 15 alone is insufficient proof.

Focused regression tests cover comment-delimiter hiding, unsupported dynamic
fixture references, scalar selectors, altered exception rationale, extra used
exceptions, and unused exceptions. These tests join the four acceptance
directions above before code review may accept the ticket.

The remediation review added three more concrete fail-closed cases:

- an inline provider object assigned or reassigned to a local variable before a
  converter call is still discovered; supported assignment flow must not be
  limited to a `json!` inside the variable's original `let` initializer;
- a literal fixture path assembled with `include_str!(concat!(...))`, including
  split directory components, is discovered, while a genuinely dynamic path is
  rejected as unsupported; and
- duplicate on-disk selector declarations fail instead of checking and counting
  the same fixture twice.

Focused tests pin reassigned inline objects, split-literal `concat!` paths, and
duplicate on-disk declarations. These are part of acceptance, not optional
parser hardening.

The next remediation review found two more ways a hand-rolled Rust scanner can
silently attest the wrong object:

- assignment discovery distinguishes `=` from equality and comparison
  operators. A later `record == json!({})` expression must not replace the
  provider object assigned to `record`; and
- a converter argument outside the explicitly supported direct-`json!` or
  simple-local flow, such as `from_nci_trial(&wrapper["study"])`, fails closed
  with the converter and source location. Unsupported argument expressions must
  never be skipped.

Focused regressions prove an equality expression cannot redirect discovery and
an indexed converter argument is rejected. These join the required discovery
tests above.

Final-gate execution exposed a repository-harness requirement. `bin/lint` is
also exercised in reduced test repositories that intentionally contain neither
BioMCP's receipt checker nor its source-capture manifest. Gate integration must
therefore follow the existing optional-check convention: run the fixture-key
audit when both `tools/check-source-capture-receipts.py` and
`testdata/sources/capture-receipts.json` are present; skip with an explicit
message when both are absent; and fail closed when exactly one is present. A
focused lint-contract test pins the partial-configuration failure. This does not
make the BioMCP gate optional, because the real repository tracks both files.

## Completed 2026-09-04

The existing source-capture receipt audit now checks provider-shaped clinical
trial fixtures as part of `make lint`. ClinicalTrials.gov fixtures are attested
recursively against the recorded provider schema; NCI fixtures are checked at
top level against the receipted unrestricted capture. On-disk selectors and
inline converter fixtures are inventoried fail-closed, authored fixtures remain
checked, and the exact 15 legacy NCI alias exceptions are narrow, required, and
linked to ticket 1138.

Design review: ACCEPT after the ticket defined path-aware evidence, a complete
fixture boundary, explicit compatibility exceptions, and the NCI evidence
limit. Code review: ACCEPT after adversarial remediation of comment and
delimiter handling, dynamic and split fixture paths, assignment and comparison
flow, unsupported converter arguments, selector types, duplicate declarations,
exception policy, and reduced-repository lint behavior.

Verification on the final tree:

- focused lint and receipt contracts: 54 passed;
- complete `make lint`: passed;
- complete `make test`: 846 passed, 3 skipped, plus strict documentation build;
- complete `make spec`: passed.

Honest limitation: NCI publishes no field schema, so this record proves only
top-level NCI trial keys. Nested NCI paths remain outside this guard until
stronger receipted evidence exists. The inventory records that limitation so
ticket 1138 cannot accidentally claim recursive coverage.
