---
flow: build
priority: 5
---

# Preserve ClinicalTrials.gov postal codes in trial locations

## Outcome

A ClinicalTrials.gov location returned by `get trial <id> locations` preserves
the provider's postal code in structured JSON and in the Markdown locations
table. An absent postal code remains absent rather than becoming an empty or
invented value.

## Current facts and evidence

The defect is live on the current tree. `CTGOV_GET_FIELDS_LOCATIONS` in
`src/sources/clinicaltrials.rs` requests `LocationZip` for `locations`; the
`all` projection incorporates the same list. `CtGovLocation.zip` successfully
deserializes the provider's nested `zip` key. However,
`extract_locations` in `src/transform/trial.rs` does not copy that value, and
the public `TrialLocation` in `src/entities/trial/mod.rs` has no field for it.
The value is therefore lost before JSON serialization and Markdown rendering.

The original claim that BioMCP pays for this field on every request was too
broad. It is requested on CTGov detail requests containing `locations` or
`all`, not on default, contacts-only, or search requests.

The existing receipted capture
`testdata/sources/ctgov/get_nct02576665_20260811.json` is sufficient proof and
needs no replacement. Its recorded request includes `LocationZip`, and its
first US location is Sarah Cannon Research Institute in Denver, Colorado with
`"zip": "80218"`. The capture also contains two other US postal codes. The
provider-schema capture already attests
`protocolSection.contactsLocationsModule.locations[].zip`.

Tickets 1126 and 1138 are complete. Their fixture/code-key ratchets verify that
provider-shaped keys and source reads are attested; they do not prove that
every requested field reaches the public entity. Do not expand this ticket
into that repository-wide request-to-output checker. This ticket pins the
known postal-code loss directly.

## Design

Add `postal_code: Option<String>` to `TrialLocation`, with the same
`skip_serializing_if = "Option::is_none"` behavior as other optional location
fields. Use the provider-neutral public name `postal_code`; keep `zip` as the
ClinicalTrials.gov source-model name. In `extract_locations`, populate it with
`clean_opt(loc.zip.as_deref())` so blank provider strings are not published.

Adding the field requires mechanical `postal_code: None` updates to the four
current hand-built `TrialLocation` values in
`src/render/markdown/root_tests.rs`, `src/render/markdown/trial/tests.rs`, and
`src/cli/trial/tests_locations.rs`. Those are compilation updates, not license
to change the behavior those tests cover.

Render postal code as its own `Postal code` column in
`templates/trial.md.j2`, between `City` and `Country`, using `-` when absent.
Do not concatenate it into the city or state value: JSON and Markdown should
both expose the same distinct fact, including non-US postal-code formats.

Add a focused receipt-backed conversion test in
`src/transform/trial/tests/ticket_1114.rs` and register that test module from
`src/transform/trial/tests.rs`. Decode the existing NCT02576665 capture through
`ClinicalTrialsClient::decode_get_response`, convert it with
`from_ctgov_study`, and locate the Sarah Cannon site by facility rather than by
list index. Assert its typed `postal_code` is `80218`, serialized JSON contains
`locations[].postal_code`, and locations Markdown contains the same value.
This is the red-before-green test: it fails at the public conversion boundary
before the model and converter change.

In the same test module, add a small provider-shaped conversion case with one
postal code padded by whitespace and one blank postal code. Assert the padded
value is trimmed, while the blank value becomes `None` and its serialized
location has no `postal_code` key. Update the existing location-rendering test
in `src/render/markdown/trial/tests.rs` to assert the full row for a
`postal_code: None` location, including `-` in the new column. These assertions
are required acceptance coverage, not merely struct-literal compilation
updates.

In `src/sources/clinicaltrials/tests/construction.rs`, pin the already-correct
request boundary: `LocationZip` occurs exactly once for `locations` and `all`,
and is absent from default and contacts-only field projections. This prevents
the conversion test from passing only because an unrestricted fixture carries
a field production requests no longer fetch.

Update the existing location assertions in `spec/entity/trial.md` to expect the
new Markdown column and add a deterministic JSON assertion that the fixture
route for NCT02576665 reports `80218` as the Sarah Cannon location's
`postal_code`. The existing CTGov spec fixture already serves the receipted
capture, so no live request or fixture-runner change is needed.

## Acceptance

- `TrialLocation` exposes optional JSON field `postal_code`; it is omitted when
  the provider value is absent or blank and preserves the provider's content
  after surrounding-whitespace cleanup.
- The receipted NCT02576665 conversion maps Sarah Cannon Research Institute's
  source `zip` to typed and serialized `postal_code == "80218"`.
- The Markdown locations table has a separate `Postal code` column and renders
  `80218` for that same site; a missing postal code renders `-`.
- CTGov detail field-plan tests prove `LocationZip` appears exactly once for
  `locations` and `all`, and not for default or contacts-only requests.
- The deterministic public CLI contract in `spec/entity/trial.md` proves both
  Markdown and JSON surfaces without network access.
- Focused Rust tests pass, followed by `make lint`, `make test`, and `make spec`.

## Scope, ownership, and dependencies

The owning path is the shared trial entity and CTGov conversion/rendering
boundary: `src/entities/trial/mod.rs::TrialLocation`,
`src/transform/trial.rs::extract_locations`, and `templates/trial.md.j2`.
JSON needs no separate renderer change because serde serializes
`TrialLocation`; MCP needs no schema change because it reuses the typed CLI
JSON result.

Do not alter CTGov search fields, NCI conversion, location inclusion or
ordering, contact projection, pagination, the twenty-row Markdown cap, or any
other location field. Do not add or recapture fixtures and do not edit receipt
records. The inline provider-shaped cleanup case must be declared in
`fixture_key_contract.inline`; that inventory entry is the sole authorized
`testdata/sources/capture-receipts.json` edit.

Dependencies: none. Tickets 1121, 1122, and 1141 touch the same
`TrialLocation` literals or Markdown table but change site completeness,
contact cardinality, and truncation respectively; they are not semantic
prerequisites. Preserve their behavior if any lands first, and resolve only
the mechanical struct-literal/table-column overlap during rebase.

## Review

- Evidence/design pass (2026-09-04): live defect confirmed; request scope
  corrected; existing receipted fixture, owning symbols, public field name,
  rendering shape, and deterministic tests identified.
- Independent design review (2026-09-04): ACCEPT after revision. Verified the
  request-field construction and deduplication, source/public models,
  conversion loss, four hand-built literals, template, receipted capture, and
  deterministic spec fixture route on the current tree. Added explicit
  whitespace, blank/omitted-JSON, and missing-value Markdown coverage because
  the positive receipt alone could not prove those acceptance claims. No
  implementation blocker or semantic dependency remains.
- Implementation evidence (2026-09-04): added `TrialLocation.postal_code`,
  mapped the CTGov `zip` through `clean_opt`, rendered a distinct Markdown
  column, and updated the four hand-built literals. Added receipt-backed and
  provider-shaped conversion/JSON/Markdown tests in
  `src/transform/trial/tests/ticket_1114.rs`, request-projection coverage in
  `src/sources/clinicaltrials/tests/construction.rs`, the missing-value row
  assertion in `src/render/markdown/trial/tests.rs`, and deterministic Markdown
  plus JSON assertions in `spec/entity/trial.md`. The focused test was red on
  the missing public field before implementation, then the ticket tests,
  request-projection test, existing location renderer test, formatting,
  no-default-features Clippy with warnings denied, the Mustmatch lint, and both
  new fixture-backed CLI assertions passed.
- Independent code review (2026-09-04): ACCEPT with no material findings.
  Verified the complete diff including the new test module, all four
  `TrialLocation` literals, request composition, whitespace cleanup, serde
  omission, Markdown column alignment and fallback, receipted fixture data,
  and deterministic spec routing. Focused Rust tests and `git diff --check`
  passed; the reviewer also ran the complete `make spec` gate successfully.
- Full-lint follow-up (2026-09-04): the source capture and fixture-key contract
  rejected the new provider-shaped cleanup case because its inline fixture was
  undeclared. Added the required `fixture_key_contract.inline` entry for
  `ctgov_postal_codes_are_trimmed_and_blank_values_are_omitted:json:1`, backed
  by the existing CTGov schema attestor; no fixture or receipt bytes changed.
  The focused source capture/fixture-key audit and affected Rust tests then
  passed.
- Independent remediation review (2026-09-04): ACCEPT with no findings. The
  selector, path, and CTGov endpoint match the sole new provider-shaped inline
  value; the existing unchanged schema attestor covers `zip`; exceptions
  remain empty. The direct audit classified 234/234 files and checked 441
  fixture keys plus 124 code keys with zero exceptions; the focused audit
  suite (16 tests), ticket tests (2 tests), and `git diff --check` passed.

## Completed 2026-09-04

ClinicalTrials.gov postal codes now survive the source-to-public conversion as
optional `TrialLocation.postal_code` values and appear as a distinct Markdown
column. Missing or blank values remain absent in JSON and render as `-`; the
request projection and deterministic receipted CLI surfaces are pinned.

Final primary-agent verification passed: `make lint`; `make test` (3,086 Rust
tests passed with 30 skipped, 883 Python tests passed with 3 skipped, and the
strict documentation build passed); and `make spec` (all routine pages, 38
parallel-isolation contracts, and 8 static specs passed).
