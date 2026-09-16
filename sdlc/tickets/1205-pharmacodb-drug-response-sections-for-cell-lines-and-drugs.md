---
flow: build
priority: 4
deps: [1202]
---

# 1205: PharmacoDB drug response sections for cell lines and drugs

## Goal

`biomcp get cell-line <accession> drug_response` and `biomcp get drug <name> cell_lines` print the published PharmacoDB summary metrics (AAC, IC50, EC50, Einf, HS, DSS1) for each experiment, labeled with the PharmacoDB dataset name (GDSC1, GDSC2, CTRPv2, PRISM, gCSI, and the rest). BioMCP reports the numbers as published. It never ranks, thresholds, or labels a cell line as sensitive or resistant. The motivating consumer is a hackathon team screening public GEO studies of drug-treated AML cell lines. That team needs to see which screens tested a drug on a given line.

## Current Facts

- No code, doc, or fixture mentions PharmacoDB today (`grep -rni pharmacodb src docs spec testdata` returns nothing).
- Drug sections are constants in `src/entities/drug/mod.rs:610-633` (`DRUG_SECTION_NAMES` at `:621`). `parse_sections_for_name` (`src/entities/drug/get.rs:53`) maps each token to an `include_*` flag. The `all` expansion (`get.rs:97-106`) leaves out `approvals`, so an opt-in section outside `all` already has precedent.
- Section outcomes use `SectionOutcome::{data, empty, unavailable, inapplicable}` (`src/entities/section_outcome.rs:39-88`) and `drug.section_outcomes.complete(key, outcome)` (`get.rs:202`, `get.rs:765`).
- `Drug` (`src/entities/drug/mod.rs:45`) carries `name`, `chembl_id`, `drugbank_id`, and `unii`, and no PubChem id. The join to PharmacoDB therefore goes by name.
- GraphQL clients follow the CIViC shape: a const base plus an env override (`src/sources/civic.rs:13-15`), `env_base` (`src/sources/mod.rs:431`), `shared_client` (`src/sources/mod.rs:1167`), and a module line in `src/sources/mod.rs:263`. `shared_client` enforces an 8 MiB default body cap (`architecture/technical/source-integration.md`, shared client section).
- Each source gets a guide in `docs/sources/` linked from `docs/sources/index.md:14-40`, a row in `docs/reference/source-licensing.md` (for example `:47`), an entry in `docs/reference/sources.json` (for example `:143`), and an API health row in `src/cli/health/catalog.rs` (CIViC GraphQL probe at `:302-308`). Fixture seam variables are listed in `docs/reference/configuration.md:67-75`.
- The `cell-line` entity, its Cellosaurus client, and its cross-reference list come from ticket 1202. Cellosaurus publishes a PharmacoDB cross-reference. Observed 2026-09-16: `https://api.cellosaurus.org/cell-line/CVCL_2119?format=txt` contains `DR   PharmacoDB; MOLM13_950_2019`.

PharmacoDB API, observed 2026-09-16 by `curl -X POST https://pharmacodb.ca/graphql`:

- The endpoint is GraphQL at `https://pharmacodb.ca/graphql` with no key. Introspection lists `cell_line(cellId, cellName, cellUID)`, `compound(compoundId, compoundName, compoundUID)`, `experiments(cellLineId, cellLineName, compoundId, compoundName, tissueId, tissueName, page, per_page, all)`, and `datasets`.
- `cell_line(cellUID: "MOLM13_950_2019")` returns `{id: 1248, name: "MOLM-13", accession_id: "CVCL_2119"}`. `cell_line` accepts no accession argument, and `cellName: "MOLM13"` fails with `Please provide a valid cell ID, Name or UID.`
- `compound(compoundName: "venetoclax")` matches case-insensitively and returns `{id: 53572, uid: "PDBC02030", name: "Venetoclax"}` under a `compound` wrapper.
- `Experiment` has `id`, `cell_line {id uid name}`, `compound {id uid name}`, `dataset {name}`, `tissue {name}`, and `profile {AAC IC50 EC50 Einf HS DSS1 DSS2 DSS3}`. Profile fields are nullable floats.
- `experiments(cellLineName: ...)` and `experiments(compoundName: ...)` fail with an upstream SQL error (`Unknown column 'cell.name'`). The id arguments work.
- `per_page` is ignored with an id argument: `experiments(compoundId: 53572, per_page: 5)` returned 3608 rows. With `all: true`, venetoclax returned 3608 rows (GDSC2 1052, GDSC1 981, PRISM 926, CTRPv2 536, NCI60 113), about 690 KB with the fields above. `experiments(cellLineId: 1248, all: true)` returned 1117 rows (GDSC1 426, CTRPv2 416, GDSC2 240, gCSI 35).
- `experiments(compoundId: 53572, cellLineId: 1248)` returned two rows: GDSC1 with `AAC: 0, IC50: null` and GDSC2 with `AAC: 0.80548938, IC50: 0.00284989`.
- `datasets` returns CCLE, CTRPv2, FIMM, GDSC1, GDSC2, GRAY, NCI60, PRISM, UHNBreast, gCSI.

Terms: PharmacoDB API and data are CC BY-NC 4.0 and its code is GPL-3.0. Ian ruled on 2026-09-16 that the non-commercial term does not block an open-source, non-commercial BioMCP. The site is a JavaScript app, so a plain fetch of `/about` shows no license text. The implementer confirms the terms URL in a browser and records it with `reviewed_on`. Citation: Feizi N, et al. PharmacoDB 2.0. Nucleic Acids Research 2022;50(D1):D1348-D1357, doi:10.1093/nar/gkab1084, PMID 34850112.

## Design

### Source module

`src/sources/pharmacodb.rs` with `PHARMACODB_BASE = "https://pharmacodb.ca"`, `BIOMCP_PHARMACODB_BASE` through `env_base`, and `shared_client`. It exposes three queries and nothing else:

- `cell_line_by_uid(uid) -> Option<PharmacoCellLine { id, uid, name, accession_id }>`
- `compound_by_name(name) -> Option<PharmacoCompound { id, uid, name }>`
- `experiments(filter: CellLineId(i64) | CompoundId(i64)) -> Vec<PharmacoExperiment>`, sent with `all: true` and the fields listed above except `DSS2` and `DSS3`.

The upstream "Please provide a valid ..." error maps to `None`. Any other GraphQL error or transport failure is a source error. The shared 8 MiB body cap stays in force. A larger body makes the section `unavailable`.

### Joins

- Cell line: take the PharmacoDB cross-reference from the 1202 Cellosaurus record, call `cell_line_by_uid`, and accept the record only when `accession_id` equals the requested CVCL accession. No PharmacoDB cross-reference gives `empty` with the message `no PharmacoDB cross-reference in Cellosaurus`. An accession mismatch gives `unavailable` with the message `PharmacoDB accession does not match`. Name search is not used.
- Drug: call `compound_by_name` with the resolved `drug.name`. If that misses and the requested name differs, call it once with the requested name. Accept a result only when its `name` equals the query ignoring ASCII case. A miss gives `empty` with the message `no PharmacoDB compound with this name`.

### Sections

- Drug: add `DRUG_SECTION_CELL_LINES = "cell_lines"` to `DRUG_SECTION_NAMES` and to `parse_sections_for_name`, with a `cell_lines` section outcome key. `all` does not include it, following `approvals`. The name follows the existing noun sections (`targets`, `indications`) and names what each row is.
- Cell line: add `drug_response` to the 1202 section list with a matching outcome key. The 1202 `all` expansion does not include it.
- Rows are sorted by counterpart name (compound for a cell line, cell line for a drug), then dataset name, then experiment id. Rows are never sorted by a metric.
- Each section returns `total` (all experiments), `datasets` (count per dataset name, sorted by name), and the first 25 rows. Each row holds `experiment_id`, `dataset`, the counterpart `name` and `uid` (and `tissue` for the drug section), and `aac`, `ic50`, `ec50`, `einf`, `hs`, `dss1` as published. A null metric stays null in JSON and prints as `-`. No unit conversion, no rounding in JSON, and Markdown prints the value with up to four significant digits.
- JSON shape: `drug.cell_lines` and `cell_line.drug_response` hold `{ "source": "PharmacoDB", "pharmacodb_id": ..., "total": N, "datasets": [{"name", "count"}], "rows": [...] }`.
- Markdown: a `## Drug response (PharmacoDB)` heading on the cell line card and a `## Cell lines (PharmacoDB)` heading on the drug card, then a `N experiments: GDSC1 426, ...` line, a table with columns `Dataset | Compound or Cell line | AAC | IC50 | EC50 | Einf | HS | DSS1`, a `Showing 25 of N` line when truncated, and one fixed line: `Values as published by PharmacoDB (CC BY-NC 4.0); BioMCP does not interpret sensitivity.`

### Docs and inventory

- `docs/sources/pharmacodb.md` in the `civic.md` shape: what BioMCP exposes, both commands, no key, official source, terms, and the citation. `docs/sources/index.md` gains a row.
- `docs/reference/source-licensing.md` row and `sources.json` entry: tier 3 (non-commercial term), `direct_api`, auth `none`, both surfaces.
- `docs/reference/configuration.md` lists `BIOMCP_PHARMACODB_BASE` as a fixture seam.
- `src/cli/health/catalog.rs` gains a PharmacoDB `PostJson` probe (`{ datasets { id } }`) that affects the drug `cell_lines` and cell line `drug_response` sections.
- `docs/user-guide/drug.md` section list, the 1202 cell line guide, `src/cli/list_reference.md`, and `docs/user-guide/cli-reference.md` list the new sections.

## Fixtures

`testdata/sources/pharmacodb/`, recorded 2026-09-16 and trimmed:

- `cell_line_molm13.json`: the `cell_line` response for `MOLM13_950_2019`.
- `cell_line_accession_mismatch.json`: the same record with a different `accession_id`.
- `compound_venetoclax.json` and `compound_not_found.json` (the upstream error body).
- `experiments_cell_line_1248.json`: 30 rows over GDSC1, GDSC2, CTRPv2, and gCSI, with at least one null IC50, so truncation at 25 is exercised.
- `experiments_compound_53572.json`: 6 rows over GDSC1, GDSC2, PRISM, CTRPv2, and NCI60, including the MOLM-13 GDSC1 row with `IC50: null`.

A spec fixture script `spec/fixtures/setup-pharmacodb-spec-fixture.sh` with a matching cleanup script serves these files on loopback, keyed by GraphQL operation and argument, and exports `BIOMCP_PHARMACODB_BASE`. The same script serves or reuses the 1202 Cellosaurus fixture for MOLM-13. No test touches the network.

## Acceptance

Focused Rust tests (fixture-backed):

1. The source client parses each fixture, maps the "valid cell ID" error to `None`, and sends `all: true` with the id argument, never a name argument, to `experiments`.
2. `get cell-line CVCL_2119 drug_response` returns `total == 30`, dataset counts in name order, 25 rows sorted by compound name, dataset, and id, and null metrics as JSON null.
3. The accession mismatch fixture gives an `unavailable` outcome and makes no `experiments` request. A Cellosaurus record without a PharmacoDB cross-reference gives `empty` and makes no PharmacoDB request.
4. `get drug venetoclax cell_lines` returns `total == 6` and rows sorted by cell line name. An unknown compound gives `empty`.
5. `get drug venetoclax all` and `get cell-line CVCL_2119 all` make no PharmacoDB request, and their output is byte-identical to the output before this ticket.
6. An unknown section error lists `cell_lines` for drugs.
7. The Markdown renders pin the heading, the counts line, a row with `-` for a null value, the `Showing 25 of 30` line, and the fixed source line.

Executable specs: `spec/entity/drug.md` gains one JSON block (`get drug venetoclax cell_lines --json`: `cell_lines.total == 6` and the first row's dataset) and one Markdown block (the heading and the fixed line). The 1202 cell line spec page gains the same pair for `drug_response`.

`make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA. The docs parity checks accept the new source page, licensing row, `sources.json` entry, and configuration row.

## Out of scope

- Paging past the first 25 rows, filters by dataset or tissue, and a `drug cell-lines` helper. A follow-up ticket may add them once the section is used.
- Dose-response curves, biomarker associations, molecular profiles, and PharmacoDB gene or tissue queries.
- Any ranking, threshold, sensitivity label, or aggregation across datasets beyond the count per dataset.
- Name-based cell line matching, and PubChem or ChEMBL matching for compounds.
- DepMap, PRISM downloads, GDSC direct access, and LINCS.
- New MCP tools. The generic `get` tool reaches both sections through its section list.

## Complexity

- Contract score: 1 (two new sections, one fixed JSON shape, explicit empty and mismatch cases)
- State and timing score: 0 (bounded read-only requests through the shared cache)
- Reach score: 2 (new source module, drug and cell line entities, health, docs inventory)
- Proof score: 1 (recorded fixtures, request assertions, byte-identical `all`)
- Cost of error score: 1 (a wrong join would show another line's numbers, and the accession check guards it)
- Total: 5
- Minimum level floor: none
- Final level: 3
- Reasons: new external source with a strict identity join and an untrusted upstream pagination contract

## Review

- Design review: pending
- Code review: pending
