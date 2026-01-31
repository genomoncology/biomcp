# Pattern: Drug Investigation

Investigate a drug's safety profile, approvals, and prescribing information.

## Quick Safety Check (1 command)

```bash
biomcp openfda adverse search --drug <name> --serious
```

This returns top serious adverse events from FAERS. Start here for rapid assessment.

## Full Workflow (run separately)

Run each command individually for comprehensive investigation:

```bash
# Step 1: Drug mechanism
biomcp drug get <name>

# Step 2: Serious adverse events
biomcp openfda adverse search --drug <name> --serious

# Step 3: FDA label (warnings, contraindications)
biomcp openfda label search --drug <name>
```

Optional additional queries:

- `biomcp openfda approval search --drug <name>` - approval history
- `biomcp openfda shortage search --drug <name>` - supply status

## Label Sections

| Section           | What to Look For                |
| ----------------- | ------------------------------- |
| Indications       | Approved uses, required testing |
| Contraindications | When not to prescribe           |
| Warnings          | Boxed warnings, serious risks   |
| Drug Interactions | Concomitant medications         |

## Tips

- Use `--serious` flag to focus on serious adverse events
- Use `--reaction <term>` to search for specific adverse reactions
- Use generic drug names for comprehensive searches
- Check indications for required companion diagnostics
- Combine with `biomcp article search -c <drug>` for case reports
