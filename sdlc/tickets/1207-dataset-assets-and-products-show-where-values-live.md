---
flow: build
priority: 3
deps: [1211]
---

# 1207: Dataset assets and products show where values live

## Goal

`biomcp get dataset <id> assets products` tells an agent where a GEO series keeps its expression values before anything large is downloaded. An **asset** is one concrete file with a URL: a series matrix file, an NCBI-computed RNA-seq count file, or a supplementary file. A **product** is what an asset means scientifically: an array value matrix from the submitter, NCBI raw gene counts, NCBI TPM, or a supplementary file of unknown meaning. The names follow the 2026-09-16 research-data decision brief. BioMCP lists links and sizes it can read cheaply. It downloads none of the listed files; ticket 1208 does that.

```
biomcp get dataset GSE48843 assets products
biomcp get dataset geo:GSE982 assets
```

## Current Facts

Measured 2026-09-16 against 588 AML series that a hackathon team screening public GEO studies of drug-treated AML cell lines had collected:

- 774 series matrix files downloaded. 120 of them (107 series) have rows between `!series_matrix_table_begin` and `!series_matrix_table_end`. The other 654 have only the `ID_REF` header. By first-sample `!Sample_library_strategy`, 490 of the empty files are RNA-Seq, 63 ChIP-Seq, and 22 ATAC-seq.
- ESummary `geo2r` was `yes` for 200 series whose matrix table was empty. For the first of these, GSE48843, the page `https://www.ncbi.nlm.nih.gov/geo/download/?type=rnaseq_counts&acc=GSE48843` links three files: `GSE48843_raw_counts_GRCh38.p13_NCBI.tsv.gz`, `GSE48843_norm_counts_FPKM_GRCh38.p13_NCBI.tsv.gz`, and `GSE48843_norm_counts_TPM_GRCh38.p13_NCBI.tsv.gz`. Each link has the form `/geo/download/?type=rnaseq_counts&acc=<GSE>&format=file&file=<name>`. The raw counts file is 1.5 MB gzip with 39,376 NCBI GeneID rows and one column per GSM. The page also links `Human.GRCh38.p13.annot.tsv.gz` without an accession.
- The same page for GSE982, an array series, returns 6,474 bytes and no `file=` links.
- NCBI describes the pipeline at `https://www.ncbi.nlm.nih.gov/geo/info/rnaseqcounts.html`.
- ESummary `suppfile` names supplementary types only. The series `suppl/` folder under `https://ftp.ncbi.nlm.nih.gov/geo/series/<prefix>/<GSE>/suppl/` lists the actual files. Among the empty-table series, types were TXT 166, CSV 42, TSV 34, XLSX 23, and BW 20.
- Ticket 1211 adds `series_prefix`, `read_header(url, lines_after_marker)`, and the `BIOMCP_GEO_FTP_BASE` override. Ticket 1203 adds `supplementary_types` and `geo2r` to dataset rows.

## Design

Sections `assets` and `products` on `get dataset`. Both are opt-in and not part of `all`, because they read the matrix files and two listing pages. `products` is derived from the same asset scan, so asking for both makes one scan.

- **Matrix.** For each platform from ESummary, build the matrix URL with 1211's rules and call `read_header` with `lines_after_marker = 2`. Report `has_values` as true when a data row follows the `ID_REF` header. Report `library_strategies` as the distinct `!Sample_library_strategy` values in file order. 
- **NCBI counts.** Fetch the `rnaseq_counts` page for the accession. Collect `file=` links whose `acc` matches. Classify each by name: `raw_counts`, `norm_counts_FPKM`, `norm_counts_TPM`, and the genome build between the kind and `_NCBI`. An empty list is a normal result. Add an override `BIOMCP_GEO_WEB_BASE` for `https://www.ncbi.nlm.nih.gov`.
- **Supplementary.** Fetch the `suppl/` listing. Report each file name and URL. Report sizes only when the listing prints them.
- **Platform annotation.** For each array platform whose matrix has values, list `https://ftp.ncbi.nlm.nih.gov/geo/platforms/<GPL prefix>/<GPL>/annot/<GPL>.annot.gz` when a HEAD request succeeds. NCBI's accession-free annotation file linked from the counts page is listed too.
- Each asset gets a stable `id` within the series: `matrix:<GPL>`, `annot:<GPL>`, `ncbi:<file name>`, or `suppl:<file name>`. Ticket 1208 downloads by this ID.
- JSON: `assets: {snapshot_id, coverage: {complete, notes}, assets: [{id, role, producer, format, platform, url, size, has_values, library_strategies}]}`. `role` is `series_matrix`, `raw_gene_counts`, `fpkm`, `tpm`, `gene_annotation`, or `supplementary`. `producer` is `submitter` or `ncbi`. `snapshot_id` is the retrieval time plus a digest of the listed IDs and URLs. It records what was seen and claims no upstream version. `coverage.complete` is false when any listing failed or hit a cap, and `notes` says which.
- `products: [{kind, producer, units, feature_ids, asset_ids, status}]`. `kind` is `array_values`, `raw_gene_counts`, `normalized_counts`, or `unknown`. `feature_ids` is `platform_probe` for arrays and `ncbi_gene_id` for NCBI counts. `status` is `usable_values` or `meaning_unknown`. A supplementary file is always `unknown` and `meaning_unknown`.
- Markdown prints an asset table and a product table, and no ranking line. Products print in the order assets were listed. BioMCP makes no claim that any file suits an analysis.

## Fixtures

Under `testdata/sources/geo/`: the GSE48843 counts page, the GSE982 counts page, one `suppl/` listing, and two trimmed matrix files, one with two data rows and one with only `ID_REF`.

## Acceptance

1. GSE982 fixtures: one matrix asset with `has_values: true`, an `annot:GPL96` asset, and no NCBI counts.
2. GSE48843 fixtures: matrix `has_values: false`, three NCBI assets with roles and build `GRCh38.p13`, and a `raw_gene_counts` product with `feature_ids: ncbi_gene_id`.
3. The scan calls `read_header` with 2 and reads nothing past the second table line.
4. A counts page whose links name another accession yields no rows. A failed `suppl/` listing sets `coverage.complete` to false and keeps the other assets.
5. The same fixtures give the same `snapshot_id` digest part on two runs.
6. `get dataset GSE982 all` makes no matrix, counts page, or `suppl/` request.
7. Spec `spec/entity/dataset.md` gains one block for each fixture series, served by the local fixture server.
8. `docs/sources/ncbi-geo.md` explains assets versus products, the three file kinds, the `geo2r` caveat, and links the NCBI counts page.
9. The recorded fixture for `Human.GRCh38.p13.annot.tsv.gz` came from a URL that returned gzip, and that URL is written in the fixture receipt. A scripted fetch of the page's link on 2026-09-16 did not return gzip, so the builder finds the working URL first.

## Out of scope

- Downloading any listed file (ticket 1208), reading count values, or mapping GeneIDs to symbols (ticket 1209).
- Probe-to-gene mapping for array platforms.
- Choosing between raw, FPKM, and TPM.
