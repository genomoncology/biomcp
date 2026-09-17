---
flow: build
priority: 3
deps: [1208]
---

# 1209: Study import builds a local study from a dataset asset

## Goal

`biomcp study import` turns one downloaded GEO value asset plus a user-written sample map into a local study in the cBioPortal DataHub layout. The existing `study list`, `study query --type expression`, and `study filter --expression-above/--expression-below` then work on GEO data with no new analysis code. This is step 3 of the 2026-09-16 research-data decision brief.

```
biomcp study import --name gse48843_counts \
  --from geo:GSE48843 --asset ncbi:GSE48843_raw_counts_GRCh38.p13_NCBI.tsv.gz \
  --annotation gene_info:Homo_sapiens --sample-map samples.tsv
biomcp study query --study gse48843_counts --gene CD34 --type expression
```

## Current Facts

- Local studies live under `resolve_study_root()` (`src/sources/cbioportal_study.rs:278`, override `BIOMCP_STUDY_DIR`, default `dirs::data_dir()/biomcp/studies` at `src/sources/cbioportal_study.rs:287`). Ticket 1208 adds `BIOMCP_DATA_DIR` above that default, with `BIOMCP_STUDY_DIR` still taking precedence.
- Ticket 1208 records `upstream_last_updated` in each dataset manifest entry and gives the manifest a series-level `imports` list. Its writes take the series `manifest.lock`.
- A study needs `meta_study.txt`, parsed by `parse_meta_study` (`src/sources/cbioportal_study.rs:1483`) with `cancer_study_identifier`, `name`, and `type_of_cancer`.
- Samples come from `data_clinical_sample.txt`. Parsers skip blank lines and lines that start with `#` (`src/sources/cbioportal_study.rs:2181`, `:2205`). `clinical_column_values` (`:991`) reads a named column.
- Expression comes from the first file in `EXPRESSION_FILES` (`src/sources/cbioportal_study.rs:15`) that exists. The format is `Hugo_Symbol`, `Entrez_Gene_Id`, then one column per sample (test fixture at `:2305`).
- Ticket 1207 marks each product with `measurement_kind`, `normalization_or_transform`, and `feature_id_type` (`ncbi_gene_id` for NCBI counts, `platform_probe` for arrays). Ticket 1208 downloads assets and records their paths.
- A script cannot fetch the counts-page annotation file (ticket 1207). NCBI Gene `Homo_sapiens.gene_info.gz` mapped 37,663 of 39,376 GSE48843 count rows (95.65%). `gene_history.gz` sorted the 1,713 misses into 1,334 retired IDs, 378 IDs replaced by a current ID, and 1 ID replaced by an ID that is not current. The symbol `TRNAV-CAC` came from two GeneIDs (experiment 203).
- `GPL96.annot.gz` has an `!Annotation_date` line (Aug 09 2016) and its table between `!platform_table_begin` and `!platform_table_end`. On GSE982, 1,223 probes list several genes, always separated by `///`. Choosing the max-mean probe over all 75 samples and over a 15-sample subset gave different probes for 275 of 12,502 genes.
- The expression reader returns the first row when several rows share a symbol. It drops blank and non-finite cells without saying so.
- Every study command failure prints `Source unavailable: cBioPortal DataHub is not available.` in Markdown and JSON (`src/error.rs:481`, `:572`). The internal reason, such as `Missing study metadata file` (`src/sources/cbioportal_study.rs:1486`), is dropped.
- `study filter --cancer-type` ignores case (`src/sources/cbioportal_study.rs:1448`).
- NCBI publishes platform annotation files with gene symbol and gene ID columns for many arrays. `https://ftp.ncbi.nlm.nih.gov/geo/platforms/GPLnnn/GPL96/annot/GPL96.annot.gz` returned HTTP 200 on 2026-09-16. The sequencing platform GPL24676 has no `annot/` folder (HTTP 404) and needs none.

## Design

### Sample map

A TSV the user writes, with header. `sample_id` (a GSM accession) is required. Every other column is copied as a clinical sample attribute, with the header uppercased (`agent` becomes `AGENT`). BioMCP does not read, check, or change the values. Ticket 1210 compares group values exactly as written. `undiff` and `Undiff` form two groups. The map may list a subset of samples. Import keeps only those samples in the expression file.

- A `sample_id` listed twice fails and names the ID.
- Map samples missing from the asset fail the import, and the error names every one.
- Asset columns absent from the map are reported as unmatched columns in the output and in `import.json`.

Docs say that sample maps usually come from characteristics keys (`treatment`, `agent`) and sample titles. The treatment protocol is usually the same for every sample. Docs also say that group values match exactly as written, while `study filter --cancer-type` ignores case.

### Feature mapping

- NCBI counts: the gene ID column becomes `Entrez_Gene_Id`. The symbol comes from NCBI Gene `gene_info` for the organism, downloaded through 1208 (`--annotation gene_info:<organism>`, required for this producer). A gene ID with no symbol is dropped. When `--gene-history gene_history` is given, dropped rows are split into retired IDs and replaced IDs. Otherwise they are counted as one number.
- Arrays: the matrix table (downloaded through 1208 as `matrix:<GPL>`) is read past the header. The GPL annotation file maps probe to symbol and gene ID. The user downloads it through 1208 as `annot:<GPL>` and passes it with `--annotation`, the same flag NCBI counts use. A platform with no annotation file fails with a message naming the platform. Multi-gene probes are split on `///`. Probes with no gene, or with several genes, are dropped and counted. `--probe-rule max-mean` is the only value today and the default. It keeps the probe with the highest mean, the `MaxMean` rule of WGCNA `collapseRows`. The mean is computed once over the samples kept by the map and never uses group labels. Blank cells are skipped in the mean. A tie goes to the lowest probe ID in byte order. `meta_study.txt` records the rule. `import.json` records the rule and the annotation file date from `!Annotation_date`.
- One row per symbol: when two features map to the same symbol, import keeps the first in file order, drops the rest, and counts them as symbol collisions.
- Values are copied unchanged. BioMCP does not normalize, log-transform, or z-score.

### Output

- Folder `<study root>/<name>/` staged, then renamed into place. An existing name fails unless `--replace`.
- `meta_study.txt`: `cancer_study_identifier: <name>`, `name` from the dataset title, `type_of_cancer: other`; when `--cancer-type` is given the same value also fills a `CANCER_TYPE` column in `data_clinical_sample.txt`, because `study filter --cancer-type` reads that column (`src/sources/cbioportal_study.rs:1442-1443`); the machine unit fields `measurement_kind`, `normalization_or_transform`, and `feature_id_type` from 1207, with `unknown` allowed; and `description` stating source ID, asset IDs, SHA-256 from the 1208 manifest, `upstream_last_updated` from the same manifest entry (or `unknown` when it is null), product kind, the probe rule, and dropped-feature counts.
- `data_clinical_sample.txt`: `SAMPLE_ID`, `PATIENT_ID` (set to the sample ID), and the map columns, with the four standard `#` header lines.
- `data_expression_imported.txt`: the mapped matrix. `EXPRESSION_FILES` gains this name last, so existing DataHub studies keep their current file.
- `import.json`: the full provenance, including the sample map SHA-256, `upstream_last_updated` copied from the 1208 manifest entry for the value asset, the import time, the three unit fields, the annotation file date, dropped-row counts (retired, replaced, and other), symbol collisions, and unmatched matrix columns. `upstream_last_updated` is `null` when the asset had none, which is the honest answer for NCBI count files and supplementary files.
- After the study folder is renamed into place, import appends `{study: <name>, imported_at: <time>, asset_id: <value asset>}` to the dataset manifest's `imports` list, under the series `manifest.lock` that ticket 1208 defines. A repeat import of the same study name with `--replace` replaces the existing entry rather than adding a second one. A failed import appends nothing. When the manifest cannot be locked or written, the study is already installed and the command exits nonzero with a message naming the manifest path and the study, so the user can re-run `study import --replace`.
- Output carries `data_as_of`, set to `upstream_last_updated` when the manifest entry has one and to the manifest `downloaded_at` otherwise, with `data_as_of_kind` naming which. Markdown ends with the GEO attribution line from ticket 1204.
- No `data_clinical_patient.txt` and no `data_mutations.txt`. Commands that need them fail with the existing missing-file messages. The study commands are unit-blind (`--expression-above` compares a bare number). The unit fields let `study score` (ticket 1212) print and check units.

### Error rendering

Study command errors keep their reason. Markdown and JSON both name the missing file, for example `data_mutations.txt` or `meta_study.txt`, in place of the bare `cBioPortal DataHub is not available` line.

`study import` is CLI-only. The MCP shell rejects it.

### Docs

- State that analysis works offline. Once the assets are on disk, `study import`, `study query`, `study filter`, `study compare`, and `study score` make no network request, and `dataset path` is the way to reach the files without a network.
- State that an imported study is frozen. Nothing refreshes it, and a later `dataset download --refresh` never changes an imported input. The study keeps the SHA-256 it recorded.
- State the study root resolution order: `BIOMCP_STUDY_DIR`, then `BIOMCP_DATA_DIR/studies/`, then the platform data directory.

## Acceptance

1. NCBI counts fixture plus annotation fixture plus a three-sample map: `study list` shows the study, `study query --type expression --gene` returns the three values unchanged.
2. Array fixture with two probes for one gene keeps the higher-mean probe. `meta_study.txt` names the rule and the dropped counts.
3. Max-mean uses only the kept samples: a fixture where the choice flips when a sample is excluded picks the probe for the kept set. A tie picks the lowest probe ID in byte order. A blank cell is skipped in the mean. A `///` probe is dropped and counted. `import.json` records the annotation date.
4. NCBI counts with a `gene_info` fixture and a `gene_history` fixture report dropped rows as retired and replaced.
5. Two GeneIDs with one symbol produce one row and a collision count of 1.
6. A map with a repeated `sample_id` fails and names it. Asset columns absent from the map appear as unmatched columns.
7. `import.json` and `meta_study.txt` carry `measurement_kind`, `normalization_or_transform`, and `feature_id_type`.
8. A platform with no annotation file fails before any study folder exists.
9. A map sample missing from the asset fails and names the sample.
10. `study filter --study <name> --expression-above GENE:X` works on the imported study.
11. Map columns come back byte-identical through `clinical_column_values`.
12. An existing DataHub fixture study still reads its original expression file.
13. The MCP shell rejects `study import`.
14. Spec `spec/entity/study.md` gains one import-then-query block on fixtures.
15. `study survival`, `study co-occurrence`, and `study compare --gene` on the imported study fail with a message that names the missing file, in Markdown and in JSON. A test pins both forms.
16. `--cancer-type AML` makes `study filter --cancer-type AML` return every imported sample.
17. `import.json` and the `meta_study.txt` description carry `upstream_last_updated` from the manifest entry. A manifest entry with a null value writes `null` in `import.json` and `unknown` in the description.
18. The dataset manifest gains one `imports` entry with the study name, the import time, and the asset ID. A second import under the same name with `--replace` leaves one entry. A failed import leaves the list unchanged.
19. `study query --type expression` on the imported study makes no network request. The test asserts an empty request log.

## Out of scope

- Writing or suggesting sample labels, pairing treated and control samples, and cross-study merging.
- Normalization, batch correction, and scaling between arrays and counts.
- Supplementary files with unknown meaning. Only 1207 products with `usable_values` import.
- Grouped comparison by a map column (ticket 1210) and signature scores (ticket 1212).

## Decisions

Open to Ian's overturn: `--gene-history` is optional because `gene_history.gz` is 162 MB. Without it, dropped GeneIDs are one count. NCBI counts map through NCBI Gene `gene_info` because a script cannot fetch the counts-page annotation file. Collisions keep the first row in file order, matching what the expression reader already returns.

Open to Ian's overturn: an import that installs the study and then fails to update the dataset manifest exits nonzero and keeps the study. The study is the expensive artifact and the manifest entry is a pointer the user can restore with `--replace`.

## Review

- Design review: pending
- Code review: pending
