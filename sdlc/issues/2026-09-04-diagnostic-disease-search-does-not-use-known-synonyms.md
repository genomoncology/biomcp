# Diagnostic disease search does not use disease synonyms that BioMCP already knows

BioMCP can resolve Bachmann-Bupp syndrome to `MONDO:0033642`. The disease record includes “Bachmann-Bupp syndrome,” “global developmental delay-alopecia-macrocephaly-facial dysmorphism-structural brain anomalies syndrome,” and “NEDABA” as synonyms. The same card suggests this command:

```bash
biomcp search diagnostic --disease "Bachmann-Bupp syndrome"
```

The suggested command returned zero results on `biomcp 0.9.0-dev.6` on 2026-09-04. A gene search found `GTR000596648.2`, named “Bachmann-Bupp syndrome: Full gene sequencing.” A search with the longer GTR condition name also found it:

```bash
biomcp search diagnostic --gene ODC1 --source gtr --limit 5
biomcp search diagnostic --disease "Neurodevelopmental disorder with alopecia and brain abnormalities" --source gtr --limit 5
```

`src/entities/diagnostic/search.rs::NormalizedSearchFilters::matches_gtr` compares the caller's phrase only with the condition strings in the local GTR index. It does not resolve the disease or try its known synonyms. The GTR files contain the test, and the MyDisease-backed disease record contains the needed synonym set. The underlying sources support the correction.

## Recommended design

Resolve a disease query once before local diagnostic filtering. Match the caller's phrase, the canonical disease name, and exact synonym phrases against GTR conditions. Preserve the matched term in structured output. If disease resolution fails, run the current literal search and report that no synonym expansion occurred.

This design costs one disease-resolution request when the local literal search cannot answer the query. Exact phrase matching limits false positives. Broad parent terms must not enter the alias set.

## Done, observably

- The Bachmann-Bupp command finds `GTR000596648.2` and reports which synonym matched.
- A literal GTR condition query keeps its current result.
- A disease resolver failure does not turn into a false confirmed zero.
- Tests cover a preferred name, a synonym, a literal condition, and an unrelated disease.

Ticket 1093 covers a disease card that loses its display name. This issue covers the separate diagnostic-filter path.
