---
title: "PharmGKB and CPIC MCP Tool for Pharmacogenomics | BioMCP"
description: "Use BioMCP to search CPIC-backed PGx guidance and add PharmGKB clinical annotations to gene-drug pharmacogenomic workflows."
---

# PharmGKB / CPIC

Pharmacogenomic workflows often need two things at once: actionable guidance and supporting annotation context. That is why BioMCP treats this page as a combined source guide instead of splitting the user-facing workflow into two nearly identical entry points.

In BioMCP, CPIC supplies the core `get pgx <gene_or_drug>` card plus recommendations, frequencies, and guidelines, while PharmGKB supplies the `annotations` enrichment section. That split matters because the main recommendations users act on come from CPIC even when PharmGKB adds broader clinical annotation context.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `search pgx` | PGx matches for genes and drugs | Search is aligned to the CPIC-backed workflow |
| `get pgx <gene_or_drug>` | Core PGx card for a gene or drug | CPIC-backed base view |
| `get pgx <gene_or_drug> interactions` | Bounded gene-drug interaction page | CPIC pair view |
| `get pgx <gene_or_drug> recommendations` | Dosing and action recommendations | CPIC recommendations section |
| `get pgx <gene_or_drug> frequencies` | Allele frequency context | CPIC frequencies section |
| `get pgx <gene_or_drug> guidelines` | Guideline references and summaries | CPIC guidelines section |
| `get pgx <gene_or_drug> annotations` | Clinical annotations and enrichment | PharmGKB-backed section |

## Example commands

```bash
biomcp search pgx -g CYP2D6 --limit 5
```

Returns PGx search matches for a gene-oriented query.

```bash
biomcp get pgx CYP2D6
```

Returns the base PGx card for the requested gene or drug.

```bash
biomcp get pgx CYP2D6 recommendations
```

Returns CPIC recommendations for the requested PGx target.

```bash
biomcp get pgx CYP2D6 annotations
```

Returns the PharmGKB-backed annotations section.

```bash
biomcp get pgx codeine recommendations frequencies
```

Returns a combined PGx view with recommendations and frequency context for codeine.

## API access

No BioMCP API key required.

## Official source

[PharmGKB](https://www.pharmgkb.org/) is the public pharmacogenomics knowledgebase paired with CPIC in BioMCP's PGx workflow.

## Related docs

- [PGX](../user-guide/pgx.md)
- [Gene](../user-guide/gene.md)
- [Drug](../user-guide/drug.md)
- [Variant](../user-guide/variant.md)
