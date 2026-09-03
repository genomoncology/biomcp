---
flow: build
priority: 5
---

# A site missing a facility, city or country vanishes from locations entirely

`extract_locations` at `src/transform/trial.rs:126-128` drops a whole site when any one of facility, city or country is absent:

```rust
let facility = clean_opt(loc.facility.as_deref())?;
let city = clean_opt(loc.city.as_deref())?;
let country = clean_opt(loc.country.as_deref())?;
```

Line 161 then turns a trial whose sites are all incomplete into `locations: None`. A trial that runs at 59 real hospitals reports that it has no sites.

This is not a rare shape. NCT00791778 carries 59 locations and every one of them has `facility: null`. Across 1,000 completed studies sampled from the live provider on 2026-09-03, 346 of 6,275 locations were incomplete.

Verified against `0.9.0-dev.6`.

## Required behavior

A site the provider described incompletely is still a site. It is kept, and the fields the provider did not supply are absent from it rather than causing the site to be dropped.

## Correct behavior

A site with any identifying field is kept, with its absent fields absent. Locations and contacts are derived from the same set of sites.

Write that as a failing test, then fix. Red before green.

This behavior is held against both 0.9 and 1.0 alike. It is recorded as case 12 of the clinical-trial conformance cases in the sibling BioData repository. **An attempt cannot read that repository**, so the statement above is this ticket's own authoritative copy. If it looks wrong, stop and say so rather than implementing something different.

The direction is settled and is not a design question: the site is kept. Dropping the site instead would also be self-consistent, but it costs a caller the location of a real trial site, while keeping it costs an absent JSON field. Design owns the mechanism — whether both paths derive from one shared site list, or the guard is relaxed to accept any identifying field — not the direction.

## The fixture is part of this work

This repository has no payload of its own. Counted 2026-09-03: eleven captures under `testdata/sources/ctgov/`, twenty-six studies, zero incomplete locations.

The payload exists one repository over and it is a **recorded capture, not authored**: `tests/fixtures/clinicaltrials-gov-v2/nct00791778-partial-sites.json` in the sibling BioData repository, receipted in that folder's manifest as a public recorded capture, taken 2026-09-02 from `https://clinicaltrials.gov/api/v2/studies/NCT00791778`, sha256 pinned, recorded for this exact case. It carries 59 locations, all with a null facility.

Copying that payload into `testdata/sources/ctgov/` with its receipt is part of this ticket. Nothing is pending: fixture ownership was ruled on 2026-09-02 — BioData records every fixture once and both projects use the same set.

## Done, observably

- Converting the 59-location payload yields 59 locations, each carrying the city and country the provider supplied, with state where present and facility absent. 37 of the 59 have no state.
- A trial whose sites are all incompletely described does not report zero locations.
- The recorded payload is in `testdata/sources/ctgov/` with its receipt, so the assertion has a case it catches.

## What this replaces

Assertions that pin the current drop-the-site behavior are expected to be restated.

`TrialLocation` at `src/entities/trial/mod.rs:85-90` declares `facility`, `city` and `country` as `String` rather than `Option`. They are required today precisely because the three guards this ticket removes were what guaranteed them, so the type changes and the JSON shape changes with it.

That reaches the markdown table at `templates/trial.md.j2:81`, which renders all three unguarded. `status` on the same row has an explicit `or "-"` fallback and the other three have none, because until now they could not be absent.

## Boundary

Do not change site ordering or which sites the provider returns.

Do not change which contacts are emitted, or their order. The shared-site-list mechanism offered above necessarily restructures `extract_contacts` at `src/transform/trial.rs:185-208`; that is permitted. Changing what it produces is not.

The related symptom — the output listing a contact for a site it says does not exist — is removed **inside the conversion** by keeping the site, because the site the contact belongs to now appears in `from_ctgov_study`'s output. What remains at that layer is only a site with no identifying field at all, no facility, no city and no country, that still carries a contact. No such site exists in 225 fixtures across both repositories or in 1,600 live studies sampled 2026-09-03, so there is no failing test to write and no ticket for it. Case 12's rule already covers it if one ever appears.

That is a statement about the conversion and not about what a caller receives. Two surfaces downstream truncate locations while leaving contacts whole, and both keep the symptom: `templates/trial.md.j2:80` caps the table at twenty locations with no overflow line, and `paginate_trial_locations` at `src/cli/trial/dispatch.rs:389-405` slices locations without touching contacts. Ticket 1141 carries that. Do not fix it here.
