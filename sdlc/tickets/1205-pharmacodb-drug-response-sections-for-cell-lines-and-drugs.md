---
flow: build
priority: 4
deps: [1202]
---

# 1205: PharmacoDB drug response sections for cell lines and drugs

## Goal

`biomcp get cell-line <accession> drug_response` and `biomcp get drug <name> cell_lines` print how many PharmacoDB experiments exist per dataset (GDSC1, GDSC2, CTRPv2, PRISM, gCSI, and the rest). `biomcp drug cell-lines <name> --cell-line <id>` prints the published summary metrics (AAC, IC50, EC50, Einf, HS, DSS1) for one drug on one cell line. Row listings for a whole drug or a whole cell line need a `--dataset` filter. PharmacoDB is BioMCP's route to GDSC data. BioMCP reports the numbers as published. It never ranks, thresholds, or labels a cell line as sensitive or resistant. The motivating consumer is a hackathon team screening public GEO studies of drug-treated AML cell lines. That team needs to see which screens tested a drug on a given line.

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

Survey of 2026-09-17 (workspace experiment 204) over ten AML and leukemia test lines:

- Common inputs exceed the 8 MiB body cap (`DEFAULT_MAX_BODY_BYTES`, `src/sources/mod.rs:338`). `experiments` for K-562 returned 78,373 rows, 19.4 MB, in 9 s. Doxorubicin returned 144,704 rows, 18.6 MB. The other test lines returned 520 to 1,199 rows (108 to 246 KB). Venetoclax, cytarabine, quizartinib, and paclitaxel returned 3,608, 10,832, 2,627, and 8,255 rows (0.3 to 1.4 MB). One drug on one line returned 0 to 3 rows, under 1 KB.
- `read_limited_body_with_limit(resp, api, max_bytes)` (`src/sources/mod.rs:1586`) reads a body under a caller-chosen cap.
- No experiment row carries a unit. MOLM-13 alone has 118 repeated compound-and-dataset pairs and 241 rows with no IC50.
- No test line has PRISM rows in PharmacoDB, although the PRISM 20Q2 release lists MOLM-13, THP-1, U-937, K-562, NB4, and OCI-AML3. Gilteritinib has only PRISM and NCI60 rows, and none of them are MOLM-13.
- `cell_line(cellUID)` returned an `accession_id` that matched Cellosaurus for all ten test lines. The list query `cell_lines` cannot return `accession_id`. PharmacoDB names match the Cellosaurus names (`OCI-AML-3`, `MOLM-13`), while `cellName: "OCI-AML3"` fails.
- HL-60(TB) is PharmacoDB id 1228, uid `HL-60(TB)6502021_`, accession CVCL_A794. Cellosaurus has no PharmacoDB link for CVCL_A794, so a join through the Cellosaurus link alone misses it. PharmacoDB also has 17 NCI60 rows for NB4.
- `cancerrxgene.org` returns HTTP 410 and points users to Cell Model Passports. The GDSC release 8.5 bulk files (2023) remain on `cog.sanger.ac.uk`. PharmacoDB carries GDSC1 and GDSC2.

Terms: corrected on 2026-09-18, see "Licence findings, 2026-09-18" below. PharmacoDB publishes no licence or terms page. Its source code is GPL-3.0 and the paper describing it is CC BY-NC 4.0; the terms for the data itself are unstated by the provider. Ian ruled on 2026-09-16 that the non-commercial term does not block an open-source, non-commercial BioMCP. The site is a JavaScript app, so a plain fetch of `/about` shows no license text. The implementer confirms the terms URL in a browser and records it with `reviewed_on`. Citation: Feizi N, et al. PharmacoDB 2.0. Nucleic Acids Research 2022;50(D1):D1348-D1357, doi:10.1093/nar/gkab1084, PMID 34850112.

## Design

### Source module

`src/sources/pharmacodb.rs` with `PHARMACODB_BASE = "https://pharmacodb.ca"`, `BIOMCP_PHARMACODB_BASE` through `env_base`, and `shared_client`. It exposes three queries and nothing else:

- `cell_line_by_uid(uid) -> Option<PharmacoCellLine { id, uid, name, accession_id }>`
- `compound_by_name(name) -> Option<PharmacoCompound { id, uid, name }>`
- `cell_line_by_name(name) -> Option<PharmacoCellLine>`, the same query with `cellName`.
- `experiment_counts(filter: CellLineId(i64) | CompoundId(i64)) -> Vec<(dataset, count)>`, sent with `all: true` and only `id` and `dataset { name }`.
- `experiments(filter: CellLineId(i64) | CompoundId(i64) | Pair(cell_line_id, compound_id)) -> Vec<PharmacoExperiment>`, sent with `all: true` and the fields listed above except `DSS2` and `DSS3`.

The upstream "Please provide a valid ..." error maps to `None`. Any other GraphQL error or transport failure is a source error.

### Size guard

- Every `experiments` and `experiment_counts` request reads its body through `read_limited_body_with_limit` with a 32 MiB cap. That cap sits above the largest observed body (K-562, 19.4 MB with all fields). A larger body makes the section `unavailable` with the message `PharmacoDB response exceeds 32 MiB`.
- Sections never list rows. They call `experiment_counts` only.
- Rows come from `experiments` only for a drug plus cell line pair, or for one side with a `--dataset` filter. The client filters by dataset after the fetch because the API has no dataset argument.

### Joins

- Cell line: take the PharmacoDB cross-reference from the 1202 Cellosaurus record and call `cell_line_by_uid`. With no cross-reference, call `cell_line_by_name` with the Cellosaurus name. Accept a record only when `accession_id` equals the requested CVCL accession. The name fallback reaches HL-60(TB) (CVCL_A794). When neither call finds a record, the outcome is `empty` with the message `no PharmacoDB cell line for this accession`. An accession mismatch gives `unavailable` with the message `PharmacoDB accession does not match`. The accession check stops a name fallback from joining the wrong line.
- Drug: call `compound_by_name` with the resolved `drug.name`. If that misses and the requested name differs, call it once with the requested name. Accept a result only when its `name` equals the query ignoring ASCII case. A miss gives `empty` with the message `no PharmacoDB compound with this name`.

### Sections

- Drug: add `DRUG_SECTION_CELL_LINES = "cell_lines"` to `DRUG_SECTION_NAMES` and to `parse_sections_for_name`, with a `cell_lines` section outcome key. `all` does not include it, following `approvals`. The name follows the existing noun sections (`targets`, `indications`) and names what each row is.
- Cell line: add `drug_response` to the 1202 section list and one key of the same name to the `section_outcomes` registry that ticket 1202 builds. The 1202 `all` expansion does not include it.
- Each section returns `total` (all experiments) and `datasets` (count per dataset name, sorted by name). It lists no rows. It ends with next commands for the helper: `biomcp drug cell-lines <drug> --cell-line <ac>` and `biomcp drug cell-lines <drug> --dataset <name>` on the drug card, and `biomcp cell-line drug-response <ac> --dataset <name>` on the cell line card.
- Helpers, following `gene pathways` under `GeneCommand`:
  - `biomcp drug cell-lines <drug> --cell-line <id>`: one pair request, 0 to 3 rows in practice. The `<id>` takes any form `get cell-line` accepts.
  - `biomcp drug cell-lines <drug> --dataset <name>`: rows for that dataset. `--cell-line` or `--dataset` is required. With neither, the command fails before any request and prints the counts command.
  - `biomcp cell-line drug-response <ac> --dataset <name>`: rows for that dataset. `--dataset` is required. This adds a `CellLineCommand` group next to `DrugCommand` (`src/cli/drug/mod.rs:78`).
  - The dataset filter matches the PharmacoDB dataset name ignoring ASCII case. An unknown name fails and lists the ten dataset names. `--limit` is 1 to 100 with a default of 25, and `--offset` defaults to 0.
- **MCP arms.** `is_allowed_mcp_command` (`src/mcp/shell.rs:472`) is an exhaustive match with no wildcard, so every new variant is classified here or the build fails. `DrugCommand::CellLines` is allowed. The new `Commands::CellLine` group and its `CellLineCommand::DrugResponse` arm are allowed. Each makes bounded read-only requests and reveals no local path, and each is reachable through the raw tool the way `gene pathways` is. `GENERIC_MCP_REJECTION_MESSAGE` (`src/mcp/shell.rs:326`) names study commands only and is unchanged.
- **Release date.** PharmacoDB publishes no version, no release name, and no release date. The GraphQL schema exposes none, and the survey of 2026-09-17 found none. Every output therefore carries `data_as_of` set to the retrieval time and `data_as_of_kind: "retrieved"`. The docs say plainly that PharmacoDB publishes no version, so a repeat query can return different numbers with no way to tell.
- **Bot checks.** PharmacoDB served no bot check in any measurement. The rule still holds: an HTTP 200 whose body is an HTML human-verification page where a GraphQL JSON body was expected is a provider error. It names the URL and the operation, and the command stops. BioMCP never retries through a check and never scrapes the page.
- Rows are sorted by counterpart name (compound for a cell line, cell line for a drug), then dataset name, then experiment id. Rows are never sorted by a metric. Repeated compound-and-dataset pairs stay as separate rows, and each row shows its experiment id so the repeat is visible. BioMCP does not merge or average them.
- Each helper result returns `total`, `datasets`, and one page of rows. Each row holds `experiment_id`, `dataset`, the counterpart `name` and `uid` (and `tissue` for the drug section), and `aac`, `ic50`, `ec50`, `einf`, `hs`, `dss1` as published. A null metric stays null in JSON and prints as `-`. No unit conversion, no rounding in JSON, and Markdown prints the value with up to four significant digits.
- JSON shape: `drug.cell_lines` and `cell_line.drug_response` hold `{ "source": "PharmacoDB", "pharmacodb_id": ..., "total": N, "datasets": [{"name", "count"}], "data_as_of": "<retrieval time>", "data_as_of_kind": "retrieved" }`. The helpers add `"filter"` and `"rows": [...]`.
- Markdown: a `## Drug response (PharmacoDB)` heading on the cell line card and a `## Cell lines (PharmacoDB)` heading on the drug card, then a `N experiments: GDSC1 426, ...` line, and the next commands. The helpers print a table with columns `Experiment | Dataset | Compound or Cell line | AAC | IC50 | EC50 | Einf | HS | DSS1` and a `Showing X of N` line when truncated. Every output prints one fixed attribution line: `Values as published by PharmacoDB. PharmacoDB publishes no licence or terms page; its source code is GPL-3.0 and the describing paper is CC BY-NC 4.0, and the terms for the data itself are unstated by the provider; treat reuse as non-commercial. PharmacoDB publishes no version; retrieved <data_as_of>. PharmacoDB gives no units; BioMCP does not interpret sensitivity.` (reworded on 2026-09-18 so the line asserts no licence the provider never published) The non-commercial term is named in the line itself, because a user cannot tell it from the data.

### Docs and inventory

- `docs/sources/pharmacodb.md` in the `civic.md` shape: what BioMCP exposes, the sections and helpers, no key, official source, terms, and the citation. It says that PharmacoDB rows carry no units, that repeated experiments stay separate, and that the GDSC site returns 410 and PharmacoDB is BioMCP's route to GDSC1 and GDSC2 data. Its examples use venetoclax on MOLM-13. No example implies PRISM rows for a test line. `docs/sources/index.md` gains a row.
- `docs/reference/source-licensing.md` row and `sources.json` entry: tier 3 (non-commercial term), `direct_api`, auth `none`, both surfaces.
- `docs/reference/configuration.md` lists `BIOMCP_PHARMACODB_BASE` as a fixture seam.
- `src/cli/health/catalog.rs` gains a PharmacoDB `PostJson` probe (`{ datasets { id } }`) that affects the drug `cell_lines` and cell line `drug_response` sections.
- `docs/user-guide/drug.md` section list, the 1202 cell line guide, `src/cli/list_reference.md`, and `docs/user-guide/cli-reference.md` list the new sections.

## Fixtures

`testdata/sources/pharmacodb/`, recorded 2026-09-16 and trimmed:

- `cell_line_molm13.json`: the `cell_line` response for `MOLM13_950_2019`.
- `cell_line_accession_mismatch.json`: the same record with a different `accession_id`.
- `compound_venetoclax.json` and `compound_not_found.json` (the upstream error body).
- `cell_line_hl60tb_by_name.json`: the `cell_line(cellName: "HL-60(TB)")` response with accession CVCL_A794.
- `experiments_cell_line_1248.json`: 30 rows over GDSC1, GDSC2, CTRPv2, and gCSI, with at least one null IC50 and at least one repeated compound-and-dataset pair, so truncation at 25 is exercised.
- `experiment_counts_cell_line_1248.json`: the counts projection for the same 30 rows.
- `experiments_compound_53572.json`: 6 rows over GDSC1, GDSC2, PRISM, CTRPv2, and NCI60, including the MOLM-13 GDSC1 row with `IC50: null`. The PRISM row is on a line outside the ten test lines.
- `experiments_pair_53572_1248.json`: the two MOLM-13 venetoclax rows.

The K-562 and doxorubicin bodies are too large to commit. Tests generate synthetic bodies instead: a counts body of 78,373 rows, a full-field body of 19.4 MB matching the measured K-562 case, and a hypothetical full body over 32 MiB.

A spec fixture script `spec/fixtures/setup-pharmacodb-spec-fixture.sh` with a matching cleanup script serves these files on loopback, keyed by GraphQL operation and argument, and exports `BIOMCP_PHARMACODB_BASE`. The same script serves or reuses the 1202 Cellosaurus fixture for MOLM-13. No test touches the network.

## Acceptance

Focused Rust tests (fixture-backed):

1. The source client parses each fixture, maps the "valid cell ID" error to `None`, and sends `all: true` with the id argument, never a name argument, to `experiments`. The counts request asks for `id` and `dataset { name }` only.
2. `get cell-line CVCL_2119 drug_response` returns `total == 30` and dataset counts in name order, lists no rows, and sends no full-field `experiments` request.
3. `cell-line drug-response CVCL_2119 --dataset GDSC1` returns rows sorted by compound name, dataset, and id, keeps both rows of the repeated pair, and prints null metrics as JSON null.
4. The accession mismatch fixture gives an `unavailable` outcome and makes no `experiments` request. A Cellosaurus record without a PharmacoDB cross-reference falls back to `cell_line_by_name`. The HL-60(TB) fixture joins to CVCL_A794.
5. `get drug venetoclax cell_lines` returns `total == 6` with no rows. `drug cell-lines venetoclax --cell-line CVCL_2119` sends one pair request and returns two rows. `drug cell-lines venetoclax` with no filter fails before any request. An unknown compound gives `empty`.
6. Size guard: a synthetic 78,373-row counts body (the K-562 case) parses and sums to 78,373. A synthetic 19.4 MB full-field body, the largest measured input, parses and returns its rows. A synthetic body over 32 MiB, which no measured input reaches, gives `unavailable` with the size message and no panic.
7. `get drug venetoclax all` and `get cell-line CVCL_2119 all` make no PharmacoDB request, and their output is byte-identical to the output before this ticket.
8. An unknown section error lists `cell_lines` for drugs. An unknown `--dataset` lists the ten dataset names.
9. The Markdown renders pin the heading, the counts line, the next commands, a helper row with `-` for a null value, the `Showing 25 of 30` line, and the fixed attribution line with the non-commercial term, the retrieval time, and the no-units note.
10. Every section and helper payload carries `data_as_of` and `data_as_of_kind: "retrieved"`, with the time supplied by an injected clock so the test pins an exact string.
11. An HTML body served with HTTP 200 for a GraphQL request is a provider error naming the URL and the operation, and no row renders.

Executable specs: `spec/entity/drug.md` gains one JSON block (`get drug venetoclax cell_lines --json`: `cell_lines.total == 6` and the first dataset count), one JSON block (`drug cell-lines venetoclax --cell-line CVCL_2119 --json`: two rows), and one Markdown block (the heading and the fixed line). The 1202 cell line spec page gains the same pair for `drug_response`.

`make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA. The docs parity checks accept the new source page, licensing row, `sources.json` entry, and configuration row.

## Out of scope

- Filters by tissue, and row listings without a pair or dataset filter.
- Dose-response curves, biomarker associations, molecular profiles, and PharmacoDB gene or tissue queries.
- Any ranking, threshold, sensitivity label, or aggregation across datasets beyond the count per dataset.
- Name-based cell line matching beyond the Cellosaurus-name fallback, and PubChem or ChEMBL matching for compounds.
- DepMap, PRISM downloads, GDSC direct access, and LINCS. PharmacoDB already serves PRISM and GDSC, so no separate PRISM or GDSC ticket follows.
- New MCP tools. The generic `get` tool reaches both sections through its section list. The helpers reach MCP through the raw tool.

## Decisions

These follow the 2026-09-17 source survey. Ian can overturn any of them.

- Sections print counts only. Rows need a pair or a dataset filter because K-562 and doxorubicin are over 18 MB.
- The experiments body cap is 32 MiB for PharmacoDB only. The shared 8 MiB default stays for every other source. The cap sits above every measured input: the largest are K-562 at 19.4 MB and doxorubicin at 18.6 MB, and both must parse. No measured input exceeds the cap.
- The cell line join falls back to the Cellosaurus name and always checks the accession.
- Repeated experiments stay as separate rows.
- `data_as_of` is the retrieval time, because PharmacoDB publishes no version. The output says so rather than leaving the field out.

## Complexity

- Contract score: 2 (two count sections, two row helpers, one fixed JSON shape, explicit empty, mismatch, and size cases)
- State and timing score: 0 (bounded read-only requests through the shared cache)
- Reach score: 2 (new source module, drug and cell line entities, health, docs inventory)
- Proof score: 1 (recorded fixtures, request assertions, byte-identical `all`)
- Cost of error score: 1 (a wrong join would show another line's numbers, and the accession check guards it)
- Total: 6
- Minimum level floor: none
- Final level: 3
- Reasons: new external source with a strict identity join and an untrusted upstream pagination contract

## Review

- Design review: covered by the 2026-09-17 ticket review. Its findings were applied before the build.
- Code review: none. The work shipped on 2026-09-19 and passed lint, test and spec on the build host at `3b1d6452`. The completion record is `sdlc/records/1205-pharmacodb-drug-response-sections-for-cell-lines-and-drugs.md`.

## Licence findings, 2026-09-18

The original Terms sentence above claimed CC BY-NC 4.0 for the PharmacoDB API and data. That claim does not hold, and it is corrected in place.

- `https://pharmacodb.ca/` serves a single-page app. Every path, including `/about`, `/documentation`, `/terms` and `/api`, returns the identical 2,258-byte HTML shell with HTTP 200. There is no server-rendered terms page to read.
- The app's JavaScript bundles contain no licence string: no "CC BY", no "Creative Commons", no "license" text that names terms for the data.
- The CC BY-NC 4.0 in the original ticket text traces to the journal article, Feizi N, et al. PharmacoDB 2.0, Nucleic Acids Research 2022;50(D1):D1348-D1357, doi:10.1093/nar/gkab1084, which is published under CC BY-NC 4.0. That licence covers the paper, not the database.
- The only licence the project itself publishes is for its source code: GPL-3.0, at `https://github.com/bhklab/PharmacoDB/blob/master/LICENSE`.
- Conclusion recorded in both registries with `reviewed_on: 2026-09-18`, tier 3, and `terms_url: null`: PharmacoDB publishes no licence or terms page, its source code is GPL-3.0, the describing paper is CC BY-NC 4.0, and redistribution terms for the data itself are unstated by the provider. The user-facing attribution line says the same and asks the reader to treat reuse as non-commercial, because a user cannot tell that from the data.
- `the_license_agrees_across_the_attribution_line_and_both_registries` in `src/entities/pharmacodb/tests.rs` pins that the attribution line, `docs/reference/sources.json` and `docs/reference/source-licensing.md` carry one value and that no bare `CC BY-NC 4.0` row returns.
