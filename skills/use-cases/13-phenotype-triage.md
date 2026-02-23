# Pattern: Phenotype-Based Disease Triage

Start from observed clinical phenotypes (HPO terms or free text) and identify candidate diseases, causal genes, and actionable next steps.

## Quick Check

```bash
biomcp search phenotype "HP:0001250,HP:0001263" --limit 5
biomcp get disease "Dravet syndrome" genes
```

## Full Workflow

```bash
# Step 1: Phenotype-to-disease matching
biomcp search phenotype "HP:0001250,HP:0001263,HP:0000252" --limit 5

# Step 2: Review top candidate disease
biomcp get disease "Dravet syndrome"

# Step 3: Gene associations for candidate
biomcp get disease "Dravet syndrome" genes

# Step 4: Variant search in causal gene
biomcp search variant -g SCN1A --significance pathogenic --limit 5

# Step 5: Trial options for candidate
biomcp disease trials "Dravet syndrome" --limit 5
```

## Reference

| Source | Provides |
|--------|----------|
| Monarch Initiative | HPO-based disease similarity scoring |
| Disease entity | Normalization, MONDO IDs, synonyms |
| Disease gene section | Gene-disease associations (DisGeNET, OMIM) |
| Variant search | Pathogenic variants in candidate genes |

## Validation Checklist

- [ ] Candidate diseases ranked by phenotype similarity
- [ ] Top candidate disease normalized and described
- [ ] Gene-disease associations retrieved
- [ ] Pathogenic variants in causal gene listed
- [ ] Trial or treatment follow-up initiated

## Tips

- Use HPO IDs for precise matching; free text works but is less specific.
- Multiple phenotypes improve specificity — 3+ terms recommended.
- For very rare conditions, combine phenotype search with literature search by gene.
