# Pattern: Literature Synthesis

Build a focused evidence summary by combining structured article search with variant/drug context checks.

> Inspired by PubMed & Beyond-style evidence workflows.

## Quick Check

```bash
biomcp search article -g BRAF -d melanoma --limit 5
biomcp get article 22663011 annotations
```

## Full Workflow

```bash
# Step 1: Core evidence search (sorted by impact)
biomcp search article -g BRAF -d melanoma --sort citations --limit 10

# Step 2: Treatment-focused refinement (review articles)
biomcp search article -g BRAF -d melanoma --type review --limit 5

# Step 3: Annotation extraction for key paper (exclude retracted)
biomcp search article -g BRAF -d melanoma --drug dabrafenib --since 2020-01-01 --exclude-retracted --limit 5
biomcp get article 22663011 annotations

# Step 4: Cross-check variant context
biomcp get variant "BRAF V600E" clinvar

# Step 5: Cross-check drug context
biomcp get drug dabrafenib targets
```

## Reference

| Source | Provides |
|--------|----------|
| Article search/get | Literature corpus and entity annotations |
| Variant entity | Clinical relevance anchor |
| Drug entity | Mechanism/target anchor |

## Validation Checklist

A complete run should produce:

- [ ] Representative article set for topic
- [ ] Refined treatment-specific evidence subset
- [ ] Annotation output for at least one article
- [ ] Variant context included in synthesis
- [ ] Drug mechanism context included in synthesis

## Tips

- Use `--since` to constrain noisy topics.
- Mix broad (`gene + disease`) and narrow (`gene + disease + drug`) searches.
- Capture PMIDs in the final summary for auditability.
- Use `biomcp article entities <PMID>` to extract structured entities from a key paper.
