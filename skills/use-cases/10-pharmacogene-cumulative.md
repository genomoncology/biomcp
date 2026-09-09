# Pattern: Pharmacogene cumulative evidence

Use this when the question asks which genes collectively influence response, dosing, or toxicity for a drug.

```bash
biomcp search pgx -d warfarin --limit 10
biomcp get pgx warfarin recommendations annotations
biomcp search article --drug warfarin -k "CYP2C9 VKORC1 dose response" --limit 10
biomcp batch article 17048007,19794411,19958090 --mode compact
```

Interpretation:
- Start with PGx recommendations and annotations to find curated gene-drug evidence.
- Use article search to catch cumulative evidence that spans multiple pharmacogenes.
- Batch the strongest candidate PMIDs instead of opening papers one at a time.
- Distinguish well-supported dosing genes from weaker association-only candidates.
