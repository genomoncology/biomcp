# Pattern: Drug Investigation

Investigate mechanism, label context, safety signals, and alternatives for a drug.

## Quick Check

```bash
biomcp get drug pembrolizumab
biomcp search adverse-event -d pembrolizumab --serious --limit 5
```

## Full Workflow

```bash
# Step 1: Drug profile and target context
biomcp get drug pembrolizumab

# Step 2: Label review
biomcp get drug pembrolizumab label

# Step 3: Safety signals
biomcp search adverse-event -d pembrolizumab --serious --limit 5

# Step 4: Recall/shortage surveillance
biomcp get drug pembrolizumab shortage
biomcp search adverse-event -d pembrolizumab --type recall --limit 5

# Step 5: Mechanism-based alternatives (filter by type)
biomcp search drug --mechanism inhibitor --type small-molecule --limit 5
```

## Reference

| Source | Provides |
|--------|----------|
| MyChem/OpenTargets | Targets and mechanism |
| OpenFDA label | Indications, warnings, usage guidance |
| OpenFDA FAERS/recall | Safety and recall signal context |
| Drug shortage section | Current supply status |

## Validation Checklist

A complete run should produce:

- [ ] Drug mechanism and target context
- [ ] Label content reviewed
- [ ] Serious adverse-event sample retrieved
- [ ] Recall/shortage status checked
- [ ] At least one plausible alternative surfaced

## Tips

- Use `--target` and `--indication` together for higher-precision alternatives.
- Confirm alternatives with label and adverse-event checks before triage decisions.
- For broad classes, run multiple mechanism queries and compare overlap.
