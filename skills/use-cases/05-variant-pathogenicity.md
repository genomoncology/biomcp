# Pattern: Variant pathogenicity evidence

Use this when the question asks whether a named variant is pathogenic, actionable, or clinically relevant in a disease context.

```bash
biomcp get variant "BRAF V600E" clinvar predictions population
biomcp get variant "BRAF V600E" civic cgi
biomcp variant trials "BRAF V600E" --limit 5
biomcp variant articles "BRAF V600E" --limit 5
```

Interpretation:
- Always request the `clinvar` section: the default headline `significance` is only the most severe RCV classification in MyVariant.info's cached copy. With `clinvar`, the headline and `clinvar.germline_classification` carry ClinVar's current record-level germline classification, review status, and evaluation date from NCBI.
- Read the record-level germline classification first; treat the per-condition RCV aggregates and SCV submissions as supporting detail, not the headline answer. When `significance_note` reports a disagreement between the cached copy and NCBI ClinVar, trust the NCBI ClinVar value and say why.
- Compare cancer knowledge bases separately from germline clinical assertions.
- Use trials and articles to anchor disease-specific relevance or therapy context.
- Say when evidence is variant-level but not disease-specific enough to answer the exact question.
