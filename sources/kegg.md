---
title: "KEGG MCP Tool for Pathway Search | BioMCP"
description: "Use BioMCP to search KEGG pathways, fetch KEGG pathway summaries, and expand to pathway genes without learning KEGG's flat-file API."
---

# KEGG

KEGG matters when you want pathway names and stable IDs that are widely recognized across papers, figures, and downstream analysis tools. It is especially useful when a user already knows a KEGG pathway identifier or wants pathway genes without committing to the deeper Reactome event model.

In BioMCP, pathway search and detail are multi-source across Reactome, KEGG, and WikiPathways. KEGG base cards stay summary-only unless you explicitly request `genes`, `events` and pathway `enrichment` are Reactome-only, and BioMCP keeps KEGG traffic within the provider's published guidance of 3 requests / second.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `search pathway` | Pathway search rows that can resolve to KEGG IDs | Search is multi-source; this page focuses on KEGG-backed rows |
| `get pathway <id>` | KEGG summary card for a pathway ID | KEGG base card stays summary-oriented |
| `get pathway <id> genes` | Member genes for a KEGG pathway | Explicit follow-up section |
| `get gene <symbol> pathways` | Gene-to-pathway links that can include KEGG rows | Cross-entity helper with mixed-source pathway output |

## Example commands

```bash
biomcp search pathway "MAPK signaling" --limit 5
```

Returns a mixed-source pathway table that can include KEGG rows. Exact title searches such as `MAPK signaling pathway` can include the human KEGG row `hsa04010` when KEGG participates.

```bash
biomcp get pathway hsa05200
```

Returns a KEGG pathway card with summary context for the resolved ID.

```bash
biomcp get pathway hsa05200 genes
```

Returns the gene membership section for that KEGG pathway.

```bash
biomcp get gene BRAF pathways
```

Returns pathway links for BRAF that can include KEGG entries.

## API access

No BioMCP API key required.

## Official source

[KEGG](https://www.kegg.jp/) is the official pathway resource behind BioMCP's KEGG search rows and pathway cards.

## Related docs

- [Pathway](../user-guide/pathway.md)
- [Gene](../user-guide/gene.md)
- [Data Sources](../reference/data-sources.md)
