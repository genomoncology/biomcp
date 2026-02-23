# Pattern: Variant to Treatment

From a detected variant, derive treatment context, trial options, and evidence support.

## Quick Check

```bash
biomcp get variant "BRAF V600E" clinvar
biomcp variant trials "BRAF V600E" --limit 5
```

## Full Workflow

```bash
# Step 1: Variant interpretation
biomcp get variant "BRAF V600E" clinvar

# Step 2: Gene pathway context
biomcp get gene BRAF pathways

# Step 3: Similar pathogenic variants (filter by population frequency)
biomcp search variant -g BRAF --significance pathogenic --consequence missense --max-frequency 0.01 --limit 5

# Step 4: Mutation-focused trial options
biomcp variant trials "BRAF V600E" --limit 5

# Step 5: Supporting literature (exclude retracted papers)
biomcp search article -g BRAF -d melanoma --since 2020-01-01 --exclude-retracted --limit 5
```

## Reference

| Source | Provides |
|--------|----------|
| ClinVar | Clinical significance and review status |
| gnomAD | Population frequency context |
| OncoKB | Oncogenicity and actionability context |
| ClinicalTrials.gov/NCI CTS | Trial opportunities |

## Validation Checklist

A complete run should produce:

- [ ] Variant significance identified
- [ ] Gene pathway context returned
- [ ] Similar variants list generated
- [ ] Trial IDs surfaced for the mutation
- [ ] Literature evidence includes relevant PMIDs

## Tips

- Use `--consequence` and `--max-frequency` to tighten variant review.
- If trial results are sparse, switch to `biomcp search trial -c <disease> --mutation "<variant>"`.
- If API responses are throttled, retry after a short pause.
