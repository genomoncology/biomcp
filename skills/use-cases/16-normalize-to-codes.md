# Pattern: Normalize a term to ontology codes

Use this when a free-text clinical or guideline term needs source-labelled MONDO, HPO, ICD-10, SNOMED, or RxNorm-style codes before downstream structuring.

```bash
biomcp discover "type 2 diabetes mellitus"
biomcp --json discover "type 2 diabetes mellitus"
biomcp discover "metformin"
```

Interpretation:
- Use the markdown view first to confirm the resolved concept and suggested next commands.
- Use JSON when you need exact identifiers; preserve source labels such as MONDO, SNOMEDCT, ICD10CM, HPO, and RxNorm.
- Do not invent absent code classes. If HPO or RxNorm is not returned for a disease term, report that the class was not present in discover output.
- Drug terms may return RxNorm-style labels while disease terms often return MONDO plus SNOMED or ICD-10 crosswalks.
