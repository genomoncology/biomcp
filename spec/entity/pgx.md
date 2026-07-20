# Pharmacogenomics Queries

BioMCP's PGx surface connects genes, drugs, and CPIC guidance without forcing
users to switch tools or guess which source backed the answer. These canaries
focus on CPIC-style interaction tables plus opt-in recommendation and frequency
detail.

## Gene-First Search

Searching by pharmacogene should keep the interaction table shape visible so a
reader can immediately see which drugs are affected.

```bash
../../tools/biomcp-ci search pgx CYP2D6 --limit 3 | mustmatch like '# PGx Search: gene=CYP2D6
| Gene | Drug | CPIC Level | PGx Testing | Guideline |
| CYP2D6 | codeine | A | Actionable PGx |'
```

The testing-recommendation filter accepts the provider's documented category
and keeps matching interactions visible.

```bash
set -o pipefail
../../tools/biomcp-ci search pgx CYP2D6 --pgx-testing "Actionable PGx" --limit 3 \
  | mustmatch like '| CYP2D6 | codeine | A | Actionable PGx |'
```

## Drug-First Search

Drug lookup is a first-class path too. It should route through the same CPIC
interaction surface instead of an undocumented special-case workflow.

```bash
../../tools/biomcp-ci search pgx --drug clopidogrel --limit 3 | mustmatch like '# PGx Search: drug=clopidogrel
| CYP2C19 | clopidogrel | A | Actionable PGx |'
```

## Recommendations Stay Opt-In

Recommendation detail belongs behind an explicit deepen path so the default
interaction card stays readable.

```bash
../../tools/biomcp-ci get pgx CYP2D6 recommendations | mustmatch like '# CYP2D6 - recommendations
## Recommendations (CPIC)
| Drug | Phenotype | Activity Score | Recommendation | Classification |'
```

## Population Frequencies

Population allele frequencies should stay available as their own section and
render with explicit population/frequency columns instead of disappearing into
free text.

```bash
../../tools/biomcp-ci get pgx CYP2D6 frequencies | mustmatch like '# CYP2D6 - frequencies
## Population Frequencies (CPIC)
| Gene | Allele | Population | Frequency | Subjects |'
```
