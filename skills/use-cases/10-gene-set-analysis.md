# Pattern: Gene Set Analysis

Analyze a gene set for pathway-level signals, interaction context, and literature-backed interpretation.

> Inspired by GeneAgent-style generate-verify workflows.

## Quick Check

```bash
biomcp enrich "BRCA1,TP53,ATM,CHEK2,PALB2" --limit 5
biomcp get gene BRCA1 pathways
```

## Full Workflow

```bash
# Step 1: Enrichment overview
biomcp enrich "BRCA1,TP53,ATM,CHEK2,PALB2" --limit 10

# Step 2: Batch comparison and pathway verification
biomcp batch gene "BRCA1,ATM,PALB2"
biomcp get gene BRCA1 pathways
biomcp get gene ATM pathways

# Step 3: Verify interaction context
biomcp get gene BRCA1 interactions
biomcp get gene TP53 interactions

# Step 4: Add literature support
biomcp search article -g BRCA1 -k "DNA repair" --limit 5

# Step 5: Disease anchoring and therapeutic context
biomcp get disease "breast cancer" genes
biomcp disease drugs "breast cancer" --limit 5
```

## Reference

| Source | Provides |
|--------|----------|
| `enrich` (g:Profiler) | Enrichment terms and significance |
| Gene pathways/interactions | Mechanistic verification |
| Article search | Evidence support |
| `batch` | Parallel gene summaries |
| Disease genes | Disease-level anchoring |

## Validation Checklist

A complete run should produce:

- [ ] Enrichment terms returned for the gene set
- [ ] Pathway verification for representative genes
- [ ] Interaction evidence for at least one claim
- [ ] Supporting literature captured
- [ ] Disease-level relevance summarized

## Tips

- If enrichment is temporarily unavailable, continue with pathway + interaction + article verification steps.
- Validate top claims with at least two independent command results.
- Keep the final summary claim-by-claim with command evidence.
