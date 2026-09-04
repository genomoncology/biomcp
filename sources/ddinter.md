---
title: "DDInter MCP Tool for Drug-Drug Interactions | BioMCP"
description: "Use BioMCP to query bounded DDInter-backed drug-drug interactions and severity levels through an installed local CSV bundle."
---

# DDInter

DDInter matters when you need a structured answer to a drug-drug interaction
question instead of free-text safety prose or ad hoc literature synthesis. It
is the right page for questions about interacting partner drugs, source-provided
severity levels, bounded pages, and the local-runtime DDInter bundle that
BioMCP keeps on disk for repeatable DDI lookups.

In BioMCP, DDInter is a local-runtime source for interaction review rather than
a live per-request API surface. Reads use the installed eight-file bundle in
`BIOMCP_DDINTER_DIR` or the default data directory without downloading or
refreshing it. `biomcp ddinter sync` is the explicit maintenance path. BioMCP
supports `biomcp drug interactions <name>` plus `get drug <name> interactions`
and shows `DDInter local data (<root>)` in full health output.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `biomcp drug interactions <name> [--limit N] [--offset N]` | Bounded partner rows, DDInter severity levels, totals, paging, and bundle freshness | Sorts and deduplicates the full local match set before returning up to 50 rows |
| `get drug <name> interactions` | The first 25 DDInter-backed rows inside the standard drug card | Reports totals and points to the helper when more rows exist |
| `biomcp health` | Local readiness for the DDInter bundle | Reports `DDInter local data (<root>)` in the non-API health view |
| `biomcp ddinter sync` | Explicit refresh of the local DDInter CSV bundle | Downloads and validates all eight files before replacing the prior usable bundle |


## Example commands

```bash
biomcp drug interactions warfarin
```

Returns the first 25 DDInter-backed partner rows for warfarin with total, page, and fresh/stale metadata.

```bash
biomcp drug interactions imatinib --limit 25 --offset 25
```

Returns the same interaction-focused report for an oncology anchor drug.

```bash
biomcp get drug warfarin interactions
```

Renders the same DDInter-backed report inside the standard `get drug` card.

```bash
biomcp ddinter sync
```

Downloads and validates a complete replacement bundle. A failed sync leaves the prior complete bundle usable.

## API access

No BioMCP API key required. Install the bundle with `biomcp ddinter sync`
or preseed all eight files in `BIOMCP_DDINTER_DIR` or the default data directory.
Normal reads never perform maintenance, even when the bundle is stale, and
report `bundle_freshness.status` as `fresh` or `stale`. DDInter's own terms warn
that absence from the database does not prove no
interaction exists, so BioMCP keeps empty results scoped to the current local
bundle instead of treating them as safety claims. When the queried drug is not
present in the loaded DDInter bundle at all, JSON includes
`coverage_status: "not_in_ddinter_coverage"` and markdown says this is a source
coverage miss.

## Official source

The official DDInter surfaces behind BioMCP's DDI workflow are:

- [DDInter download bundle](https://ddinter.scbdd.com/download/)
- [DDInter explanation page](https://ddinter.scbdd.com/explanation/)
- [DDInter terms and conditions](https://ddinter.scbdd.com/terms/)

## Related docs

- [Drug](../user-guide/drug.md)
- [Data Sources](../reference/data-sources.md)
- [Source Licensing and Terms](../reference/source-licensing.md)
- [Troubleshooting](../troubleshooting.md)
