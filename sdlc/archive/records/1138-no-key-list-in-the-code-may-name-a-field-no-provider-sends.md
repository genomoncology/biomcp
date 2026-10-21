---
flow: build
priority: 8
---

# Provider-key reads must be attested at their paths or removed

Tickets 1126, 1132, and 1136 are complete. Ticket 1126 now checks provider-
shaped trial fixtures against a receipted ClinicalTrials.gov schema and an
unrestricted NCI trial capture. Ticket 1132 repaired five NCI mappings, and
1136 repaired structured NCI eligibility text. This ticket extends the same
manifest and checker in the opposite direction: selected code that reads a
provider key must be checked against that evidence too.

The check is deliberately narrower than the title of the original proposal.
It covers the ClinicalTrials.gov serde model rooted at `CtGovStudy` and NCI
top-level trial-record reads in three named conversion functions. It does not
claim to audit every JSON key in the repository.

## Defects and evidence

Ticket 1095 fixed three ClinicalTrials.gov serde names that did not exist in
the provider schema: `interventionType`, `armGroupType`, and `referenceType`.
The correct key at each path is `type`.

One equivalent CTGov defect remains. `CtGovLocation.central_contacts` models
`protocolSection.contactsLocationsModule.locations[].centralContacts`, which
the recorded provider schema does not contain. The real `centralContacts` is a
sibling of `locations` on `CtGovContactsLocationsModule`, and that module-level
field is correct. The location field feeds two unreachable fallbacks in
`extract_locations` and `extract_contacts`; removing them changes no output.

Ticket 1132 removed the five broken NCI mapping groups that motivated this
guard, but 16 individually unattested legacy alternatives remain in
`from_nci_hit` and `from_nci_trial`:

| Group | Attested top-level name | Unattested alternatives to remove |
| --- | --- | --- |
| trial ID | `nct_id` | `nctId`, `nctID` |
| title | `brief_title` | `briefTitle`, `title` |
| status | `current_trial_status` | `status`, `overallStatus` |
| phase | `phase` | `phase_code`, `phaseCode` |
| sponsor | `lead_org` | `lead_organization`, `leadSponsor`, `sponsor` |
| conditions | `diseases` | `conditions` |
| start date | `start_date` | `startDate` |
| completion date | `completion_date` | `completionDate` |
| summary | `brief_summary` | `briefSummary`, `summary` |

These converter functions are crate-private in practice and are called only
with NCI provider records. No public or provider compatibility evidence was
found for the alternatives. Remove all 16 rather than turning the original
defect class into permanent exceptions. Rewrite the three synthetic tests that
currently use those spellings to use the attested snake_case provider names.
Preserve the status-value casing coverage in
`trial_status_normalization_variants`; only its invented keys change.

The 15 fixture exceptions shipped by ticket 1126 are occurrences of these
aliases in those three tests, not 15 distinct aliases. Once the fixtures use
attested names, delete all 15 entries and `exception_policy` from
`fixture_key_contract`. Amend the checker so an empty fixture exception set is
the only accepted closed set. Do not replace them with code-read exceptions.

## Required rule

Every alternative in a covered code-read group is judged independently. A
group containing one attested name does not bless its other names. Each read is
attested at its provider path; this ticket authorizes no code-read exceptions.
The group exists for diagnostics only.

An unattested read fails with a diagnostic naming the endpoint, source file and
function or root type, group/read site, and provider path. An unsupported or
new undeclared read form inside the covered boundary fails closed rather than
silently escaping discovery.

Attestation means the evidence supports that exact path. It does not mean one
sample proves a provider can never publish another optional field.

## One manifest and one gate

Extend `tools/check-source-capture-receipts.py`, its existing tests in
`tests/test_capture_receipts.py`, and
`testdata/sources/capture-receipts.json`. Add `code_key_contract` beside the
shipped `fixture_key_contract`; do not create another provenance registry or a
Rust implementation.

`code_key_contract` declares:

- one CTGov source/root: `src/sources/clinicaltrials.rs`, `CtGovStudy`;
- three NCI source/function/root triples:
  `src/transform/trial.rs:from_nci_hit(hit)`,
  `src/transform/trial.rs:from_nci_trial(trial)`, and
  `src/entities/trial/get.rs:nci_eligibility_text(trial)`;
- the two exact CTGov supplemental paths and their evidence described below;
- an empty, closed code-read exception set.

That boundary is itself closed: the checker requires exactly those four
source/root declarations and rejects a missing, altered, duplicate, or extra
entry. A declaration whose source file, function, root parameter, or root type
does not exist also fails; selecting zero reads is never success.

The checker computes group identities from the function, read helper, and
stable occurrence order, and reports the current source line for people. The
manifest supplies endpoint, source boundary, and root rather than duplicating
the discovered key lists.

The existing `bin/lint` invocation owns enforcement, so this runs under
`make lint`. There is no new Rust test, no `sdlc/scripts/` change, and no new
gate.

## ClinicalTrials.gov discovery

Starting from `CtGovStudy`, recursively discover the serde fields and their
paths through the local struct graph in `src/sources/clinicaltrials.rs`. Honor
the forms used by that graph: struct-level `rename_all = "camelCase"`, explicit
field `serde(rename = "...")`, path-neutral `serde(default)`, and nesting
through `Option<T>` and `Vec<T>`. Emit every field at its own path without an
array marker. Only when traversing from a `Vec<T>` or `Option<Vec<T>>` field to
a child, append `[]` between that field and child: `centralContacts` is the
array field and `centralContacts[].name` is its child. Compare those full paths
to the existing `ctgov_schema` attestor. Other serde attributes, aliases,
flattening, unsupported generic/container shapes, or unresolved local field
types in this covered graph fail closed with the source field and reason.

The graph currently emits 107 provider paths. The recorded CTGov metadata
treats `geoPoint` as an opaque `GeoPoint` leaf and does not enumerate `lat` and
`lon`, although the provider sends both. Add exactly these supplemental
attestations to `code_key_contract`:

- `protocolSection.contactsLocationsModule.locations[].geoPoint.lat`
- `protocolSection.contactsLocationsModule.locations[].geoPoint.lon`

Each entry must name the opaque-schema limitation and the supporting receipted
full record `ctgov/get_nct06131398_full_20260903.json`, which contains the path.
The checker verifies the evidence path exists in that receipt-backed record and
rejects extra, duplicate, altered, or unused supplemental entries. These are
positive attestations, not exceptions. Do not globally permit descendants of a
schema leaf.

The guard must reject `interventionType`, `armGroupType`, and `referenceType`
when each is reintroduced through the corresponding serde field, and must
reject location-level `centralContacts` while accepting the same name at the
module-level path.

## NCI discovery and evidence limit

Within the three declared functions, discover every top-level access on the
named root parameter. The only supported forms are:

- `json_get_string(root, &[literal, ...])`;
- `nci_conditions(root, &[literal, ...], ...)`; and
- direct `root.get("literal")`, including the root segment of a chained read.

Check every candidate separately against the top-level keys in the existing
`nci_top_level_capture` attestor. Inventory all root-record access forms in the
declared functions; `root[...]`, `.pointer(...)`, passing the root to an
unregistered helper, aliasing the root, a computed key, a nonliteral candidate
list, or any other unproved root access fails closed. Calls on derived nested
receivers are outside top-level attestation.

That limitation is intentional and must appear in the completion record. NCI
publishes no schema, and one unrestricted trial proves that its recursively
present paths are real but cannot prove absent optional nested paths invalid.
This ticket therefore guards NCI top-level reads only. It may prove the roots
`eligibility` and `arms`; it must not claim recursive proof for
`eligibility.structured.min_age`, `arms[].interventions[].name`, or the ticket
1136 eligibility-entry fields. Do not change ticket 1126's top-level attestor
or its recorded limitation.

The attestor is an unrestricted `/trials?size=1` search record, while the NCI
detail client calls `/trials/{nct_id}` and expects a raw record. Both conversion
paths consume the same trial-record shape, but this evidence does not attest
transport-envelope equivalence. Preserve that provenance caveat.

Request projection lists, query-parameter keys, `NciSearchResponse` transport
envelope aliases, non-trial providers, and JSON reads outside the three named
functions are excluded.

Both source scanners are lexical, not raw regular-expression searches. They
must correctly ignore line/block comments and string, raw-string, byte-string,
and character literals when finding covered declarations and delimiters, and
must fail on malformed or unclosed covered constructs. A commented fake struct,
function, attribute, or read must neither create a declaration nor hide a real
one that follows it.

The first code review exposed three concrete consequences of that rule. Prefix
boundaries used to associate serde attributes with a struct must be derived
from the aligned comment/literal-masked source; a raw `rfind("}")` or
`rfind(";")` lets those characters inside a block comment hide a real
unsupported attribute. Comments within supported NCI calls are whitespace:
`root.get(/* comment */ "key")` remains a literal read, while a quoted token
inside a comment in a helper candidate array is not a candidate. Root-position
bookkeeping also uses the aligned masked arguments: in
`json_get_string(/* root */ root, &[...])`, the comment occurrence must not be
marked as the live argument or cause the real root to look unsupported. Focused
mutations must pin all four cases.

## Authorized product cleanup

Besides removal of the 16 NCI aliases and corresponding test-fixture exception
cleanup, the only converter/model cleanup authorized is:

- delete `CtGovLocation.central_contacts` in
  `src/sources/clinicaltrials.rs`;
- delete `.or_else(|| loc.central_contacts.first())` in `extract_locations`;
- delete `.chain(loc.central_contacts.iter())` in `extract_contacts`.

Do not touch `CtGovContactsLocationsModule.central_contacts` or its module-
level conversion. Do not otherwise change provider conversion behavior or
capture bytes. Ticket 1122 overlaps this deletion and must treat the location-
level fallback as already removed when this ticket completes.

The guard cannot detect an omitted read such as hardcoding `why_stopped` to
`None`, and it cannot decide that one real key is semantically wrong for a
field, such as using `primary_purpose` as study type. Ticket 1132's behavior
tests hold those cases. Do not claim this structural guard catches them.

## Done, observably

- The current CTGov serde graph passes with exactly the two verified `geoPoint`
  supplemental attestations; the location-level dead field and fallbacks are
  gone, while module-level central contacts remain.
- All 16 unattested NCI alternatives are gone. The attested names and existing
  output behavior remain, and all 15 obsolete fixture exceptions are gone.
- Mutation tests prove: an unattested alias beside an attested NCI member fails;
  an entirely unattested NCI group fails; an unattested direct root `.get`
  fails; each of the three historical CTGov wrong names fails; and location-level
  `centralContacts` fails while the module-level path passes.
- Focused tests prove the root of a chained NCI read is checked without claiming
  its nested descendants, and unsupported root alias, index, pointer, computed
  key, and unregistered-helper forms fail closed.
- Focused tests prove an unauthorized, altered, duplicate, and unused
  supplemental attestation fails. Diagnostics carry endpoint, source/function
  or root, group/read site, and path.
- Focused tests prove each exact boundary declaration is mandatory: missing,
  altered, duplicate, extra, and unresolved declarations fail. Commented fake
  declarations/reads do not affect discovery or hide later real code, and
  malformed/unclosed covered constructs fail closed.
- A real unsupported serde attribute separated from its struct by a comment
  containing `}` or `;` still fails. Comments around a real NCI key are ignored,
  commented-out key literals do not become reads, and a root name in a comment
  before the actual helper argument does not disturb root discovery.
- `make lint`, `make test`, and `make spec` pass.

## History

Proposed 2026-09-03 after the NCI mapping audit found five silent field defects.
The first ticket text assumed sampled captures could prove absence and treated
the five repairs as separate future work. Ticket 1126 then recorded the CTGov
schema and NCI capture, ticket 1132 bundled and landed all five repairs, and
ticket 1136 added another structured NCI reader. The design was rewritten from
those shipped facts before implementation. Independent review also forced the
per-alternative rule, explicit retirement of legacy aliases, bounded discovery
syntax, honest NCI depth limit, and single Python lint owner recorded above.

## Completed 2026-09-03

The shared receipt checker now audits the exact four code boundaries described
above. It checks 107 CTGov serde paths and 17 NCI top-level reads, validates the
two receipt-backed `geoPoint` supplements, and reports zero exceptions. The 16
unattested NCI aliases and all 15 obsolete fixture exceptions are gone. The
dead location-level `central_contacts` model and fallbacks are gone; real
module-level central contacts remain unchanged.

Independent design review: ACCEPT after two ticket-amendment rounds.
Independent code review: ACCEPT after two remediation rounds. The review
findings about comment-delimiter attribute hiding, comment-wrapped keys, and
commented root-position bookkeeping are part of the permanent requirements and
mutation suite above.

The NCI proof remains intentionally top-level only. Its unrestricted search
capture attests the shared trial-record keys used by search/detail conversion,
not optional nested-key absence or detail transport-envelope equivalence.

Repository gates passed: `make lint`; `make test` (3,060 Rust tests, then 876
Python tests passed and 3 skipped, plus strict documentation build); and
`make spec` (all declared mustmatch and static contract batches passed). The
final focused receipt/code audit covered 234 files, 431 fixture paths, and 124
code reads with zero exceptions.
