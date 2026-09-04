---
title: "ClinVar MCP Tool for Variant Interpretation | BioMCP"
description: "Use BioMCP to pull ClinVar clinical significance, review status, and disease context for human variants through one variant lookup workflow."
---

# ClinVar

ClinVar is the most recognizable public archive for germline and somatic clinical significance claims, so it is often the first source people want when they ask whether a variant is pathogenic, uncertain, or well reviewed. It matters because the labels are familiar to labs, researchers, and reviewers even when the upstream submission evidence is messy.

In BioMCP, ClinVar is an indirect-only source. BioMCP does not act as a direct ClinVar API client; instead, ClinVar assertions are surfaced through MyVariant.info payloads and then normalized into the variant workflow. That keeps the lookup fast, but it means this page is about ClinVar-backed provenance inside BioMCP rather than a standalone ClinVar transport.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `get variant <id>` | Base variant card with ClinVar-backed significance signals when present | ClinVar arrives indirectly through MyVariant.info |
| `get variant <id> clinvar` | Focused ClinVar section with significance, review status, and disease context | Indirect-only provider surface |
| `search variant -g <gene> --significance <value>` | Variant search filtered by ClinVar significance labels | Search rows can surface ClinVar-derived review and significance hints |

## Example commands

```bash
biomcp get variant rs113488022
```

Returns a base variant card that can include ClinVar-backed summary fields when they are available.

```bash
biomcp get variant rs113488022 clinvar
```

Returns a ClinVar section with significance, review status, and disease context.

```bash
biomcp get variant "BRAF V600E" clinvar
```

Returns the same ClinVar section for a gene-plus-protein variant ID.

```bash
biomcp search variant -g BRCA1 --significance pathogenic --limit 5
```

Returns variant rows filtered by ClinVar significance labels.

## API access

No standalone BioMCP key path; ClinVar content is surfaced indirectly via MyVariant.info. The [Variant](../user-guide/variant.md) guide covers the broader workflow that hosts this section.

## Official source

[ClinVar](https://www.ncbi.nlm.nih.gov/clinvar/) is the official NCBI archive for clinical variant interpretations.

## Related docs

- [Variant](../user-guide/variant.md)
- [How to annotate variants](../how-to/annotate-variants.md)
- [Source Licensing and Terms](../reference/source-licensing.md)
