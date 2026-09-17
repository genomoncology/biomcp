---
flow: build
priority: 4
deps: [1202]
---

# 1213: HPA expression across cell lines for one gene

## Goal

`biomcp gene cell-lines <symbol> --group <cancer group>` prints the Human Protein Atlas RNA level (nTPM) of one gene in every HPA cell line of one cancer group, for example FLT3 across the 91 leukemia lines. Each row carries the Cellosaurus accession when the HPA name matches exactly one human Cellosaurus line. BioMCP reports the values as published and adds no labels or thresholds. The motivating consumer is a hackathon team screening public GEO studies of drug-treated AML cell lines. That team needs to see which lines express a target gene before it picks a study.

## Current Facts

- The HPA client exists. `HpaClient` (`src/sources/hpa.rs:16`) uses `shared_client()` and `env_base(HPA_BASE, HPA_BASE_ENV)` (`:22-26`), with `HPA_BASE` `https://www.proteinatlas.org` (`:12`) and `BIOMCP_HPA_BASE` (`:14`). Its one call, `protein_data` (`:60`), reads `<ensembl>.xml` through `protein_data_plan` (`:29`). Nothing reads cell line expression today.
- The gene entity calls it for the `hpa` section (`GENE_SECTION_HPA`, `src/entities/gene.rs:253`, client call at `:1668`). The health probe is the `HPA` row in `src/cli/health/catalog.rs:448`. Recorded fixtures live in `testdata/sources/hpa/` (`braf_20260811.xml`).
- Per-gene helpers with flags live under `GeneCommand` (`src/cli/gene/mod.rs:79`), for example `Pathways` (`:164`). `get gene` sections take no flags (`src/cli/gene/mod.rs:40`), so the group filter needs a helper.
- The repo records HPA as tier 3, CC BY-SA 4.0 for copyrightable parts (`docs/reference/source-licensing.md:68`, terms `https://www.proteinatlas.org/about/licence`). The 2026-09-17 survey read CC BY 4.0 from the site. The implementer rechecks the terms page and updates `reviewed_on` in both files.

### HPA search download, observed 2026-09-17 (workspace experiment 204)

- `GET https://www.proteinatlas.org/api/search_download.php?search=FLT3&format=json&columns=g,eg,cell_RNA_leukemia&compress=no` returned 11,043 bytes. The body is a JSON array with one object per gene that matched the search, three for FLT3. Each object has `Gene`, `Ensembl`, and one key per cell line, for example `"Cell line RNA - MOLM-13 [nTPM]": "23.0"`. Values are strings.
- The column keys are `cell_RNA_<group>`. The HPA data access page (`/about/help/dataaccess`) lists 30 groups covering 1,193 lines. Examples: `leukemia`, `lymphoma`, `myeloma`, `breast_cancer`, `lung_cancer`, `neuroblastoma`. The leukemia group holds 91 lines.
- All ten AML and leukemia test lines appear in the leukemia group under their Cellosaurus names: MOLM-13, MV4-11, HL-60, K-562, KG-1, THP-1, OCI-AML-3, Kasumi-1, U-937, NB4. No test line has an HPA link in Cellosaurus, so the join goes by name.
- The gene JSON (`/<ensembl>.json`) holds only summaries. The full cell line file `rna_celline.tsv.zip` is 205,861,141 bytes, so an all-genes view for one line is out of reach.
- HPA files were dated 2025-11-05. No rate limit appears in the headers.

## Design

### Source

Add `cell_line_rna(ensembl_id, group)` to `HpaClient` with a `cell_line_rna_plan` next to `protein_data_plan`. It requests `api/search_download.php` with `search=<ensembl id>`, `format=json`, `columns=g,eg,cell_RNA_<group>`, and `compress=no`. It keeps only the array object whose `Ensembl` equals the requested ID. It returns `(line name, nTPM)` pairs in upstream key order, parsing each value as `f64` and keeping a non-numeric value as `null`. A missing object gives an empty list.

`group` must be one of the 30 group names, stored as a constant list copied from the data access page at implementation time. An unknown group fails before any request and lists the valid names.

### Join to Cellosaurus

- Normalize each HPA name with the 1202 normalizer.
- Look up the names in batches of 20 with one Cellosaurus search per batch, `q=id:("<name1>" OR "<name2>" ...)`, the 1202 escape, and `fields=ac,id,ox`. The recorded fixture confirms the `id:` field. If Cellosaurus rejects `id:`, the batch uses `idsy:` with the same filter below.
- A row gets an accession only when exactly one human record's normalized identifier equals the normalized HPA name. Otherwise the accession is `null` and the row notes `no unique Cellosaurus match`.
- A Cellosaurus failure leaves every accession `null` and adds one note. The expression rows still print.

### Surface

- `biomcp gene cell-lines <symbol> --group <group> [--limit N] [--offset N]` under `GeneCommand`. The symbol resolves through the existing gene lookup to its Ensembl ID. Limit is 1 to 100 with a default of 100, so one call shows a whole leukemia group.
- Rows keep HPA's order. `--sort nTPM` is not offered. Each row prints the HPA line name, the accession or `-`, and the nTPM value as published.
- JSON: `{ "source": "Human Protein Atlas", "gene", "ensembl_id", "group", "total", "data_as_of", "data_as_of_kind", "rows": [{"name", "accession", "ntpm"}] }`.
- `data_as_of` is the HPA file date, measured 2025-11-05 on 2026-09-17, with `data_as_of_kind: "release"`. HPA publishes the file date on its data access page and not in the `search_download.php` response, so the date is a constant in the source module next to the group list, refreshed the same way the group list is. The docs say the constant is a recorded file date and name the page it came from. When a future response carries a date, the parser prefers it.
- Markdown ends with the HPA attribution line, `Human Protein Atlas, files dated 2025-11-05, <license>. proteinatlas.org`, and `biomcp get cell-line <first accession>` as the next command. `<license>` is the term the implementer confirms on the terms page, CC BY-SA 4.0 in `docs/reference/source-licensing.md:68` today and CC BY 4.0 in the 2026-09-17 survey. One value lands in the line, in `source-licensing.md`, and in `sources.json`, and a test pins that the three agree.

### Bot checks

HPA served no bot check in any measurement. The rule still holds: an HTTP 200 whose body is an HTML human-verification page where JSON was expected is a provider error. It names the URL and the file, and the command stops. BioMCP never retries through a check, never rewrites the request to get around one, and never scrapes the page.
- The existing HPA health row gains the new surface in its `affects` text.

### Docs

- `docs/sources/human-protein-atlas.md` gains the helper, the group list, and the note that values are nTPM as published.
- `docs/user-guide/gene.md`, `docs/user-guide/cli-reference.md`, and `src/cli/list_reference.md` list the helper. The 1202 cell line guide links to it.

## Fixtures

Record through the production request path, each with a `real_and_receipted` receipt:

- `testdata/sources/hpa/cell_rna_leukemia_flt3_20260917.json`: the full FLT3 leukemia response, three objects.
- `testdata/sources/cellosaurus/search_id_leukemia_batch1_20260917.json`: one batch of 20 leukemia names, including MOLM-13 and OCI-AML-3.

## Acceptance

Fixture-backed Rust tests, no live network:

1. The plan targets `api/search_download.php` with the four parameters. An unknown group fails before any request and lists the group names.
2. The parser keeps only the FLT3 object whose `Ensembl` is ENSG00000122025, returns 91 rows in key order, and reads `MOLM-13` as 23.0.
3. The join gives MOLM-13 CVCL_2119 and OCI-AML-3 CVCL_1844. A name with no unique human match gets `null` and the note.
4. A Cellosaurus error leaves the rows intact with `null` accessions and one note.
5. `--limit` and `--offset` page the rows, and `total` stays 91.
6. The HPA health row names the new surface.
7. Every output carries `data_as_of` `2025-11-05` and `data_as_of_kind: "release"`, and the Markdown ends with the attribution line naming that date and the confirmed license.
8. The license string in the attribution line, `docs/reference/source-licensing.md`, and `docs/reference/sources.json` are the same value.
9. An HTML body served with HTTP 200 for the search download is a provider error naming the URL, and no row renders.

Executable spec: `spec/entity/gene.md` gains one JSON block (`gene cell-lines FLT3 --group leukemia --json`: `total == 91` and the MOLM-13 accession) and one Markdown block (the attribution line).

`make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA.

## Out of scope

- A view of all genes for one cell line. It needs the 206 MB file.
- Protein level, subcellular location, and single-cell data for cell lines.
- Ranking, thresholds, or "high" and "low" labels.
- A `get cell-line` section. HPA has no Cellosaurus link, and a name join per card adds a request for one value.

## Decisions

Ian can overturn these.

- The helper sits on the gene side because HPA answers one gene at a time in about 11 KB.
- The join accepts only a unique human identifier match and shows `null` otherwise. A wrong accession costs more than a missing one.
- `data_as_of` is a recorded file date held as a constant, because the response carries no date. The alternative is a second request to the data access page on every call, for a value that changes a few times a year.

## Complexity

- Contract score: 1 (one helper, one fixed JSON shape)
- State and timing score: 0 (bounded read-only requests)
- Reach score: 1 (existing HPA client, gene CLI, Cellosaurus search from 1202, docs)
- Proof score: 1 (recorded fixtures)
- Cost of error score: 1 (a wrong name join would label a row with another line)
- Total: 4
- Final level: 2
- Reasons: existing source, one new query shape, and a guarded name join

## Review

- Design review: pending
- Code review: pending
