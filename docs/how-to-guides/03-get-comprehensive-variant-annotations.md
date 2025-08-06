# How to Get Comprehensive Variant Annotations

This guide demonstrates how to retrieve and interpret genetic variant information using BioMCP's integrated databases.

## Overview

BioMCP provides variant annotations from multiple sources:

- **MyVariant.info**: Core variant database with clinical significance ([BioThings Reference](../backend-services-reference/02-biothings-suite.md))
- **External Annotations**: TCGA cancer data, 1000 Genomes population frequencies
- **cBioPortal Integration**: Cancer-specific mutation context ([API Reference](../backend-services-reference/03-cbioportal.md))
- **BioThings Links**: Connected gene, disease, and drug information ([BioThings Suite](../backend-services-reference/02-biothings-suite.md))

## Basic Variant Lookup

### Search by rsID

Find variant information using dbSNP identifiers:

```bash
# CLI
biomcp variant get rs121913529

# Python
variant = await client.variants.get("rs121913529")

# MCP Tool
variant_getter(variant_id="rs121913529")
```

### Search by HGVS Notation

Use standard HGVS notation:

```python
# Protein change
variant = await variant_getter("NP_004324.2:p.Val600Glu")

# Coding DNA change
variant = await variant_getter("NM_004333.4:c.1799T>A")

# Genomic coordinates
variant = await variant_getter("NC_000007.13:g.140453136A>T")
```

### Search by Genomic Position

```python
# Search by coordinates
variants = await variant_searcher(
    chromosome="7",
    start=140453136,
    end=140453136,
    assembly="hg38"  # or hg19
)
```

## Understanding Variant Annotations

### Clinical Significance

```python
# Get variant details
variant = await variant_getter("rs121913529")

# Check clinical significance
print(f"Clinical Significance: {variant.clinical_significance}")
# Output: "Pathogenic"

print(f"ClinVar Review Status: {variant.review_status}")
# Output: "reviewed by expert panel"
```

### Population Frequencies

```python
# Access frequency data
if variant.frequencies:
    print("Population Frequencies:")
    print(f"  gnomAD: {variant.frequencies.gnomad}")
    print(f"  1000 Genomes: {variant.frequencies.thousand_genomes}")
    print(f"  ExAC: {variant.frequencies.exac}")
```

### Functional Predictions

```python
# In silico predictions
if variant.predictions:
    print(f"CADD Score: {variant.predictions.cadd}")
    print(f"PolyPhen: {variant.predictions.polyphen}")
    print(f"SIFT: {variant.predictions.sift}")
```

## Advanced Variant Searches

### Filter by Clinical Significance

```python
# Find pathogenic BRCA1 variants
pathogenic_variants = await variant_searcher(
    gene="BRCA1",
    significance="pathogenic",
    limit=20
)

# Multiple significance levels
variants = await variant_searcher(
    gene="TP53",
    significance=["pathogenic", "likely_pathogenic"]
)
```

### Filter by Frequency

Find rare variants:

```python
# Rare variants (MAF < 1%)
rare_variants = await variant_searcher(
    gene="CFTR",
    frequency_max=0.01,
    significance="pathogenic"
)

# Ultra-rare variants
ultra_rare = await variant_searcher(
    gene="SCN1A",
    frequency_max=0.0001
)
```

### Filter by Prediction Scores

```python
# High-impact variants
high_impact = await variant_searcher(
    gene="MLH1",
    cadd_score_min=20,  # CADD > 20 suggests deleteriousness
    polyphen_prediction="probably_damaging"
)
```

## External Database Integration

For technical details on external data sources, see the [BioThings Suite Reference](../backend-services-reference/02-biothings-suite.md).

### TCGA Cancer Data

Variants automatically include TCGA annotations when available:

```python
variant = await variant_getter("rs121913529", include_external=True)

# Check TCGA data
if variant.external_data.get("tcga"):
    tcga = variant.external_data["tcga"]
    print(f"TCGA Studies: {tcga['study_count']}")
    print(f"Cancer Types: {', '.join(tcga['cancer_types'])}")
    print(f"Sample Count: {tcga['sample_count']}")
```

### 1000 Genomes Project

Population-specific frequencies:

```python
# Access 1000 Genomes data
if variant.external_data.get("thousand_genomes"):
    tg_data = variant.external_data["thousand_genomes"]
    print("Population Frequencies:")
    for pop, freq in tg_data["populations"].items():
        print(f"  {pop}: {freq}")
```

### Ensembl VEP Annotations

```python
# Consequence predictions
if variant.consequences:
    for consequence in variant.consequences:
        print(f"Gene: {consequence.gene}")
        print(f"Impact: {consequence.impact}")
        print(f"Consequence: {consequence.consequence_terms}")
```

## Integration with Other BioMCP Tools

BioMCP's unified architecture allows seamless integration between variant data and other biomedical information. For implementation details, see the [Transport Protocol Guide](../developer-guides/04-transport-protocol.md).

### Variant to Gene Information

```python
# Get variant
variant = await variant_getter("rs121913529")

# Get associated gene details
gene_symbol = variant.gene.symbol  # "BRAF"
gene_info = await gene_getter(gene_symbol)

print(f"Gene: {gene_info.name}")
print(f"Function: {gene_info.summary}")
```

### Variant to Disease Context

```python
# Find disease associations
diseases = variant.disease_associations

for disease in diseases:
    # Get detailed disease info
    disease_info = await disease_getter(disease.name)
    print(f"Disease: {disease_info.name}")
    print(f"Definition: {disease_info.definition}")
    print(f"Synonyms: {', '.join(disease_info.synonyms)}")
```

### Variant to Clinical Trials

```python
# Search trials for specific variant
gene = variant.gene.symbol
mutation = variant.protein_change  # e.g., "V600E"

trials = await trial_searcher(
    other_terms=[f"{gene} {mutation}", f"{gene} mutation"],
    recruiting_status="OPEN"
)
```

## Practical Workflows

### Workflow 1: Cancer Variant Analysis

```python
async def analyze_cancer_variant(hgvs: str):
    # Think about the analysis
    await think(
        thought=f"Analyzing cancer variant {hgvs}",
        thoughtNumber=1
    )

    # Get variant details
    variant = await variant_getter(hgvs, include_external=True)

    # Get gene context
    gene = await gene_getter(variant.gene.symbol)

    # Search for targeted therapies
    drugs = await search(
        query=f"drugs.targets:{variant.gene.symbol}",
        domain="drug"
    )

    # Find relevant trials
    trials = await trial_searcher(
        other_terms=[
            variant.gene.symbol,
            variant.protein_change,
            "targeted therapy"
        ],
        recruiting_status="OPEN"
    )

    # Search literature
    articles = await article_searcher(
        genes=[variant.gene.symbol],
        variants=[hgvs],
        keywords=["therapy", "treatment", "resistance"]
    )

    return {
        "variant": variant,
        "gene": gene,
        "potential_drugs": drugs,
        "clinical_trials": trials,
        "literature": articles
    }
```

### Workflow 2: Rare Disease Variant

```python
async def rare_disease_variant_analysis(gene: str, phenotype: str):
    # Find all pathogenic variants
    variants = await variant_searcher(
        gene=gene,
        significance=["pathogenic", "likely_pathogenic"],
        frequency_max=0.001  # Rare
    )

    # Analyze each variant
    results = []
    for v in variants[:10]:  # Top 10
        # Get full annotations
        full_variant = await variant_getter(v.id)

        # Check phenotype associations
        if phenotype.lower() in str(full_variant.phenotypes).lower():
            results.append({
                "variant": full_variant,
                "phenotype_match": True,
                "frequency": full_variant.frequencies.gnomad or 0
            })

    # Sort by relevance
    results.sort(key=lambda x: x["frequency"])
    return results
```

### Workflow 3: Pharmacogenomics

```python
async def pharmacogenomic_analysis(drug_name: str):
    # Get drug information
    drug = await drug_getter(drug_name)

    # Find pharmGKB annotations
    pgx_variants = []

    # Search for drug-related variants
    if drug.targets:
        for target in drug.targets:
            variants = await variant_searcher(
                gene=target,
                keywords=[drug_name, "pharmacogenomics", "drug response"]
            )
            pgx_variants.extend(variants)

    # Get detailed annotations
    annotated = []
    for v in pgx_variants:
        full = await variant_getter(v.id)
        if full.pharmacogenomics:
            annotated.append(full)

    return {
        "drug": drug,
        "pgx_variants": annotated,
        "affected_genes": list(set(v.gene.symbol for v in annotated))
    }
```

## Interpreting Results

### Clinical Actionability

```python
def assess_actionability(variant):
    """Determine if variant is clinically actionable"""

    actionable = False
    reasons = []

    # Check pathogenicity
    if variant.clinical_significance in ["pathogenic", "likely_pathogenic"]:
        actionable = True
        reasons.append("Pathogenic variant")

    # Check for drug associations
    if variant.drug_associations:
        actionable = True
        reasons.append(f"Associated with {len(variant.drug_associations)} drugs")

    # Check guidelines
    if variant.clinical_guidelines:
        actionable = True
        reasons.append("Clinical guidelines available")

    return {
        "actionable": actionable,
        "reasons": reasons,
        "recommendations": variant.clinical_guidelines
    }
```

### Report Generation

```python
def generate_variant_report(variant):
    """Create a clinical variant report"""

    report = f"""
## Variant Report: {variant.id}

### Basic Information
- **Gene**: {variant.gene.symbol}
- **Protein Change**: {variant.protein_change or "N/A"}
- **Genomic Location**: chr{variant.chr}:{variant.pos}
- **Reference**: {variant.ref} → **Alternate**: {variant.alt}

### Clinical Significance
- **Status**: {variant.clinical_significance}
- **Review**: {variant.review_status}
- **Last Updated**: {variant.last_updated}

### Population Frequency
- **gnomAD**: {variant.frequencies.gnomad or "Not found"}
- **1000 Genomes**: {variant.frequencies.thousand_genomes or "Not found"}

### Predictions
- **CADD Score**: {variant.predictions.cadd or "N/A"}
- **PolyPhen**: {variant.predictions.polyphen or "N/A"}
- **SIFT**: {variant.predictions.sift or "N/A"}

### Associated Conditions
{format_conditions(variant.conditions)}

### Clinical Resources
- **ClinVar**: {variant.clinvar_url}
- **dbSNP**: {variant.dbsnp_url}
"""
    return report
```

## Best Practices

### 1. Use Multiple Identifiers

```python
# Try multiple formats if one fails
identifiers = [
    "rs121913529",
    "NM_004333.4:c.1799T>A",
    "7:140453136:A:T"
]

for id in identifiers:
    try:
        variant = await variant_getter(id)
        break
    except:
        continue
```

### 2. Check Data Completeness

```python
# Not all variants have all annotations
if variant.frequencies:
    # Use frequency data
    pass
else:
    # Note that frequency unavailable
    pass
```

### 3. Consider Assembly Versions

```python
# Specify genome assembly
variants_hg38 = await variant_searcher(
    chromosome="7",
    start=140453136,
    assembly="hg38"
)

variants_hg19 = await variant_searcher(
    chromosome="7",
    start=140153336,  # Different coordinate!
    assembly="hg19"
)
```

## Troubleshooting

### Variant Not Found

1. **Check notation**: Ensure proper HGVS format
2. **Try alternatives**: rsID, genomic coordinates, protein change
3. **Verify gene symbol**: Use official HGNC symbols

### Missing Annotations

- Not all variants have all data types
- Rare variants may lack population frequencies
- Novel variants won't have ClinVar data

### Performance Issues

- Use pagination for large searches
- Limit external data requests when not needed
- Cache frequently accessed variants

## Next Steps

- Learn to [predict variant effects](04-predict-variant-effects-with-alphagenome.md)
- Explore [article searches](01-find-articles-and-cbioportal-data.md) for variant literature
- Set up [logging and monitoring](05-logging-and-monitoring-with-bigquery.md)
