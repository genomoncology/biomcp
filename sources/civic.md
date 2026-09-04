---
title: "CIViC MCP Tool for Clinical Variant Evidence | BioMCP"
description: "Use BioMCP to surface CIViC evidence, disease-associated variants, and therapy context across BioMCP variant, gene, drug, and disease workflows."
---

# CIViC

CIViC matters when you want clinically oriented cancer-variant evidence instead of a generic annotation alone. It is a good fit for workflows that need therapy relevance, disease context, or evidence assertions tied to specific molecular profiles and clinical interpretations.

In BioMCP, CIViC is exposed as section-gated enrichment rather than a full alternate entity system. The main entry points are `get variant <id> civic`, `get gene <symbol> civic`, `get drug <name> civic`, and disease workflows such as `get disease <id> variants` when you want CIViC-backed molecular profile context. Drug `targets` output can also add a separate CIViC-backed variant-target annotation line while leaving the full CIViC evidence table opt-in.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `get variant <id> civic` | Variant-level CIViC evidence and assertions | Explicit opt-in section |
| `get gene <symbol> civic` | Gene-level CIViC evidence summary | Explicit opt-in section |
| `get drug <name> targets` | Variant-specific therapy target annotations that map to displayed generic targets | Additive CIViC line; not merged into generic target/mechanism fields |
| `get drug <name> civic` | Therapy-context evidence for a drug | CIViC therapy evidence section |
| `get disease <id> civic` | Disease-context evidence and assertions | CIViC disease evidence section |
| `get disease <id> variants` | Disease-associated molecular profiles and variants | CIViC augments the disease variants workflow |

## Example commands

```bash
biomcp get variant "BRAF V600E" civic
```

Returns a CIViC section with variant evidence and assertion summaries.

```bash
biomcp get gene BRAF civic
```

Returns a gene-level CIViC evidence summary for BRAF.

```bash
biomcp get drug vemurafenib civic
```

Returns therapy-context CIViC evidence for the drug workflow.

```bash
biomcp get disease MONDO:0005105 civic
```

Returns disease-context CIViC evidence and assertions for the disease record.

```bash
biomcp get disease MONDO:0005105 variants
```

Returns disease-associated variants with CIViC-backed molecular profile context.

## API access

No BioMCP API key required.

## Official source

[CIViC](https://civicdb.org/home) is the public clinical interpretation knowledgebase behind BioMCP's CIViC evidence sections.

## Related docs

- [Variant](../user-guide/variant.md)
- [Gene](../user-guide/gene.md)
- [Disease](../user-guide/disease.md)
- [CIViC Sections](../reference/civic-sections.md)
