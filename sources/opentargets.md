---
title: "OpenTargets MCP Tool for Target and Disease Context | BioMCP"
description: "Use BioMCP to surface OpenTargets disease associations, drug-target context, and druggability signals in BioMCP gene, disease, and drug workflows."
---

# OpenTargets

OpenTargets matters when you need ranked target-disease context instead of a flat list of associations. It is especially useful for deciding whether a gene looks disease-relevant, whether a target appears tractable, and how much supporting signal exists behind a drug or disease pivot.

In BioMCP, OpenTargets powers the gene `druggability` section and contributes ranked evidence to `get gene <symbol> diseases` and `get disease <id> genes`. Those surfaces remain mixed-source, but OpenTargets scores are the shared signal behind disease ranking, prevalence context, and parts of the drug target and indication workflow.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `get gene <symbol> diseases` | Ranked disease associations for a gene | OpenTargets scores anchor the disease ranking |
| `get gene <symbol> druggability` | Tractability, safety, and targetability context | Combined with DGIdb interactions in one gene section |
| `get drug <name> targets` | Generic drug-target context for known therapies | Mixed with ChEMBL target evidence; CIViC may add separate variant-target annotations |
| `get drug <name> indications` | Disease and indication context for drugs | Mixed with ChEMBL indication enrichment |
| `get disease <id> genes` | Ranked associated genes for a disease | OpenTargets can add OT-only disease-gene rows and also attach scores to Monarch/CIViC rows |
| `get disease <id> prevalence` | Prevalence-like evidence and disease burden context | OpenTargets-backed disease section |

## Example commands

```bash
biomcp get gene BRAF diseases
```

Returns disease associations for BRAF with OpenTargets-backed ranking context.

```bash
biomcp get gene BRAF druggability
```

Returns a druggability section with tractability and safety signals.

```bash
biomcp get disease MONDO:0005105 genes
```

Returns disease-associated genes where OpenTargets can both add disease-gene rows and attach score summaries.

```bash
biomcp get disease MONDO:0005105 prevalence
```

Returns prevalence-like evidence for the normalized disease record.

```bash
biomcp get drug pembrolizumab targets
```

Returns a drug targets section where OpenTargets joins ChEMBL generic target context, with CIViC optionally adding a separate variant-target line.

## API access

No BioMCP API key required.

## Official source

[OpenTargets](https://platform.opentargets.org/) is the official target-disease platform behind BioMCP's OpenTargets-backed ranking and tractability signals.

## Related docs

- [Gene](../user-guide/gene.md)
- [Disease](../user-guide/disease.md)
- [Drug](../user-guide/drug.md)
- [Data Sources](../reference/data-sources.md)
