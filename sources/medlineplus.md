---
title: "MedlinePlus MCP Tool for Plain-Language Disease Context | BioMCP"
description: "Use BioMCP to add MedlinePlus plain-language context to discover results."
---

# MedlinePlus

MedlinePlus is the right source when you need plain-language disease or symptom context alongside structured biomedical identifiers. In BioMCP, MedlinePlus supplements `biomcp discover` for disease and symptom-oriented prompts; it is suppressed for gene, drug, pathway, and other flows where consumer-health prose would add noise.

Disease `clinical_features` now uses Monarch/HPO phenotype annotations instead of MedlinePlus clinical summaries, so MedlinePlus remains a discover-only supplemental source.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `biomcp discover <query>` | Plain-language context for disease and symptom queries | Supplemental only; OLS4 remains the required structured-concept backbone |

## Example commands

```bash
biomcp discover "symptoms of Marfan syndrome"
```

Returns structured discover follow-ups with supplemental MedlinePlus plain-language context when the query resolves as a disease or symptom flow.

```bash
biomcp discover "chest pain"
```

Adds consumer-health context only when the resolved concept is disease- or symptom-oriented.

```bash
biomcp discover "Marfan syndrome"
```

Keeps MedlinePlus supplemental to structured concept resolution.

## API access

No BioMCP API key required. BioMCP uses the public MedlinePlus Search endpoint
for supplemental discover context.

## Official source

[MedlinePlus](https://medlineplus.gov/) is the official NLM consumer-health
site. BioMCP uses the public [MedlinePlus Search](https://wsearch.nlm.nih.gov/ws/query)
endpoint for the surfaces described here.

## Related docs

- [Discover](../user-guide/discover.md)
- [Disease](../user-guide/disease.md)
- [Data Sources](../reference/data-sources.md)
- [Source Licensing](../reference/source-licensing.md)
