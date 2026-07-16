---
title: "Reactome MCP Tool for Pathway Analysis | BioMCP"
description: "Use BioMCP to search Reactome pathways, inspect pathway genes and events, and connect pathway context to downstream trial and article workflows."
---

# Reactome

Reactome is the best source page to reach for when a question has moved past single genes and into mechanism, signaling flow, or pathway membership. It matters because pathway context is where isolated biomarkers start to look like a biological story instead of a disconnected list of hits.

BioMCP pathway search and detail are multi-source across Reactome, KEGG, and WikiPathways. This page covers the Reactome-backed part of that surface: Reactome IDs, Reactome detail cards, contained events, and Reactome-gated enrichment sections. Top-level `biomcp enrich` is a g:Profiler workflow and does not belong on this page. JSON/MCP detail output records requested `genes`, `events`, and `enrichment` under `section_outcomes`; `empty` is a successful zero, while `unavailable` means the requested evidence could not be retrieved and is reported in Markdown too.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `search pathway` | Pathway search rows that can resolve to Reactome IDs | Pathway search is multi-source; this page focuses on Reactome-backed rows |
| `get pathway <id>` | Reactome pathway summary card | Reactome ID path |
| `get pathway <id> genes` | Member genes for a Reactome pathway | Reactome detail section |
| `get pathway <id> events` | Contained-events view | Reactome-only section |
| `get pathway <id> enrichment` | Reactome-gated enrichment summary | This is a Reactome pathway workflow, not generic top-level enrichment |
| `get gene <symbol> pathways` | Gene-to-pathway links that can include Reactome entries | Cross-entity helper that surfaces Reactome pathway links |

## Example commands

```bash
biomcp search pathway "MAPK signaling" --limit 5
```

Returns a pathway table with source, ID, and name columns.

```bash
biomcp get pathway R-HSA-5673001
```

Returns a Reactome pathway card with source and summary context.

```bash
biomcp get pathway R-HSA-5673001 genes
```

Returns the genes section for that Reactome pathway.

```bash
biomcp get pathway R-HSA-5673001 events
```

Returns a contained-events section for the Reactome record.

```bash
biomcp get gene BRAF pathways
```

Returns a gene pathways section that can include Reactome IDs and links.

## API access

No BioMCP API key required.

## Official source

[Reactome](https://reactome.org/) is the official curated pathway knowledgebase behind BioMCP's Reactome-specific pathway views.

## Related docs

- [Pathway](../user-guide/pathway.md)
- [Gene](../user-guide/gene.md)
- [Data Sources](../reference/data-sources.md)
