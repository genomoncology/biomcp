---
title: "OncoKB MCP Tool for Oncology Variant Annotation | BioMCP"
description: "Use BioMCP to run the explicit OncoKB variant helper for oncogenicity, evidence levels, and treatment implications on actionable variants."
---

# OncoKB

OncoKB matters when you need a focused oncology interpretation instead of general variant context. It is the source page to use when a workflow has already narrowed to an actionable somatic variant and you want evidence levels, oncogenicity framing, and therapy implications from a well-known cancer interpretation source.

In BioMCP, OncoKB is an explicit helper rather than an automatic section on every variant lookup. Use the `variant oncokb` command when you want the registration-gated production source. That helper requires `ONCOKB_TOKEN`, and BioMCP keeps the upstream `TOKEN` naming because OncoKB uses the same term in its own registration flow.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `variant oncokb <id>` | Oncogenicity, treatment implications, and evidence levels | Explicit helper, not a default section on `get variant` |

## Example commands

```bash
biomcp variant oncokb "BRAF V600E"
```

Returns an OncoKB interpretation for a common actionable melanoma variant.

```bash
biomcp variant oncokb "EGFR L858R"
```

Returns an OncoKB interpretation focused on evidence levels and treatment context.

```bash
biomcp variant oncokb "KRAS G12C"
```

Returns an OncoKB interpretation for a targeted oncology biomarker.

## API access

Requires `ONCOKB_TOKEN` for the production OncoKB API. Configure it with the [API Keys](../getting-started/api-keys.md) guide and register at [OncoKB](https://www.oncokb.org/account/register).

## Official source

[OncoKB](https://www.oncokb.org/) is the official precision-oncology knowledgebase behind BioMCP's explicit OncoKB helper.

## Related docs

- [Variant](../user-guide/variant.md)
- [How to annotate variants](../how-to/annotate-variants.md)
- [API Keys](../getting-started/api-keys.md)
