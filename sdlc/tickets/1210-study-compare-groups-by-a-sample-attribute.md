---
flow: build
priority: 3
deps: [1209]
---

# 1210: Study compare groups by a sample attribute

## Goal

`biomcp study compare --study <id> --group-by <COLUMN> --type expression --target <GENE>` compares one gene's expression across the groups a clinical sample column defines. With an imported GEO study (ticket 1209), a user-written `AGENT` column gives a drug versus DMSO comparison. BioMCP computes the numbers and never says which group is the control or whether a difference is good.

```
biomcp study compare --study gse48843_counts --group-by AGENT --type expression --target CD34
biomcp study compare --study gse48843_counts --group-by AGENT --type expression --target CD34 --groups venetoclax,DMSO
```

## Current Facts

- `study compare` requires `--gene` and splits samples into mutant and wildtype (`compare_expression_with_root`, `src/entities/study.rs:719`, calling `compare_expression_by_mutation`).
- `study filter` already reads clinical labels (`--cancer-type`). `clinical_column_values` reads any named column (`src/sources/cbioportal_study.rs:991`).
- `expression_pairs_by_sample` returns sample and value pairs (`src/entities/study.rs:523`).

## Design

- `--group-by <COLUMN>` and `--gene` are mutually exclusive, and one is required. Existing `--gene` behavior is unchanged.
- `--group-by` works with `--type expression` only. `--type mutations` with `--group-by` fails with a message.
- Groups are the distinct non-empty column values, compared exactly as written. Samples with an empty value are left out and counted.
- `--groups a,b` keeps only those groups, in that order. An unknown group name fails and lists the values present.
- Per group: sample count, mean, median, standard deviation, minimum, maximum. With exactly two groups, each with at least two samples, add the difference in means (second minus first) and a Welch t-test p-value. With more than two groups, or a group under two samples, report the summaries and state why no test ran.
- JSON follows the existing expression comparison shape with a `grouping: {column, groups}` field in place of the mutation gene.
- Output never labels a group as control or treatment and never states a direction as better or worse.

## Acceptance

1. A fixture study with `AGENT` values `drugA` (3 samples), `DMSO` (3), and one blank: two groups, one excluded sample, summaries match hand-computed values, and a Welch p-value matches a reference computed outside BioMCP and pinned in the test.
2. `--groups DMSO,drugA` reverses the difference sign.
3. A group with one sample reports summaries and no test.
4. `--gene` and `--group-by` together fail. `--type mutations --group-by` fails.
5. Existing mutation-group compare tests pass unchanged.
6. The MCP shell allows the command, as it allows `study compare` today.
7. Spec `spec/entity/study.md` gains one grouped compare block.

## Out of scope

- Signature scores (a later `study score` ticket if demand appears).
- Pairing samples within a group, multiple-testing correction, and cross-study ranking.
