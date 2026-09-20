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
biomcp study score --study gse48843_counts --weights GENE_A:0.5,GENE_B:-0.25 --group-by AGENT --groups DMSO,venetoclax
```

The motivating consumer is a hackathon team that scores drug-treated and DMSO-treated AML cell lines with published LSC6 and LSC17 weights and compares the groups within one study.

## Current Facts

- `expression_values_by_sample` (`src/sources/cbioportal_study.rs:1183`) returns one gene's value per sample as `HashMap<String, f64>`.
- `expression_group_stats` (`:1971`) and `mann_whitney_u_test` (`:1919`) build group summaries and the test. Ticket 1210 adds grouping by a clinical column.
- Ticket 1209 imports GEO values unchanged and writes `measurement_kind`, `normalization_or_transform`, and `feature_id_type` to `import.json` and `meta_study.txt`.
- Experiment 203 measured the weighted sum. Doubling one sample's values doubled its score exactly and moved it from rank 17 to rank 2 of 75. On GSE48843 raw counts, per-million scaling changed the rank of 27 of 32 samples. Dropping a missing gene and filling it with zero give the same sum. Dropping one gene changed the rank of 4 of 32 samples.
- The MCP shell allows the read-only `study` subcommands (`GENERIC_MCP_REJECTION_MESSAGE`, `src/mcp/shell.rs:326`) and withholds local paths. That constant lists the allowed study commands by name in its own text.
- `is_allowed_mcp_command` (`src/mcp/shell.rs:472`) is an exhaustive match over `Commands` and subcommand variants with no wildcard arm, so a new `StudyCommand` variant fails the build until it is classified.

## Design

- Signature input, exactly one of:
  - `--signature <file>`: a TSV with header columns `gene` and `weight`. CLI-only, because it reads a local path.
  - `--weights GENE:W,GENE:W`: inline, allowed in MCP.
- Gene symbols are normalized the way `study query` normalizes them. A duplicate gene is an error. A weight that does not parse as a finite number is an error naming the row.
- A signature gene absent from the expression file fails the command and lists every absent gene. Refusing is the default. With `--allow-missing`, the score uses the genes found. Every run prints genes used out of genes given and the missing list, because dropping a gene and filling it with zero give the same sum.
- Per sample: `score = sum(weight × value)` over the genes used. A sample missing a value for any used gene gets no score and is counted.
- Values are used as stored. No scaling, log transform, or z-score. The output states that no normalization was applied and prints the study's `measurement_kind`, `normalization_or_transform`, and `feature_id_type`, or `unknown`.
- When `measurement_kind` is `raw_counts`, the output warns that a score scales with each sample's sequencing depth.
- Output per sample: sample ID, score, genes used out of genes given. With `--group-by`, the 1210 group summaries of the scores follow, with the same two-group Mann-Whitney rule, `--groups` ordering, and exclusions. The method statement matches 1210: Mann-Whitney U, normal approximation with tie and continuity correction, two-sided, independent samples only.
- JSON: `{study, units: {measurement_kind, normalization_or_transform, feature_id_type}, warnings, signature: {genes, source}, effective_signature: [{gene, weight}], missing_genes, samples: [{sample_id, score, genes_used}], unscored, grouping}`. `signature.source` is `file:<sha256>` or `inline`.
- Output never names a direction as better or worse and never applies a threshold.
- MCP arms: `StudyCommand::Score` is allowed with `--weights` and rejected with `--signature`, which reads a workstation-local path. `GENERIC_MCP_REJECTION_MESSAGE` (`src/mcp/shell.rs:326`) gains `study score --weights` in the list of allowed study commands it names, so a caller reading the rejection learns which form works. The `--signature` rejection reuses `LOCAL_INPUT_MCP_REJECTION_MESSAGE` (`src/mcp/shell.rs:328`), which already covers a CLI-only local file argument.

## Acceptance

1. A fixture study with three genes and four samples: scores match hand-computed values for a file signature and the same inline signature.
2. An absent gene fails and lists it. With `--allow-missing`, the score uses the remaining genes, prints genes used out of genes given, and lists the absent one.
3. A sample with a blank value for a used gene is unscored and counted.
4. `--group-by AGENT --groups DMSO,drugA` returns 1210's summary shape over scores. The p-value matches R `wilcox.test(exact=FALSE, correct=TRUE)` within 1e-6, and JSON carries the method name.
5. The MCP shell rejects `--signature` with the local-input message and allows `--weights`. A test pins that `GENERIC_MCP_REJECTION_MESSAGE` names `study score --weights`.
6. A duplicate gene and a non-numeric weight fail with row-specific messages.
7. A study with `measurement_kind: raw_counts` prints the units and the raw counts warning. A study with no unit fields prints `unknown`.
8. Doubling one fixture sample's values doubles its score exactly.
9. Spec `spec/entity/study.md` gains one score block and one grouped score block on fixtures.

## Out of scope

- Shipping, naming, or suggesting any signature or weights.
- Thresholds, positive or negative calls, pairing treated and control samples, and ranking drugs across studies.
- Scaling within or across studies.

## Decisions

Open to Ian's overturn: a missing gene fails by default. `--allow-missing` reports the effective signature on every run. The score warns on raw counts and never rescales them.


## Review

- Design review: pending
- Code review: pending
