---
title: "MedlinePlus MCP Tool for Plain-Language Disease Context | BioMCP"
description: "Use BioMCP to add MedlinePlus plain-language context to discover results and opt-in disease clinical-feature summaries."
---

# MedlinePlus

MedlinePlus is the right source when you need plain-language disease or symptom context alongside structured biomedical identifiers. In BioMCP, MedlinePlus supplements `biomcp discover` for disease and symptom-oriented prompts; it is suppressed for gene, drug, pathway, and other flows where consumer-health prose would add noise.

MedlinePlus also backs the opt-in `get disease <name_or_id> clinical_features` section. That section surfaces reviewed clinical-summary rows for configured diseases and can fall back to embedded reviewed fixtures when live MedlinePlus search is unavailable.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `biomcp discover <query>` | Plain-language context for disease and symptom queries | Supplemental only; OLS4 remains the required structured-concept backbone |
| `get disease <name_or_id> clinical_features` | MedlinePlus clinical-summary feature rows | Opt-in disease section that keeps consumer-health context out of default disease cards |

## Example commands

```bash
biomcp discover "symptoms of Marfan syndrome"
```

Returns structured discover follow-ups with supplemental MedlinePlus plain-language context when the query resolves as a disease or symptom flow.

```bash
biomcp get disease "uterine leiomyoma" clinical_features
```

Fetches the opt-in MedlinePlus-backed clinical-features section for a configured disease.

```bash
biomcp get disease MONDO:0007947 clinical_features
```

Uses a resolved disease identifier while keeping the MedlinePlus clinical-summary section explicit.

## API access

No BioMCP API key required. BioMCP uses the public MedlinePlus Search endpoint
for supplemental discover context and disease clinical-feature summaries, with
embedded reviewed fixtures as the offline fallback for configured diseases.

## Official source

[MedlinePlus](https://medlineplus.gov/) is the official NLM consumer-health
site. BioMCP uses the public [MedlinePlus Search](https://wsearch.nlm.nih.gov/ws/query)
endpoint for the surfaces described here.

## Related docs

- [Discover](../user-guide/discover.md)
- [Disease](../user-guide/disease.md)
- [Data Sources](../reference/data-sources.md)
- [Source Licensing](../reference/source-licensing.md)
