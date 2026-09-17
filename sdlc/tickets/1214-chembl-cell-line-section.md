---
flow: build
priority: 5
deps: [1202]
---

# 1214: ChEMBL cell line section

## Goal

`biomcp get cell-line <accession> chembl` prints the ChEMBL cell line record that names this Cellosaurus accession: ChEMBL ID, EFO ID, CLO ID, and the number of ChEMBL assays run on the line. The count tells an agent whether ChEMBL literature assays exist for the line. BioMCP lists no assays. The motivating consumer is a hackathon team screening public GEO studies of drug-treated AML cell lines.

## Current Facts

- The ChEMBL client exists. `ChemblClient` (`src/sources/chembl.rs:14`) uses `shared_client()` and `env_base(CHEMBL_BASE, CHEMBL_BASE_ENV)` (`:20-25`), with `CHEMBL_BASE` `https://www.ebi.ac.uk/chembl/api/data` (`:11`) and `BIOMCP_CHEMBL_BASE` (`:12`). Request plans follow `drug_targets_plan` (`:27`) and `target_summary_plan` (`:40`). Responses decode through `decode_json_response` (`:50`) and `get_json` (`:63`). Nothing reads cell line records today.
- The drug entity calls the client from `src/entities/drug/targets.rs:26`. The health probe is the `ChEMBL` row in `src/cli/health/catalog.rs:441`. Parsing tests live in `src/sources/chembl/tests/parsing.rs`, and recorded fixtures in `testdata/sources/chembl/`.
- Ticket 1202 adds the `cell-line` entity and its section list. Ticket 1205 adds the first cell line section that calls another source.

### ChEMBL, observed 2026-09-17 (workspace experiment 204)

- `GET /cell_line.json?cellosaurus_id=CVCL_2119` returned one record in 424 bytes: `cell_chembl_id` CHEMBL3706573, `cell_name` MOLM-13, `cell_source_tax_id` 9606, `cellosaurus_id` CVCL_2119, `efo_id` null, `clo_id` null.
- The `cellosaurus_id` join matched all ten AML and leukemia test lines in both directions. Name filters do not: `cell_name__iexact` missed K-562, OCI-AML3, Kasumi-1, and NB4 because ChEMBL spells them `K562`, `Kasumi 1`, and `NB-4`.
- `GET /assay.json?cell_chembl_id=CHEMBL3706573&limit=1` returned about 1 KB with `page_meta.total_count`. Assay counts ran from 127 to 8,042 per test line (1,408 for MOLM-13). The assays are free-text literature records.
- Activity values sit on a separate cell-line target (CHEMBL3706572 for MOLM-13). No such target was found by name for K-562, OCI-AML-3, Kasumi-1, or NB4, so this ticket does not read targets.
- `status.json` reported ChEMBL_37, released 2026-05-01. Terms are CC BY-SA 3.0.

## Design

### Source

Add two calls to `ChemblClient`, each with a plan function:

- `cell_line_by_cellosaurus(accession) -> Option<ChemblCellLine { chembl_id, name, tax_id, efo_id, clo_id }>`: `cell_line.json?cellosaurus_id=<ac>`. Zero records give `None`. More than one record keeps all of them in ChEMBL order.
- `cell_line_assay_count(chembl_id) -> u64`: `assay.json?cell_chembl_id=<id>&limit=1`, reading `page_meta.total_count`.

### Section

- Add `chembl` to the 1202 cell line section list with an outcome key. `all` does not include it.
- The section calls `cell_line_by_cellosaurus` with the requested accession, then `cell_line_assay_count` for each record. It never searches by name.
- No record gives `empty` with the message `no ChEMBL cell line lists this accession`. A transport or decode failure gives `unavailable`.
- JSON: `cell_line.chembl` holds `{ "source": "ChEMBL", "data_as_of", "data_as_of_kind", "records": [{"chembl_id", "name", "efo_id", "clo_id", "assay_count"}] }`.
- `data_as_of` is the ChEMBL release name and date read from `status.json`, measured 2026-09-17 as `ChEMBL_37 (2026-05-01)`, with `data_as_of_kind: "release"`. Add `status()` to `ChemblClient` with its own plan function. The section calls it once, the shared HTTP cache holds the answer, and a failed call falls back to the retrieval time with `data_as_of_kind: "retrieved"`. The release is not hard-coded.
- Markdown: a `## ChEMBL` heading, one row per record, and the attribution line `ChEMBL_37 (2026-05-01), CC BY-SA 3.0. ChEMBL is produced by EMBL-EBI.` The release part is the `data_as_of` value. BioMCP lists no assays and no activity values.

### Bot checks

ChEMBL served no bot check in any measurement. The rule still holds: an HTTP 200 whose body is an HTML human-verification page where JSON was expected is a provider error. It names the URL and the endpoint, and the command stops. BioMCP never retries through a check, never rewrites the request to get around one, and never scrapes the page.
- The `ChEMBL` health row adds the cell line section to its `affects` text.

### Docs

`docs/sources/chembl.md`, the 1202 cell line guide, `docs/user-guide/cli-reference.md`, and `src/cli/list_reference.md` list the section.

## Fixtures

Record through the production request path into `testdata/sources/chembl/`, each with a `real_and_receipted` receipt:

- `cell_line_cvcl_2119_20260917.json`: the MOLM-13 record.
- `cell_line_cvcl_0004_20260917.json`: the K-562 record, whose ChEMBL name `K562` differs from the Cellosaurus name.
- `cell_line_none_20260917.json`: an empty result for an accession ChEMBL does not list.
- `assay_count_chembl3706573_20260917.json`: the `limit=1` assay page.
- `status_20260917.json`: the `status.json` body reporting ChEMBL_37, released 2026-05-01.

## Acceptance

Fixture-backed Rust tests, no live network:

1. The plans target `cell_line.json` with `cellosaurus_id` and `assay.json` with `cell_chembl_id` and `limit=1`.
2. `get cell-line CVCL_2119 chembl` returns CHEMBL3706573, null EFO and CLO, and the recorded assay count.
3. `get cell-line CVCL_0004 chembl` returns the K-562 record through the accession despite the name difference.
4. The empty fixture gives `empty` with the message and makes no assay request.
5. `get cell-line CVCL_2119 all` makes no ChEMBL request.
6. The catalog lists `chembl` among the cell line sections.
7. The section carries `data_as_of` `ChEMBL_37 (2026-05-01)` and `data_as_of_kind: "release"` from the `status.json` fixture, and the Markdown ends with the CC BY-SA 3.0 attribution line naming that release. A failing `status.json` gives a retrieval time and `data_as_of_kind: "retrieved"`, and the records still render.
8. An HTML body served with HTTP 200 for a ChEMBL JSON request is a provider error naming the URL, and no row renders.

Executable spec: the 1202 cell line spec page gains one JSON block (`get cell-line CVCL_2119 chembl --json`: `chembl.records[0].chembl_id == "CHEMBL3706573"`).

`make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA.

## Out of scope

- Assay lists, activity values, and cell-line targets.
- Name-based matching.
- A ChEMBL-to-Cellosaurus reverse lookup. Ticket 1202 already resolves ChEMBL IDs through `dr:`.

## Decisions

Ian can overturn these.

- The section shows a count and no assays. Assays are free-text literature records in the thousands per line.
- The join goes only through `cellosaurus_id` because name filters missed four of ten test lines.
- `data_as_of` is read from `status.json` at runtime rather than pinned in the source, so a new ChEMBL release needs no code change.

## Complexity

- Contract score: 1 (one section, one fixed JSON shape)
- State and timing score: 0 (two bounded read-only requests)
- Reach score: 1 (existing ChEMBL client, cell line entity, docs)
- Proof score: 1 (recorded fixtures)
- Cost of error score: 0 (a count and three IDs)
- Total: 3
- Final level: 2
- Reasons: existing source and an exact identifier join

## Review

- Design review: pending
- Code review: pending
