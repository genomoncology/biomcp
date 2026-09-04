---
flow: build
priority: 6
---

# Find diagnostic tests through known disease synonyms

## Goal

A diagnostic search uses disease names that BioMCP already recognizes and finds tests filed under an equivalent name. On 2026-09-04, BioMCP resolved Bachmann-Bupp syndrome to `MONDO:0033642` and knew its synonyms, but `biomcp search diagnostic --disease "Bachmann-Bupp syndrome" --source gtr --limit 5` returned zero results. The same GTR data returned `GTR000596648.2` through its longer condition name and through `--gene ODC1`. The reproduction and code evidence came from `sdlc/issues/2026-09-04-diagnostic-disease-search-does-not-use-known-synonyms.md` in commit `f8ff2a78`.

## Desired functionality

BioMCP matches a diagnostic disease query against the requested name, the resolved disease name, and exact known synonyms. Results identify the term that matched. A failed or unavailable disease lookup remains distinguishable from a confirmed empty diagnostic search. The search does not broaden into disease ancestors or loosely related concepts.

## Success criteria

- The Bachmann-Bupp syndrome command returns `GTR000596648.2`.
- Searching the longer GTR condition name keeps returning the same test.
- Structured output identifies the disease name or synonym that matched.
- An unavailable disease resolver does not produce a false confirmed zero.
- An unrelated disease synonym does not admit the Bachmann-Bupp test.
- Existing gene, test-type, manufacturer, and WHO IVD filters retain their behavior.

## Boundaries

This ticket adds exact disease-name and synonym matching to diagnostic discovery. It does not add ontology-parent expansion, fuzzy disease matching, diagnostic interpretation, or a new GTR ingestion system.
