---
title: "EMA MCP Tool for EU Drug Regulatory Data | BioMCP"
description: "Use BioMCP to search EMA-backed EU drug records in BioMCP and retrieve regulatory, safety, and shortage context through the local EMA batch."
---

# EMA

EMA matters when you need European regulatory context that does not appear in U.S.-only drug sources. It is the right page for questions about EU approvals, regional safety wording, or shortage status when those answers depend on the European Medicines Agency's published human-medicines data.

In BioMCP, EMA is a local-runtime source for drug name/alias lookups and region-aware sections rather than a live per-request API surface. BioMCP auto-downloads the six EMA human-medicines JSON feeds into `BIOMCP_EMA_DIR` or the default data directory on first use, supports the canonical `--region eu|all` grammar, accepts `--region ema` as an input alias for `eu`, and exposes `biomcp ema sync` when you want a forced refresh.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `search drug <name> --region eu` | EU drug matches by name or alias | Uses the local EMA batch for region-aware name/alias lookups; `ema` is accepted as an input alias for `eu` |
| `search drug <name> --region all` | Combined U.S., EU, and WHO name/alias search | Merges EMA local results with U.S. and WHO data |
| `get drug <name> regulatory --region eu|all` | EU or combined regulatory context | EMA-backed regional section; omitting `--region` on this direct regulatory path checks U.S. and EU data |
| `get drug <name> safety --region eu|all` | EU or combined safety context | EMA-backed regional section |
| `get drug <name> shortage --region eu|all` | EU or combined shortage context | EMA-backed regional section |

## Example commands

```bash
biomcp search drug Keytruda --region eu --limit 3
```

Returns EU-focused drug matches from the local EMA dataset.

```bash
biomcp get drug Keytruda regulatory --region eu
```

Returns EMA-backed regulatory context for the EU region.

```bash
biomcp get drug Dupixent regulatory --region ema
```

Returns the same EMA-backed regulatory context while using the accepted `ema` input alias for the canonical `eu` region.

```bash
biomcp get drug Ozempic safety --region eu
```

Returns EMA-backed safety context for the EU region.

```bash
biomcp ema sync
```

Refreshes the local EMA batch without waiting for the next automatic sync.

## API access

No BioMCP API key required. BioMCP auto-downloads the EMA human-medicines JSON batch into `BIOMCP_EMA_DIR` or the default data directory on first use.

## Official source

[EMA](https://www.ema.europa.eu/en/about-us/about-website/download-website-data-json-data-format) is the official European Medicines Agency download surface behind BioMCP's EU drug context.

## Related docs

- [Drug](../user-guide/drug.md)
- [Data Sources](../reference/data-sources.md)
- [Troubleshooting](../troubleshooting.md)
