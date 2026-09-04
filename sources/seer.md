---
title: "SEER Explorer MCP Tool for Cancer Survival | BioMCP"
description: "Use BioMCP to surface SEER Explorer 5-year relative survival summaries for mapped cancers in the BioMCP disease survival section."
---

# SEER Explorer

SEER Explorer matters when a disease question is really a cancer-outcome question and you need a population-level survival anchor instead of more genes, pathways, or phenotype context.

In BioMCP, SEER Explorer is visible through the disease `survival` section. That section ships a sex-split, all-ages and all-races 5-year relative survival view for mapped cancer sites, and because the integration relies on undocumented SEER Explorer UI endpoints, BioMCP falls back to a stable note when a disease does not map cleanly or the upstream is unavailable.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `get disease <id> survival` | Latest observed 5-year relative survival by sex plus recent yearly history | Opt-in disease section backed by the live SEER site catalog and filtered to all ages / all races |
| `get disease <id> all` | Adds disease survival when the normalized disease maps to one SEER cancer site | Included in `all`; unmapped or unavailable cases render a stable note instead of an error |

## Example commands

```bash
biomcp get disease "chronic myeloid leukemia" survival
```

Returns the SEER-backed survival section for the mapped CML site with latest observed survival by sex.

```bash
biomcp get disease "Hodgkin lymphoma" survival
```

Returns disease survival data for another mapped cancer label and confirms the site resolver is not CML-specific.

```bash
biomcp get disease "Marfan syndrome" survival
```

Returns the stable no-data note when the disease does not map to one SEER cancer site.

```bash
biomcp get disease "chronic myeloid leukemia" all
```

Returns the full disease card with the survival section included alongside the other non-key-gated disease sections.

## API access

No BioMCP API key required.

## Official source

[SEER Explorer](https://seer.cancer.gov/statistics-network/explorer/) is the National Cancer Institute's public cancer-statistics explorer behind BioMCP's disease survival section.

## Related docs

- [Disease](../user-guide/disease.md)
- [CLI Reference](../user-guide/cli-reference.md)
- [Data Sources](../reference/data-sources.md)
