---
flow: build
priority: 4
---

# Contacts survive a truncation that drops the sites they belong to

Two surfaces cut the location list and leave the contact list whole, so a caller is given someone to phone at a site the same output does not list.

`templates/trial.md.j2:80` renders `{% for loc in locations[:20] %}`, capped at twenty with no overflow line. Line 46 renders `{% for contact in contacts %}` uncapped, and each contact prints a `- Location:` line naming its facility, city, state and country. Any trial with more than twenty sites can print contacts for sites its own Locations table never showed.

`paginate_trial_locations` at `src/cli/trial/dispatch.rs:389-405` slices `trial.locations` to an offset and a limit and does not touch `trial.contacts`. A request for locations twenty through thirty returns every contact for sites zero through nineteen beside it.

Measured against the live provider on 2026-09-03: of 600 recruiting studies, 26 carry more than twenty locations, and **16 of those have site contacts past the cap**. NCT04796350 has 66 locations with 34 contacted sites beyond the cutoff.

Verified against `0.9.0-dev.6`.

## Required behavior

When a surface shows only some of a trial's sites, the contacts it shows belong to the sites it showed.

A reader is never given a contact for a site the same output withheld.

## Correct behavior

Locations and contacts presented together describe the same set of sites. Truncating one truncates the other, and a truncation says that it happened.

Write that as a failing test, then fix. Red before green.

This is the presentation half of the rule case 12 of the clinical-trial conformance cases states, recorded in the sibling BioData repository. **An attempt cannot read that repository**, so the statement above is this ticket's own authoritative copy. Case 12's conversion half is ticket 1121. If this looks wrong, stop and say so rather than implementing something different.

## Why this is not part of 1121

1121 fixes the conversion, where a site was dropped for being incompletely described. After it lands, `from_ctgov_study` emits one site list that both paths derive from, and the conversion no longer orphans anything.

These two surfaces orphan contacts by truncating, which is a different mechanism at a different layer, and it is live today independently of 1121. 1121 does make it more visible: NCT00791778 goes from rendering zero locations to rendering twenty of fifty-nine, so that trial becomes an instance of this defect the moment 1121 lands.

## Done, observably

- A trial with more than twenty sites does not render a contact for a site absent from its rendered locations table.
- A paginated request for a range of locations does not return contacts belonging to sites outside that range.
- A truncated location list says it was truncated, rather than presenting twenty of fifty-nine sites as though that were all of them.

## What this replaces

The current uncapped contacts rendering is being replaced deliberately. Assertions pinning it in the markdown render tests and the CLI location tests are expected to be restated.

Whether the fix filters contacts to the shown sites, caps both lists together, or lifts the cap is design's call. This ticket requires only that the two agree and that a cut is disclosed.

## Boundary

Do not change which sites the conversion produces. That is 1121.

Do not change contact ordering or which contact is treated as primary.
