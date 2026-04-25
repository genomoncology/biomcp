---
title: "DDInter MCP Tool for Drug-Drug Interactions | BioMCP"
description: "Use BioMCP to query DDInter-backed drug-drug interactions, severity levels, and class summaries through the local DDInter CSV bundle."
---

# DDInter

DDInter matters when you need a structured answer to a drug-drug interaction
question instead of free-text safety prose or ad hoc literature synthesis. It
is the right page for questions about interacting partner drugs, source-provided
severity levels, class rollups, and the local-runtime DDInter bundle that
BioMCP keeps on disk for repeatable DDI lookups.

In BioMCP, DDInter is a local-runtime source for interaction review rather than
a live per-request API surface. BioMCP auto-downloads the eight public DDInter
CSV files into `BIOMCP_DDINTER_DIR` or the default data directory on first use,
supports `biomcp drug interactions <name>` plus `get drug <name> interactions`,
shows `DDInter local data (<root>)` in full health output, and exposes
`biomcp ddinter sync` when you want a forced refresh of the bundle.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `biomcp drug interactions <name>` | Structured partner rows, DDInter severity levels, and class summaries for one anchor drug | Uses the local DDInter bundle as the canonical DDI source and resolves the anchor drug first |
| `get drug <name> interactions` | The same DDInter-backed interaction report inside the standard drug card | Keeps helper and section parity instead of splitting the contract |
| `biomcp health` | Local readiness for the DDInter bundle | Reports `DDInter local data (<root>)` in the non-API health view |
| `biomcp ddinter sync` | Forced refresh of the local DDInter CSV bundle | Refreshes all eight public DDInter CSV files without waiting for the next automatic sync |

## Example commands

```bash
biomcp drug interactions warfarin
```

Returns the structured DDInter-backed interaction report for warfarin, including class rollups and partner rows.

```bash
biomcp drug interactions imatinib
```

Returns the same interaction-focused report for an oncology anchor drug.

```bash
biomcp get drug warfarin interactions
```

Renders the same DDInter-backed report inside the standard `get drug` card.

```bash
biomcp ddinter sync
```

Refreshes the local DDInter bundle without waiting for the next automatic sync.

## API access

No BioMCP API key required. BioMCP auto-downloads the eight public DDInter CSV
files into `BIOMCP_DDINTER_DIR` or the default data directory on first use.
DDInter's own terms warn that absence from the database does not prove no
interaction exists, so BioMCP keeps empty results scoped to the current local
bundle instead of treating them as safety claims.

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
