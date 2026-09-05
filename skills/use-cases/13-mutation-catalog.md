# Pattern: Mutation catalog for one gene and disease

Use this when the question asks for mutations in one gene that are linked to a named disease or phenotype.

```bash
biomcp get gene PLN
biomcp search variant -g PLN --limit 10
biomcp search variant -g PLN --hgvsp L39X --limit 5
biomcp search article -g PLN -d cardiomyopathy --type review --limit 10
```

Interpretation:
- Start with the gene card so the symbol and disease context are grounded.
- Use broad variant search to find candidate changes before asking about one protein change.
- In an exact result, treat `transcript_annotations` as paired SnpEff explanations. A `matched` tuple can differ from the `displayed` tuple because the independent dbNSFP identity facts retained the row; neither role selects a clinically preferred transcript or proves equivalence.
- Use disease-anchored reviews to separate causal mutations from unrelated polymorphisms.
- Avoid presenting the result as exhaustive unless the evidence source itself is comprehensive.
