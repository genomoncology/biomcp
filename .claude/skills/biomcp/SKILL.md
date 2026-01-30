---
name: biomcp
description: Query biomedical databases for clinical trials, research articles, genetic variants, and FDA data. Use when asked about genes, mutations, drug safety, clinical trials, variant pathogenicity, drug approvals, or literature search.
---

# BioMCP CLI

BioMCP provides AI agents with direct access to biomedical data sources via command-line interface.

## When to Use

- Literature search: "find papers about BRAF melanoma", "evidence for this variant"
- Clinical trials: "recruiting trials for lung cancer", "trials near Boston"
- Variant analysis: "is BRCA1 p.E1250K pathogenic", "variant frequency"
- Drug safety: "adverse events for pembrolizumab", "drug interactions"
- FDA data: "drug approvals in 2023", "current drug shortages"

## Installation

```bash
uv tool install biomcp-python
biomcp --version
biomcp health check
```

## Commands

| Command | Purpose | Example |
|---------|---------|---------|
| `biomcp article search` | Find literature | `--gene BRAF --disease melanoma` |
| `biomcp article get` | Fetch paper | `38768446` |
| `biomcp trial search` | Find trials | `--condition "lung cancer" --status OPEN` |
| `biomcp trial get` | Trial details | `NCT04280705` |
| `biomcp variant search` | Query variants | `--gene BRCA1 --significance pathogenic` |
| `biomcp variant get` | Variant details | `rs113488022` |
| `biomcp openfda adverse search` | Drug safety | `--drug pembrolizumab --serious` |
| `biomcp openfda approval search` | Drug approvals | `--year 2023 --limit 10` |
| `biomcp openfda shortage search` | Drug shortages | `--status current` |

## Output Formats

Use `-j` or `--json` for JSON output:

```bash
biomcp variant search --gene BRAF --significance pathogenic -j
biomcp trial search --condition melanoma --status OPEN -j
biomcp article search --gene EGFR --disease "lung cancer" -j
```

## Normalization

Normalize entity names before searching:

- Genes: Use HGNC symbols (HER2 → ERBB2)
- Variants: Use HGVS notation (V600E → p.Val600Glu)
- Diseases: Use standard terms via `biomcp disease get`

## Use Cases

Detailed workflows in `use-cases/`:

1. [Precision Oncology](use-cases/01-precision-oncology.md) - Tumor profiling reports
2. [Trial Matching](use-cases/02-trial-matching.md) - Patient-trial matching
3. [Drug Safety](use-cases/03-drug-safety.md) - Adverse event investigation
4. [Variant Interpretation](use-cases/04-variant-interpretation.md) - Clinical significance
5. [Rare Disease](use-cases/05-rare-disease.md) - Orphan disease research
6. [Drug Shortages](use-cases/06-drug-shortages.md) - Supply chain monitoring
7. [Cell Therapy](use-cases/07-cell-therapy.md) - CAR-T trial landscape
8. [Immunotherapy](use-cases/08-immunotherapy.md) - Biomarker-driven trials
9. [Drug Labels](use-cases/09-drug-labels.md) - FDA label search
10. [Hereditary Cancer](use-cases/10-hereditary-cancer.md) - Germline variants

## Common Workflows

### Variant pathogenicity + trials (e.g., BRAF V600E melanoma)
```bash
biomcp variant search --gene BRAF --hgvsp "p.Val600Glu" --significance pathogenic -j
biomcp trial search --condition melanoma --required-mutation "BRAF V600E" --status OPEN -j
```

### Drug safety investigation
```bash
biomcp openfda adverse search --drug pembrolizumab --serious -j
biomcp openfda approval search --drug pembrolizumab -j
```

### Literature + trial synthesis
```bash
biomcp article search --gene EGFR --disease "lung cancer" --keyword resistance -j
biomcp trial search --condition "lung cancer" --term "EGFR resistance" --phase PHASE3 --status OPEN -j
```

## Troubleshooting

```bash
# Check API status
biomcp health check --verbose

# Enable debug logging
BIOMCP_LOG_LEVEL=DEBUG biomcp article search --gene TP53
```

## Resources

- [Documentation](https://biomcp.org)
- [GitHub](https://github.com/genomoncology/biomcp)
