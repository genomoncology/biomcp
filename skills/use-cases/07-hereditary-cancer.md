# Pattern: Hereditary Cancer Syndromes

Investigate hereditary syndrome context, associated genes and variants, and relevant trial/evidence follow-up.

## Quick Check

```bash
biomcp get disease "Lynch syndrome"
biomcp get gene MLH1 diseases
```

## Full Workflow

```bash
# Step 1: Normalize syndrome name
biomcp get disease "Lynch syndrome"

# Step 1.5: Phenotype-based investigation
biomcp search phenotype "HP:0003002,HP:0002027,HP:0100279" --limit 5

# Step 2: Gene-level syndrome context
biomcp get gene MLH1 diseases

# Step 3: Variant review in key gene (pathogenic + consequence filter)
biomcp search variant -g MLH1 --significance pathogenic --consequence missense --limit 5

# Step 4: Trial search
biomcp search trial -c "Lynch syndrome" --status recruiting --limit 5

# Step 5: Supporting literature
biomcp search article -g MLH1 -d "colorectal cancer" --limit 5
```

## Reference

| Source | Provides |
|--------|----------|
| Disease entity | Syndrome naming/synonyms |
| Gene disease section | Hereditary association context |
| Variant search | Candidate variant context |
| Trial/article search | Next-step clinical and evidence data |

## Validation Checklist

A complete run should produce:

- [ ] Syndrome normalized and described
- [ ] Syndrome-gene association confirmed
- [ ] Variant context retrieved for core gene(s)
- [ ] Trial options listed with IDs
- [ ] Phenotype-based differential diagnosis checked
- [ ] Supporting literature evidence captured

## Tips

- Include both syndrome term and disease phenotype terms in trial queries.
- Use `--consequence` or `--significance` filters to reduce variant noise.
- Keep a syndrome-to-gene table during case review.
