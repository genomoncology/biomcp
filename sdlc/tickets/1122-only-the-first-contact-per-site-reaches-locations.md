---
flow: build
priority: 5
deps: ["1121"]
---

# Locations keep one contact per site while contacts keeps them all

Locations take only the first contact a site lists. The contacts section keeps every one. So one payload produces two different contact counts for the same site depending on which part of the output a reader looks at.

`extract_locations` at `src/transform/trial.rs:129-132` takes `loc.contacts.first()`, and lines 139-142 fold that one contact into four scalar fields. `TrialLocation` at `src/entities/trial/mod.rs:85-105` has room for exactly one. `extract_contacts` at `:197` iterates every contact the site lists. Verified 2026-09-03 against `0.9.0-dev.6`.

## Required behavior

Every contact a site lists is reachable from that site's location, in the order the provider lists them.

The two views of the same site agree about how many contacts it has.

## Correct behavior

A site's contacts are the same set on both paths. A per-site contact count taken from locations equals the count taken from contacts, and both equal the number the payload states.

Write that as a failing test, then fix. Red before green.

This behavior is held against both 0.9 and 1.0 alike. It is recorded as case 13 of the clinical-trial conformance cases in the sibling BioData repository. **An attempt cannot read that repository**, so the statement above is this ticket's own authoritative copy. If it looks wrong, stop and say so rather than implementing something different.

## The fixture is part of this work

No payload in this repository has a site with more than one contact. Counted 2026-09-03: eleven captures under `testdata/sources/ctgov/`, twenty-six studies, zero central contacts and zero location contacts. Every contact byte tested against today is synthetic on `example.test`.

The payload exists one repository over, authored rather than captured, with a receipt: `tests/fixtures/clinical-trial-parity/case-13-location-contacts.json` in the sibling BioData repository. It is a one-site study whose site lists two contacts, and it is the input this ticket's test needs.

Copying that payload into `testdata/sources/clinicaltrials/` with a receipt is part of this ticket, not a precondition someone else satisfies. Without it the parity assertion is vacuously true, because no site in the corpus has any contact at all.

A trial contact is patient-bearing, so this fixture is authored by policy and will never be a recorded capture. That is the intended state, not a gap waiting to be filled.

It lands in `testdata/sources/clinicaltrials/`, not in `testdata/sources/ctgov/`, which is named for recorded captures.

Ticket 1126 has now established the `authored` classification and requires an
`authored_reason`; authored fixtures remain checked against provider paths. Add
the copied fixture to `testdata/sources/capture-receipts.json` using that shipped
contract rather than inventing another provenance form.

**Tickets 1126 and 1138 are complete. Ticket 1121 remains the declared dependency.**

1138 removed `central_contacts` from `CtGovLocation`, a key ClinicalTrials.gov
does not send at the location path, and removed both unreachable fallbacks.
`extract_locations` now selects `loc.contacts.first()` and `extract_contacts`
iterates `loc.contacts`. Build the new multi-contact shape from those real site
contacts; do not recreate the deleted location-level central-contact path.

1126 already built the fixture-key audit. This ticket must declare its new
authored fixture and selector in that existing inventory, with no exception for
provider-invalid keys.

## Done, observably

- A site listing two contacts reports two on both paths, in the order the payload lists them.
- The per-site counts agree for every site on the trial.
- The authored two-contact payload is in `testdata/sources/clinicaltrials/` with a receipt, so the assertion above has a case it catches rather than passing over an empty set.

## What this replaces

The current single-contact locations rendering is being replaced deliberately. The change reaches the JSON shape of a location, the locations table in `templates/trial.md.j2`, and the pinned locations assertion in `spec/entity/trial.md`. Tests that assert a single contact on a location are expected to be restated. They are not listed here, because no list written before the code is read is complete.

The new shape is not specified here. Design owns it.

## Boundary

Do not change contact ordering or which contact is treated as primary.

Do not change which sites appear. Ticket 1121 settles that, and this ticket depends on it: until every site appears on both paths, a per-site parity assertion is not well defined.

Do not rework how the two paths derive their site list. 1121 owns that mechanism and deliberately leaves it open. This ticket is about the contact count per site and nothing wider.
