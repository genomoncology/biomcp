# Pattern: Clinical Trial Searching and Patient Matching

Search trials by condition and molecular filters, then perform criterion-level matching for a specific patient scenario.

## Quick Check

```bash
biomcp search trial -c melanoma --status recruiting --limit 5
biomcp get trial NCT02576665 eligibility
```

## Full Workflow

```bash
# Step 1: Condition-level discovery
biomcp search trial -c melanoma --limit 5

# Step 2: Add molecular filter
biomcp search trial -c melanoma --mutation "BRAF V600E" --status recruiting --limit 5

# Step 3: Add facility or geography constraints
biomcp search trial -c melanoma --facility "Dana-Farber" --status recruiting --limit 5

# Step 4: Retrieve eligibility text for candidate trial
biomcp get trial NCT02576665 eligibility

# Step 5: Check intervention context
biomcp get drug dabrafenib targets
```

## Patient Matching Workflow

> Inspired by TrialGPT-style criterion-level reasoning.

```bash
# Step 1: Characterize patient mutation context
biomcp get variant "BRAF V600E" clinvar

# Step 2: Candidate trial shortlist (with demographic filters)
biomcp search trial -c melanoma --mutation "BRAF V600E" --age 55 --sex female --status recruiting --limit 5

# Step 3: Eligibility review for top trial
biomcp get trial NCT02576665 eligibility

# Step 4: Supporting treatment-evidence context
biomcp search article -g BRAF -d melanoma --drug dabrafenib --limit 5
```

## Reference

| Source | Provides |
|--------|----------|
| ClinicalTrials.gov / NCI CTS | Trial records and filters |
| Trial eligibility section | Inclusion/exclusion criteria |
| Variant and drug entities | Molecular and intervention context |

## Validation Checklist

A complete run should produce:

- [ ] Trial IDs found using condition and status filters
- [ ] Mutation filter affects shortlist composition
- [ ] Eligibility text reviewed for at least one trial
- [ ] Trial phase/intervention captured in summary
- [ ] Patient-matching rationale references explicit criteria

## Tips

- Use `--source nci` for biomarker-heavy searches.
- If mutation filtering is sparse, broaden to gene-level terms and refine.
- Keep a short eligibility evidence table for comparison across top candidates.
- Use `--count-only` for landscape sizing before drilling into individual trials.
