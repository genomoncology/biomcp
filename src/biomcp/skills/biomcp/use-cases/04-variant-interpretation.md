# Use Case: Variant Interpretation

Interpret clinical significance of genetic variants for pathogenicity and actionability.

## Workflow

1. **Search variants in gene** - Find known pathogenic variants
2. **Get specific variant details** - Deep dive by rsID
3. **Search by HGVS notation** - Query using cDNA or protein notation
4. **Find related trials** - Trials for mutation carriers
5. **Search literature** - Functional studies, case reports

## Variant Annotation Sources

| Source        | Data Provided                        |
| ------------- | ------------------------------------ |
| ClinVar       | Clinical significance, review status |
| gnomAD        | Population allele frequencies        |
| COSMIC        | Somatic mutation frequency           |
| dbSNP         | rsID, genomic coordinates            |
| CADD          | Deleteriousness score                |
| PolyPhen/SIFT | Functional predictions               |

## Tips

- Use `--significance pathogenic` to focus on disease-causing variants
- Check population frequency to rule out common benign polymorphisms
- Cross-reference ClinVar review status for confidence level
