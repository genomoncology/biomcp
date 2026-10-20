---
flow: build
priority: 5
---

# A site missing a facility, city or country vanishes from locations entirely

## Outcome

Every meaningfully described site in a ClinicalTrials.gov `locations` array
reaches the public trial location list. Provider-omitted facility, city, and
country values remain absent in structured JSON and render as `-` in the
Markdown table instead of causing the entire site to disappear. A wholly empty
provider object does not become a fabricated all-placeholder site.

## Verified current facts and reproducer

The defect is live on `0.9.0-dev.6`. ClinicalTrials.gov's source model is
already truthful: `CtGovLocation.facility`, `.city`, `.state`, `.zip`, and
`.country` are all `Option<String>` in `src/sources/clinicaltrials.rs`, and the
`locations` and `all` detail projections already request the corresponding
provider fields. No request-field or source-deserialization change is needed.

The loss occurs in `src/transform/trial.rs::extract_locations`. Its three
`clean_opt(...)?` guards require facility, city, and country simultaneously.
The subsequent `(!out.is_empty()).then_some(out)` consequently turns a study
whose sites all fail a guard into `locations: None`. In contrast,
`extract_contacts` iterates the same source module's complete `locations` array,
so a contact can survive after its site was discarded.

The public model makes that accidental filter part of its type contract:
`TrialLocation.facility`, `.city`, and `.country` are required `String` fields
in `src/entities/trial/mod.rs`. Serde therefore cannot express their absence.
JSON serialization is otherwise direct. `templates/trial.md.j2` renders those
three values without missing-value fallbacks; its City cell also needs to avoid
a leading comma when city is absent but state is present.

The defect then reaches every public detail surface. `from_ctgov_study` assigns
`extract_locations(study)` directly to `Trial.locations`.
`entities::trial::get` retains it when `locations` or `all` is requested.
`cli::trial::dispatch::render_loaded_card` serializes the same typed trial to
JSON or Markdown. The raw MCP `biomcp` tool and typed MCP `get` tool both map to
that CLI execution and renderer (`src/mcp/shell.rs::execute_cli`), so they have
no independent location conversion or schema to update.

The deterministic evidence is the sibling BioData recorded capture
`tests/fixtures/clinicaltrials-gov-v2/nct00791778-partial-sites.json`. It is an
exact public response from
`https://clinicaltrials.gov/api/v2/studies/NCT00791778`, captured
`2026-09-02T22:50:57.611106Z`, with SHA-256
`d220af8b0a55ca2a907fb660afea4bf972f04ec42c395a50db35d7ebc4f9f5d9`.
Inspection on 2026-09-04 confirms 59 source locations. All 59 **omit** the
`facility` key (they do not contain `facility: null`), all 59 have non-empty
city and country values, 37 omit state, and none has a site contact. The current
BioMCP corpus has 13 files under `testdata/sources/ctgov/`, 27 study records,
and 31 locations; none is missing facility, city, or country. The old counts of
11 files and 26 studies, and the `facility: null` description, are stale.

The normative direction is case 12 in BioData's clinical-trial conformance
matrix: a site with any identifying field is kept with absent fields absent,
and a contact is not orphaned by a different site filter. It does **not** say
that an empty `{}` must become a public site. Preserve a source member when any
cleaned place field, finite coordinate, or extractable named contact identifies
the site; status alone is not a location identity. Discard only a member for
which every identifying value is absent or blank. This keeps facility-only,
city-only, state-only, postal-only, country-only, coordinate-only, and
contact-bearing sites while avoiding a row made entirely from invented
placeholders. The same predicate must be used when deciding which source sites
can contribute site contacts, so conversion itself cannot orphan a contact.

## Accepted design

Copy the sibling capture byte-for-byte to
`testdata/sources/ctgov/get_nct00791778_20260902.json`. Add a
`real_and_receipted` entry to `testdata/sources/capture-receipts.json` with the
provider URL, capture timestamp, exact SHA-256 above, a statement that no
fields were removed or values redacted, and the public-provider origin
statement. Keep it in `ctgov/`, the repository's recorded-capture directory;
do not move it to `clinicaltrials/` merely to satisfy that directory's existing
discovery special case, minimize away its coordinates, or alter its bytes to
fit an audit limitation.

Ticket 1126's fixture-key audit currently discovers consumed trial documents
only under `clinicaltrials/` and `nci_cts/`. That closed list misses recorded
CTGov captures already fed directly to the conversion layer, including this
ticket's `include_bytes!`, and therefore rejects a truthful `on_disk`
declaration as "not consumed." Extend
`tools/check-source-capture-receipts.py::_consumed_trial_files` so `ctgov/`
uses the same literal-include discovery and fail-closed dynamic-reference rules
as the other two directories. Enroll the complete set of five CTGov documents
then discovered from Rust tests, not just this ticket's file:

- `ctgov/get_nct00791778_20260902.json`, selector `/`;
- `ctgov/get_nct02576665_20260811.json`, selector `/`;
- `ctgov/get_nct02576665_full_20260903.json`, selector `/`;
- `ctgov/search_keytruda_limit3_20260811.json`, selector `/studies/*`; and
- `ctgov/search_phelan_limit5_20260811.json`, selector `/studies/*`.

Each declaration uses endpoint `ctgov`. The latter four are pre-existing,
already receipted consumed captures exposed by correcting discovery; this is an
inventory migration, not permission to edit or reclassify them. A file consumed
from more than one Rust test still has one path/selector declaration.

The provider schema attestor deliberately exposes `geoPoint` as an opaque leaf,
while the copied capture and two pre-existing detail captures contain its real
`lat` and `lon` children. Ticket 1138 already proves exactly those two provider
paths from the receipted unrestricted NCT06131398 record and enforces them as a
closed pair in `code_key_contract.supplemental_attestations`. Make the fixture-
key audit reuse only that already validated pair when checking CTGov fixtures.
Do not admit every descendant of an opaque schema leaf, create a fixture-key
exception, duplicate a second supplemental inventory, or bypass validation of
the evidence path and limitation. The overall audit must still fail if either
shared supplemental declaration is missing, altered, duplicated, extra, or no
longer supported by the receipted evidence.

Extend the existing receipt-audit tests in `tests/test_capture_receipts.py` to
prove a declared, literally consumed `ctgov/` detail/search fixture is
discovered with its correct selector, an undeclared or dynamic CTGov reference
fails closed, the exact `geoPoint.lat`/`.lon` paths pass through the shared
positive attestations, and an unknown sibling or arbitrary opaque-leaf
descendant still fails. Do not add a new audit tool, manifest, exception, or
test file.

In `src/entities/trial/mod.rs::TrialLocation`, change only `facility`, `city`,
and `country` to `Option<String>` and give each the existing
`skip_serializing_if = "Option::is_none"` contract. `state`, `postal_code`,
status, contact fields, and coordinates retain their current types and names.
JSON therefore omits an unavailable value; it must not serialize an empty
string or invent a placeholder. This is an intentional Rust model API change
for callers constructing or reading `TrialLocation`; for JSON, records whose
three fields are present retain their existing keys and string values, while
deserialization becomes able to accept their absence.

In `src/transform/trial.rs::extract_locations`, populate all five textual place
fields through `clean_opt` and remove the three required-field guards. Retain
every source member satisfying the meaningful-site predicate above, and use
that same predicate in `extract_contacts` before a site's contacts are emitted.
Preserve the existing stable recruiting-first sort and the outer `None`
behavior when no meaningful locations remain. Optional facility, city, and
country values do not participate in sorting, and members with equal
recruiting classification retain provider order. Do not otherwise change which
contacts are emitted or their order.

In `templates/trial.md.j2`, render missing facility and country as `-`. Render
the City cell as `city, state` when both exist, city alone or state alone when
only one exists, and `-` when both are absent. Every provider-derived value in
the location table must pass through the existing canonical
`render::markdown::support::markdown_cell` behavior so pipes, newlines, terminal
controls, and blank strings cannot break the table; expose that helper as a
narrow Minijinja filter in the existing Markdown environment rather than
inventing a second escaping rule. Keep the existing Postal code, Status, and
Contact columns and the twenty-row template cap unchanged.

Update the four hand-built `TrialLocation` literals currently in
`src/render/markdown/root_tests.rs`, `src/render/markdown/trial/tests.rs` (two),
and `src/cli/trial/tests_locations.rs` to wrap their present facility, city,
and country strings in `Some`. Update facility comparisons in
`src/transform/trial/tests/ticket_1114.rs` to compare through `as_deref()`;
those are mechanical consequences of the public type change, not permission to
alter ticket 1114's postal-code behavior.

Add the focused receipt-backed regression to the existing
`src/transform/trial/tests.rs` rather than creating another Rust test file.
Decode the copied bytes through
`ClinicalTrialsClient::decode_get_response`, convert with `from_ctgov_study`,
and first establish red-before-green with the current API by asserting the
typed location count is 59 (today it fails because `locations` is `None`). The
completed test must also assert all 59 preserve non-empty city and country, all
59 have `facility == None`, exactly 37 have no state, and serialized JSON has
59 locations with no `facility` key. Render locations Markdown and assert a
known row starts with `| - | La Jolla, California | 92037 | United States |`.

Add a small provider-shaped conversion case in the same existing test module
covering facility-only, state-only, postal-only, country-only, contact-bearing,
status-only, and wholly blank sites, plus a provider string containing a pipe,
newline, and terminal control. Assert every identified source member remains
present, the status-only and wholly blank members are discarded, whitespace is
cleaned, absent/blank fields are `None` and omitted from JSON, the existing
stable recruiting-first/provider ordering survives optional values, and the
state-only Markdown City cell contains the state without a leading comma.
Assert the special characters are sanitized/escaped without changing the
table's column count. Declare this inline case in
`fixture_key_contract.inline`; do not add a provenance exception.
Its coordinate-only member legitimately carries `geoPoint.lat`; it is covered
by the same shared positive attestation above, not by deleting the case or
pretending the opaque schema leaf recursively attests arbitrary children.

Make the public CLI proof deterministic by adding the copied capture to
`CTGOV_DETAIL` in
`spec/fixtures/setup-ctgov-intervention-alias-spec-fixture.sh`. Extend
`spec/entity/trial.md` with a `--json get trial NCT00791778 --limit 59
locations` assertion proving `locations | length == 59`,
`location_pagination.total == 59`, and every serialized location lacks
`facility` while retaining city and country. Add a Markdown assertion for the
same known La Jolla row. The explicit `--limit 59` is required because the CLI
locations surface otherwise paginates to twenty before rendering. Also pin the
fixture server's request log for this ID to the detail path and location-field
projection, so the spec cannot pass by serving the capture for a different
request shape. This ticket does not change the template's independent
twenty-row Markdown cap or its disclosure; ticket 1141 owns that behavior.

No separate MCP regression is required. Both raw and typed MCP invoke the same
CLI parser/execution and the same serde/Markdown renderer, and neither defines
a `TrialLocation` output schema. The typed tool does not expose location
`limit`, so its explicit `locations` section uses the CLI's default twenty-row
page; its `all` section retains the CLI's existing unpaginated structured
behavior, while raw MCP can pass an explicit limit. Those presentation and
pagination differences do not create a second conversion path and are outside
this ticket; ticket 1141 separately owns contact alignment and disclosure when
a surface truncates locations.

## Acceptance

- Receipt-backed conversion of NCT00791778 yields 59 typed locations, not
  `None`; all preserve provider city/country, all omit facility, and 37 omit
  state.
- `TrialLocation.facility`, `.city`, and `.country` are optional. Missing and
  blank provider values become `None` and their JSON keys are omitted.
- No meaningfully described member of a provider `locations` array is dropped
  merely because another location field is absent. Focused single-field,
  contact-bearing, and wholly blank cases prove the inclusion boundary is
  generic rather than hardcoded to NCT00791778.
- Markdown uses `-` for missing facility/country, renders state-only City cells
  without a leading comma, escapes/sanitizes provider values without corrupting
  table shape, and retains the existing postal/status/contact columns.
- The deterministic CLI JSON spec reports all 59 sites with `--limit 59`; the
  Markdown spec proves the known rendered row, and the request-log assertion
  pins the detail location projection without network access. Raw MCP can make
  the same 59-site request; typed MCP inherits the corrected conversion and
  renderer while retaining its existing section-dependent pagination behavior.
- Existing recruiting-first ordering, contact extraction/cardinality, location
  pagination, and postal-code behavior remain unchanged.
- Consumed-file discovery includes recorded `ctgov/` fixtures with the same
  literal/dynamic fail-closed behavior as `clinicaltrials/` and `nci_cts/`; all
  five currently discovered CTGov captures have truthful record selectors.
- Fixture-key checks accept only the two receipt-backed `geoPoint.lat` and
  `geoPoint.lon` supplemental paths already closed by ticket 1138. Unknown
  opaque-leaf descendants, altered evidence, and new exceptions still fail.
- The receipt audit and focused Rust/spec assertions pass, followed by
  `make lint`, `make test`, and `make spec`.

## Scope, dependencies, and constraints

Owning files are `src/entities/trial/mod.rs`, `src/transform/trial.rs`,
`src/render/markdown/mod.rs`, `templates/trial.md.j2`, their existing test
modules, the copied CTGov fixture and receipt inventory, the existing CTGov
spec fixture server, `spec/entity/trial.md`, and the narrowly extended existing
receipt checker and its existing test module. Do not change provider request
fields, source models, NCI conversion, site ordering, which contacts are
emitted, contact order or cardinality, pagination, or the Markdown twenty-row
cap/disclosure behavior. Do not alter any pre-existing capture bytes or
provenance classifications.

Dependencies: none. Ticket 1122 depends on this ticket because it changes a
site's location from one embedded contact to every provider contact; it must
build on the corrected site set and optional `TrialLocation` fields. Ticket
1141 is not an implementation prerequisite: it owns contact filtering and
disclosure when Markdown or CLI pagination presents only a subset of an
already-correct converted site list. Whichever ticket lands second must resolve
only mechanical `TrialLocation` literal/template overlap.

Tickets 1126 and 1138 are completed foundations, not open prerequisites. This
ticket extends 1126's consumed-file boundary to the recorded CTGov directory
and reuses 1138's already closed positive evidence for the two opaque
`GeoPoint` children. It must preserve both ratchets: zero fixture-key
exceptions and the exact two-member supplemental set.

The Cargo package inventory is already exactly 1,300 files, its enforced
ceiling. The new `testdata` capture is excluded from `cargo package`, but a new
Rust test sidecar would exceed the package boundary. Keep tests in existing
files and do not weaken, raise, or add an exception to that limit.
`src/transform/trial.rs`, `src/entities/trial/mod.rs`, and the affected existing
test files are below the 1,000-line Rust source-size threshold. Avoid edits to
the already-ratcheted `src/mcp/shell.rs`; MCP needs none for this shared model
fix.

## Review

- Evidence/design pass (2026-09-04): live filter, source request/model,
  conversion, public type, JSON/Markdown, CLI/MCP reuse, fixture provenance,
  spec routing, package ceiling, and 1122/1141 overlap verified. Corrected the
  stale fixture counts and `null` claim, identified the site-inclusion boundary,
  receipt/key inventory and spec fixture changes, and constrained coverage to
  existing packaged test files.
- Independent design review (2026-09-04): **ACCEPT**. Verified
  the sibling bytes and manifest independently (SHA-256, timestamp, exact-byte
  provenance, 59 locations, 59 omitted facility keys, 37 omitted states, no
  site contacts), the current 1,300-file Cargo package inventory, source/model/
  converter/serde paths, stable status-only sorting, CLI pagination and spec
  route, raw/typed MCP reuse, receipt/key declarations, and the 1122/1141
  boundaries. Revised the unsafe "map every array member" claim to retain every
  meaningful site but reject a wholly empty object, required the same site
  predicate for contact extraction, recorded the intentional Rust API change,
  and added canonical Markdown table escaping plus a pinned request-shape
  assertion.
- Reopened design review (2026-09-04): **REVISE, then ACCEPT after this
  amendment**. The implementation correctly stopped when the receipt audit
  disproved two design assumptions: `ctgov/` was outside consumed-file
  discovery, and ticket 1138's opaque-`GeoPoint` supplements applied only to
  code reads. Independent inspection found five unique CTGov captures consumed
  from Rust tests (the new capture plus four pre-existing files); enrolling
  them introduces no unattested paths beyond the already proven
  `geoPoint.lat`/`.lon` pair. The corrected design keeps the exact recorded
  bytes in `ctgov/`, extends discovery and inventory completely, and shares the
  closed positive attestations with fixture checking. Moving the capture solely
  into a directory the scanner already recognizes, minimizing away valid
  coordinates, or declaring exceptions would conceal the audit gap and is
  rejected.
- Implementation evidence (2026-09-04): the receipt-backed test first failed
  with `locations` equal to `None` instead of 59, then passed after the shared
  meaningful-site predicate, optional public fields, and canonical Markdown
  rendering were implemented. The copied fixture remains byte-identical at
  SHA-256 `d220af8b0a55ca2a907fb660afea4bf972f04ec42c395a50db35d7ebc4f9f5d9`.
  The initial receipt audit exposed the `ctgov/` discovery and opaque-GeoPoint
  contradictions; implementation stopped until the accepted amendment, then
  extended the closed audit with exactly five consumed CTGov captures and the
  exact two pre-existing supplements. The completed audit reports 235 files,
  1,103 fixture keys, zero fixture exceptions, and 124 code keys; all 71 audit
  tests pass. Focused Rust lanes pass 27 transform, 12 trial-Markdown, and 13
  location-CLI tests. The deterministic CLI proof returns 59 locations and a
  pagination total of 59, omits every facility key, renders the La Jolla row,
  and records the exact detail location projection. Formatting, Clippy with
  warnings denied, Ruff, the quality ratchet, static specs, diff whitespace,
  and the exact 1,300-file Cargo package boundary pass. Changing
  `TrialLocation.facility`, `.city`, and `.country` to `Option<String>` is an
  intentional Rust construction/deserialization API change; JSON with present
  values is unchanged, while absent values are now accepted and omitted.
- Independent code review (2026-09-04): **REVISE**. The product conversion,
  JSON, Markdown, fixture, and spec changes were clean. The audit supplement
  validator still accepted any limitation containing `opaque`, and focused
  tests did not explicitly prove the CTGov search selector, an undeclared
  consumed CTGov capture, or both geo coordinates.
- Remediation (2026-09-04): supplement validation now requires each of the two
  exact canonical four-field declaration objects, so altered limitation text,
  evidence, endpoint, paths, duplicates, omissions, extras, or extra object
  fields fail closed. Existing audit tests now separately prove CTGov detail
  and `/studies/*` search consumption, undeclared and dynamic CTGov rejection,
  both `geoPoint.lat` and `.lon`, and rejection of an unknown opaque child. All
  71 receipt tests, the direct 235-file audit, focused product regressions,
  formatting, Ruff, Clippy with warnings denied, quality ratchet, diff check,
  and the exact 1,300-file Cargo package boundary pass.
- Independent remediation review (2026-09-04): **ACCEPT** with no findings.
  Independently verified fail-closed behavior for altered limitations, extra
  fields, duplicate or missing declarations, wrong paths/endpoints/evidence,
  and bad evidence classification; exact declaration reordering remains valid.
  The expanded selector/coordinate tests, 71-test receipt suite, direct audit,
  focused product lanes, fixture SHA, package inventory, and `git diff --check`
  all passed.

## Completed 2026-09-04

Meaningfully identified ClinicalTrials.gov sites now survive partial provider
data. Facility, city, and country are optional public fields; absent values are
omitted from JSON and rendered safely in Markdown. Wholly empty or status-only
objects remain excluded, and site contacts use the same inclusion predicate.
The byte-identical 59-site capture and the expanded closed CTGov fixture audit
provide deterministic evidence.

Final primary-agent verification passed: `make lint`; `make test` (3,092 Rust
tests passed with 30 skipped, 890 Python tests passed with 3 skipped, the
1,300-file Cargo package boundary passed, and strict documentation built); and
`make spec` (all routine pages, 38 parallel-isolation contracts, and 8 static
specs passed).
