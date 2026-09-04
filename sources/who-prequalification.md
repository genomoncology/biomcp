---
title: "WHO Prequalification MCP Tool for Global Drug Access | BioMCP"
description: "Use BioMCP to search WHO Prequalification-backed drug and vaccine records in BioMCP and retrieve global regulatory context through the local WHO finished-pharma, API, and vaccine exports."
---

# WHO Prequalification

WHO Prequalification matters when you need global regulatory context for medicines and WHO-prequalified vaccines used across procurement and access programs outside the U.S./EU-only lens. It is the right page for questions about whether a medicine appears in the WHO finished-pharmaceutical-products list or the WHO active-pharmaceutical-ingredients list, whether a vaccine appears in the WHO vaccine prequalification export, and how that status compares with other regional drug views.

In BioMCP, WHO Prequalification is a local-runtime source for drug name/alias lookups and regional regulatory sections rather than a live per-request API surface. BioMCP auto-downloads the finished-pharmaceutical-products CSV, the active-pharmaceutical-ingredients CSV, and the vaccine export into `BIOMCP_WHO_DIR` or the default data directory on first use, supports `--region who|all`, lets structured `--region who` searches filter structured U.S. hits through WHO prequalification for finished-pharma/API rows, supports `--product-type <finished_pharma|api|vaccine>` on explicit WHO searches, keeps vaccine support search-only in this ticket, and exposes `biomcp who sync` when you want a forced refresh of all three exports.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `search drug <name> --region who` | WHO-prequalified drug matches by name or alias | Uses the local WHO finished-pharma + API batch for WHO-only name/alias lookups |
| `search drug <name> --region all` | Combined U.S., EU, and WHO name/alias search | Adds WHO rows to the split all-region output |
| `search drug --indication <disease> --region who` | WHO-prequalified rows filtered from structured U.S. search hits | Keeps MyChem structured semantics and narrows results through the WHO batch |
| `search drug <name> --region who --product-type <finished_pharma|api|vaccine>` | WHO-only source-type filtering | Narrows WHO search rows to finished pharmaceutical products, active pharmaceutical ingredients, or vaccine rows |
| `get drug <name> regulatory --region who|all` | WHO or combined regulatory context | WHO-backed regional regulatory section |

## Example commands

```bash
biomcp search drug trastuzumab --region who --limit 5
```

Returns WHO-prequalified trastuzumab rows from the local WHO dataset.

```bash
biomcp search drug artesunate --region who --product-type api --limit 5
```

Returns only WHO active-pharmaceutical-ingredient rows for artesunate. `--product-type` requires explicit `--region who`.

```bash
biomcp search drug BCG --region who --product-type vaccine --limit 5
```

Returns WHO vaccine rows for BCG. WHO vaccine support is explicit search-only in this release, so use a vaccine name or brand with `--product-type vaccine` instead of `get drug`.

```bash
biomcp search drug --indication malaria --region who --limit 5
```

Filters structured U.S. malaria hits through the local WHO prequalification batch.

```bash
biomcp get drug trastuzumab regulatory --region who
```

Returns WHO-backed regulatory context for the WHO region.

## API access

No BioMCP API key required. BioMCP auto-downloads the WHO finished-pharmaceutical-products CSV, active-pharmaceutical-ingredients CSV, and vaccine CSV into `BIOMCP_WHO_DIR` or the default data directory on first use.

## Official source

The official WHO CSV exports behind BioMCP's WHO drug context are:

- [Finished pharmaceutical products](https://extranet.who.int/prequal/medicines/prequalified/finished-pharmaceutical-products/export?page&_format=csv)
- [Active pharmaceutical ingredients](https://extranet.who.int/prequal/medicines/prequalified/active-pharmaceutical-ingredients/export?page&_format=csv)
- [Vaccines](https://extranet.who.int/prequal/vaccines/prequalified/export)

## Related docs

- [Drug](../user-guide/drug.md)
- [Data Sources](../reference/data-sources.md)
- [Troubleshooting](../troubleshooting.md)
