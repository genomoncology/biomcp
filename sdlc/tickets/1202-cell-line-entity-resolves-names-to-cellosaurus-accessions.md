---
flow: build
priority: 2
deps: []
---

# 1202: A cell-line entity resolves names to Cellosaurus accessions

## Goal

`biomcp search cell-line <name>` turns any common spelling of a cell line (MOLM13, MOLM-13, MV4-11, MV4;11) into its Cellosaurus accession. `biomcp get cell-line <CVCL_xxxx>` returns one card with name, synonyms, species, disease, category, sex, and age. Its `xrefs` section lists the cross-references other sources join on, and its `variants` section lists the curated sequence variations. `get cell-line` also accepts a DepMap, Cell Model Passports, ChEMBL, or PharmacoDB ID and resolves it to the accession. The accession is the join key for tickets 1205 and 1206. The motivating consumer is a hackathon team screening public GEO studies of drug-treated AML cell lines, which met the same line under four spellings.

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
- `fields=ac,id,sy,ox,di,ca,sx,dr` cuts the MOLM-13 record from 156 KB to 19 KB. The same projection is 323 KB for K-562, which carries 1,013 ENCODE and 207 GEO cross-references. Without `dr` the MOLM-13 projection is 2,073 bytes, and adding `var` makes it 4,449 bytes (survey of 2026-09-17, workspace experiment 204). The projected record carries `accession-list` (`type` primary or secondary), `name-list` (`type` identifier or synonym), `species-list` (NCBI_TaxID `accession` and `label`), `disease-list` (`database` NCIt or ORDO, `accession`, `label`), `category`, `sex`, and `xref-list` (`database`, `accession`, `url`).
- The xref `database` names the ticket needs are `DepMap` (ACH-000362 for MOLM-13, ACH-000045 for MV4-11), `Cosmic-CLP`, `ChEMBL-Cells`, `Cell_Model_Passport`, `GDSC`, `PharmacoDB`, and `LINCS_LDP`. Across ten AML and leukemia test lines, DepMap, Cell Model Passports, ChEMBL, and PharmacoDB links exist for all ten. U-937 (CVCL_0007) has no `GDSC` and no `Cosmic-CLP` link. No test line has an HPA link. The record also carries 25 `Cosmic` sample rows for MOLM-13. Those are sample IDs and stay out of the card. No `rrid` field exists. Cellosaurus is the RRID authority for cell lines, and the RRID is `RRID:` plus the primary accession.
- The default keyword search ranks badly for exact names. `q=K562` finds 374 records and puts K-562 (CVCL_0004) at position 383 of the TSV output. `q=HL60` puts four derived lines ahead of HL-60.
- The fielded query `q=idsy:"<name>"` searches the name and synonyms. `idsy:"MV4;11"` returns 3 records, `idsy:"MOLM13"` returns 1, `idsy:"K562"` returns 366. `idsy:"KO"` fills the 1000-row window, so a window can end before the exact match.
- Normalized comparison finds the intended line in every AML case tried: lowercase and drop every non-alphanumeric character from the query, the name, and each synonym. MOLM13 matches CVCL_2119, MV4;11 matches synonym MV4;11 on CVCL_0064, HL60 matches CVCL_0002. The 2026-09-16 KG1 facts are stale. On 2026-09-17 `idsy:"KG1"` returned three records in this order: CVCL_E3VV (mouse), CVCL_0374 (KG-1, human), and CVCL_UD72. All three match exactly after normalization.
- Results change with spelling. `idsy:"KG-1"` returns CVCL_0374, CVCL_2971 (KG-1-C), and CVCL_1S07. `idsy:"NB-4"` returns CVCL_0005 alone.
- Upstream order can put the wrong line first. `idsy:"NB4"` returns SJNB-4 (CVCL_8821, a neuroblastoma line that lists `NB4` as a synonym) at position 16 and the real NB4 (CVCL_0005) at position 21. Both match exactly after normalization. NB4 is the line's identifier. On SJNB-4 it is only a synonym. The exact match can also sit near the end of the window: K562 at 360 of 366, THP1 at 68 of 69.
- HL-60 (CVCL_0002) and the NCI-60 line HL-60(TB) (CVCL_A794) are separate records. The `NCI-DCTD` link on CVCL_A794 holds the name `HL-60`, so a name join from NCI-60 lands on the wrong line. Cellosaurus has no PharmacoDB link for CVCL_A794.
- Reverse lookup works through the `dr:` field. `q=dr:ACH-000362`, `q=dr:SIDM00437`, and `q=dr:MOLM13_950_2019` each return CVCL_2119 alone.
- `sequence-variation-list` (field `var`) holds 1 to 5 curated entries per test line. Each entry has a variation type, an HGVS description, an HGNC cross-reference, zygosity, and PubMed sources. OCI-AML-3 lists DNMT3A p.Arg882Cys, NPM1 p.Trp288Cysfs*12, and NRAS p.Gln61Leu.
- `/release-info` reported release 56.0, 2026-06-25, 168,970 lines. No rate limit appears in the headers.
- Terms: Cellosaurus data is CC BY 4.0. The requested citation is Bairoch A. "The Cellosaurus, a cell line knowledge resource." J. Biomol. Tech. 29:25-38 (2018). The site publishes no rate limit. BioMCP needs no key.

## Design

### Source module

`src/sources/cellosaurus.rs`, declared in `src/sources/mod.rs`, uses `shared_client()` and `env_base("https://api.cellosaurus.org", "BIOMCP_CELLOSAURUS_BASE")`. Add a `rate_limit.rs` policy for that origin at 334 ms, matching KEGG, because Cellosaurus publishes no limit. Three calls:

- `get(accession, fields)`: `/cell-line/{ac}?format=json&fields=<fields>`. The card fields are `ac,id,sy,ox,di,ca,sx,ag`. The request adds `var` when the `variants` section is asked for and `dr` when `xrefs` or a join section (tickets 1205, 1206, and later ones) is asked for. The card alone never fetches `dr`. A 404 maps to the repo's not-found error.
- `search_names(name)`: `/search/cell-line?q=idsy:"<escaped>"&format=json&fields=ac,id,sy,ox,di,ca&rows=1000`. The escape backslash-quotes `"` and `\`. It returns the rows and a `window_full` flag set when 1000 rows came back.
- `search_xref(id)`: `/search/cell-line?q=dr:"<escaped>"&format=json&fields=ac,id&rows=10`. The recorded fixture confirms that the quoted form returns the same record as the unquoted form.

### Entity

`src/entities/cell_line.rs` owns the types, the normalizer, and ranking.

- `CellLine`: `accession`, `secondary_accessions`, `rrid`, `name`, `synonyms`, `species` (taxon id and label), `diseases` (database, accession, label), `category`, `sex`, `age`, `variants`, and `xrefs`. `xrefs` holds only the join keys, in this fixed order: `depmap`, `cosmic_clp`, `chembl`, `cell_model_passport`, `gdsc`, `pharmacodb`, `lincs_ldp`. Each is a list, so a record with two DepMap IDs keeps both. Every key is present. A missing link is an empty list, so U-937 shows `gdsc: []`. `source` is `Cellosaurus` and the card carries the CC BY 4.0 attribution line.
- `variants`: one row per `sequence-variation-list` entry with gene symbol, HGNC ID, variation type, HGVS description, zygosity, and PubMed IDs, in upstream order and as published. BioMCP adds no interpretation.
- Search row: `accession`, `name`, `species`, `category`, first disease label, `match` (`exact` or `partial`), and `matched_on` (`name` or `synonym`, set on exact rows). `exact` means the normalized query equals the normalized name or any normalized synonym. `matched_on: name` means the query equals the record's identifier name. Rows sort in this order: exact identifier matches, then exact synonym-only matches, then partial matches. Within each group human lines (taxon 9606) come first, then upstream order. `--offset` and `--limit` apply after sorting. The total is the row count when the window was not full. A full window makes the total unknown and adds one note: `Cellosaurus returned 1000 rows; an exact match may lie past this window. Use a more specific name or the CVCL accession.`
- Spelling: when the query contains a hyphen, search runs twice, once with the raw query and once with every hyphen removed. The rows merge by accession before ranking. A full window on either request sets the total to unknown.
- More than one exact match is reported as it is. The rows show the species and the match kind that tell KG-1 from KG1 and NB4 from SJNB-4. The ranking orders the rows. BioMCP still picks no winner and hides no row.
- A query that already looks like an accession (`^CVCL_[A-Z0-9]{4}$`, case-insensitive) searches `ac:` instead and returns that one row as `exact`.
- Reverse lookup: `get cell-line <id>` treats any ID that is not a CVCL accession as a source ID and calls `search_xref`. Examples are DepMap `ACH-000362`, Cell Model Passports `SIDM00437`, ChEMBL `CHEMBL3706573`, and PharmacoDB `MOLM13_950_2019`. One result opens that record. Zero results give the not-found error with the hint `biomcp search cell-line <id>`. More than one result fails and lists the accessions. The card prints the ID it was resolved from.

### CLI and MCP

- `SearchEntity::CellLine(CellLineSearchArgs)` with the pathway argument shape: positional query or `-q`, `--limit` 1-25 default 10, `--offset` default 0. Search takes no filters.
- `GetEntity::CellLine(CellLineGetArgs)` with `id` and trailing `sections`. Sections are `variants`, `xrefs`, and `all`. The default card shows the card fields only and ends with the next commands `biomcp get cell-line <ac> variants` and `biomcp get cell-line <ac> xrefs`. The requested sections widen the `fields` list of the one record request, so no section makes an extra call and the entity needs no `section_outcomes` registry row. A reverse lookup adds one search request before it.
- Catalog: `("cell-line", true, true)` in `ENTITY_FLAGS` and `CELL_LINE_SECTION_NAMES` in the section map. Typed MCP `get` picks it up from the catalog. Typed MCP `search` stays at eight entities. Cell-line search reaches MCP through the raw tool, like pathway.
- Search rows print `biomcp get cell-line <ac>` in `_meta.next_commands` for the first exact row.
- Health: one `SourceDescriptor` for Cellosaurus probing `/release-info?format=json`, affects "cell-line search and detail".
- The review note wrote `cell_line`. This ticket uses `cell-line` because every multi-word entity in the CLI and catalog is kebab case (`adverse-event`).

### Docs

- `docs/sources/cellosaurus.md` on the KEGG page model, a row in `docs/sources/index.md`, and a nav entry in `mkdocs.yml`.
- `docs/user-guide/cell-line.md`, a nav entry, and command rows in `docs/user-guide/cli-reference.md` and `src/cli/list_reference.md`. Help `after_help` examples: `search cell-line MOLM13`, `search cell-line "MV4;11"`, `get cell-line CVCL_2119`, `get cell-line CVCL_2119 xrefs`, `get cell-line CVCL_1844 variants`, `get cell-line ACH-000362`. The user guide names the HL-60 and HL-60(TB) trap.
- A tier 1 row in `docs/reference/source-licensing.md` and an object in `docs/reference/sources.json`: `direct_api`, auth `none`, CC BY 4.0, attribution and citation required on reuse, terms URL `https://www.cellosaurus.org/description.html`, reviewed 2026-09-16.

## Fixtures

Record these through the production request path into `testdata/sources/cellosaurus/`, each with a `real_and_receipted` receipt. No fields are removed.

- `search_idsy_molm13_20260916.json`: one row, CVCL_2119.
- `search_idsy_mv4_11_semicolon_20260916.json` for `idsy:"MV4;11"`: three rows, CVCL_0064 exact.
- `search_idsy_kg1_20260917.json`: three rows (CVCL_E3VV, CVCL_0374, CVCL_UD72), all exact. It replaces the stale 2026-09-16 capture.
- `search_idsy_kg_1_20260917.json` for `idsy:"KG-1"`: CVCL_0374, CVCL_2971, CVCL_1S07.
- `search_idsy_nb4_20260917.json`: the full window with CVCL_8821 before CVCL_0005.
- `search_dr_ach_000362_20260917.json`, `search_dr_sidm00437_20260917.json`, `search_dr_chembl3706573_20260917.json`, and `search_dr_molm13_950_2019_20260917.json`: one row each, CVCL_2119.
- `get_cvcl_2119_20260917.json` and `get_cvcl_0064_20260917.json`: card fields plus `var` and `dr`.
- `get_cvcl_1844_var_20260917.json`: OCI-AML-3 card fields plus `var`.
- `get_cvcl_0007_20260917.json`: U-937 card fields plus `dr`, with no GDSC or Cosmic-CLP link.
- `get_cvcl_0005_20260917.json`: NB4 card fields plus `dr`.

One synthetic 1000-row page is generated in the test from a template. It is not a recorded file. Extend `setup-provider-contract-spec-fixture.sh` to serve `/cellosaurus/...` from these files, answer an empty search for any other `dr:` query, answer 404 for any other accession, and export `BIOMCP_CELLOSAURUS_BASE`.

## Acceptance

Rust tests, fixture-backed, no live network:

1. Normalizer: `MOLM13`, `molm-13`, and `Molm 13` normalize equal. `MV4;11`, `MV4-11`, and `MV 4;11` normalize equal.
2. `MOLM13`, `MOLM-13`, and `MV4;11` searches each return the expected accession as the first row with `match: exact`.
3. `KG1` returns the exact set {CVCL_0374, CVCL_E3VV, CVCL_UD72}, each with its species. CVCL_0374 is first because it is a human identifier match. The test checks the set and the first row, not the order of the rest. `KG-1` runs two requests and merges them by accession with no duplicate rows.
4. `NB4` returns CVCL_0005 first with `matched_on: name`. CVCL_8821 appears later with `matched_on: synonym`.
5. A full 1000-row window sets the total to unknown and prints the note. A 3-row window reports total 3 and no note.
6. The search request escapes `"` and `\` and targets `idsy:`. `CVCL_2119` as a query targets `ac:` instead.
7. `get cell-line CVCL_2119` parses accession, RRID `RRID:CVCL_2119`, name `MOLM-13`, synonyms, species 9606, NCIt disease C8263, category `Cancer cell line`, and sex `Male`. The card request has no `dr` in `fields`. The `xrefs` section lists DepMap `ACH-000362`, adds `dr` to the request, and leaves out the `Cosmic` sample rows.
8. `get cell-line CVCL_0064 xrefs` lists DepMap `ACH-000045`, COSMIC cell line `908156`, and ChEMBL `CHEMBL3308063`. `get cell-line CVCL_0007 xrefs` prints every key, with `gdsc` and `cosmic_clp` as empty lists.
9. `get cell-line CVCL_1844 variants` lists DNMT3A, NPM1, and NRAS rows with their HGVS descriptions as published.
10. `get cell-line ACH-000362`, `SIDM00437`, `CHEMBL3706573`, and `MOLM13_950_2019` each resolve to CVCL_2119. An unmatched source ID fails with the not-found error and the search hint.
11. An unknown accession fails with the not-found error. An unknown section fails before any request.
12. The catalog lists `cell-line` as searchable and gettable with sections `variants`, `xrefs`, and `all`. The typed MCP `get` schema gains a `cell-line` branch. Typed MCP `search` still rejects `cell-line`.
13. The health catalog test counts the new Cellosaurus row. The rate-limit test resolves Cellosaurus URLs to the new policy.

Executable spec `spec/entity/cell-line.md`, added to `SPEC_ROUTINE_PATHS`:

- `search cell-line "MV4;11"` Markdown shows a table row with `CVCL_0064`, `MV4-11`, and `exact`.
- `search cell-line KG1 --json` has three `exact` results and `results[0].accession == "CVCL_0374"`.
- `search cell-line NB4 --json` has `results[0].accession == "CVCL_0005"`.
- `get cell-line ACH-000362 --json` has `accession == "CVCL_2119"`.
- `get cell-line CVCL_2119 --json` has `accession == "CVCL_2119"` and `xrefs.depmap == ["ACH-000362"]`.
- `get cell-line CVCL_2119` Markdown shows the attribution line and the `get cell-line CVCL_2119 xrefs` next command.
- `get cell-line CVCL_1844 variants` Markdown shows a variants table with `NPM1`.

`make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA.

## Decisions

These choices follow the 2026-09-17 source survey. Ian can overturn any of them.

- Cellosaurus is the identity source for every cell line section. Every other source joins to it through the CVCL accession.
- Ranking puts identifier matches before synonym-only matches and human lines before other species. Upstream order failed on NB4.
- The card never fetches `dr`. Only `xrefs` and join sections pay for it.
- Sources read in other tickets: PharmacoDB (1205), DepMap through Figshare (1206), HPA expression across cell lines (1213), and ChEMBL cell line lookup (1214).
- Sources skipped, one reason each:
  - Cell Model Passports: it repeats Cellosaurus and DepMap, several calls timed out at 60 s, and it has no drug endpoint.
  - GDSC direct: the site returns 410, the bulk files are 2023 spreadsheets, and PharmacoDB carries GDSC1 and GDSC2.
  - CellMiner: it covers 60 lines by name with no accession, and it carries the HL-60(TB) name trap. PharmacoDB carries NCI60.
  - COSMIC Cell Lines: it needs an account, and commercial users go to Qiagen.
  - LINCS/CLUE: the API needs a user key. The card still lists the `LINCS_LDP` link.
  - ENCODE: its records are assay samples, not cell line facts.

## Out of scope

- Pivots such as `cell-line datasets`. Drug response and dependency sections are tickets 1205 and 1206.
- STR profiles, HLA typing, child lines, publications, and every other record field not named above.
- A species or disease filter on search, and any rule that picks one line among several exact matches and hides the rest.
- The `misspelling` field, fuzzy matching, and paging past the 1000-row window.
- Typed MCP `search` support, batch support, and local mirroring of Cellosaurus.
