# Pattern: Resistance and Next-Line Options

Analyze resistance mechanisms and identify next-line trial or treatment options after progression.

## Quick Check

```bash
biomcp search article -g EGFR --drug osimertinib -k resistance --limit 5
biomcp variant trials "EGFR C797S" --limit 5
```

## Full Workflow

```bash
# Step 1: Resistance-focused literature
biomcp search article -g EGFR --drug osimertinib -k resistance --limit 5

# Step 2: Resistance variant context (protein-level search)
biomcp search variant -g EGFR --hgvsp C797S --significance pathogenic --consequence missense --limit 5

# Step 3: Mutation-specific trial lookup
biomcp variant trials "EGFR C797S" --limit 5

# Step 4: Post-progression trial search (line-of-therapy filter)
biomcp search trial -c "non-small cell lung cancer" --progression-on osimertinib --line-of-therapy 2L --status recruiting --limit 5

# Step 5: Alternative target/mechanism options
biomcp search drug --target EGFR --mechanism inhibitor --limit 5
```

## Reference

| Source | Provides |
|--------|----------|
| Article search | Mechanism and progression evidence |
| Variant search | Candidate resistance alterations |
| Trial helpers/search | Next-line trial opportunities |
| Drug search | Alternative treatment classes |

## Validation Checklist

A complete run should produce:

- [ ] Resistance literature evidence returned
- [ ] Resistance variants identified
- [ ] Mutation-linked trial options listed
- [ ] Post-progression trial search executed
- [ ] Alternative mechanism shortlist produced

## Tips

- Pair gene + drug + `resistance` keyword for focused evidence retrieval.
- Use progression filters early when trial space is large.
- Compare resistance patterns across multiple articles before prioritization.
- Use `--results-available` to find completed trials with published outcome data.
