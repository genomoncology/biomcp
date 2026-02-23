# Pattern: Pharmacogenomics

Assess gene-drug interactions, CPIC guideline levels, and dosing recommendations for precision medicine.

## Quick Check

```bash
biomcp search pgx -g CYP2D6 --limit 5
biomcp get pgx CYP2D6 recommendations
```

## Full Workflow

```bash
# Step 1: Gene-drug interaction overview
biomcp search pgx -g CYP2D6 --limit 10

# Step 2: Drug-focused PGx lookup
biomcp search pgx -d codeine --limit 5

# Step 3: Full PGx profile with recommendations
biomcp get pgx CYP2D6 recommendations

# Step 4: Clinical annotation context
biomcp get pgx CYP2D6 annotations

# Step 5: Cross-check drug label for PGx language
biomcp get drug codeine label
```

## Reference

| Source | Provides |
|--------|----------|
| PharmGKB | Gene-drug associations and clinical annotations |
| CPIC | Guideline levels (A, B, C, D) |
| Drug label section | Pharmacogenomic prescribing language |

## Validation Checklist

- [ ] Gene-drug interactions listed with CPIC levels
- [ ] Drug-specific PGx associations returned
- [ ] Dosing recommendations retrieved
- [ ] Clinical annotation evidence captured
- [ ] Drug label PGx language cross-checked

## Tips

- CPIC Level A = strongest evidence for dosing changes.
- Start with gene if the patient genotype is known; start with drug if reviewing a prescription.
- Combine PGx with variant search if a specific star allele is involved.
