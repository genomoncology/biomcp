---
flow: build
priority: 2
deps: []
---

# A cell-line entity resolves names to Cellosaurus accessions

BioMCP carries a `cell-line` entity backed by Cellosaurus. `biomcp search cell-line <name>` normalizes any common spelling and ranks exact identifier matches ahead of synonym-only matches and human lines ahead of other species. `biomcp get cell-line <CVCL_xxxx>` prints one card with name, synonyms, species, disease, category, sex, and age, plus `xrefs` and `variants` sections. `get cell-line` also accepts a DepMap, Cell Model Passports, ChEMBL, or PharmacoDB identifier and resolves it through the Cellosaurus `dr:` field. Every output names the Cellosaurus release read from `/release-info` and carries the CC BY 4.0 attribution line with the requested citation. The accession is the join key the later cell-line tickets use.

## Evidence

The work landed on `origin/main` and gated green on the build host at commit `37336c22`, which also carries the spec-runner and release-test repairs the entity needed. `make lint` passed. `make test` passed with 3,654 of 3,654 Rust tests and 952 Python contracts passing with three skips. `make spec` passed across 30 pages and 41 Python contracts. Recorded Cellosaurus fixtures under `testdata/sources/cellosaurus/` answer every test and every spec block, so no lane touched the network.

## Boundary

The ticket added no pivot commands, no STR profile, HLA typing, child-line or publication fields, no species or disease filter on search, and no rule that picks one line among several exact matches. Typed MCP `search` still rejects `cell-line`; the raw tool reaches it. Drug response, dependency, expression, and ChEMBL sections belong to tickets 1205, 1206, 1213, and 1214.
