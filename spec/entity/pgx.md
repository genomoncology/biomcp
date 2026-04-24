# Pharmacogenomics Queries

BioMCP's PGx surface connects genes, drugs, and CPIC guidance without forcing
users to switch tools or guess which source backed the answer. These canaries
focus on CPIC-style interaction tables plus opt-in recommendation and frequency
detail.

## Gene-First Search

Searching by pharmacogene should keep the interaction table shape visible so a
reader can immediately see which drugs are affected.

```bash
out="$(../../tools/biomcp-ci search pgx CYP2D6 --limit 3)"
echo "$out" | mustmatch like "# PGx Search: gene=CYP2D6"
echo "$out" | mustmatch like "| Gene | Drug | CPIC Level | PGx Testing | Guideline |"
echo "$out" | mustmatch like "| CYP2D6 | codeine | A | Actionable PGx |"
```

## Drug-First Search

Drug lookup is a first-class path too. It should route through the same CPIC
interaction surface instead of an undocumented special-case workflow.

```bash
out="$(../../tools/biomcp-ci search pgx --drug clopidogrel --limit 3)"
echo "$out" | mustmatch like "# PGx Search: drug=clopidogrel"
echo "$out" | mustmatch like "| CYP2C19 | clopidogrel | A | Actionable PGx |"
```

## Recommendations Stay Opt-In

Recommendation detail belongs behind an explicit deepen path so the default
interaction card stays readable.

```bash
out="$(../../tools/biomcp-ci get pgx CYP2D6 recommendations)"
echo "$out" | mustmatch like "# CYP2D6 - recommendations"
echo "$out" | mustmatch like "## Recommendations (CPIC)"
echo "$out" | mustmatch like "| Drug | Phenotype | Activity Score | Recommendation | Classification |"
```

## Population Frequencies

Population allele frequencies should stay available as their own section and
render with explicit population/frequency columns instead of disappearing into
free text.

```bash
out="$(../../tools/biomcp-ci get pgx CYP2D6 frequencies)"
echo "$out" | mustmatch like "# CYP2D6 - frequencies"
echo "$out" | mustmatch like "## Population Frequencies (CPIC)"
echo "$out" | mustmatch like "| Gene | Allele | Population | Frequency | Subjects |"
```
