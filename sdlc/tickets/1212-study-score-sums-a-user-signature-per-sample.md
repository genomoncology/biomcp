---
flow: build
priority: 4
deps: [1210]
---

# 1212: Study score sums a user signature per sample

## Goal

`biomcp study score` computes one number per sample: the sum of each signature gene's expression times its weight. The user supplies the genes and weights. With `--group-by`, it compares the scores across the groups a sample column defines, using the 1210 comparison. BioMCP ships no signature, sets no threshold, and never calls a score good or bad.

```
biomcp study score --study gse48843_counts --signature lsc17.tsv
biomcp study score --study gse48843_counts --weights DNMT3B:0.0874,CD34:0.0171 --group-by AGENT --groups DMSO,venetoclax
```

The motivating consumer is a hackathon team that scores drug-treated and DMSO-treated AML cell lines with published LSC6 and LSC17 weights and compares the groups within one study.

## Current Facts

- `expression_values_by_sample` (`src/sources/cbioportal_study.rs:1183`) returns one gene's value per sample as `HashMap<String, f64>`.
- `expression_group_stats` (`:1971`) and `mann_whitney_u_test` (`:1366`) build group summaries and the test. Ticket 1210 adds grouping by a clinical column.
- Ticket 1209 imports GEO values unchanged and records units only in the `meta_study.txt` description.
- The MCP shell allows the read-only `study` subcommands (`GENERIC_MCP_REJECTION_MESSAGE`, `src/mcp/shell.rs:326`) and withholds local paths.

## Design

- Signature input, exactly one of:
  - `--signature <file>`: a TSV with header columns `gene` and `weight`. CLI-only, because it reads a local path.
  - `--weights GENE:W,GENE:W`: inline, allowed in MCP.
- Gene symbols are normalized the way `study query` normalizes them. A duplicate gene is an error. A weight that does not parse as a finite number is an error naming the row.
- A signature gene absent from the expression file fails the command and lists every absent gene, unless `--allow-missing` is given. With `--allow-missing`, the score uses the genes found, and the output lists the absent genes on every run.
- Per sample: `score = sum(weight × value)` over the genes used. A sample missing a value for any used gene gets no score and is counted.
- Values are used as stored. No scaling, log transform, or z-score. The output repeats the units line from the study description when one exists.
- Output per sample: sample ID, score, genes used out of genes given. With `--group-by`, the 1210 group summaries of the scores follow, with the same two-group Mann-Whitney rule, `--groups` ordering, and exclusions.
- JSON: `{study, signature: {genes, source}, missing_genes, samples: [{sample_id, score, genes_used}], unscored, grouping}`. `signature.source` is `file:<sha256>` or `inline`.
- Output never names a direction as better or worse and never applies a threshold.

## Acceptance

1. A fixture study with three genes and four samples: scores match hand-computed values for a file signature and the same inline signature.
2. An absent gene fails and lists it. With `--allow-missing`, the score uses the remaining genes and lists the absent one.
3. A sample with a blank value for a used gene is unscored and counted.
4. `--group-by AGENT --groups DMSO,drugA` returns 1210's summary shape over scores, with the pinned Mann-Whitney p-value.
5. The MCP shell rejects `--signature` and allows `--weights`.
6. A duplicate gene and a non-numeric weight fail with row-specific messages.
7. Spec `spec/entity/study.md` gains one score block and one grouped score block on fixtures.

## Out of scope

- Shipping, naming, or suggesting any signature or weights.
- Thresholds, positive or negative calls, pairing treated and control samples, and ranking drugs across studies.
- Scaling within or across studies.
