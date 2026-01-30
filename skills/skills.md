# BioMCP Agent Skills

BioMCP is a biomedical Model Context Protocol implementation that provides AI agents with direct access to clinical trials, research articles, genetic variants, and FDA data.

## Installation

### Using uv (Recommended)

```bash
uv tool install biomcp-python
```

### Using pip

```bash
pip install biomcp-python
```

### Verify Installation

```bash
biomcp --version
biomcp health check
```

## CLI Overview

BioMCP provides these command domains:

| Command | Purpose | Data Source |
|---------|---------|-------------|
| `biomcp article` | Search biomedical literature | PubMed, PubTator3 |
| `biomcp trial` | Find clinical trials | ClinicalTrials.gov |
| `biomcp variant` | Query genetic variants | MyVariant.info |
| `biomcp disease` | Get disease information | MyDisease.info |
| `biomcp openfda adverse` | Drug adverse events | FDA FAERS |
| `biomcp openfda label` | Drug product labels | FDA SPL |
| `biomcp openfda approval` | Drug approvals | Drugs@FDA |
| `biomcp biomarker` | Trial biomarkers | NCI CTS API |
| `biomcp intervention` | Trial interventions | NCI CTS API |
| `biomcp organization` | Trial sponsors | NCI CTS API |

### Common Patterns

```bash
# Search with multiple filters
biomcp article search --gene BRAF --disease melanoma --keyword "resistance"

# Find recruiting trials
biomcp trial search --condition "lung cancer" --status open --phase phase3

# Get variant annotations
biomcp variant search --gene BRCA1 --significance pathogenic

# Check drug safety
biomcp openfda adverse search --drug pembrolizumab --serious
```

## Use Cases

These use cases demonstrate practical BioMCP workflows for biomedical research:

| # | Use Case | Domain | Key Commands |
|---|----------|--------|--------------|
| 1 | [Precision Oncology Report](use-cases/01-precision-oncology.md) | Oncology | variant, trial, article |
| 2 | [Clinical Trial Matching](use-cases/02-trial-matching.md) | Clinical | trial, biomarker |
| 3 | [Drug Safety Investigation](use-cases/03-drug-safety.md) | Pharmacovigilance | openfda adverse |
| 4 | [Variant Interpretation](use-cases/04-variant-interpretation.md) | Genetics | variant, trial |
| 5 | [Rare Disease Research](use-cases/05-rare-disease.md) | Rare Disease | trial, disease, article |
| 6 | [Drug Shortage Monitoring](use-cases/06-drug-shortages.md) | Supply Chain | openfda shortage |
| 7 | [Cell Therapy Trials](use-cases/07-cell-therapy.md) | Immunotherapy | trial, intervention |
| 8 | [Immunotherapy Biomarkers](use-cases/08-immunotherapy.md) | Oncology | trial, biomarker |
| 9 | [FDA Label Search](use-cases/09-drug-labels.md) | Regulatory | openfda label |
| 10 | [Hereditary Cancer Syndromes](use-cases/10-hereditary-cancer.md) | Genetics | variant, trial, disease |

## Quick Reference

### Article Search

```bash
biomcp article search --gene <GENE> --disease <DISEASE> --keyword <TERM>
biomcp article get <PMID>
```

### Trial Search

```bash
biomcp trial search --condition <CONDITION> --status open --phase phase3
biomcp trial search --term <MUTATION> --intervention-type biological
biomcp trial get <NCT_ID>
```

### Variant Search

```bash
biomcp variant search --gene <GENE> --significance pathogenic
biomcp variant search --rsid <RSID>
biomcp variant get <RSID>
```

### FDA Data

```bash
biomcp openfda adverse search --drug <DRUG> --serious
biomcp openfda label search --name <DRUG>
biomcp openfda label search --indication <CONDITION>
biomcp openfda approval search --drug <DRUG>
```

## Resources

- [BioMCP Documentation](https://biomcp.org)
- [GitHub Repository](https://github.com/genomoncology/biomcp)
- [API Reference](https://biomcp.org/reference)
