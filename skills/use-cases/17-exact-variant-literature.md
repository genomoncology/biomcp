# Pattern: Exact-variant literature retrieval

Use this when you have a complete gene plus protein change and need a compact, evidence-first literature shortlist without making a clinical classification.

```bash
biomcp search variant MSH2 p.L341P --limit 5
biomcp variant articles "MSH2 p.L341P" --limit 5
biomcp article batch 26951660 31433521
biomcp get article 26951660 fulltext
biomcp --json get article 26951660 assets
biomcp article citations 26951660 --limit 5
biomcp article references 26951660 --limit 5
```

Interpretation:
- Resolve the strict identity first; do not treat a search spelling as a normalized result.
- Use the default `variant articles` union route as the compact shortlist, then compare candidate summaries in one batch.
- Request full text and linked assets only for selected papers.
- Expand citations or references only when the primary papers still lack the needed evidence.
