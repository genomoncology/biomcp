---
flow: build
priority: 3
deps: [1208]
---

# 1209: Study import builds a local study from a dataset asset

## Goal

`biomcp study import` turns one fetched GEO value asset plus a user-written sample map into a local study in the cBioPortal DataHub layout. The existing `study list`, `study query --type expression`, and `study filter --expression-above/--expression-below` then work on GEO data with no new analysis code. This is step 3 of the 2026-09-16 research-data decision brief.

```
biomcp study import --name gse48843_counts \
  --from geo:GSE48843 --asset ncbi:GSE48843_raw_counts_GRCh38.p13_NCBI.tsv.gz \
  --sample-map samples.tsv
biomcp study query --study gse48843_counts --gene CD34 --type expression
```

## Current Facts

- Local studies live under `resolve_study_root()` (`src/sources/cbioportal_study.rs:278`, override `BIOMCP_STUDY_DIR`).
- A study needs `meta_study.txt`, parsed by `parse_meta_study` (`src/sources/cbioportal_study.rs:1483`) with `cancer_study_identifier`, `name`, and `type_of_cancer`.
- Samples come from `data_clinical_sample.txt`. Parsers skip blank lines and lines that start with `#` (`src/sources/cbioportal_study.rs:2181`, `:2205`). `clinical_column_values` (`:991`) reads a named column.
- Expression comes from the first file in `EXPRESSION_FILES` (`src/sources/cbioportal_study.rs:15`) that exists. The format is `Hugo_Symbol`, `Entrez_Gene_Id`, then one column per sample (test fixture at `:2305`).
- Ticket 1207 marks NCBI counts with `feature_ids: ncbi_gene_id` and arrays with `platform_probe`. Ticket 1208 fetches assets and records their paths.
- NCBI publishes platform annotation files with gene symbol and gene ID columns for many arrays. `https://ftp.ncbi.nlm.nih.gov/geo/platforms/GPLnnn/GPL96/annot/GPL96.annot.gz` returned HTTP 200 on 2026-09-16. The sequencing platform GPL24676 has no `annot/` folder (HTTP 404) and needs none.

## Design

### Sample map

A TSV the user writes, with header. `sample_id` (a GSM accession) is required. Every other column is copied as a clinical sample attribute, with the header uppercased (`agent` becomes `AGENT`). BioMCP does not read, check, or change the values. The map may list a subset of samples. Import keeps only those samples in the expression file. A map sample that is missing from the asset is an error naming it.

### Feature mapping

- NCBI counts: the gene ID column becomes `Entrez_Gene_Id`. The symbol comes from the NCBI annotation asset for that build, fetched through 1208 (`--annotation <asset-id>`, required for this producer). A gene ID with no symbol is dropped and counted.
- Arrays: the matrix table (fetched through 1208 as `matrix:<GPL>`) is read past the header. The GPL annotation file maps probe to symbol and gene ID. The user fetches it through 1208 as `annot:<GPL>` and passes it with `--annotation`, the same flag NCBI counts use. A platform with no annotation file fails with a message naming the platform. Probes with no gene, or with several genes, are dropped and counted. Several probes for one gene keep the probe with the highest mean across the mapped samples. This is the only aggregation rule, and `meta_study.txt` records it.
- Values are copied unchanged. BioMCP does not normalize, log-transform, or z-score.

### Output

- Folder `<study root>/<name>/` staged, then renamed into place. An existing name fails unless `--replace`.
- `meta_study.txt`: `cancer_study_identifier: <name>`, `name` from the dataset title, `type_of_cancer: other` unless `--cancer-type` is given, and `description` stating source ID, asset ID, SHA-256 from the 1208 manifest, product kind, units as reported by 1207, the probe rule, and dropped-feature counts.
- `data_clinical_sample.txt`: `SAMPLE_ID`, `PATIENT_ID` (set to the sample ID), and the map columns, with the four standard `#` header lines.
- `data_expression_imported.txt`: the mapped matrix. `EXPRESSION_FILES` gains this name last, so existing DataHub studies keep their current file.
- `import.json`: the full provenance, including the sample map SHA-256.

`study import` is CLI-only. The MCP shell rejects it.

## Acceptance

1. NCBI counts fixture plus annotation fixture plus a three-sample map: `study list` shows the study, `study query --type expression --gene` returns the three values unchanged.
2. Array fixture with two probes for one gene keeps the higher-mean probe. `meta_study.txt` names the rule and the dropped counts.
3. A platform with no annotation file fails before any study folder exists.
4. A map sample missing from the asset fails and names the sample.
5. `study filter --study <name> --expression-above GENE:X` works on the imported study.
6. Map columns come back byte-identical through `clinical_column_values`.
7. An existing DataHub fixture study still reads its original expression file.
8. The MCP shell rejects `study import`.
9. Spec `spec/entity/study.md` gains one import-then-query block on fixtures.

## Out of scope

- Writing or suggesting sample labels, pairing treated and control samples, and cross-study merging.
- Normalization, batch correction, and scaling between arrays and counts.
- Supplementary files with unknown meaning. Only 1207 products with `usable_values` import.
- Grouped comparison by a map column (ticket 1210).
