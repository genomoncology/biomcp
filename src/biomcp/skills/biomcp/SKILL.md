---
name: biomcp
description: Query biomedical databases for clinical trials, research articles, genetic variants, and FDA data. Use when asked about genes, mutations, drug safety, clinical trials, variant pathogenicity, drug approvals, or literature search.
---

# BioMCP CLI

Query biomedical databases: PubTator3, ClinicalTrials.gov, MyVariant.info, MyGene.info, MyChem.info, OpenFDA.

## When to Use

- Literature: "find papers about BRAF", "evidence for this variant"
- Trials: "recruiting trials for lung cancer", "trials near Boston"
- Genes: "what does ERBB2 do", "gene function and pathways"
- Variants: "is BRCA1 p.E1250K pathogenic", "predict variant effect"
- Drugs: "information about pembrolizumab", "drug mechanisms"
- Diseases: "what is Lynch syndrome", "disease-gene associations"
- FDA: "adverse events for pembrolizumab", "drug shortages", "device recalls"

## Installation

```bash
uv tool install biomcp-python
biomcp --version
```

## Commands

### Literature (PubTator3)

PubTator3 provides entity-annotated article search with filters for genes, variants, diseases, and chemicals.

| Command                 | Purpose                                                   |
| ----------------------- | --------------------------------------------------------- |
| `biomcp article search` | Search articles by entity (gene/variant/disease/chemical) |
| `biomcp article get`    | Get article details by PMID or DOI                        |

**Article search options:** `-g/--gene`, `-v/--variant`, `-d/--disease`, `-c/--chemical`, `-k/--keyword`

### Clinical Trials (ClinicalTrials.gov / NCI CTS)

| Command                          | Purpose                                     |
| -------------------------------- | ------------------------------------------- |
| `biomcp trial search`            | Find trials by disease, intervention, phase |
| `biomcp trial get`               | Get trial details by NCT ID                 |
| `biomcp biomarker search`        | Find biomarkers in trial eligibility        |
| `biomcp organization search/get` | Search trial sites and organizations        |
| `biomcp intervention search/get` | Search drugs, devices, procedures           |
| `biomcp intervention types`      | List intervention type values               |

**Trial search filters:**

- Location: `--lat <lat> --lon <lon> --distance <miles>`
- Eligibility: `--required-mutation`, `--excluded-mutation`, `--line-of-therapy 1L|2L|3L+`
- Design: `--sponsor-type`, `--study-design`, `--purpose`

### Genes (MyGene.info)

| Command              | Purpose                                   |
| -------------------- | ----------------------------------------- |
| `biomcp gene search` | Search genes by symbol, name, or keyword  |
| `biomcp gene get`    | Get gene details (function, pathways, GO) |

**Enrichment analysis:** `biomcp gene get <symbol> --enrich <type>`
Types: `pathway`, `kegg`, `reactome`, `ontology`, `go_process`, `diseases`, `tissues`

### Variants (MyVariant.info + AlphaGenome)

| Command                  | Purpose                                   |
| ------------------------ | ----------------------------------------- |
| `biomcp variant search`  | Search variants by gene, HGVS, or rsID    |
| `biomcp variant get`     | Get variant annotations (ClinVar, gnomAD) |
| `biomcp variant predict` | Predict variant effects via AlphaGenome   |

### Drugs (MyChem.info)

| Command              | Purpose                                |
| -------------------- | -------------------------------------- |
| `biomcp drug search` | Search drugs by name or identifier     |
| `biomcp drug get`    | Get drug details (targets, mechanisms) |

### Diseases (MyDisease.info)

| Command                 | Purpose                                 |
| ----------------------- | --------------------------------------- |
| `biomcp disease search` | Search diseases by name or keyword      |
| `biomcp disease get`    | Get disease details (genes, phenotypes) |

### FDA Data (OpenFDA)

| Command                          | Purpose                       |
| -------------------------------- | ----------------------------- |
| `biomcp openfda adverse search`  | Drug adverse events (FAERS)   |
| `biomcp openfda approval search` | Drug approvals (Drugs@FDA)    |
| `biomcp openfda shortage search` | Drug shortages                |
| `biomcp openfda label search`    | Drug product labels (SPL)     |
| `biomcp openfda device search`   | Device adverse events (MAUDE) |
| `biomcp openfda recall search`   | Drug recalls (Enforcement)    |

**Adverse event filters:** `--drug <name>`, `--reaction <term>`, `--serious`

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

| File                      | Pattern                                        |
| ------------------------- | ---------------------------------------------- |
| `01-variant-to-treatment` | Variant → interpretation → trials → treatment  |
| `02-drug-investigation`   | Drug → adverse events → labels → approvals     |
| `03-trial-matching`       | Patient criteria → filtered trial search       |
| `04-rare-disease`         | Rare condition → gene therapy → registries     |
| `05-drug-shortages`       | Drug → shortage status → alternatives          |
| `06-advanced-therapies`   | CAR-T, immunotherapy trial landscape           |
| `07-hereditary-cancer`    | Syndrome → genes → surveillance trials         |
| `08-resistance`           | Prior therapy → resistance → next-line options |

## Troubleshooting

```bash
biomcp health check --verbose
BIOMCP_LOG_LEVEL=DEBUG biomcp article search --gene TP53
```
