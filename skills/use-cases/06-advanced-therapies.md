# Pattern: Advanced Therapy Trials

Find and triage advanced-therapy trials (CAR-T, checkpoint, and combination immunotherapy) with eligibility and safety context.

## Quick Check

```bash
biomcp search trial -i "CAR-T" -c lymphoma --status recruiting --limit 5
biomcp get trial NCT02445248 eligibility
```

## Full Workflow

```bash
# Step 1: Therapy-type trial search
biomcp search trial -i "CAR-T" -c lymphoma --status recruiting --limit 5

# Step 2: Eligibility detail for top candidate
biomcp get trial NCT02445248 eligibility

# Step 3: Recent open-access therapy evidence
biomcp search article -d lymphoma --drug axicabtagene --open-access --since 2022-01-01 --limit 5

# Step 4: Target pathway context
biomcp get gene CTLA4 pathways
biomcp get gene PDCD1 pathways

# Step 5: Targeted safety signal check
biomcp search adverse-event -d axicabtagene --serious --reaction "cytokine release" --limit 5
```

## Reference

| Source | Provides |
|--------|----------|
| Trial search/get | Trial options and eligibility |
| Article search | Recent therapy evidence |
| Gene pathways | Immunotherapy target context |
| Adverse-event search | Post-market safety signals |

## Validation Checklist

A complete run should produce:

- [ ] Advanced-therapy trial shortlist with trial IDs
- [ ] Eligibility reviewed for at least one candidate
- [ ] Supporting literature pulled
- [ ] Pathway context for relevant targets
- [ ] Safety signals checked for representative agent

## Tips

- Pair `-i` intervention strings with disease filters for better precision.
- Keep an explicit note of prior-therapy constraints from eligibility sections.
- If results are sparse, broaden intervention keywords and retry.
