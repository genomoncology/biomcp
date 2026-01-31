# Pattern: Hereditary Cancer Syndromes

Research multi-gene syndromes for variant interpretation and prevention trials.

## Quick Lookup (1 command each)

```bash
# Gene function and associated cancers
biomcp gene get MLH1 --enrich diseases

# Prevention/surveillance trials
biomcp trial search --condition "Lynch syndrome" --term "prevention"
```

## Full Workflow (run separately)

```bash
# Step 1: Gene info
biomcp gene get <symbol>

# Step 2: Disease associations
biomcp gene get <symbol> --enrich diseases

# Step 3: Syndrome trials
biomcp trial search --condition "<syndrome>" --status OPEN
```

## Common Syndromes

| Syndrome    | Key Genes              |
| ----------- | ---------------------- |
| Lynch       | MLH1, MSH2, MSH6, PMS2 |
| HBOC        | BRCA1, BRCA2           |
| Li-Fraumeni | TP53                   |
| FAP         | APC                    |

## Trial Types

| Type         | Search Approach                   |
| ------------ | --------------------------------- |
| Surveillance | `--term "screening"` + syndrome   |
| Prevention   | `--term "chemoprevention"`        |
| Treatment    | `--condition <associated cancer>` |

## Tips

- Search ALL genes in a syndrome panel, not just one
- Include both syndrome name and gene names in searches
- Check for risk-reduction trials (e.g., PARP inhibitors for BRCA carriers)
