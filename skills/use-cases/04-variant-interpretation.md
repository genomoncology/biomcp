# Use Case: Variant Interpretation

Interpret the clinical significance of genetic variants.

## Scenario

A patient has a BRCA1 variant detected on germline testing. Determine pathogenicity and clinical actionability.

## Workflow

### Step 1: Search Pathogenic Variants in Gene

```bash
biomcp variant search --gene BRCA1 --significance pathogenic --size 10
```

Returns known pathogenic variants with:
- ClinVar classification
- Population frequencies
- Functional predictions

### Step 2: Get Specific Variant Details

```bash
biomcp variant get rs80357906
```

Deep dive on a specific variant by rsID.

### Step 3: Search by HGVS Notation

```bash
biomcp variant search --gene BRCA1 --hgvsc "c.5266dupC"
```

Search using cDNA notation.

### Step 4: Find Related Trials

```bash
biomcp trial search --condition "breast cancer" --term "BRCA" --status open
```

Find trials for BRCA mutation carriers.

### Step 5: Search Literature

```bash
biomcp article search --gene BRCA1 --keyword "pathogenic variant" --page 1
```

## Variant Annotation Sources

| Source | Data Provided |
|--------|---------------|
| ClinVar | Clinical significance, review status |
| gnomAD | Population allele frequencies |
| COSMIC | Somatic mutation frequency |
| dbSNP | rsID, genomic coordinates |
| CADD | Deleteriousness score |
| PolyPhen/SIFT | Functional predictions |

## Expected Output

- Pathogenicity: Pathogenic (ClinVar, multiple submitters)
- Population frequency: Rare (<0.01%)
- Functional impact: Loss of function
- Clinical actionability: PARP inhibitor eligibility, enhanced surveillance

## Search Parameters

| Parameter | Example | Description |
|-----------|---------|-------------|
| `--gene` | BRCA1 | Gene symbol |
| `--rsid` | rs80357906 | dbSNP ID |
| `--hgvsp` | p.Cys61Gly | Protein notation |
| `--hgvsc` | c.181T>G | cDNA notation |
| `--significance` | pathogenic | ClinVar classification |
| `--max-frequency` | 0.01 | gnomAD filter |

## Tips

- Use `--significance pathogenic` to focus on disease-causing variants
- Check population frequency to rule out common benign polymorphisms
- Cross-reference with ClinVar review status for confidence
- Link to trial search for actionability assessment
