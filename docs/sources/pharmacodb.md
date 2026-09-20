---
title: "PharmacoDB MCP Tool for Cell Line Drug Response | BioMCP"
description: "Use BioMCP to read published PharmacoDB drug sensitivity experiments for a cell line or a compound, scoped by dataset, without writing GraphQL."
---

# PharmacoDB

PharmacoDB matters when you have a cell line and want to know which compounds were tested on it, and in which study. It gathers the large pharmacogenomic screens, CTRPv2, GDSC1, GDSC2, gCSI, NCI60, PRISM and the rest, behind one GraphQL endpoint, and reports one row per experiment with the published sensitivity metrics.

In BioMCP, PharmacoDB answers two questions from two directions. `get cell-line <accession> drug_response` counts the experiments a line has in each dataset, and `get drug <name> cell_lines` counts them for a compound. Both sections are asked for by name and never load under `all`, because the row listings behind them are large: the unscoped K-562 response is 19.4 MB of body, so every row helper demands a scope and refuses without one. The rows are the values PharmacoDB published. There are no units, no thresholds, and no sort BioMCP invented, and a repeated cell line and compound pair in one dataset stays as separate rows with separate experiment IDs.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `get cell-line <accession> drug_response` | Experiment counts per dataset for one cell line | The join goes through the Cellosaurus PharmacoDB xref, then the line name, and the accession PharmacoDB returns must match |
| `get drug <name> cell_lines` | Experiment counts per dataset for one compound | PharmacoDB publishes no ChEMBL or DrugBank key on a compound, so the join goes by name and the name it returns must be the name asked for |
| `cell-line drug-response <accession> --dataset <name>` | One page of experiment rows for that line inside one dataset | `--dataset` is required; an unscoped listing is megabytes of body |
| `drug cell-lines <name> --cell-line <accession>` | The rows for one compound on one cell line, usually 0 to 3 | One of `--cell-line` or `--dataset` is required |
| `drug cell-lines <name> --dataset <name>` | One page of rows for that compound inside one dataset | Rows carry AAC, IC50, EC50, Einf, HS, and DSS1 exactly as published, with `-` for an absent value |

## Example commands

```bash
biomcp get cell-line CVCL_2119 drug_response
```

Returns the MOLM-13 counts, 1117 experiments across CTRPv2, gCSI, GDSC1, and GDSC2.

```bash
biomcp cell-line drug-response CVCL_2119 --dataset GDSC1
```

Returns the first 25 of the 426 GDSC1 rows for MOLM-13, each naming the compound.

```bash
biomcp get drug venetoclax cell_lines
```

Returns the venetoclax counts, 3608 experiments across CTRPv2, GDSC1, GDSC2, NCI60, and PRISM.

```bash
biomcp drug cell-lines venetoclax --cell-line CVCL_2119
```

Returns the two published venetoclax-on-MOLM-13 experiments, one in GDSC1 and one in GDSC2.

```bash
biomcp drug cell-lines venetoclax --dataset NCI60
```

Returns the first 25 of the 113 NCI60 rows for venetoclax, each naming the cell line and its tissue.

## Release and attribution

PharmacoDB publishes no version or release date, so every output reports the retrieval time as `data_as_of` with kind `retrieved`. PharmacoDB also publishes no licence or terms page: its source code is GPL-3.0 and the paper describing it is CC BY-NC 4.0, and the terms for the data itself are unstated by the provider. The attribution line on every output says so and asks you to treat reuse as non-commercial.

## API access

No BioMCP API key required. Set `BIOMCP_PHARMACODB_BASE` to point at a different host.

## Official source

[PharmacoDB](https://pharmacodb.ca/) is the pharmacogenomic experiment resource behind BioMCP's cell line drug response and drug cell lines sections. Its GraphQL endpoint is unauthenticated and read-only.

## Related docs

- [Cell Line](../user-guide/cell-line.md)
- [Drug](../user-guide/drug.md)
- [Data Sources](../reference/data-sources.md)
- [Source Licensing](../reference/source-licensing.md)
