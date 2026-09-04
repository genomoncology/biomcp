---
title: "Monarch Initiative MCP Tool for Phenotype and Disease Matching | BioMCP"
description: "Use BioMCP to query Monarch-backed disease genes, phenotype matches, and model evidence in BioMCP disease and phenotype workflows."
---

# Monarch Initiative

Monarch Initiative matters when a disease workflow depends on phenotype evidence, cross-species model context, or phenotype-to-disease matching instead of a single disease identifier lookup. It is particularly useful when you need a phenotype-first starting point and then want to pivot into disease records with supporting evidence.

In BioMCP, Monarch is visible in the disease `genes` section, the disease `models` section, and `search phenotype` for ranked HPO-set matching. There is no `get phenotype` subcommand, so phenotype work begins with search and then pivots back into disease records and sections.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `get disease <id> genes` | Disease-gene associations with relationship and provenance context | Monarch-backed disease section that can be augmented with other source scores |
| `get disease <id> phenotypes` | Phenotype associations for a disease | Monarch-backed disease section |
| `get disease <id> models` | Model-organism evidence for a disease | Monarch-backed disease section |
| `search phenotype` | Ranked disease matches from phenotype terms | Search-first phenotype workflow |

## Example commands

```bash
biomcp get disease MONDO:0005105 genes
```

Returns disease-associated genes with Monarch-backed evidence context.

```bash
biomcp get disease MONDO:0005105 phenotypes
```

Returns disease phenotypes and qualifiers for the requested disease.

```bash
biomcp get disease MONDO:0005105 models
```

Returns model-organism evidence for the requested disease.

```bash
biomcp search phenotype "HP:0001250 HP:0001263" --limit 10
```

Returns ranked disease matches from the supplied HPO term set.

## API access

No BioMCP API key required.

## Official source

[Monarch Initiative](https://monarchinitiative.org/) is the official integrated phenotype and disease platform behind BioMCP's Monarch-backed disease and phenotype workflows.

## Related docs

- [Disease](../user-guide/disease.md)
- [Phenotype](../user-guide/phenotype.md)
- [Data Sources](../reference/data-sources.md)
