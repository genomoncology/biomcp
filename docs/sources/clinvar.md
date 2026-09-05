---
title: "ClinVar MCP Tool for Variant Interpretation | BioMCP"
description: "Use BioMCP to pull ClinVar clinical significance, review status, and disease context for human variants through one variant lookup workflow."
---

# ClinVar

ClinVar is the most recognizable public archive for germline and somatic clinical significance claims, so it is often the first source people want when they ask whether a variant is pathogenic, uncertain, or well reviewed. It matters because the labels are familiar to labs, researchers, and reviewers even when the upstream submission evidence is messy.

BioMCP retrieves the current NCBI Variation Archive record directly for an explicit `clinvar` section (and for `all`) after MyVariant.info resolves a numeric ClinVar Variation ID. The default variant card remains a fast indirect summary. If direct retrieval fails, a usable MyVariant.info snapshot is returned as a clearly degraded fallback with its available evaluation date and submitter count.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `get variant <id>` | Base variant card with ClinVar-backed significance signals when present | Indirect summary through MyVariant.info; no direct ClinVar request |
| `get variant <id> clinvar` | Current condition-specific RCV aggregates and separate current SCV submissions | Direct NCBI ClinVar, with MyVariant.info fallback |
| `search variant -g <gene> --significance <value>` | Variant search filtered by ClinVar significance labels | Search rows can surface ClinVar-derived review and significance hints |

## Example commands

```bash
biomcp get variant rs113488022
```

Returns a base variant card that can include ClinVar-backed summary fields when they are available.

```bash
biomcp get variant rs113488022 clinvar
```

Returns the current direct ClinVar record with condition-specific aggregates and each current submission kept separate. Domains such as germline, somatic clinical impact, and oncogenicity are not combined into a consensus.

```bash
biomcp get variant "BRAF V600E" clinvar
```

Returns the same ClinVar section for a gene-plus-protein variant ID.

```bash
biomcp search variant -g BRCA1 --significance pathogenic --limit 5
```

Returns variant rows filtered by ClinVar significance labels.

## API access

NCBI E-utilities EFetch (`db=clinvar`, `rettype=vcv`, Variation ID lookup) powers explicit `clinvar` and `all` requests. The [Variant](../user-guide/variant.md) guide covers the broader workflow that hosts this section.

No credential is required. An optional `NCBI_API_KEY` is sent to NCBI E-utilities when configured and enables the provider's higher request budget.

## Official source

[ClinVar](https://www.ncbi.nlm.nih.gov/clinvar/) is the official NCBI archive for clinical variant interpretations.

## Related docs

- [Variant](../user-guide/variant.md)
- [How to annotate variants](../how-to/annotate-variants.md)
- [Source Licensing and Terms](../reference/source-licensing.md)
