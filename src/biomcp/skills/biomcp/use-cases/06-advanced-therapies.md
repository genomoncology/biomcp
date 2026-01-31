# Pattern: Advanced Therapy Trials

Find trials for cell therapies (CAR-T, TIL) and immunotherapies (checkpoint inhibitors).

## Quick Search (1 command)

```bash
# CAR-T trials for a cancer
biomcp trial search --term "CAR-T" --condition "<cancer>" --status OPEN

# Checkpoint inhibitor trials by biomarker
biomcp trial search --biomarker "PD-L1" --condition "<cancer>" --status OPEN
```

## Full Workflow (run separately)

```bash
# Step 1: Find trials by therapy type
biomcp trial search --term "CAR-T" --condition "<cancer>"

# Step 2: Get specific trial details
biomcp trial get <NCT_ID>

# Step 3: Check drug safety (if needed)
biomcp openfda adverse search --drug "<drug name>" --serious
```

## Search Terms

| Therapy Type | Search Term                                              |
| ------------ | -------------------------------------------------------- |
| CAR-T        | `--term "CAR-T"` or `--term "chimeric antigen receptor"` |
| TIL          | `--term "tumor infiltrating lymphocytes"`                |
| TCR-T        | `--term "TCR T cell"`                                    |
| PD-1/PD-L1   | `--term "pembrolizumab"` or `--biomarker "PD-L1"`        |
| Combinations | `--term "ipilimumab nivolumab"`                          |

## Key Biomarkers

| Biomarker | Relevance                     |
| --------- | ----------------------------- |
| CD19      | B-cell CAR-T target           |
| BCMA      | Multiple myeloma CAR-T        |
| PD-L1     | Checkpoint inhibitor response |
| TMB-H     | Tumor mutational burden high  |
| MSI-H     | Microsatellite instability    |

## Tips

- CAR-T trials have complex eligibility (prior therapies, organ function)
- Check for prior immunotherapy exposure requirements
- Biomarker cutoffs vary by trial (PD-L1 ≥1% vs ≥50%)
- Use `biomcp article search -c <drug> -d <cancer>` for efficacy data
