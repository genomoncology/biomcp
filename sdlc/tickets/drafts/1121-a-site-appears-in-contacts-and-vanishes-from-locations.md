---
flow: build
priority: 5
---

# A site missing a facility, city or country vanishes from locations but keeps its contact

Locations drop a site that lacks any of facility, city or country. Contacts keep it. So the output carries a contact for a site that, according to the same output, does not exist. When the incomplete site was the trial's only site, the trial reports no locations at all while still listing someone to call.

Verified in `src/transform/trial.rs` on 2026-09-02 against `0.9.0-dev.6`.

## Required behavior

A site appears in locations and in contacts consistently. A site is either present in both or absent from both.

A site the provider described incompletely is still a site, and what is known about it is reported.

## Why this is a draft

The conformance case needs a recorded provider payload that does not exist in `testdata/sources/` yet: a CTGov trial carrying a site that is missing a facility or a country.

ADR 0017 requires fixtures recorded from the provider rather than hand-written, and defect 17 is what happens when that rule is broken. This ticket waits on a decision Ian owns: who records the missing payloads, and whether both projects share one recorded set. Promote it once that payload exists.
## Done, observably

- No trial reports a contact for a site absent from its locations.
- A trial whose only site is incompletely described does not report zero locations.

## Where correct behavior is written

`repos/biodata/sdlc/planning/clinical-trial-conformance/cases.json`, case 12. Take the assertion from that case, write it as a failing test, then fix. Do not copy the expected behavior into this repository as a second statement of it.

Reported by the BioData lead in `notes/biomcp/feedback/2026-09-02-seventeen-trial-defects-to-fix-in-0-9.md`, defect 12 of seventeen.
## Boundary

Do not change site ordering or which sites the provider returns.
