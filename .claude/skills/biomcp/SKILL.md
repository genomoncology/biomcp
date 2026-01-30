---
name: biomcp
description: Query biomedical databases for clinical trials, research articles, genetic variants, and FDA data. Use when asked about genes, mutations, drug safety, clinical trials, variant pathogenicity, drug approvals, or literature search.
---

# BioMCP CLI

Query biomedical databases: PubMed, ClinicalTrials.gov, ClinVar, OpenFDA.

## When to Use

- Literature: "find papers about BRAF", "evidence for this variant"
- Trials: "recruiting trials for lung cancer", "trials near Boston"
- Variants: "is BRCA1 p.E1250K pathogenic", "variant frequency"
- FDA: "adverse events for pembrolizumab", "drug shortages"

## Installation

```bash
uv tool install biomcp-python
biomcp --version
```

## Commands

| Command                          | Purpose                  |
| -------------------------------- | ------------------------ |
| `biomcp article search`          | Find literature (PubMed) |
| `biomcp article get`             | Get article by PMID      |
| `biomcp trial search`            | Find clinical trials     |
| `biomcp trial get`               | Get trial by NCT ID      |
| `biomcp variant search`          | Query ClinVar variants   |
| `biomcp variant get`             | Get variant by rsID      |
| `biomcp openfda adverse search`  | Drug adverse events      |
| `biomcp openfda approval search` | Drug approvals           |
| `biomcp openfda shortage search` | Drug shortages           |
| `biomcp openfda label search`    | Drug labels              |

**Discover options:** `biomcp [command] [subcommand] --help`

**Important:** BioMCP is a command-line tool. Always use `biomcp` commands directly—do not import it as a Python library.

## Key Patterns

- Use `-j` or `--json` for structured output
- Status enums are UPPERCASE (OPEN, RECRUITING, PHASE3)
- Normalize before searching (see below)

## Normalization

- **Genes:** Use HGNC symbols (HER2 → ERBB2)
- **Variants:** Use HGVS notation (V600E → p.Val600Glu)
- **Diseases:** Use standard terms via `biomcp disease get`

## Use Cases

See `use-cases/` for detailed clinical workflows:

| File                        | Pattern                                        |
| --------------------------- | ---------------------------------------------- |
| `01-mutation-to-trial`      | Mutation → annotations → trials → literature   |
| `02-drug-investigation`     | Drug → adverse events → approvals → labels     |
| `03-trial-matching`         | Patient criteria → filtered trial search       |
| `04-variant-interpretation` | Variant → pathogenicity → actionability        |
| `05-rare-disease`           | Rare condition → gene therapy → registries     |
| `06-drug-shortages`         | Drug → shortage status → alternatives          |
| `07-cell-therapy`           | CAR-T/TIL/TCR-T trial landscape                |
| `08-immunotherapy`          | Biomarker-driven checkpoint inhibitor trials   |
| `09-drug-labels`            | FDA labels → indications → warnings            |
| `10-hereditary-cancer`      | Syndrome → genes → surveillance trials         |
| `11-resistance-research`    | Prior therapy → resistance → next-line options |

## Troubleshooting

```bash
biomcp health check --verbose
BIOMCP_LOG_LEVEL=DEBUG biomcp article search --gene TP53
```
