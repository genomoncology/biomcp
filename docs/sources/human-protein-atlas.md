---
title: "Human Protein Atlas MCP Tool for Tissue Expression | BioMCP"
description: "Use BioMCP to surface Human Protein Atlas tissue expression and localization data through the BioMCP gene hpa section."
---

# Human Protein Atlas

Human Protein Atlas matters when gene-level summaries are not enough and you need tissue-aware protein context. It is the page to use when a workflow depends on where a protein is expressed, how it localizes inside cells, or whether cancer-expression context helps explain why a target matters.

In BioMCP, Human Protein Atlas is surfaced through the gene `hpa` section. That section focuses on tissue expression, subcellular localization, and cancer expression rather than trying to reproduce the full Human Protein Atlas website or every assay detail.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `get gene <symbol> hpa` | Protein tissue expression, localization, and cancer-expression context | Explicit gene section backed by Human Protein Atlas |
| `gene cell-lines <symbol> --group <group>` | The RNA level (nTPM) of one gene in every HPA cell line of one cancer group | One search download call per gene. Each row carries the Cellosaurus accession when exactly one human cell line carries the HPA name |

## Example commands

```bash
biomcp get gene BRAF hpa
```

Returns Human Protein Atlas tissue-expression and localization context for BRAF.

```bash
biomcp get gene EGFR hpa
```

Returns Human Protein Atlas tissue-expression context for EGFR.

```bash
biomcp get gene TP53 hpa
```

Returns Human Protein Atlas tissue and cancer-expression context for TP53.

```bash
biomcp gene cell-lines FLT3 --group leukemia
```

Returns the nTPM of FLT3 in each of the 93 cell lines of the HPA leukemia group, as published. BioMCP adds no labels and no thresholds, and it offers no sort.

The 30 cancer groups are `adrenocortical_cancer`, `bile_duct_cancer`, `bladder_cancer`, `bone_cancer`, `brain_cancer`, `breast_cancer`, `cervical_cancer`, `colorectal_cancer`, `esophageal_cancer`, `gallbladder_cancer`, `gastric_cancer`, `head_and_neck_cancer`, `kidney_cancer`, `leukemia`, `liver_cancer`, `lung_cancer`, `lymphoma`, `myeloma`, `neuroblastoma`, `non-cancerous`, `ovarian_cancer`, `pancreatic_cancer`, `prostate_cancer`, `rhabdoid`, `sarcoma`, `skin_cancer`, `testis_cancer`, `thyroid_cancer`, `uncategorized`, and `uterine_cancer`. They cover 1,206 cell lines. An unknown group fails before any request and lists the names.

`data_as_of` is `2025-11-05`, the date of the HPA cell line RNA file. The search download response carries no date, so the value is a constant next to the group list in the source module. It was read from the `Last-Modified` header of the `rna_celline.tsv.zip` file the [data access page](https://www.proteinatlas.org/about/help/dataaccess) links, and it is refreshed the same way the group list is. A future response that carries its own date is preferred over the constant.

## API access

No BioMCP API key required.

## Official source

[Human Protein Atlas](https://www.proteinatlas.org/) is the official tissue-expression and localization resource behind BioMCP's gene `hpa` section.

## Related docs

- [Gene](../user-guide/gene.md)
- [Data Sources](../reference/data-sources.md)
- [Source Licensing and Terms](../reference/source-licensing.md)
