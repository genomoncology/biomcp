---
flow: build
priority: 5
---

# Contacts survive a location truncation that drops their site

Status: proposed

## Outcome

Whenever Markdown or a paginated trial response presents only a subset of a
trial's sites, every top-level site contact it presents belongs to that visible
subset. Central contacts remain visible. A capped Markdown location table says
how many sites it shows, and an explicit locations page is not silently cut a
second time by the template.

## Verified current facts and reproducer

The defect remains live on `main` at `84f2343f` (`0.9.0-dev.6`) after tickets
1121 and 1122. The commits after `e7ba28e4` change only planning/issues and do
not alter this ticket's runtime, fixture, spec, documentation, package, or
quality-ratchet evidence.

There are two independent cuts:

- `src/cli/trial/dispatch.rs::paginate_trial_locations` replaces
  `trial.locations` with the requested `offset`/`limit` window but does not
  change `trial.contacts`. Thus `get trial <id> contacts locations --offset 20
  --limit 10` can return site contacts from outside the ten returned sites.
  JSON truthfully emits `location_pagination`, but its top-level contact list is
  not page-scoped.
- `templates/trial.md.j2` renders `locations[:20]` while rendering every
  top-level contact. This affects unpaginated `all` Markdown and batch Markdown,
  and it also silently re-caps an explicit locations page whose requested limit
  exceeds 20. The template emits no cap disclosure. The CLI footer can therefore
  say that 59 locations were shown even though the table rendered only 20.

Section selection matters. `entities::trial::get` omits `contacts` unless
`contacts` or `all` is requested and omits `locations` unless `locations` or
`all` is requested. A contacts-only request intentionally shows every contact
without a location table and is not a parity defect. A locations-only request
has no top-level contacts, and each returned `TrialLocation` already owns its
site contacts. Alignment is required only when contacts and a location subset
are presented together.

Ticket 1121 is complete. `TrialLocation.facility`, `.city`, and `.country` are
optional, and conversion now retains every meaningful site. Ticket 1122 is
also complete. Each location now carries the ordered, cleaned
`TrialLocation.contacts: Vec<TrialSiteContact>` collection, while the legacy
`contact_*` scalars remain literal-first-contact compatibility aliases. The
top-level `Trial.contacts` list still contains module-level central contacts
followed by every named site contact in provider location/contact order.

The nested collection removes the earlier identity problem for current CTGov
conversion: visible site-contact membership can be derived from the visible
locations rather than guessed from facility/city text. The implementation must
still preserve top-level ordering and handle duplicate public values, so it
must filter the existing list against a counted membership set rather than
rebuild the list in location-sort order. The membership key is the exact public
tuple `(facility, city, state, country, name, role, phone, email)`. Postal code
and coordinates cannot participate because `TrialContact` does not carry them.
Indistinguishable duplicate tuples are handled as a multiset: each visible
nested contact authorizes one matching top-level occurrence.

For backward-compatible deserialized or hand-built locations whose nested
`contacts` vector is empty, a nonblank legacy `contact_name` plus the other
legacy scalar and site fields authorizes one matching occurrence. Do not use
the legacy fallback when the nested collection is nonempty. This preserves the
one-contact compatibility representation without allowing unrelated same-site
contacts through. Current CTGov conversion always supplies the authoritative
nested collection for every named site contact.

Module-level central contacts have no owning location and must never be removed
by a location window. Only contacts whose level equals `site`
case-insensitively are candidates for filtering; preserve central and unknown
future levels in their original positions. Preserve the relative order and
exact values of every retained contact. If filtering leaves no contacts,
restore the converter's ordinary absence shape (`contacts: None`) rather than
serializing a fabricated empty section.

The raw MCP `biomcp` tool and typed MCP `get` tool both reuse the CLI parser,
entity conversion, serde serializer, and Markdown renderer through
`src/mcp/shell.rs`; neither owns a trial-location schema or a second conversion.
Raw MCP can express location offset/limit flags. Typed MCP exposes trial
sections but no location paging fields, so `locations` uses the CLI default
20-site page; typed `contacts locations` gets the same contact-aligned default
page. A standalone typed `all` retains the existing unpaginated JSON behavior
and gets the Markdown display cap described below. As in the CLI, adding a
literal `locations` section alongside `all` opts into location paging. No MCP
implementation change is needed.

The sibling BioData evidence does not define this presentation behavior. Its
clinical-trial conformance case 12 requires retaining an identifiable partial
site during conversion, and case 13 requires every contact to remain attached
to its site. Those completed foundations make this ticket implementable, but
the old claim that this ticket is a "presentation half" of case 12 is stale.
The page-alignment and truthful-disclosure rule in this ticket is BioMCP's own
public-output contract.

The 2026-09-03 live survey recorded in the original report found 26 of 600
recruiting studies with more than 20 sites, 16 with contacted sites after the
first 20, and NCT04796350 with 66 sites and 34 contacted sites past the cap.
Treat that as prevalence evidence, not as a mutable-network acceptance
fixture. The repository currently has 13 CTGov response JSON files under
`testdata/sources/ctgov/` (plus field metadata), covering 28 study records and
90 locations; the recorded provider responses intentionally contain no person
contacts. Deterministic contact coverage must therefore remain authored.

## Accepted behavior and design

Add one shared trial projection helper in the existing trial entity module (or
an existing child of it) that filters top-level site contacts to an already
selected `TrialLocation` slice using the counted exact key above. It must retain
central/unknown levels and original contact order, consume duplicate key counts
one occurrence at a time, use the legacy scalar fallback only for an empty
nested collection, and normalize an empty result to `None`. Both CLI paging and
Markdown display projection must call this helper; do not maintain two matching
algorithms.

`paginate_trial_locations` continues to select the same stable
recruiting-first location window and return the same
`LocationPaginationMeta { total, offset, limit, has_more }`. After selecting the
window, align `trial.contacts` to it. Do not change nested location contacts,
location count/order, offset semantics, limit validation, or metadata.

Replace the template's literal `locations[:20]` with an already selected
Markdown location slice and make the renderer own the distinction between two
presentation modes:

1. An explicit CLI `locations` request is already paginated. Render every site
   in that page without a second template cap. The existing CLI footer remains
   the disclosure and reports the actual rendered page count, original total,
   offset, limit, and `more available` state. An explicit `--limit 25` can
   therefore render 25 rows; the default remains 20.
2. Generic/unpaginated Markdown rendering, including `all`, direct renderer
   callers, and `batch trial --sections ...`, keeps a 20-site display cap.
   Before rendering, clone/project the trial to its first 20 locations, align
   top-level site contacts to those locations, and emit exactly
   `Locations: showing 20 of 25 (display cap 20).` when truncated (with the
   actual counts substituted). Emit no cap line when all locations fit.

Expose a narrow paginated-trial Markdown entry point for
`cli::trial::dispatch`; keep the existing `trial_markdown` entry point as the
generic capped behavior used by batch and other callers. Both entry points
must share one internal rendering implementation. Pass selected locations and
the optional disclosure through render context; do not put matching logic,
count arithmetic, or section parsing into Jinja.

Only align top-level contacts when both the contacts and locations sections are
being rendered. Contacts-only Markdown remains complete. Locations-only
Markdown continues to show its nested per-site contacts in the table but no
top-level Contacts section. `all` Markdown shows all central contacts plus only
site contacts belonging to its displayed 20 locations. Empty/out-of-range
location pages likewise retain central contacts and remove all site contacts.

Structured JSON behavior is intentionally split by request shape:

- explicit `contacts locations` JSON is page-scoped and carries the unchanged
  `location_pagination` object;
- `locations` JSON has the selected page and its nested site contacts, with no
  top-level contacts because that section was not requested;
- contacts-only JSON remains complete and still omits `locations`;
- standalone `all` JSON and batch JSON remain complete and unpaginated, so all
  locations and all contacts remain present and no new metadata is invented;
- a request containing literal `locations` is explicitly paginated even when
  `all` is also present. If `all` supplies the contacts section, those contacts
  are aligned to the page and the unchanged pagination metadata is present.

This changes no public Rust or JSON field names/types. It deliberately changes
which site entries appear in a jointly paginated top-level `contacts` array,
adds truthful Markdown cap text, and makes explicit Markdown limits above 20
effective. Existing central-contact output, nested-contact output, legacy
location aliases, complete JSON, and contact-only output remain compatible.

## Test-first implementation plan

Establish red before green in existing files; do not add a Rust test sidecar.

In `src/cli/trial/tests_locations.rs`, construct at least 25 locations with
unique nested contacts plus a central contact, page at offset 20 with limit 3,
and first assert the current failure: all 25 top-level site contacts survive.
The completed test must prove the page contains exactly locations 20--22,
pagination metadata is unchanged, the central contact survives, only those
three site contacts survive, and retained contacts keep their original order.
Add cases for an out-of-range page, duplicate identical contact/site tuples
(counted membership, not set membership), a same-place contact belonging to an
omitted location, an empty nested collection with a populated legacy alias,
and central/unknown contact levels interleaved with site contacts. The
out-of-range coverage must separately prove both central-contact survival and
`contacts: None` when an all-site list has no authorized member. Also cover a
site whose optional facility/city/country identity is absent, uppercase `SITE`,
a blank legacy name, and a nonempty nested collection with a stale legacy alias
to prove that fallback is neither over-broad nor used alongside authoritative
nested contacts. These cases pin the matching boundary and prevent
facility-only filtering.

In `src/render/markdown/trial/tests.rs`, construct 21 locations with nested
contacts and a central contact. Generic `trial_markdown(..., ["all"])` must
render 20 rows, include the exact cap disclosure with 20 and 21, include the
central and first-20 site contacts, and omit both the 21st location and its
top-level contact. A nontruncated case must omit the disclosure. Exercise the
paginated Markdown entry point with more than 20 already-selected locations and
prove it renders the last row with no hidden second cap. Retain the existing
safe multi-contact cell and legacy-scalar fallback assertions from ticket 1122.

Extend the existing authored `CONTACTS_ELIGIBILITY_STUDY` in
`spec/fixtures/setup-ctgov-intervention-alias-spec-fixture.sh` to 25 deterministic
sites, each with a unique named site contact, while preserving the current
first site and central contact used by existing contracts. This is an in-memory
fixture-server value, introduces no new provider keys or person data, and needs
no capture receipt or new file. Do not alter the byte-pinned case-13 fixture.

In `spec/entity/trial.md`, add executable contracts proving:

- `--json get trial NCT41300001 --offset 20 --limit 3 contacts locations`
  returns three locations, only their three top-level site contacts, the
  central contact, and exact `total: 25`, `offset: 20`, `limit: 3`,
  `has_more: true` metadata;
- the Markdown form shows the same central and selected site contacts, omits a
  known off-page contact, and emits the exact existing footer counts, offset,
  limit, and `more available` state for `showing 3 of 25`;
- `get trial NCT41300001 --limit 25 contacts locations` renders the 25th row,
  proving the old template cap is gone for an explicit page, and its footer
  truthfully reports 25 shown with no `more available`; and
- `get trial NCT41300001 all` keeps the 20-row generic cap, omits the 21st
  site's top-level contact, and prints the exact display-cap disclosure.

Use `jq`, fixed synthetic names, and `mustmatch`/exact negative shell checks so
the spec proves counts and membership, not merely the presence of one lucky
row. Keep the existing request-log assertions valid; no provider request-field
change belongs here.

Update `docs/sources/clinicaltrials-gov.md` and `docs/user-guide/trial.md` to
state the default 20-site locations page, explicit offset/limit behavior,
page-scoped top-level site contacts when contacts and locations are combined,
central-contact survival, complete contacts-only/full JSON behavior, and the
20-site disclosed cap for unpaginated `all`/batch Markdown. Do not document a
new flag or response field.

No separate raw- or typed-MCP test is required: the CLI executable contracts
exercise the exact shared execution and rendering path, and MCP owns no
alternate model. Do not edit `src/mcp/shell.rs`.

## Acceptance

- A jointly requested location page contains only top-level site contacts
  authorized by the returned locations' nested contact collections; central
  contacts survive, including on an empty page.
- Filtering preserves original top-level order and cardinality, including
  indistinguishable duplicate tuples, and does not infer ownership from place
  fields alone. The documented legacy one-contact fallback works only when a
  location's nested collection is empty.
- Default explicit locations requests still page at 20. Their JSON metadata is
  unchanged, their Markdown footer is accurate, and an explicit Markdown limit
  above 20 renders the complete requested page rather than only 20 rows.
- Unpaginated `all` and batch Markdown render at most 20 location rows, align
  the Contacts section to those rows, and disclose the exact shown/total
  counts. No disclosure appears when there is no truncation.
- Contacts-only output remains complete; locations-only output remains
  page-scoped through nested contacts; standalone `all` JSON and batch JSON
  remain complete and unpaginated. A literal `locations` combined with `all`
  remains an explicit paginated request and aligns the contacts supplied by
  `all`.
- Location order/count, within-site contact order, global retained-contact
  order, central-contact semantics, public serde shapes, provider projections,
  conversion, and the ticket-1121 meaningful-site predicate do not change.
- Focused Rust tests and deterministic trial specs pass, followed by
  `make lint`, `make test`, and `make spec`.

## Scope, dependencies, and constraints

Owning files are the existing trial entity projection/model module,
`src/cli/trial/dispatch.rs`, `src/cli/trial/tests_locations.rs`,
`src/render/markdown/trial.rs`, `src/render/markdown/trial/tests.rs`,
`templates/trial.md.j2`, the existing CTGov spec fixture setup,
`spec/entity/trial.md`, and the two trial documentation pages named above.

Do not change CTGov source structs or request fields, conversion/site inclusion,
recruiting-first site ordering, nested contact extraction/order, legacy scalar
semantics, NCI conversion, contact redaction policy, pagination flag grammar,
pagination metadata, JSON `all`/batch completeness, or the default page size.
Do not add a contact/location identifier, reconstruct contacts in sorted
location order, drop central contacts, modify recorded fixture files or the
byte-pinned case-13 fixture, edit sibling BioData, or touch unrelated issue
files. The explicitly authorized authored fixture-server value above is the
only fixture mutation in scope.

Dependencies: none. Tickets 1121 and 1122 are completed foundations, not open
dependencies. Their accepted optional-location and nested-contact models are
the inputs to this design.

`cargo package --list --allow-dirty --no-verify` currently reports exactly
1,300 files, the enforced ceiling. Keep tests in existing files and do not add
a packaged sidecar, raise the ceiling, or add an exception. Current relevant
files remain below the 1,000-line Rust source-size limit
(`dispatch.rs` 574, `tests_locations.rs` 346, Markdown `trial.rs` 215, and its
tests 467 lines before this work); keep them below that limit. Test data is
authored in the existing fixture-server script, not added as a package file.

## Review

- Independent design review (2026-09-04): **ACCEPT after amendment**. Verified
  `main` at `84f2343f`; movement since `e7ba28e4` is confined to unrelated
  planning/issues. The review independently checked the two owning cuts,
  conversion and public serde models, direct/batch and JSON/Markdown request
  shapes, raw/typed MCP reuse, exact counted membership, optional site fields,
  nested-versus-legacy contact authority, retained ordering and contact levels,
  empty-result normalization, disclosures, the authored fixture and existing
  specs/docs, the exact 1,300-file package inventory, and current Rust file
  sizes. The amendment makes standalone `all` versus `all locations` explicit
  and requires negative fallback, optional-identity, case-insensitive site,
  interleaved-order, `None`, and exact-footer proofs. No current issue or code
  change invalidates the design.
- Implementation evidence (2026-09-04): test-first red reproduced with
  `cargo test --no-default-features
  cli::trial::tests_locations::paginate_trial_locations_aligns_site_contacts_to_the_page
  -- --exact --nocapture`: the requested sites were 20--22, but the assertion
  received the central contact plus all 25 site contacts. The shared counted
  projection, CLI paging alignment, capped generic Markdown projection and
  disclosure, and uncapped explicit-page renderer path then made the focused
  trial pagination suite (16 tests) and Markdown suite (15 tests) pass. The
  boundary coverage includes duplicate cardinality, exact same-place
  nonmembership, optional site identity, case-insensitive `site`, interleaved
  central/unknown levels, empty-page `None`, blank legacy names, legacy-only
  authorization, and authoritative nested contacts overriding stale aliases.
  No design assumption was disproved. `make lint`, `make test` (3,101 Rust
  tests passed, 30 skipped; 890 Python tests passed, 3 skipped; strict docs
  build passed), and `make spec` passed. `git diff --check` passed; package
  inventory remains exactly 1,300 files, and the four touched Rust files remain
  below 1,000 lines (575, 577, 266, and 581 lines respectively).
- Code review (2026-09-04): **REJECT**. The explicit `--limit 25` Markdown
  contract searched the whole output for `Fixture Site 25` and `Site
  Coordinator 25`, so those strings could come from the top-level Contacts
  section while a restored `locations[:20]` template cap still hid the 25th
  Locations-table row. The blank legacy `contact_name` case also had no
  otherwise exact blank-name top-level site candidate, so deleting the
  production blank-name guard did not affect the test result.
- Remediation evidence (2026-09-04): the explicit-page contract now isolates
  the `## Locations` section and pins the complete 25th table row, including
  facility, city/state, country, status, contact role, phone, and email. With a
  temporary `locations[:20]` mutation, the focused assertion failed because
  that row was missing even though the truthful 25-of-25 footer remained. The
  legacy-boundary test now includes an otherwise exact top-level site contact
  whose name is three spaces and directly asserts its removal. Temporarily
  deleting only `filter(|name| !name.trim().is_empty())` failed the test with
  the blank contact retained. After restoring production code, the exact
  boundary test passed, the full pagination suite passed 16 tests, the full
  trial Markdown suite passed 15 tests, the formatting and diff checks passed,
  and `make spec` passed its complete routine and static contract lanes.
  Product behavior is unchanged by this remediation.
- Code re-review (2026-09-04): **REJECT**. The stale legacy-alias candidate was
  not otherwise an exact match for the location's legacy tuple: constructing
  it from `contact("site", "Stale Alias", Some("Site 07"))` gave it
  `stale-alias@example.test`, while `authoritative.contact_email` remained
  `contact-07@example.test`. The test therefore still passed if legacy fallback
  was incorrectly allowed alongside a nonempty authoritative nested-contact
  collection.
- Second remediation evidence (2026-09-04): the stale-alias top-level contact
  is now based on the exact `Contact 07` tuple and changes only its name to
  `Stale Alias`, matching every legacy scalar and site field, including
  `contact-07@example.test`; the test directly asserts that candidate is
  removed while the authoritative nested `Contact 07` survives. Temporarily
  authorizing the legacy tuple in the nonempty-nested branch made the focused
  test fail at the new stale-alias assertion. After restoring the unchanged
  production branch, the exact test and all 16 focused pagination tests passed,
  as did `cargo fmt --all -- --check` and `git diff --check`. No production
  change was required.
- Final independent code re-review (2026-09-04): **ACCEPT with no findings**.
  Verified that all three rejected assertions are now mutation-resistant: the
  complete 25th Locations-table row disappears under the old template cap,
  the exact blank-name candidate survives without the blank guard, and the
  otherwise exact stale legacy alias survives if fallback is wrongly combined
  with authoritative nested contacts. Reassessed and accepted the counted
  projection, ordering and level preservation, empty normalization, paging and
  rendering modes, disclosures, fixture/docs scope, 1,300-file package limit,
  and source-size limits. Independently passed all 16 pagination tests, all 15
  trial Markdown tests, `make spec`, formatting, and diff checks.

## Completed 2026-09-04

Joint contact/location pages now retain central contacts and only the ordered
site contacts authorized by the selected locations. Explicit Markdown pages
render their complete requested window without a second cap. Generic `all` and
batch Markdown retain the 20-location display cap, align site contacts to those
rows, and disclose the exact shown and total counts. Contacts-only and complete
JSON behavior remain unchanged.

Final primary-agent verification on the integrated `origin/main` state passed:
`make lint`; `make test` (3,101 Rust tests passed with 30 skipped, 890 Python
tests passed with 3 skipped, and strict documentation built); and `make spec`
(all routine pages, 38 parallel-isolation contracts, and 8 static specs
passed). The Cargo package inventory remained exactly 1,300 files.
