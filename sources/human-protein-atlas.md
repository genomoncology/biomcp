---
title: "Human Protein Atlas MCP Tool for Tissue Expression | BioMCP"
description: "Use BioMCP to surface Human Protein Atlas tissue expression and localization data through the BioMCP gene hpa section."
---

# Human Protein Atlas

Human Protein Atlas matters when gene-level summaries are not enough and you need tissue-aware protein context. It is the page to use when a workflow depends on where a protein is expressed, how it localizes inside cells, or whether cancer-expression context helps explain why a target matters.

In BioMCP, Human Protein Atlas is surfaced through the gene `hpa` section. That section focuses on tissue expression, subcellular localization, and cancer expression rather than trying to reproduce the full Human Protein Atlas website or every assay detail.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `get gene <symbol> hpa` | Protein tissue expression, localization, and cancer-expression context | Explicit gene section backed by Human Protein Atlas |

## Example commands

```bash
biomcp get gene BRAF hpa
```

Returns Human Protein Atlas tissue-expression and localization context for BRAF.

```bash
biomcp get gene EGFR hpa
```

Returns Human Protein Atlas tissue-expression context for EGFR.

```bash
biomcp get gene TP53 hpa
```

Returns Human Protein Atlas tissue and cancer-expression context for TP53.

## API access

No BioMCP API key required.

## Official source

[Human Protein Atlas](https://www.proteinatlas.org/) is the official tissue-expression and localization resource behind BioMCP's gene `hpa` section.

## Related docs

- [Gene](../user-guide/gene.md)
- [Data Sources](../reference/data-sources.md)
- [Source Licensing and Terms](../reference/source-licensing.md)
