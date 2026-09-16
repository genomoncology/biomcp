---
flow: build
priority: 3
deps: [1204]
---

# 1207: Dataset files section shows where values live

## Goal

`biomcp get dataset <GSE> files` tells an agent where a GEO series keeps its expression values before anything large is downloaded. It lists three kinds of file with URLs: series matrix files and whether each carries a value table, NCBI-computed RNA-seq count files, and the series supplementary files. BioMCP lists links and sizes it can read cheaply. It downloads none of the listed files.

```
biomcp get dataset GSE48843 files
biomcp get dataset GSE982 files
```

## Current Facts

Measured 2026-09-16 against 588 AML series that a hackathon team screening public GEO studies of drug-treated AML cell lines had collected:

- 774 series matrix files downloaded. 120 of them (107 series) have rows between `!series_matrix_table_begin` and `!series_matrix_table_end`. The other 654 have only the `ID_REF` header. By first-sample `!Sample_library_strategy`, 490 of the empty files are RNA-Seq, 63 ChIP-Seq, and 22 ATAC-seq.
- ESummary `geo2r` was `yes` for 200 series whose matrix table was empty. For the first of these, GSE48843, the page `https://www.ncbi.nlm.nih.gov/geo/download/?type=rnaseq_counts&acc=GSE48843` links three files: `GSE48843_raw_counts_GRCh38.p13_NCBI.tsv.gz`, `GSE48843_norm_counts_FPKM_GRCh38.p13_NCBI.tsv.gz`, and `GSE48843_norm_counts_TPM_GRCh38.p13_NCBI.tsv.gz`. Each link has the form `/geo/download/?type=rnaseq_counts&acc=<GSE>&format=file&file=<name>`. The raw counts file is 1.5 MB gzip with 39,376 NCBI GeneID rows and one column per GSM. The page also links `Human.GRCh38.p13.annot.tsv.gz` without an accession. A scripted fetch of that link did not return gzip, so the fixture recording must confirm the working URL.
- The same page for GSE982, an array series, returns 6,474 bytes and no `file=` links.
- NCBI describes the pipeline at `https://www.ncbi.nlm.nih.gov/geo/info/rnaseqcounts.html`.
- ESummary `suppfile` names supplementary types only. The series `suppl/` folder under `https://ftp.ncbi.nlm.nih.gov/geo/series/<prefix>/<GSE>/suppl/` lists the actual files. Among the empty-table series, types were TXT 166, CSV 42, TSV 34, XLSX 23, and BW 20.
- Ticket 1204 adds `series_prefix`, the streaming matrix reader, and the `BIOMCP_GEO_FTP_BASE` override. Ticket 1203 adds `supplementary_types` and `geo2r` to dataset rows.

## Design

Section `files` on `get dataset`. It is opt-in and not part of `all`, because it reads the matrix files.

- **Matrix.** For each platform from ESummary, build the matrix URL with 1204's rules. Stream the file with 1204's reader to the table marker, then read at most two lines past it. Report `has_values` as true when a data row follows the `ID_REF` header. Report `library_strategies` as the distinct `!Sample_library_strategy` values in file order. Stop reading at the second line after the marker.
- **NCBI counts.** Fetch the `rnaseq_counts` page for the accession. Collect `file=` links whose `acc` matches. Classify each by name: `raw_counts`, `norm_counts_FPKM`, `norm_counts_TPM`, and the genome build between the kind and `_NCBI`. An empty list is a normal result. Add an override `BIOMCP_GEO_WEB_BASE` for `https://www.ncbi.nlm.nih.gov`.
- **Supplementary.** Fetch the `suppl/` listing. Report each file name and URL. Report sizes only when the listing prints them.
- JSON: `files: {matrix: [{platform, url, has_values, library_strategies}], ncbi_counts: [{kind, build, url}], supplementary: [{name, url, size}]}`. Markdown prints three short tables and one line that names the first non-empty source in the order matrix values, NCBI counts, supplementary.
- The line is a pointer. BioMCP makes no claim that any file suits an analysis.

## Fixtures

Under `testdata/sources/geo/`: the GSE48843 counts page, the GSE982 counts page, one `suppl/` listing, and two trimmed matrix files, one with two data rows and one with only `ID_REF`.

## Acceptance

1. GSE982 fixtures: one matrix row with `has_values: true`, no NCBI counts, pointer names the matrix.
2. GSE48843 fixtures: matrix `has_values: false`, three NCBI count files with kinds and build `GRCh38.p13`, pointer names NCBI counts.
3. The reader never reads more than two lines past the table marker. A test counts bytes consumed.
4. A counts page whose links name another accession yields no rows.
5. `get dataset GSE982 all` makes no matrix, counts page, or `suppl/` request.
6. Spec `spec/entity/dataset.md` gains one block for each fixture series, served by the local fixture server.
7. `docs/sources/ncbi-geo.md` explains the three file kinds, the `geo2r` caveat, and links the NCBI counts page.

## Out of scope

- Downloading any listed file, reading count values, or mapping GeneIDs to symbols.
- Probe-to-gene mapping for array platforms.
- Choosing between raw, FPKM, and TPM.
