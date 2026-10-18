---
priority: 3
---

# 2217 — Correct the Reactome database license summary

Status: COMPLETE.

Milestone: 0.9.4

Reviews: revision 8a022cd, accept

Reviews: revision d0cc62e, accept

## Outcome

Reactome's source terms distinguish CC0 database data and derived files from CC BY4 illustrations/art/branding and the separately licensed software, matching the official license for BioMCP's actual integration.

- Evidence: 2204 review and coordinator independent verification of https://reactome.org/license section1(c): all database data and files derived from it are CC0. Current registry describes pathway content as CC BY4 with only some exports CC0. Sections1(a) and1(b) cover art and software separately. PM ruled priority3 in0.9.4.
- Changes: `docs/reference/sources.json` `docs/reference/source-licensing.md` `CHANGELOG.md`. Preserve source availability and avoid adding commercial restrictions to CC0 data. Use existing registry/document checks, no test duplicating prose.
- Proof: compare current text with the official license and actual source calls, make the narrow correction, and run existing source registry/licensing checks through a build job. Fresh review and normal landing follow.

## Progress

- 2026-10-18 started

Reproduction and reviewed outcome: current sources.json and licensing table call pathway content CC BY4 with only some exports CC0. Read-only rg of the Reactome registry entry and table reproduces the incorrect wording; actual integration only fetches ContentService JSON/data endpoints. Official license sections1(a)-(c), verified2026-10-18, distinguish CC0 database/derived data from CC BY4 art and separately licensed software. Make that narrow correction, leave commercial_terms/source availability unchanged, and use the existing licensing contract checks without a redundant prose test.
