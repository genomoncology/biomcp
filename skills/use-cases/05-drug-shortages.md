# Pattern: Drug Shortage Monitoring

Track shortage status and identify feasible therapeutic alternatives when supply is constrained.

## Quick Check

```bash
biomcp get drug carboplatin shortage
biomcp search drug --indication "ovarian cancer" --limit 5
```

## Full Workflow

```bash
# Step 1: Confirm shortage status
biomcp get drug carboplatin shortage

# Step 2: Review base drug profile
biomcp get drug carboplatin

# Step 3: Find indication-based alternatives (filter by type)
biomcp search drug --indication "ovarian cancer" --type small-molecule --limit 5

# Step 4: Find mechanism-based alternatives
biomcp search drug --mechanism inhibitor --limit 5

# Step 5: Safety sample for alternative candidate (filter by reaction)
biomcp search adverse-event -d cisplatin --serious --reaction nephrotoxicity --limit 5
```

## Reference

| Source | Provides |
|--------|----------|
| Drug shortage section | Current shortage signals |
| Drug profile | Targets/mechanism context |
| Drug search | Alternative candidates |
| Adverse-event search | Safety signal sampling |

## Validation Checklist

A complete run should produce:

- [ ] Shortage status confirmed for index drug
- [ ] At least one alternative by indication
- [ ] At least one alternative by mechanism
- [ ] Safety check run on an alternative
- [ ] Final shortlist includes rationale

## Tips

- Use both indication and mechanism paths; they often return different candidate sets.
- Re-run shortage checks periodically during active supply disruptions.
- Keep safety checks lightweight first, then deep-dive on finalists.
- Use `--count-only` on trial search to gauge how many alternative pipeline trials exist.
