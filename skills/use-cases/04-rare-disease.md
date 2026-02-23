# Pattern: Rare Disease Research

Combine disease normalization, gene association context, trial discovery, and literature support for rare conditions.

## Quick Check

```bash
biomcp get disease "spinal muscular atrophy"
biomcp search trial -c "spinal muscular atrophy" --status recruiting --limit 5
```

## Full Workflow

```bash
# Step 1: Normalize disease term
biomcp get disease "spinal muscular atrophy"

# Step 2: Ontology-aware lookup
biomcp search disease -q "spinal muscular atrophy" --source mondo --limit 5

# Step 2.5: Phenotype-based candidate search
biomcp search phenotype "HP:0001250,HP:0001263" --limit 5

# Step 3: Gene-disease context
biomcp get gene SMN1 diseases

# Step 4: Trial inventory
biomcp search trial -c "spinal muscular atrophy" --status recruiting --limit 5

# Step 5: Recent supporting evidence
biomcp search article -g SMN1 -d "spinal muscular atrophy" --since 2020-01-01 --limit 5
```

## Reference

| Source | Provides |
|--------|----------|
| Disease entity | Normalized terms and synonyms |
| MONDO-filtered search | Ontology alignment |
| Gene disease section | Association context |
| Phenotype search | Monarch Initiative HPO-based disease matching |
| Trial/article search | Actionable and evidence follow-up |

## Validation Checklist

A complete run should produce:

- [ ] Disease normalized to stable terminology
- [ ] Gene-disease association context captured
- [ ] Recruiting trial list returned (or explicitly none)
- [ ] Recent literature evidence retrieved
- [ ] Phenotype-based candidate diseases identified
- [ ] Synonyms/alternate terms used when needed

## Tips

- If direct disease search is sparse, retry with syndrome and phenotype synonyms.
- For very rare entities, combine gene + phenotype in article queries.
- Keep trial filters broad initially, then narrow by intervention/status.
