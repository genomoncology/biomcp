---
title: "ChEMBL MCP Tool for Drug Target Enrichment | BioMCP"
description: "Use BioMCP to pull ChEMBL drug-target activity and indication context for drug lookups without working directly with the ChEMBL API."
---

# ChEMBL

ChEMBL matters when you want drug-target evidence that is still close to assay and mechanism data instead of a high-level summary alone. It is one of the fastest ways to explain why a drug is connected to a target and which indication context is public enough to surface in a lightweight lookup.

In BioMCP, ChEMBL mainly appears inside the drug `targets` section and the drug `indications` section. Those sections are mixed with OpenTargets, but ChEMBL is the part that contributes activity and mechanism context rather than a standalone ChEMBL search workflow.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `get drug <name> targets` | Generic drug-target activity, mechanism, and target context | Mixed-source section: ChEMBL generic targets/mechanisms plus OpenTargets target context; CIViC may add a separate variant-target annotation line |
| `get drug <name> indications` | Drug indication context linked to known use areas | ChEMBL contributes indication enrichment alongside OpenTargets |

## Example commands

```bash
biomcp get drug pembrolizumab targets
```

Returns a targets section with ChEMBL-backed activity and mechanism context.

```bash
biomcp get drug pembrolizumab indications
```

Returns an indications section with ChEMBL-linked use-area context.

```bash
biomcp get drug dabrafenib targets
```

Returns a target-focused view for a kinase inhibitor with mechanism-oriented enrichment.

## API access

No BioMCP API key required.

## Official source

[ChEMBL](https://www.ebi.ac.uk/chembl/) is EMBL-EBI's public bioactivity database for drug-like molecules and targets.

## Related docs

- [Drug](../user-guide/drug.md)
- [Data Sources](../reference/data-sources.md)
- [Source Licensing and Terms](../reference/source-licensing.md)
