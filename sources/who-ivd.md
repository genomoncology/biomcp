---
title: "WHO Prequalified IVD MCP Tool for Infectious Disease Diagnostics | BioMCP"
description: "Use BioMCP to search WHO prequalified infectious-disease diagnostics, fetch source-native WHO IVD product cards, and manage the local WHO IVD CSV lifecycle without learning the raw export format."
---

# WHO Prequalified IVD

WHO Prequalified IVD is the right source when you need infectious-disease
diagnostic products rather than gene-centric genetic tests. In BioMCP, it is
the WHO-backed branch of the multi-source `diagnostic` entity:
`search diagnostic --source who-ivd` stays on the WHO CSV, while the default
`--source all` route merges WHO IVD with GTR and keeps row provenance explicit.

WHO IVD product codes can contain spaces, so BioMCP preserves the source-native
code and quotes follow-up `get diagnostic` commands automatically.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `search diagnostic --disease <name> --source who-ivd` | WHO infectious-disease diagnostic search rows | Minimum-length word/phrase boundary match over `Pathogen/Disease/Marker` |
| `search diagnostic --type <assay_format> --source who-ivd` | WHO assay-format filtered search | Exact match over WHO `Assay Format` |
| `search diagnostic --manufacturer <name> --source who-ivd` | WHO manufacturer search | Case-insensitive substring over `Manufacturer name` |
| `get diagnostic "<product_code>"` | WHO source-native diagnostic summary card | Product code is the detail identifier |
| `get diagnostic "<product_code>" conditions` | WHO target/marker section | WHO IVD supports `conditions` but not `genes` or `methods` |
| `get diagnostic "<product_code>" regulatory` | FDA device clearance/approval overlay | OpenFDA 510(k)/PMA overlay matched from the WHO product name; not WHO-native data |
| `biomcp health` | WHO IVD readiness row | Reports the local CSV lifecycle and root path |
| `biomcp who-ivd sync` | Explicit WHO IVD refresh | Force-refreshes `who_ivd.csv` |

## Example commands

```bash
biomcp search diagnostic --disease HIV --source who-ivd --limit 5
```

Returns WHO IVD infectious-disease diagnostics with a `Source` column so merged search pages remain truthful.

```bash
biomcp get diagnostic "<product_code>"
```

Fetches the WHO source-native summary card with assay format, manufacturer, target/marker, regulatory version, and prequalification year.

```bash
biomcp get diagnostic "<product_code>" conditions
```

Expands the WHO-supported `conditions` section only.

```bash
biomcp who-ivd sync
```

Force-refreshes the local WHO IVD CSV export.

```bash
biomcp health
```

Shows the `WHO IVD local data` readiness row alongside the other local-runtime bundles.

## API access

No BioMCP API key required. BioMCP downloads the WHO IVD CSV export on first
use into `BIOMCP_WHO_IVD_DIR` or the default platform data directory, then
refreshes stale data after 72 hours. The optional `regulatory` overlay also
benefits from `OPENFDA_API_KEY` for OpenFDA quota headroom.

## Official source

[WHO Prequalified IVD](https://extranet.who.int/prequal/vitro-diagnostics/prequalified/in-vitro-diagnostics)
is the official WHO landing page for the prequalified IVD program and export.

## Related docs

- [Diagnostic](../user-guide/diagnostic.md)
- [Data Sources](../reference/data-sources.md)
- [Source Licensing](../reference/source-licensing.md)
- [Troubleshooting](../troubleshooting.md)
