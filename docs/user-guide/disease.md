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

## Disease sections

Genes (Monarch-backed associations with relationship/source when available):

```bash
biomcp get disease MONDO:0005105 genes
```

Phenotypes (HPO phenotypes with qualifiers):

```bash
biomcp get disease MONDO:0005105 phenotypes
```

Variants (Monarch disease-associated variants):

```bash
biomcp get disease MONDO:0005105 variants
```

Models (Monarch model-organism evidence):

```bash
biomcp get disease MONDO:0005105 models
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

## Practical tips

- Prefer MONDO IDs in automation workflows.
- Keep raw labels in user-facing notes for readability.
- Pair disease normalization with biomarker filters for trial matching.

## Related guides

- [Trial](trial.md)
- [Article](article.md)
- [Data sources](../reference/data-sources.md)
