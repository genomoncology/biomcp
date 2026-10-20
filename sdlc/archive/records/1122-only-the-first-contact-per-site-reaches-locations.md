---
flow: build
priority: 5
---

# Every named contact at a site reaches that location

## Outcome

Every nonblank named contact in a ClinicalTrials.gov site's `contacts` array is
exposed from that public `TrialLocation`, in provider order. The existing
top-level `Trial.contacts` view exposes the same ordered contact set for each
site, while the four existing scalar `TrialLocation.contact_*` fields remain
compatible first-contact aliases. This does not reorder the top-level contacts
to match the recruiting-first location sort; global cross-site contact order is
existing behavior and outside this ticket.

## Verified current facts and evidence

The defect remains live on `0.9.0-dev.6` after ticket 1121. The CTGov source
model is already complete: `src/sources/clinicaltrials.rs::CtGovLocation` owns
`contacts: Vec<CtGovContact>`, and both the `contacts` and `locations` request
projections include `LocationContactName`, `LocationContactRole`,
`LocationContactPhone`, and `LocationContactEMail`. No provider request or
source-model change is needed.

The loss is in `src/transform/trial.rs::extract_locations`: it reads
`loc.contacts.first()` and copies only that source member into
`TrialLocation.contact_name`, `contact_role`, `contact_phone`, and
`contact_email`. `src/entities/trial/mod.rs::TrialLocation` has no collection
in which a second contact can survive. In contrast,
`src/transform/trial.rs::extract_contacts` iterates every contact at every site
accepted by the shared `is_meaningful_site` predicate and appends each contact
whose cleaned name is nonblank. Thus the current public location view has
cardinality at most one while the top-level site-contact view has cardinality
zero to many.

Ticket 1121 is complete. It made facility, city, and country optional and made
`extract_locations` and `extract_contacts` share `is_meaningful_site`; this
ticket must build on that site set and must not change the predicate. Ticket
1138 is also complete: real site contacts come only from
`CtGovLocation.contacts`; do not recreate the deleted, provider-invalid
`CtGovLocation.central_contacts` fallback. Module-level central contacts remain
valid and are unrelated to a location's nested contacts.

The deterministic conformance input is
`../biodata/tests/fixtures/clinical-trial-parity/case-13-location-contacts.json`,
SHA-256
`f1c6e779e842ca3a1f1dae52264ba6e2015b205624ad32a96370a75d934d9e2e`.
Independent inspection on 2026-09-04 confirms that it is an MIT-redistributable,
authored public synthetic fixture containing one meaningful site and two named
contacts, `First Synthetic Contact`/`CONTACT` followed by
`Second Synthetic Contact`/`BACKUP`. BioData case 13 requires every path that
exposes site contacts to preserve both in that order.

The old corpus statement is stale. There are currently 13 recorded CTGov
response files under `testdata/sources/ctgov/` (plus the field-metadata file),
covering 28 study records and 90 locations; those recorded responses still
contain zero central or site contacts. The on-disk authored
`clinicaltrials/study_contacts.json` contains one central contact but no site
contact. However, `src/transform/trial/tests.rs::ctgov_meaningful_sites_keep_partial_identity_and_safe_markdown`
already has an inline provider-shaped site with two named contacts and proves
both reach `Trial.contacts`; it also incidentally demonstrates that only the
first reaches the current `TrialLocation`. The new copied fixture is therefore
not needed to avoid a wholly vacuous unit test, but it is still required as the
byte-pinned cross-repository case-13 evidence and deterministic CLI/spec input.
Person-bearing recorded captures remain intentionally unsuitable.

The defect reaches direct serde JSON and `templates/trial.md.j2`, whose Contact
cell reads only the four scalars. `entities::trial::get` merely includes or
removes the converted `contacts` and `locations` sections.
`cli::trial::dispatch::render_loaded_card` uses the same typed trial for JSON
and Markdown. Raw MCP reaches `cli::execute_mcp_cli` through
`BioMcpServer::execute_cli`; typed MCP `get` maps its input through `get_args`
onto the same CLI grammar and execution path. Neither MCP tool defines a
separate `TrialLocation` response schema or conversion. Typed MCP exposes trial
sections but no location offset/limit; a locations request therefore retains
the CLI default page of 20, while raw MCP may express CLI paging flags.

## Accepted public representation and compatibility

Add this public model in `src/entities/trial/mod.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialSiteContact {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}
```

Add to `TrialLocation`:

```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub contacts: Vec<TrialSiteContact>,
```

`contacts` is an ordered zero-to-many collection owned by the site. It contains
exactly the source contacts that have a nonblank cleaned name, matching the
existing `TrialContact` inclusion rule; role, phone, and email remain optional
and cleaned. Do not count or invent an entry for a nameless source object.
Central contacts never appear in this location collection.

Retain `contact_name`, `contact_role`, `contact_phone`, and `contact_email`
with their current names, types, serde behavior, and values. They continue to
reflect the provider's literal first source contact through the current
`.first()` logic, even if that member has a blank name, so existing JSON
consumers and Rust readers retain the old primary-contact behavior. The new
array is additive in JSON, and `default` allows previously serialized locations
without `contacts` to deserialize. Adding a field is nevertheless an
intentional Rust construction API change: every in-repository `TrialLocation`
literal must add `contacts: Vec::new()` or its intended contact list.

Do not replace the scalars, change them to parallel arrays, reuse
`TrialContact` as the nested type, or duplicate a site into one location per
contact. `TrialContact` includes top-level-only `level` and repeated site
identity, while duplicated locations would corrupt location counts,
pagination, and ticket 1141's site boundary.

In Markdown, keep one row per site and the existing six columns. Render every
entry in `loc.contacts` inside the Contact cell in provider order, using the
existing `Name (role) phone email` field order and `<br>` between contacts.
Continue applying the existing `markdown_cell` filter to every provider value.
When `loc.contacts` is empty, fall back to the four legacy scalar fields so old
deserialized values and callers that construct only the compatibility fields
render exactly as before. A one-contact converted site therefore has unchanged
Markdown; the case-13 cell is exactly:

```text
First Synthetic Contact (CONTACT)<br>Second Synthetic Contact (BACKUP)
```

## Implementation scope

- In `src/entities/trial/mod.rs`, add `TrialSiteContact` and the defaulted,
  omitted-when-empty `TrialLocation.contacts` field exactly as above.
- In `src/transform/trial.rs`, add one shared helper that cleans a
  `CtGovContact` into `TrialSiteContact` only when its name is nonblank. Use it
  from both `extract_locations` and the site branch of `extract_contacts` so
  the nested and top-level site-contact sets cannot drift. Preserve the
  existing module-level central-contact conversion separately. Preserve the
  legacy scalar population from `loc.contacts.first()` unchanged.
- In `templates/trial.md.j2`, render the nested collection and legacy fallback
  exactly as specified above. Keep one table row per site, the twenty-row
  template cap, all columns, and all place-field behavior from ticket 1121.
- Copy the sibling fixture byte-for-byte to
  `testdata/sources/clinicaltrials/case-13-location-contacts.json`. Add it to
  `testdata/sources/capture-receipts.json` as `classification: "authored"`
  with an `authored_reason` recording the person-bearing capture exclusion,
  its byte-identical MIT BioData origin, and the verified SHA-256. Declare it
  once in `fixture_key_contract.on_disk` with selector `/` and endpoint
  `ctgov`. Authorship does not exempt its provider-shaped keys from the ticket
  1126 audit. No new attestation or exception is needed: every key is already
  in the CTGov schema inventory.
- Add the fixture to `CTGOV_DETAIL` in
  `spec/fixtures/setup-ctgov-intervention-alias-spec-fixture.sh` under
  `nct00000000`. The existing fixture server already logs detail requests and
  serves raw source bytes for this map.
- Update `docs/sources/clinicaltrials-gov.md` to document the ordered JSON
  `locations[].contacts` collection, all-contact Markdown rendering, and the
  retained first-contact scalar aliases.
- Make only mechanical `contacts` additions to the four existing hand-built
  `TrialLocation` literals in `src/render/markdown/root_tests.rs`,
  `src/render/markdown/trial/tests.rs` (two), and
  `src/cli/trial/tests_locations.rs`.

## Focused tests and executable contract

Add the fixture declaration through the existing receipt/key audit; do not
create a new audit tool or exception.

In the existing `src/transform/trial/tests.rs`, decode the copied bytes through
`ClinicalTrialsClient::decode_get_response` and convert with
`from_ctgov_study`. Establish red against the current API first by serializing
the trial and asserting the two ordered names under `locations[0].contacts`;
today that JSON key is absent. The completed test must additionally assert:

- one typed location with two nested contacts in exact name/role order;
- two top-level `level == "site"` contacts in the same order and equal cleaned
  name/role/phone/email values;
- per-site nested count, top-level site count, and provider count are all two;
- the legacy scalar fields still contain only the first source contact; and
- serialized JSON has `locations[0].contacts` with both ordered objects and
  retains the first-contact scalar keys.

Also pin both serde compatibility directions in an existing test file: a
`TrialLocation` serialized with an empty nested collection omits `contacts`,
and a legacy location JSON object without `contacts` deserializes with an empty
collection. Add a provider-shaped case whose literal first contact has a blank
name but nonblank role/phone and whose second contact is named. It must prove
that the nested and top-level collections retain only the named second contact,
while all four legacy scalars still derive independently from the literal first
source object (`contact_name` remains absent, and its nonblank role/phone remain
present). This guards against implementing the aliases from the first cleaned
collection member.

Extend the existing inline
`ctgov_meaningful_sites_keep_partial_identity_and_safe_markdown` assertions so
its two named contacts also reach that location in order. This corroborates
that the implementation is generic and continues to use the shared meaningful-
site boundary rather than hardcoding the copied NCT ID.

In `src/render/markdown/trial/tests.rs`, extend the existing contact/location
case with two nested contacts and assert both render in one Contact cell with
the exact `<br>` separator and safe field rendering. Retain an assertion using
an empty nested collection plus populated legacy scalars to prove the backward-
compatible fallback. Existing one-contact output remains pinned.

Extend `spec/entity/trial.md` with a deterministic
`--json get trial NCT00000000 contacts locations` assertion that compares the
ordered names and roles in `locations[0].contacts` with the ordered top-level
site contacts, proves both lengths are two, and pins
`locations[0].contact_name` to `First Synthetic Contact`. Add a Markdown
assertion for the exact two-contact table cell. Pin the request log for this ID
to the exact detail path and sorted, deduplicated `fields` query produced by the
combined `contacts locations` request, including all four `LocationContact*`
fields, so the spec cannot pass against a different route or request shape.

As a focused fixture check, run `sha256sum` against both copies and require the
exact recorded digest
`f1c6e779e842ca3a1f1dae52264ba6e2015b205624ad32a96370a75d934d9e2e`
before running the existing receipt/key audit. The receipt audit intentionally
requires only `authored_reason` for authored fixtures, so it does not by itself
prove byte identity.

No separate raw- or typed-MCP regression is required. The executable CLI proof
exercises their shared entity conversion and renderers, and the MCP code has no
independent output model. Do not edit the already-ratcheted
`src/mcp/shell.rs`; ticket 1141, not this ticket, owns MCP-visible contact
filtering when only a page of locations is presented.

## Acceptance

- The copied case-13 payload exposes both named site contacts, in provider
  order, in `locations[0].contacts` and in the top-level site-contact view;
  their per-site counts and values agree with the source payload.
- Location JSON uses the exact ordered `TrialSiteContact` array above. Empty
  arrays are omitted, old JSON without the field deserializes, and the existing
  scalar keys and literal-first-source-contact semantics remain unchanged even
  when the first source object is nameless and therefore absent from the new
  collection.
- Markdown renders all nested contacts in one site row with `<br>`, preserves
  the existing one-contact appearance, safely filters every provider value,
  and falls back to legacy scalars when the new collection is empty.
- The existing inline two-contact site proves the converter is generic and
  still respects ticket 1121's meaningful-site predicate.
- The authored fixture is byte-identical to the sibling SHA-256, is truthfully
  classified and declared under the existing receipt/key contract, and passes
  the CTGov provider-key audit with zero exceptions.
- The deterministic CLI JSON and Markdown specs pass and the request log pins
  the location-contact projection. Raw and typed MCP inherit the same result
  through their shared CLI execution path.
- Location count, recruiting-first site order, within-site provider contact
  order, module-level central contacts, location pagination metadata, and all
  behavior owned by ticket 1141 remain unchanged.
- Focused Rust tests and the receipt audit pass, followed by `make lint`,
  `make test`, and `make spec`.

## Exclusions, dependencies, and constraints

Do not change provider request fields or source structs, the meaningful-site
predicate, which sites appear, recruiting-first site sorting, central-contact
semantics, NCI conversion, contact redaction policy, location pagination, the
Markdown twenty-row cap/disclosure, or filtering of top-level contacts after a
location page is selected. Do not remove or reinterpret the legacy scalar
fields. Do not touch unrelated GenCC work, create another fixture/audit
mechanism, or edit sibling BioData.

Dependencies: none. Tickets 1121, 1126, 1138 are completed foundations.

Ticket 1141 overlaps only in presentation. After this ticket, a retained
location carries its own ordered contacts, which gives 1141 an unambiguous
source for aligning top-level site contacts to a paginated/rendered site set.
This ticket must not preempt that work: `paginate_trial_locations` continues to
slice only `trial.locations`, the template retains its current 20-site cap,
and `trial.contacts` remains otherwise untouched. If 1141 lands first, resolve
only mechanical model/template/test overlap and preserve its filtering and
disclosure behavior.

The current Cargo package inventory is exactly 1,300 files, its enforced
ceiling. `testdata/` is excluded from the package, so the copied JSON does not
raise that count; keep Rust tests in existing files and do not add a test
sidecar, raise the ceiling, or add an exception. Current relevant Rust files
are below the 1,000-line source-size threshold (`src/entities/trial/mod.rs` 280,
`src/transform/trial.rs` 673, `src/transform/trial/tests.rs` 483,
`src/render/markdown/trial/tests.rs` 445, and
`src/cli/trial/tests_locations.rs` 345 lines).

## Review

- Evidence/design pass (2026-09-04): **ACCEPT for independent design review**.
  Verified the post-1121 source/model/converter behavior, shared meaningful-
  site predicate, JSON/Markdown/CLI paths, raw and typed MCP reuse, paging
  boundary, existing inline two-contact evidence, sibling bytes/manifest/hash,
  current recorded-corpus counts, fixture receipt/key contracts, docs, all
  `TrialLocation` literals, package ceiling, source sizes, and ticket 1141
  overlap. Corrected the stale dependency/corpus/vacuity claims and selected an
  additive nested model with explicit legacy behavior rather than duplicating
  sites or breaking the existing JSON fields.
- Independent design review (2026-09-04): **ACCEPT**. Independently verified
  the source projection/model/converter, serde and Markdown surfaces, fixture
  bytes and BioData provenance, receipt/key-contract shape, CLI request-log
  route, MCP reuse, four hand-built literals, 1,300-file package inventory, and
  ticket 1141 boundary. Tightened the plan to prove omit-empty/default serde,
  literal-first alias behavior when that contact is nameless, exact fixture
  SHA, exact request shape, and the distinction between within-site provider
  order and unchanged global cross-site ordering.
- Implementation pass (2026-09-04): Added the accepted nested model, shared
  site-contact cleaner, literal-first scalar compatibility, all-contact safe
  Markdown rendering with legacy fallback, byte-identical case-13 fixture and
  provenance/key declarations, fixture-server route, docs, unit coverage, and
  CLI contracts. Red was observed in
  `ctgov_meaningful_sites_keep_partial_identity_and_safe_markdown`: the new
  serialized `locations[6].contacts` assertion received `Null` instead of the
  expected ordered two-contact array. Green evidence: all five focused contact
  and Markdown tests passed; both fixture copies matched
  `f1c6e779e842ca3a1f1dae52264ba6e2015b205624ad32a96370a75d934d9e2e`;
  the receipt/provider-key audit passed with zero exceptions; `cargo package
  --list --allow-dirty --no-verify` remained 1,300 files; `cargo fmt --check`,
  `git diff --check`, `make lint`, `make test` (3,095 Rust tests and 890 Python
  tests passed, 30 and 3 skipped respectively; strict docs build passed), and
  `make spec` passed. No design deviation. One test-harness discovery required
  direct `../../../testdata/...` include paths because the nested Rust test
  module resolves `include_bytes!` relative to `src/transform/trial/`; the
  pre-existing CTGov include was changed mechanically to the same direct form
  so the closed fixture audit could recognize both directories without its
  dynamic-concat guard firing.
- Independent code review (2026-09-04): **ACCEPT with no findings**. Verified
  the additive serde-compatible model, literal-first scalar aliases including
  the blank-name edge case, ordered nested/top-level site-contact parity,
  unchanged meaningful-site and central-contact behavior, safe all-contact
  Markdown plus legacy fallback, exact fixture provenance/hash and request
  shape, documentation, the 1,300-file package ceiling, source-size limits,
  and the ticket 1141 boundary. Independently passed the focused Rust and
  Markdown tests, receipt/provider-key audit, formatting/diff checks, and the
  full executable spec gate. No unrelated changes were present.

## Completed 2026-09-04

Every named ClinicalTrials.gov site contact now survives in provider order in
the owning location's additive `contacts` array and in the top-level site-
contact view. The existing scalar location contact fields remain first-source-
contact compatibility aliases. Markdown renders all nested contacts safely in
one site row and retains a scalar fallback for older serialized locations. The
byte-identical authored case-13 fixture and executable CLI contracts pin the
two-contact JSON, Markdown, and provider request behavior.

Final primary-agent verification passed: `make lint`; `make test` (3,095 Rust
tests passed with 30 skipped, 890 Python tests passed with 3 skipped, and strict
documentation built); and `make spec` (all routine pages, 38 parallel-
isolation contracts, and 8 static specs passed). The Cargo package inventory
remained exactly 1,300 files.
