---
flow: build
priority: 5
---

# Show FDA orphan designations in drug regulatory results

## Goal

Drug regulatory results include FDA orphan designations and keep designation separate from marketing approval. On 2026-09-04, `biomcp --json get drug eflornithine regulatory` returned Drugs@FDA, EMA, and WHO data but omitted FDA's March 11, 2024 orphan designation for eflornithine hydrochloride in Bachmann-Bupp syndrome. The FDA record states that the orphan indication has not received approval. The reproduction, public-service evidence, and source limits came from `sdlc/issues/2026-09-04-drug-regulatory-data-omits-orphan-designations.md` in commit `f8ff2a78`.

## Desired functionality

The drug regulatory section returns matching FDA orphan designations with their indication, designation date, sponsor, current approval state, available approval or exclusivity facts, and exact FDA source link. Every output clearly distinguishes designation from marketing approval. Source status distinguishes unavailable acquisition from a confirmed search with no matching designation.

## Success criteria

- Eflornithine regulatory output includes the Bachmann-Bupp syndrome orphan designation dated March 11, 2024.
- The result states that FDA designated the orphan use and has not approved that use.
- A drug with an approved orphan indication shows designation and approval as separate dated facts.
- Human-readable, JSON, and MCP output link to the exact FDA designation record.
- Source health distinguishes unavailable data from a successful search with no match.
- Existing Drugs@FDA approval, label, EMA, and WHO results retain their meaning.

## Boundaries

This ticket adds public FDA orphan-designation records to drug regulatory results. It does not treat designation as approval, infer clinical appropriateness, change drug labels, or add non-FDA orphan programs.
