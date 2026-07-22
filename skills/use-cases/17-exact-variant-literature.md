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

For an assembly-aware one-item request, save `[{"request_id":"atm-grch38","gene":"ATM","transcript":"NM_000051.4","coding":"c.1066-6T>G","accession":"NC_000011.10","build":"GRCh38","position":108248927,"ref":"T","alt":"G"}]` as `variants.json`, then run `biomcp --json variant articles --input variants.json`.

Interpretation:
- Resolve the strict identity first; do not treat a search spelling as a normalized result.
- `caller_supplied` means BioMCP accepted the supplied fields as one caller assertion; it validated syntax but did not establish cross-coordinate equivalence.
- `provider_confirmed` means MyVariant uniquely confirmed the request; otherwise inspect `provider_validation` for `not_found`, `indeterminate`, `contradictory`, or `unavailable` and its nullable matched alias or contradictory field.
- RefSeq exact work uses only caller-present transcript/coding, gene/coding, and genomic aliases; BioMCP performs no liftover, accession-to-`chr` conversion, strand flip, or inferred coordinate expansion.
- Use the union shortlist, compare summaries in one batch, request full text/assets only for selected papers, and expand citations or references only when needed. `provenance.query_aliases` is a retrieval input, not evidence that a paper contains the alias; inspect `--debug-plan` for strict and discovery requests. For a conservative captured-evidence pass, run `biomcp --json variant articles "MSH2 p.L341P" --verify-identity --confirmed-only`. Verification occurs before ranking and pagination. `identity.observations` records provider, locator, linked gene, observed alias, and captured-content hash; an absent or incomplete observation is not confirmation. The debug artifact hashes are post-response audit facts, never retrieval-cache keys or query aliases, and verification does not fetch supplements automatically.
