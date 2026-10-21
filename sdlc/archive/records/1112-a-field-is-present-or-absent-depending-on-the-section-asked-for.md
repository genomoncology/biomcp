---
flow: build
priority: 6
---

# Keep CTGov intervention descriptions invariant across detail sections

## Outcome

A default ClinicalTrials.gov structured trial detail and the same detail
requested with `arms` expose the same source-supplied intervention name,
alternate names, type, and description. An absent description then means the
provider did not supply one, rather than that BioMCP did not request it.

## Current facts

`Trial.intervention_details` is part of the ordinary structured detail model,
not the optional `arms` field. `from_ctgov_study` fills each
`TrialIntervention` unconditionally from
`protocolSection.armsInterventionsModule.interventions`, including
`description`, and JSON/MCP serialization exposes every populated value.

`CTGOV_GET_FIELDS_BASE` currently requests `InterventionName`,
`InterventionOtherName`, and `InterventionType`, but not
`InterventionDescription`. `CTGOV_GET_FIELDS_ARMS` requests all four. Therefore
the default response omits each intervention description while `get trial
<id> arms` can populate it for the same record. Optional fields use
`skip_serializing_if = "Option::is_none"`, so the current structured symptom is
field omission rather than a JSON `null`.

The type half of the original report is no longer live: ticket 1095 added
`InterventionType` to the base field list and fixed its provider key mapping.
The source construction test already protects that field.

The receipted unrestricted CTGov capture
`testdata/sources/ctgov/get_nct02576665_full_20260903.json` supplies exact
names, types, alternate names, and non-empty descriptions for both Toca 511 and
Toca FC. The provider schema capture
`testdata/sources/ctgov/field_metadata_20260903.json` identifies the nested
provider key as `description` and its selectable field piece as
`InterventionDescription`. The older receipted restricted capture for the same
trial demonstrates the historical request artifact, but predates ticket 1095
and therefore cannot represent today's complete base request.

## Design

Add `InterventionDescription` to `CTGOV_GET_FIELDS_BASE`. Keep the existing
converter and public `TrialIntervention` shape: the owning defect is request
construction, and the converter already preserves a supplied description.

Write the failing request-contract test first. It must compare the default and
`arms` field plans and require all four inputs consumed by
`TrialIntervention`—name, alternate name, type, and description—on both paths.
This fails on `InterventionDescription` before the production change and also
prevents a later section-only regression.

Add deterministic provider-shaped coverage by decoding the unrestricted
receipted capture and asserting that conversion preserves the two recorded
intervention descriptions together with their corresponding names and types.
Serialize the converted trial and assert the descriptions are present in
`intervention_details`; this covers the structured CLI output reused by MCP.
Do not compare against the older restricted fixture as if its stale request
were the new default contract.

## Acceptance

- The default and `arms` CTGov request plans both include exactly one each of
  `InterventionName`, `InterventionOtherName`, `InterventionType`, and
  `InterventionDescription`.
- Conversion of the unrestricted receipted NCT02576665 record preserves the
  recorded intervention name/type/description associations in
  `intervention_details`, including serialized JSON.
- The existing intervention-type, arm, reference, and fixture-key contract
  tests continue to pass.
- `make lint`, `make test`, and `make spec` pass.

## Scope and boundaries

This is a CTGov detail-request correction. Do not change trial search fields,
the NCI path, section parsing, the `arms` model or renderer, intervention
ordering/bounds, the public JSON shape, or Markdown presentation. No MCP schema
change is needed because MCP executes the same typed CLI get path and returns
the same JSON entity.

The existing receipted full capture and provider schema already attest the
field and key shape. Do not recapture data or modify
`testdata/sources/capture-receipts.json`; the fixture-key checker does not
require a new declaration merely because a test consumes a receipted file from
`testdata/sources/ctgov/`.

Dependencies: none. Ticket 1095 is already landed; this ticket preserves that
behavior without reopening it.

## Review

- Design review: ACCEPT (2026-09-04) — the request-layer fix is bounded, the
  receipted capture supports the conversion contract, and exact-once field-plan
  parity proves the section invariant.
- Code review: REJECT (2026-09-04) — the first implementation did not assert
  the complete alternate-name vectors and serialized associations for both
  receipted interventions. Remediated with complete typed-vector assertions and
  an exact serialized `intervention_details` array; independent re-review then
  ACCEPTED the implementation on 2026-09-04.

## Completed 2026-09-04

Added `InterventionDescription` to the CTGov base detail projection. Default and
`arms` requests now include exactly one each of the intervention name,
alternate-name, type, and description fields. The existing converter and public
model remain unchanged; a provider omission still serializes as field absence,
while source-supplied descriptions now survive the default path.

Receipt-backed tests preserve both NCT02576665 interventions as exact ordered
name/type/description/alternate-name associations through typed conversion and
serialized JSON. Independent design review accepted the corrected description-
only scope after confirming ticket 1095 had already fixed type. Fresh code
review rejected incomplete alternate-name assertions; after the tests pinned
both complete typed vectors and the exact serialized array, independent
re-review accepted the implementation.

Final gates passed on the reviewed tree: `make lint`; `make test`, including the
complete Rust lane, 877 Python tests passed (3 skipped), and the strict
documentation build; and `make spec`, including its static lane.
