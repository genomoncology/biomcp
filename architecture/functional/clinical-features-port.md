# Disease Clinical Features Architecture

This document describes the shipped disease `clinical_features` section. It is
an opt-in Monarch/HPO-backed view over the same backend phenotype annotations
used by `get disease <name_or_id> phenotypes`.

## Contract

`get disease <name_or_id> clinical_features` is explicit opt-in disease
section. It is not included in `all`, so broad disease lookups do not add a
second phenotype-style table unexpectedly.

The section renders backend `DiseasePhenotype` rows directly. It does not
reshape those rows into the former clinical-summary table shape.

## Source and provenance

The source is Monarch Initiative / HPO phenotype annotation data. The markdown
heading is `Clinical Features (Monarch / HPO)`, row sources are the backend
phenotype source labels, and `_meta.section_sources` reports Monarch Initiative
and HPO.

If the backend has no phenotype annotations for the disease, the section shows a
truthful Monarch/HPO empty state rather than a curated substitute.

## Runtime flow

1. Section parsing recognizes `clinical_features` only when the caller names it.
2. Disease enrichment computes a shared phenotype need flag for `phenotypes` or
   `clinical_features`.
3. That shared flag runs the existing Monarch/HPO enrichment path once:
   `add_monarch_phenotypes` and `add_phenotypes_section`.
4. Requested `clinical_features` receives the backend phenotype rows directly.
5. Requested `phenotypes clinical_features` uses the same enrichment pass and
   does not duplicate backend calls.

## Output shape

Markdown uses the phenotype table columns:

| HPO ID | Name | Evidence | Frequency | Onset | Sex | Stage | Source |

JSON exposes requested clinical-feature rows with the same backend phenotype row
shape. There is no compatibility projection to `Rank | Feature | HPO |
Confidence | Evidence | Source`.

## Validation

- `spec/entity/disease.md::Clinical Features` exercises the public live CLI path
  with a disease outside the former curated set.
- Routine gates remain `make lint`, `make test`, and `make spec`.
- Live/operator verification remains `make verify`.
