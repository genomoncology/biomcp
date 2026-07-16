---
title: "cBioPortal MCP Tool for Cohort Variant Context | BioMCP"
description: "Use BioMCP to add cBioPortal cohort-frequency context to variants and download local study datasets for BioMCP study analytics."
---

# cBioPortal

cBioPortal matters when a single variant or gene question turns into a cohort question. It is the source page to use when you need cancer-study frequencies, downloadable study datasets, or local analytics that operate on a concrete study instead of a live one-record API lookup.

In BioMCP, the variant `cbioportal` section adds best-effort cohort frequency context, while `study` is BioMCP's local cBioPortal analytics family for downloaded datasets. Use `study list` to see the local cohort list, `study download` to install a study into your local root, and `study query` when you want per-study mutation, CNA, expression, or structural-variant/fusion summaries from local files. If a query names a study outside the local snapshot, BioMCP returns `not_in_local_cohorts` with a `study download <study_id>` hint rather than a hollow cohort result.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `get variant <id> cbioportal` | Cohort-frequency context for a variant | Best-effort cBioPortal enrichment section |
| `study download --list` | List of downloadable study IDs | Local analytics entry point |
| `study download <study_id>` | Local installation of one study dataset | Downloads into the default study root or `BIOMCP_STUDY_DIR` |
| `study query --study <id> --gene <symbol> --type <mutations|cna|expression|sv>` | Per-study summaries for one gene, including structural variants/fusions from `data_sv.txt` | Local analytics workflow over downloaded files |

## Example commands

```bash
biomcp get variant "BRAF V600E" cbioportal
```

Returns a variant section with best-effort cBioPortal cohort frequency context.

```bash
biomcp study download --list
```

Returns a list of downloadable cBioPortal-style study IDs.

```bash
biomcp study download msk_impact_2017
```

Downloads the named study into the configured local study root. `biomcp study list` documents which cohorts are local in that root.

```bash
biomcp study query --study msk_impact_2017 --gene TP53 --type mutations
biomcp study query --study msk_impact_2017 --gene RET --type sv
```

Returns a per-study mutation summary for TP53 from local study files. Mutation summaries remain mutation-only; if local `data_sv.txt` is present, BioMCP notes that fusions/SV are excluded and points to `--type sv`. For a non-local study ID such as an undownloaded melanoma cohort, the query returns `not_in_local_cohorts` and suggests `biomcp study download <study_id>`.

## API access

No BioMCP API key required. Local study analytics use downloaded datasets in the default study root or `BIOMCP_STUDY_DIR`.

Study archives are capped at 2 GiB compressed and 100,000 physical tar entries,
with 1 GiB per regular member, 8 GiB aggregate payload, and 1 MiB of path
extension metadata. BioMCP validates the complete archive in staging before the
final install rename. A limit or unsafe archive-metadata failure is reported as
source unavailable without exposing archive paths or payloads, and partial
download/staging files are removed.

## Official source

[cBioPortal](https://www.cbioportal.org/) is the official cancer-genomics portal behind BioMCP's cohort-frequency enrichment and study download workflows.

## Related docs

- [Variant](../user-guide/variant.md)
- [CLI Reference](../user-guide/cli-reference.md)
- [Quick Reference](../reference/quick-reference.md)
