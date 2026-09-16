---
flow: build
priority: 2
deps: []
---

# 1202: A cell-line entity resolves names to Cellosaurus accessions

## Goal

`biomcp search cell-line <name>` turns any common spelling of a cell line (MOLM13, MOLM-13, MV4-11, MV4;11) into its Cellosaurus accession. `biomcp get cell-line <CVCL_xxxx>` returns one card with name, synonyms, species, disease, category, sex, and the cross-references other sources join on. The accession is the join key for tickets 1205 and 1206. The motivating consumer is a hackathon team screening public GEO studies of drug-treated AML cell lines, which met the same line under four spellings.

## Current Facts

- No code, doc, or fixture mentions Cellosaurus today. `grep -rli cellosaurus` matches only one recorded Europe PMC article body under `testdata/sources/variant_articles_683/`.
- The entity inventory is `ENTITY_FLAGS` (`src/cli/list/catalog.rs:38`), with section names mapped per entity at `src/cli/list/catalog.rs:78`. Multi-word entities use kebab case: `("adverse-event", true, true)` (`src/cli/list/catalog.rs:52`) and `biomcp search adverse-event` (`src/cli/commands.rs:507`).
- `SearchEntity` (`src/cli/commands.rs:269`) and `GetEntity` (`src/cli/commands.rs:522`) hold one clap variant per entity. Dispatch runs through `src/cli/outcome.rs` (`GetEntity::Pathway` at `:129`, `SearchEntity::Pathway` at `:232`) and `src/cli/response_contract.rs` (`:126`, `:287`).
- The typed MCP `get` tool derives its entity list and section enum from the catalog (`typed_get_capabilities`, `src/mcp/shell/typed_get.rs:12`). A new gettable entity reaches typed MCP `get` with no tool change. The typed MCP `search` tool covers a fixed list of eight entities (`src/mcp/shell.rs:285`). Pathway search reaches MCP only through the raw tool, and a test pins that typed rejection (`src/mcp/shell.rs:1825`).
- Per-origin pacing and base-URL overrides live in `src/sources/rate_limit.rs`. KEGG is the model: `policy("kegg", "BIOMCP_KEGG_BASE", "https://rest.kegg.jp", 334 ms)` (`:167`). Health probes are `SourceDescriptor` rows (`src/cli/health/catalog.rs:410`).
- Pathway search args are the model for a name search: positional query plus `-q`, `--limit` 1-25 default 10, `--offset` default 0 (`src/cli/pathway/mod.rs:6`). Pathway get takes `id` plus trailing `sections` (`src/cli/pathway/mod.rs:28`).
- Each source has a page under `docs/sources/` (for example `docs/sources/kegg.md`), a row in `docs/sources/index.md` (`:44` for KEGG), a nav entry in `mkdocs.yml` (`:70`), a summary row and a tier section in `docs/reference/source-licensing.md` (tier 1 starts at `:114`), and an object in `docs/reference/sources.json` (`"id": "kegg"` at `:781`). Each entity has a user guide under `docs/user-guide/`.
- Recorded fixtures live in `testdata/sources/<source>/` with a receipt in `testdata/sources/capture-receipts.json` (153 entries are `real_and_receipted`). Routine specs replay them through `spec/fixtures/setup-provider-contract-spec-fixture.sh`, which serves `/kegg/...` (`:474`) and exports `BIOMCP_KEGG_BASE` (`:619`). `spec/entity/pathway.md` is in `SPEC_ROUTINE_PATHS` (`scripts/run-specs.sh:22`).

### Cellosaurus API, observed 2026-09-16 (release 56.0)

- `GET https://api.cellosaurus.org/cell-line/{ac}?format=json&fields=...` returns `{"Cellosaurus":{"cell-line-list":[record]}}`. An unknown accession returns HTTP 404.
- `GET /search/cell-line?q=<solr>&format=json&fields=...&rows=N&start=M` returns the same envelope with no total count. `rows` defaults to 1000. The OpenAPI spec (`/openapi.json`) also lists `sort`.
- `fields=ac,id,sy,ox,di,ca,sx,dr` cuts the MOLM-13 record from 156 KB to 19 KB. The projected record carries `accession-list` (`type` primary or secondary), `name-list` (`type` identifier or synonym), `species-list` (NCBI_TaxID `accession` and `label`), `disease-list` (`database` NCIt or ORDO, `accession`, `label`), `category`, `sex`, and `xref-list` (`database`, `accession`, `url`).
- The xref `database` names the ticket needs are `DepMap` (ACH-000362 for MOLM-13, ACH-000045 for MV4-11), `Cosmic-CLP`, `ChEMBL-Cells`, `Cell_Model_Passport`, `GDSC`, and `PharmacoDB`. The record also carries 25 `Cosmic` sample rows for MOLM-13. Those are sample IDs and stay out of the card. No `rrid` field exists. Cellosaurus is the RRID authority for cell lines, and the RRID is `RRID:` plus the primary accession.
- The default keyword search ranks badly for exact names. `q=K562` finds 374 records and puts K-562 (CVCL_0004) at position 383 of the TSV output. `q=HL60` puts four derived lines ahead of HL-60.
- The fielded query `q=idsy:"<name>"` searches the name and synonyms. `idsy:"MV4;11"` returns 3 records, `idsy:"MOLM13"` returns 1, `idsy:"K562"` returns 366. `idsy:"KO"` fills the 1000-row window, so a window can end before the exact match.
- Normalized comparison finds the intended line in every AML case tried: lowercase and drop every non-alphanumeric character from the query, the name, and each synonym. MOLM13 matches CVCL_2119, MV4;11 matches synonym MV4;11 on CVCL_0064, HL60 matches CVCL_0002. KG1 matches two records: KG-1 (CVCL_0374, human) and KG1 (CVCL_UD72, mouse).
- Terms: Cellosaurus data is CC BY 4.0. The requested citation is Bairoch A. "The Cellosaurus, a cell line knowledge resource." J. Biomol. Tech. 29:25-38 (2018). The site publishes no rate limit. BioMCP needs no key.

## Design

### Source module

`src/sources/cellosaurus.rs`, declared in `src/sources/mod.rs`, uses `shared_client()` and `env_base("https://api.cellosaurus.org", "BIOMCP_CELLOSAURUS_BASE")`. Add a `rate_limit.rs` policy for that origin at 334 ms, matching KEGG, because Cellosaurus publishes no limit. Two calls:

- `get(accession)`: `/cell-line/{ac}?format=json&fields=ac,id,sy,ox,di,ca,sx,dr`. A 404 maps to the repo's not-found error.
- `search_names(name)`: `/search/cell-line?q=idsy:"<escaped>"&format=json&fields=ac,id,sy,ox,di,ca&rows=1000`. The escape backslash-quotes `"` and `\`. It returns the rows and a `window_full` flag set when 1000 rows came back.

### Entity

`src/entities/cell_line.rs` owns the types, the normalizer, and ranking.

- `CellLine`: `accession`, `secondary_accessions`, `rrid`, `name`, `synonyms`, `species` (taxon id and label), `diseases` (database, accession, label), `category`, `sex`, `xrefs`. `xrefs` holds only the join keys, in this fixed order: `depmap`, `cosmic_clp`, `chembl`, `cell_model_passport`, `gdsc`, `pharmacodb`. Each is a list, so a record with two DepMap IDs keeps both. `source` is `Cellosaurus` and the card carries the CC BY 4.0 attribution line.
- Search row: `accession`, `name`, `species`, `category`, first disease label, and `match` (`exact` or `partial`). `exact` means the normalized query equals the normalized name or any normalized synonym. Rows sort exact first, then upstream order. `--offset` and `--limit` apply after sorting. The total is the row count when the window was not full. A full window makes the total unknown and adds one note: `Cellosaurus returned 1000 rows; an exact match may lie past this window. Use a more specific name or the CVCL accession.`
- More than one exact match is reported as it is. The rows show the species that tells KG-1 and KG1 apart. BioMCP picks no winner.
- A query that already looks like an accession (`^CVCL_[A-Z0-9]{4}$`, case-insensitive) searches `ac:` instead and returns that one row as `exact`.

### CLI and MCP

- `SearchEntity::CellLine(CellLineSearchArgs)` with the pathway argument shape: positional query or `-q`, `--limit` 1-25 default 10, `--offset` default 0. Search takes no filters.
- `GetEntity::CellLine(CellLineGetArgs)` with `id` and trailing `sections`. Sections are `xrefs` and `all`. The default card shows everything except the cross-reference table and ends with the next command `biomcp get cell-line <ac> xrefs`. All sections come from the one record request. No section makes an extra call, so the entity needs no `section_outcomes` registry row.
- Catalog: `("cell-line", true, true)` in `ENTITY_FLAGS` and `CELL_LINE_SECTION_NAMES` in the section map. Typed MCP `get` picks it up from the catalog. Typed MCP `search` stays at eight entities. Cell-line search reaches MCP through the raw tool, like pathway.
- Search rows print `biomcp get cell-line <ac>` in `_meta.next_commands` for the first exact row.
- Health: one `SourceDescriptor` for Cellosaurus probing `/release-info?format=json`, affects "cell-line search and detail".
- The review note wrote `cell_line`. This ticket uses `cell-line` because every multi-word entity in the CLI and catalog is kebab case (`adverse-event`).

### Docs

- `docs/sources/cellosaurus.md` on the KEGG page model, a row in `docs/sources/index.md`, and a nav entry in `mkdocs.yml`.
- `docs/user-guide/cell-line.md`, a nav entry, and command rows in `docs/user-guide/cli-reference.md` and `src/cli/list_reference.md`. Help `after_help` examples: `search cell-line MOLM13`, `search cell-line "MV4;11"`, `get cell-line CVCL_2119`, `get cell-line CVCL_2119 xrefs`.
- A tier 1 row in `docs/reference/source-licensing.md` and an object in `docs/reference/sources.json`: `direct_api`, auth `none`, CC BY 4.0, attribution and citation required on reuse, terms URL `https://www.cellosaurus.org/description.html`, reviewed 2026-09-16.

## Fixtures

Record these through the production request path into `testdata/sources/cellosaurus/`, each with a `real_and_receipted` receipt. No fields are removed.

- `search_idsy_molm13_20260916.json`: one row, CVCL_2119.
- `search_idsy_mv4_11_semicolon_20260916.json` for `idsy:"MV4;11"`: three rows, CVCL_0064 exact.
- `search_idsy_kg1_20260916.json`: four rows, two exact across two species.
- `get_cvcl_2119_20260916.json` and `get_cvcl_0064_20260916.json`: projected records.

One synthetic 1000-row page is generated in the test from a template. It is not a recorded file. Extend `setup-provider-contract-spec-fixture.sh` to serve `/cellosaurus/...` from these files, answer 404 for any other accession, and export `BIOMCP_CELLOSAURUS_BASE`.

## Acceptance

Rust tests, fixture-backed, no live network:

1. Normalizer: `MOLM13`, `molm-13`, and `Molm 13` normalize equal. `MV4;11`, `MV4-11`, and `MV 4;11` normalize equal.
2. `MOLM13`, `MOLM-13`, and `MV4;11` searches each return the expected accession as the first row with `match: exact`.
3. `KG1` returns CVCL_0374 and CVCL_UD72 as the first two rows, both exact, with their species, in upstream order.
4. A full 1000-row window sets the total to unknown and prints the note. A 3-row window reports total 3 and no note.
5. The search request escapes `"` and `\` and targets `idsy:`. `CVCL_2119` as a query targets `ac:` instead.
6. `get cell-line CVCL_2119` parses accession, RRID `RRID:CVCL_2119`, name `MOLM-13`, synonyms, species 9606, NCIt disease C8263, category `Cancer cell line`, and sex `Male`. The `xrefs` section lists DepMap `ACH-000362` and leaves out the `Cosmic` sample rows.
7. `get cell-line CVCL_0064 xrefs` lists DepMap `ACH-000045`, COSMIC cell line `908156`, and ChEMBL `CHEMBL3308063`.
8. An unknown accession fails with the not-found error. An unknown section fails before any request.
9. The catalog lists `cell-line` as searchable and gettable with sections `xrefs` and `all`. The typed MCP `get` schema gains a `cell-line` branch. Typed MCP `search` still rejects `cell-line`.
10. The health catalog test counts the new Cellosaurus row. The rate-limit test resolves Cellosaurus URLs to the new policy.

Executable spec `spec/entity/cell-line.md`, added to `SPEC_ROUTINE_PATHS`:

- `search cell-line "MV4;11"` Markdown shows a table row with `CVCL_0064`, `MV4-11`, and `exact`.
- `search cell-line KG1 --json` has two `exact` results.
- `get cell-line CVCL_2119 --json` has `accession == "CVCL_2119"` and `xrefs.depmap == ["ACH-000362"]`.
- `get cell-line CVCL_2119` Markdown shows the attribution line and the `get cell-line CVCL_2119 xrefs` next command.

`make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA.

## Out of scope

- Pivots such as `cell-line datasets` or `cell-line drugs`. Tickets 1205 and 1206 own those.
- STR profiles, sequence variations, HLA typing, child lines, publications, and every other record field.
- A species or disease filter on search, and any rule that picks one line among several exact matches.
- The `misspelling` field, fuzzy matching, and paging past the 1000-row window.
- Typed MCP `search` support, batch support, and local mirroring of Cellosaurus.
