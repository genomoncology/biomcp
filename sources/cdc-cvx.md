---
title: "CDC CVX/MVX MCP Tool for Vaccine Identity Bridge | BioMCP"
description: "Use BioMCP to bridge vaccine brand names into EMA-backed drug matches and explicit WHO vaccine searches with the local CDC CVX, trade-name, and MVX bundle."
---

# CDC CVX/MVX

CDC CVX/MVX matters when a vaccine brand name such as Gardasil, Prevnar, Fluzone, or Comirnaty does not normalize cleanly through U.S.-first drug identity sources but you still need EU vaccine context or an explicit WHO vaccine search. It is the right page for questions about vaccine-brand matching, local vaccine identity setup, and how BioMCP expands brand names into vaccine-antigen terms before retrying the EMA path or explicit WHO vaccine search.

In BioMCP, CDC CVX/MVX is a local-runtime vaccine identity source rather than a live per-request API surface. BioMCP auto-downloads `cvx.txt`, `TRADENAME.txt`, and `mvx.txt` into `BIOMCP_CVX_DIR` or the default data directory on first use, augments the EMA/default plain-name vaccine search path when MyChem identity resolution fails, supports omitted `--region` plain-name vaccine lookups plus explicit `--region eu|all`, also augments explicit WHO vaccine name/brand searches when you pass `--region who --product-type vaccine`, leaves `--region us` untouched, and exposes `biomcp cvx sync` when you want a forced refresh.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `search drug <vaccine_brand> --region eu` | EU vaccine matches by bridged brand name | Uses the local CDC bundle to expand vaccine brand names into EMA-searchable aliases after MyChem identity misses |
| `search drug <vaccine_brand> --region all` | Combined U.S., EU, and WHO search with EMA vaccine bridge | Adds CDC-derived EMA alias expansion while keeping the split all-region output |
| `search drug <vaccine_brand> --region who --product-type vaccine` | Explicit WHO vaccine search by vaccine name or brand | Uses the local CDC bundle to expand vaccine brand names into WHO-searchable aliases after MyChem identity misses |
| `search drug <vaccine_brand>` | Default plain-name U.S.+EU+WHO search | The CDC bridge affects the EU bucket on this path; WHO aliases stay unchanged unless the user explicitly selects WHO vaccine search |
| `search adverse-event <vaccine_query> --source vaers\|all` | Vaccine identity classification before VAERS aggregation | A healthy no-match may classify the query; missing, unreadable, or unsynchronized CVX data reports `vaers.status: unavailable` instead of `query_not_vaccine` |
| `biomcp health` | Local readiness for the CDC CVX/MVX bundle | Reports `CDC CVX/MVX local data (<root>)` in the non-API health view |
| `biomcp cvx sync` | Forced refresh of the local CDC vaccine identity bundle | Refreshes `cvx.txt`, `TRADENAME.txt`, and `mvx.txt` without waiting for automatic sync |

## Example commands

```bash
biomcp search drug gardasil --region eu --limit 5
```

Bridges the Gardasil brand name through CDC CVX/MVX into EMA vaccine matches.

```bash
biomcp search drug prevnar --region eu --limit 5
```

Uses the CDC bridge to recover EMA pneumococcal vaccine matches even though `Prevnar` is not the EMA product name.

```bash
biomcp search drug fluzone --region eu --limit 5
```

Uses CDC influenza vaccine terms to recover EMA influenza-vaccine rows.

```bash
biomcp search drug gardasil --region who --product-type vaccine --limit 5
```

Uses the CDC bridge to recover WHO HPV vaccine rows from the Gardasil brand name. This WHO vaccine path is search-only in this ticket.

```bash
biomcp health
```

Shows whether the local CDC CVX/MVX bundle is configured, stale, missing files, or available at the default path.

## API access

No BioMCP API key required. BioMCP auto-downloads `cvx.txt`, `TRADENAME.txt`, and `mvx.txt` into `BIOMCP_CVX_DIR` or the default data directory on first use.

## Official source

The official CDC IIS downloads behind BioMCP's vaccine identity bridge are:

- [CVX codes](https://www2.cdc.gov/vaccines/iis/iisstandards/downloads/cvx.txt)
- [Trade names](https://www2.cdc.gov/vaccines/iis/iisstandards/downloads/TRADENAME.txt)
- [MVX manufacturers](https://www2.cdc.gov/vaccines/iis/iisstandards/downloads/mvx.txt)

## Related docs

- [Drug](../user-guide/drug.md)
- [Data Sources](../reference/data-sources.md)
- [Troubleshooting](../troubleshooting.md)
