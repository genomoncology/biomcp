---
title: "NCBI Genetic Testing Registry MCP Tool for Diagnostics | BioMCP"
description: "Use BioMCP to search GTR-backed genetic tests, fetch source-native diagnostic cards, and manage the local NCBI Genetic Testing Registry bundle."
---

# NCBI Genetic Testing Registry

NCBI Genetic Testing Registry (GTR) is the right source when you need gene-centric genetic tests, laboratory offerings, testing methods, or condition-linked diagnostics. In BioMCP, GTR is the local-runtime backbone for the multi-source `diagnostic` entity: `search diagnostic --source gtr` stays on the NCBI bundle, while the default `--source all` route can merge GTR rows with WHO IVD rows and keeps source provenance visible.

BioMCP auto-downloads the GTR `test_version.gz` and `test_condition_gene.txt` bulk exports into `BIOMCP_GTR_DIR` or the default platform data directory on first diagnostic use, refreshes stale data after 7 days, reports `GTR local data (<root>)` in full health output, and exposes `biomcp gtr sync` for forced refreshes.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `search diagnostic --gene <symbol> --source gtr` | Gene-linked genetic test rows | Uses the local GTR condition/gene relation bundle for gene-centric lookup |
| `search diagnostic --disease <name> --source gtr` | Condition-linked GTR diagnostic rows | Applies the same bounded disease phrase matching used by the multi-source diagnostic route |
| `search diagnostic --type <method> --source gtr` | GTR testing-method filtered rows | Matches source-native GTR methods |
| `search diagnostic --manufacturer <name> --source gtr` | Laboratory or provider filtered rows | Filters over GTR lab/provider metadata when available |
| `get diagnostic GTR000006692.3` | Source-native GTR diagnostic summary card | GTR accessions are the detail identifiers |
| `get diagnostic GTR000006692.3 genes` | Joined gene symbols for the test | JSON keeps the full deduped symbol arrays even when markdown rows are compact |
| `get diagnostic GTR000006692.3 conditions` | Joined condition names for the test | Preserves GTR condition provenance |
| `get diagnostic GTR000006692.3 methods` | GTR source-native testing methods | GTR supports `genes`, `conditions`, `methods`, and opt-in `regulatory` overlay sections |
| `biomcp health` | GTR local readiness row | Reports whether the local bundle is configured, stale, missing, or available at the default path |
| `biomcp gtr sync` | Explicit GTR refresh | Force-refreshes the local GTR bulk files |

## Example commands

```bash
biomcp search diagnostic --gene BRCA1 --source gtr --limit 5
```

Returns GTR genetic-test rows linked to BRCA1 with accession IDs ready for `get diagnostic`.

```bash
biomcp search diagnostic --disease "hereditary breast cancer" --source gtr --limit 5
```

Searches GTR condition-linked diagnostic rows using the bounded disease phrase filter.

```bash
biomcp get diagnostic GTR000006692.3 genes conditions methods
```

Fetches a source-native GTR diagnostic card with the main GTR-supported detail sections.

```bash
biomcp gtr sync
```

Force-refreshes the local GTR bulk bundle without waiting for the next automatic refresh.

```bash
biomcp health
```

Shows the `GTR local data` readiness row alongside WHO IVD and the other local-runtime bundles.

## API access

No BioMCP API key required. BioMCP auto-downloads the public GTR bulk files into
`BIOMCP_GTR_DIR` or the default data directory on first use. The optional
`regulatory` section for diagnostic cards is an OpenFDA device overlay and can
benefit from `OPENFDA_API_KEY`; base GTR search and detail do not require a key.

## Official source

[NCBI Genetic Testing Registry](https://www.ncbi.nlm.nih.gov/gtr/) is the
official NCBI diagnostic-test registry behind BioMCP's GTR-backed diagnostic
workflow. BioMCP reads the public bulk export at
<https://ftp.ncbi.nlm.nih.gov/pub/GTR/data/>.

## Related docs

- [Diagnostic](../user-guide/diagnostic.md)
- [Disease](../user-guide/disease.md)
- [Data Sources](../reference/data-sources.md)
- [Troubleshooting](../troubleshooting.md)
