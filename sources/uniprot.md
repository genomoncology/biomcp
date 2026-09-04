---
title: "UniProt MCP Tool for Protein Data | BioMCP"
description: "Use BioMCP to search UniProt proteins, fetch canonical protein cards, and surface structure-linked context for AI agents and research workflows."
---

# UniProt

UniProt is the canonical protein reference many life-science workflows quietly depend on, which makes it the right source page when your question is about accession-level identity, curated function, or structure-linked protein context. It matters because a strong protein card can ground the rest of an agent workflow before you fan out into pathway, interaction, or structural detail.

In BioMCP, UniProt backs the main protein card and the gene `protein` section. Domains, interactions, and complexes are separate provider sections elsewhere in BioMCP, while structure IDs are surfaced through UniProt cross-references to PDB and AlphaFold rather than through a standalone structural database client on this page.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `search protein` | Protein search rows with accessions and names | UniProt-backed search surface |
| `get protein <accession_or_symbol>` | Canonical protein card with accession, gene, function, and references | UniProt is the primary provider for the base card |
| `get gene <symbol> protein` | Gene-linked protein summary | Surfaces the UniProt-backed protein section inside the gene workflow |
| `get protein <accession> structures` | PDB and AlphaFold identifiers linked from UniProt | Structure IDs arrive via UniProt cross-references |

## Example commands

```bash
biomcp search protein BRAF --limit 3
```

Returns a protein search table with accession, name, gene, and species columns.

```bash
biomcp get protein P15056
```

Returns a protein card with accession, gene, function, and UniProt evidence links.

```bash
biomcp get gene BRAF protein
```

Returns a gene detail view with a `Protein (UniProt)` section, including legacy names and isoforms when UniProt annotates them.

When UniProt annotates legacy protein names, the section includes an `Also known as:` line
with alternative full names and short names. When UniProt annotates alternative products,
the section also includes an `Isoforms (N)` line with isoform names and the canonical record length.

```bash
biomcp get protein P15056 structures
```

Returns a structures section with PDB and AlphaFold IDs surfaced through UniProt.

## API access

No BioMCP API key required.

## Official source

[UniProt](https://www.uniprot.org/) is the official protein knowledgebase behind BioMCP's core protein card.

## Related docs

- [Protein](../user-guide/protein.md)
- [Gene](../user-guide/gene.md)
- [Data Sources](../reference/data-sources.md)
