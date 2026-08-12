---
title: "gnomAD MCP Tool for Variant Frequency Analysis | BioMCP"
description: "Use BioMCP to pull gnomAD population frequencies and gene constraint metrics for variant interpretation, rarity checks, and gene-level context."
---

# gnomAD

gnomAD is the quickest way to answer the question that often changes an interpretation: how rare is this variant, and how constrained is this gene in population data? It matters because frequency and constraint are among the fastest filters for separating plausible signals from common background variation.

In BioMCP, gene constraint comes from the gnomAD source path directly, and
variant population data now does too. Variant population requests are pinned to
the `gnomad_r4` dataset and require a trustworthy GRCh38 coordinate. This
replaces population fields previously copied from MyVariant.info payloads.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `get gene <symbol> constraint` | Gene-level constraint metrics such as LOEUF-style context | Direct gnomAD-backed gene section |
| `get variant <id> population` | Separate exome/genome frequency, ancestry counts, grpmax FAF95, and quality flags | Direct `gnomad_r4` GraphQL query for the resolved GRCh38 coordinate |
| `search variant -g <gene> --max-frequency <value>` | Rarity-filtered variant search rows | Search filter uses population-frequency context aligned with gnomAD fields |

## Example commands

```bash
biomcp get gene BRAF constraint
```

Returns a constraint section with gnomAD provenance and LOEUF-style metrics.

```bash
biomcp get variant rs113488022 population
```

Returns direct gnomAD v4 exome and genome population results.
JSON keeps raw numeric frequencies, counts, FAF95, ancestry rows, and source flag names.

```bash
biomcp get variant "chr7:g.140453136A>T" population
```

Returns the same population section after resolving a trustworthy GRCh38 identity.
A GRCh37-only or unknown-build result explains the requirement and does not query gnomAD.

```bash
biomcp search variant -g BRCA1 --max-frequency 0.01 --limit 5
```

Returns variant rows constrained by a rarity filter.

## API access

No BioMCP API key required.

## Official source

[gnomAD](https://gnomad.broadinstitute.org/) is the official Broad Institute population-resource homepage behind these frequency and constraint views.

## Related docs

- [Gene](../user-guide/gene.md)
- [Variant](../user-guide/variant.md)
- [Source Licensing and Terms](../reference/source-licensing.md)
