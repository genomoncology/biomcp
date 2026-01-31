# Pattern: Variant to Treatment

From detected variant to actionable treatment options.

## Workflow

1. **Get variant details** - `biomcp variant get <rsID>` for ClinVar, gnomAD annotations
2. **Search gene context** - `biomcp gene get <symbol>` for pathways, function
3. **Enrich pathways** - `biomcp gene get <symbol> --enrich pathway` for KEGG/Reactome
4. **Predict effects** - `biomcp variant predict` for functional impact (AlphaGenome)
5. **Find matching trials** - `biomcp trial search --biomarker <gene>` for mutation-specific trials
6. **Search literature** - `biomcp article search -g <gene> -v <variant>` for evidence

## Key Annotations

| Source  | Provides                                      |
| ------- | --------------------------------------------- |
| ClinVar | Clinical significance, review status          |
| gnomAD  | Population frequency (common = likely benign) |
| CADD    | Deleteriousness score                         |

## Tips

- Use `--significance pathogenic` to filter for disease-causing variants
- Check population frequency: >1% usually benign polymorphism
- Combine gene + disease + variant for specific article results
- Cross-reference variant pathogenicity with trial eligibility
