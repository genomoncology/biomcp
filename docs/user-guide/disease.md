# Disease

Use disease commands for normalization and disease-centric cross-entity pivots.

## Search diseases

```bash
biomcp search disease -q melanoma --limit 5
biomcp search disease -q glioblastoma --source mondo --limit 5
```

Search resolves common labels toward canonical ontology-backed identifiers.

## Get disease records

By label:

```bash
biomcp get disease melanoma
```

By MONDO identifier:

```bash
biomcp get disease MONDO:0005105
```

The base disease card includes concise OpenTargets gene-score summaries when OpenTargets
returns ranked associated targets.

## Disease sections

Genes (Monarch-backed associations with relationship/source when available, augmented with OpenTargets scores when present):

```bash
biomcp get disease MONDO:0005105 genes
```

Phenotypes (HPO phenotypes with qualifiers):

```bash
biomcp get disease MONDO:0005105 phenotypes
```

Variants (CIViC disease-associated variants):

```bash
biomcp get disease MONDO:0005105 variants
```

When the variants section is loaded, JSON also exposes `top_variant` as the
highest-ranked CIViC-backed association, and markdown shows the same compact
anchor above the full variants table.

Models (Monarch model-organism evidence):

```bash
biomcp get disease MONDO:0005105 models
```

Pathways (associated pathways):

```bash
biomcp get disease MONDO:0005105 pathways
```

Prevalence (prevalence data):

```bash
biomcp get disease MONDO:0005105 prevalence
```

CIViC (clinical evidence):

```bash
biomcp get disease MONDO:0005105 civic
```

Combined sections:

```bash
biomcp get disease MONDO:0005105 genes phenotypes variants models
biomcp get disease MONDO:0005105 all
```

## Phenotype-to-disease search

Use HPO term sets for ranked disease candidates:

```bash
biomcp search phenotype "HP:0001250 HP:0001263" --limit 10
```

You can pass terms space-separated or comma-separated.

## Typical disease-centric workflow

1. Normalize disease label.
2. Pull disease sections (`genes`, `phenotypes`, `variants`, `models`) for context.
3. Use normalized concept for trial or article searches.

Example:

```bash
biomcp get disease MONDO:0005105 genes phenotypes
biomcp search trial -c melanoma --status recruiting --limit 5
biomcp search article -d melanoma --limit 5
```

## JSON mode

```bash
biomcp --json get disease MONDO:0005105 all
biomcp --json search phenotype "HP:0001250 HP:0001263"
```

`biomcp --json get disease MONDO:0005105` now includes `top_gene_scores[]` with
overall OpenTargets scores and any available GWAS, rare-variant, or somatic subtype scores.

## Practical tips

- Prefer MONDO IDs in automation workflows.
- Keep raw labels in user-facing notes for readability.
- Pair disease normalization with biomarker filters for trial matching.

## Related guides

- [Trial](trial.md)
- [Article](article.md)
- [Data sources](../reference/data-sources.md)
