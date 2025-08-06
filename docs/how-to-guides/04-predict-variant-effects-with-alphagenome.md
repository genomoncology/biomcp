# How to Predict Variant Effects with AlphaGenome

This guide demonstrates how to use Google DeepMind's AlphaGenome to predict regulatory effects of genetic variants on gene expression, chromatin accessibility, and splicing.

## Overview

AlphaGenome predicts how DNA variants affect:

- **Gene Expression**: Log-fold changes in nearby genes
- **Chromatin Accessibility**: ATAC-seq/DNase-seq signal changes
- **Splicing**: Effects on splice sites and exon inclusion
- **Regulatory Elements**: Impact on enhancers, promoters, and TFBS
- **3D Chromatin**: Changes in chromatin interactions

For technical details on the AlphaGenome integration, see the [AlphaGenome API Reference](../backend-services-reference/07-alphagenome.md).

## Setup and API Key

### Get Your API Key

1. Visit [AlphaGenome Portal](https://deepmind.google.com/science/alphagenome)
2. Register for non-commercial use
3. Receive API key via email

For detailed setup instructions, see [Authentication and API Keys](../getting-started/03-authentication-and-api-keys.md#alphagenome).

### Configure API Key

**Option 1: Environment Variable (Personal Use)**

```bash
export ALPHAGENOME_API_KEY="your-key-here"
```

**Option 2: Per-Request (AI Assistants)**

```
"Predict effects of BRAF V600E. My AlphaGenome API key is YOUR_KEY_HERE"
```

**Option 3: Configuration File**

```python
# .env file
ALPHAGENOME_API_KEY=your-key-here
```

### Install AlphaGenome (Optional)

For local predictions:

```bash
git clone https://github.com/google-deepmind/alphagenome.git
cd alphagenome && pip install .
```

## Basic Variant Prediction

### Simple Prediction

Predict effects of BRAF V600E mutation:

```bash
# CLI
biomcp variant predict chr7 140753336 A T

# Python
result = await client.variants.predict(
    chromosome="chr7",
    position=140753336,
    reference="A",
    alternate="T"
)

# MCP Tool
result = await alphagenome_predictor(
    chromosome="chr7",
    position=140753336,
    reference="A",
    alternate="T"
)
```

### Understanding Results

```python
# Gene expression changes
for gene in result.gene_expression:
    print(f"{gene.name}: {gene.log2_fold_change}")
    # Positive = increased expression
    # Negative = decreased expression
    # |value| > 1.0 = strong effect

# Chromatin accessibility
for region in result.chromatin:
    print(f"{region.type}: {region.change}")
    # Positive = more open chromatin
    # Negative = more closed chromatin

# Splicing effects
for splice in result.splicing:
    print(f"{splice.event}: {splice.delta_psi}")
    # PSI = Percent Spliced In
    # Positive = increased inclusion
```

## Tissue-Specific Predictions

### Single Tissue Analysis

Predict effects in specific tissues using UBERON terms:

```python
# Breast tissue analysis
result = await alphagenome_predictor(
    chromosome="chr17",
    position=41246481,
    reference="G",
    alternate="A",
    tissue_types=["UBERON:0000310"]  # breast
)

# Common tissue codes:
# UBERON:0000310 - breast
# UBERON:0002107 - liver
# UBERON:0002367 - prostate
# UBERON:0000955 - brain
# UBERON:0002048 - lung
# UBERON:0001155 - colon
```

### Multi-Tissue Comparison

Compare effects across tissues:

```python
tissues = [
    "UBERON:0000310",  # breast
    "UBERON:0002107",  # liver
    "UBERON:0002048"   # lung
]

results = {}
for tissue in tissues:
    results[tissue] = await alphagenome_predictor(
        chromosome="chr17",
        position=41246481,
        reference="G",
        alternate="A",
        tissue_types=[tissue]
    )

# Compare gene expression across tissues
for tissue, result in results.items():
    print(f"\n{tissue}:")
    for gene in result.gene_expression[:3]:
        print(f"  {gene.name}: {gene.log2_fold_change}")
```

## Analysis Window Sizes

### Choosing the Right Interval

Different interval sizes capture different regulatory effects:

```python
# Short-range (promoter effects)
result_2kb = await alphagenome_predictor(
    chromosome="chr7",
    position=140753336,
    reference="A",
    alternate="T",
    interval_size=2048  # 2kb
)

# Medium-range (enhancer-promoter)
result_128kb = await alphagenome_predictor(
    chromosome="chr7",
    position=140753336,
    reference="A",
    alternate="T",
    interval_size=131072  # 128kb (default)
)

# Long-range (TAD-level effects)
result_1mb = await alphagenome_predictor(
    chromosome="chr7",
    position=140753336,
    reference="A",
    alternate="T",
    interval_size=1048576  # 1Mb
)
```

**Interval Size Guide:**

- **2kb**: Promoter variants, TSS mutations
- **16kb**: Local regulatory elements
- **128kb**: Enhancer-promoter interactions (default)
- **512kb**: Long-range regulatory
- **1Mb**: TAD boundaries, super-enhancers

## Clinical Workflows

### Workflow 1: VUS (Variant of Unknown Significance) Analysis

```python
async def analyze_vus(chromosome: str, position: int, ref: str, alt: str):
    # Step 1: Think about the analysis
    await think(
        thought=f"Analyzing VUS at {chromosome}:{position} {ref}>{alt}",
        thoughtNumber=1
    )

    # Step 2: Get variant annotations
    variant_id = f"{chromosome}:g.{position}{ref}>{alt}"
    try:
        known_variant = await variant_getter(variant_id)
        if known_variant.clinical_significance:
            return f"Already classified: {known_variant.clinical_significance}"
    except:
        pass  # Variant not in databases

    # Step 3: Predict regulatory effects
    prediction = await alphagenome_predictor(
        chromosome=chromosome,
        position=position,
        reference=ref,
        alternate=alt,
        interval_size=131072
    )

    # Step 4: Analyze impact
    high_impact_genes = [
        g for g in prediction.gene_expression
        if abs(g.log2_fold_change) > 1.0
    ]

    # Step 5: Search literature
    if high_impact_genes:
        gene_symbols = [g.name for g in high_impact_genes[:3]]
        articles = await article_searcher(
            genes=gene_symbols,
            keywords=["pathogenic", "disease", "mutation"]
        )

    return {
        "variant": f"{chromosome}:{position} {ref}>{alt}",
        "high_impact_genes": high_impact_genes,
        "regulatory_assessment": assess_regulatory_impact(prediction),
        "literature_support": len(articles) if high_impact_genes else 0
    }

def assess_regulatory_impact(prediction):
    """Classify regulatory impact severity"""
    max_expression_change = max(
        abs(g.log2_fold_change) for g in prediction.gene_expression
    ) if prediction.gene_expression else 0

    if max_expression_change > 2.0:
        return "HIGH - Strong regulatory effect"
    elif max_expression_change > 1.0:
        return "MODERATE - Significant regulatory effect"
    elif max_expression_change > 0.5:
        return "LOW - Mild regulatory effect"
    else:
        return "MINIMAL - No significant regulatory effect"
```

### Workflow 2: Non-coding Variant Prioritization

```python
async def prioritize_noncoding_variants(variants: list[dict], disease_genes: list[str]):
    """Rank non-coding variants by predicted impact on disease genes"""

    results = []

    for variant in variants:
        # Predict effects
        prediction = await alphagenome_predictor(
            chromosome=variant["chr"],
            position=variant["pos"],
            reference=variant["ref"],
            alternate=variant["alt"]
        )

        # Check impact on disease genes
        disease_impact = {}
        for gene in prediction.gene_expression:
            if gene.name in disease_genes:
                disease_impact[gene.name] = gene.log2_fold_change

        # Calculate priority score
        if disease_impact:
            max_impact = max(abs(v) for v in disease_impact.values())
            results.append({
                "variant": variant,
                "disease_genes_affected": disease_impact,
                "priority_score": max_impact,
                "chromatin_changes": len([c for c in prediction.chromatin if c.change > 0.5])
            })

    # Sort by priority
    results.sort(key=lambda x: x["priority_score"], reverse=True)
    return results

# Example usage
variants_to_test = [
    {"chr": "chr17", "pos": 41246000, "ref": "A", "alt": "G"},
    {"chr": "chr17", "pos": 41246500, "ref": "C", "alt": "T"},
    {"chr": "chr17", "pos": 41247000, "ref": "G", "alt": "A"}
]

breast_cancer_genes = ["BRCA1", "BRCA2", "TP53", "PTEN"]
prioritized = await prioritize_noncoding_variants(variants_to_test, breast_cancer_genes)
```

### Workflow 3: Splicing Analysis

```python
async def analyze_splicing_variant(gene: str, exon: int, variant_pos: int, ref: str, alt: str):
    """Analyze potential splicing effects of a variant"""

    # Get gene information
    gene_info = await gene_getter(gene)
    chromosome = f"chr{gene_info.genomic_location.chr}"

    # Predict splicing effects
    prediction = await alphagenome_predictor(
        chromosome=chromosome,
        position=variant_pos,
        reference=ref,
        alternate=alt,
        interval_size=16384  # Smaller window for splicing
    )

    # Analyze splicing predictions
    splicing_effects = []
    for event in prediction.splicing:
        if abs(event.delta_psi) > 0.1:  # 10% change in splicing
            splicing_effects.append({
                "type": event.event_type,
                "change": event.delta_psi,
                "affected_exon": event.exon,
                "interpretation": interpret_splicing(event)
            })

    # Search for similar splicing variants
    articles = await article_searcher(
        genes=[gene],
        keywords=[f"exon {exon}", "splicing", "splice site"]
    )

    return {
        "variant": f"{gene} exon {exon} {ref}>{alt}",
        "splicing_effects": splicing_effects,
        "likely_consequence": predict_consequence(splicing_effects),
        "literature_precedent": len(articles)
    }

def interpret_splicing(event):
    """Interpret splicing changes"""
    if event.delta_psi > 0.5:
        return "Strong increase in exon inclusion"
    elif event.delta_psi > 0.1:
        return "Moderate increase in exon inclusion"
    elif event.delta_psi < -0.5:
        return "Strong exon skipping"
    elif event.delta_psi < -0.1:
        return "Moderate exon skipping"
    else:
        return "Minimal splicing change"
```

## Research Applications

### Enhancer Variant Analysis

```python
async def analyze_enhancer_variant(chr: str, pos: int, ref: str, alt: str, target_gene: str):
    """Analyze variant in potential enhancer region"""

    # Use larger window to capture enhancer-promoter interactions
    prediction = await alphagenome_predictor(
        chromosome=chr,
        position=pos,
        reference=ref,
        alternate=alt,
        interval_size=524288  # 512kb
    )

    # Find target gene effect
    target_effect = None
    for gene in prediction.gene_expression:
        if gene.name == target_gene:
            target_effect = gene.log2_fold_change
            break

    # Analyze chromatin changes
    chromatin_opening = sum(
        1 for c in prediction.chromatin
        if c.change > 0 and c.type == "enhancer"
    )

    return {
        "variant_location": f"{chr}:{pos}",
        "target_gene": target_gene,
        "expression_change": target_effect,
        "enhancer_activity": "increased" if chromatin_opening > 0 else "decreased",
        "likely_enhancer": abs(target_effect or 0) > 0.5 and chromatin_opening > 0
    }
```

### Pharmacogenomic Predictions

```python
async def predict_drug_response_variant(drug_target: str, variant: dict):
    """Predict how variant affects drug target expression"""

    # Get drug information
    drug_info = await drug_getter(drug_target)
    target_genes = drug_info.targets

    # Predict variant effects
    prediction = await alphagenome_predictor(
        chromosome=variant["chr"],
        position=variant["pos"],
        reference=variant["ref"],
        alternate=variant["alt"],
        tissue_types=["UBERON:0002107"]  # liver for drug metabolism
    )

    # Check effects on drug targets
    target_effects = {}
    for gene in prediction.gene_expression:
        if gene.name in target_genes:
            target_effects[gene.name] = gene.log2_fold_change

    # Interpret results
    if any(abs(effect) > 1.0 for effect in target_effects.values()):
        response = "Likely altered drug response"
    elif any(abs(effect) > 0.5 for effect in target_effects.values()):
        response = "Possible altered drug response"
    else:
        response = "Unlikely to affect drug response"

    return {
        "drug": drug_target,
        "variant": variant,
        "target_effects": target_effects,
        "prediction": response,
        "recommendation": "Consider dose adjustment" if "altered" in response else "Standard dosing"
    }
```

## Best Practices

### 1. Validate Input Coordinates

```python
# Always use "chr" prefix
chromosome = "chr7"  # ✅ Correct
# chromosome = "7"   # ❌ Wrong

# Use 1-based positions (not 0-based)
position = 140753336  # ✅ 1-based
```

### 2. Handle API Errors Gracefully

```python
try:
    result = await alphagenome_predictor(...)
except Exception as e:
    if "API key" in str(e):
        print("Please provide AlphaGenome API key")
    elif "Invalid sequence" in str(e):
        print("Check chromosome and position")
    else:
        print(f"Prediction failed: {e}")
```

### 3. Combine with Other Tools

```python
# Complete variant analysis pipeline
async def comprehensive_variant_analysis(variant_id: str):
    # 1. Get known annotations
    known = await variant_getter(variant_id)

    # 2. Predict regulatory effects
    prediction = await alphagenome_predictor(
        chromosome=f"chr{known.chr}",
        position=known.pos,
        reference=known.ref,
        alternate=known.alt
    )

    # 3. Search literature
    articles = await article_searcher(
        variants=[variant_id],
        genes=[known.gene.symbol]
    )

    # 4. Find relevant trials
    trials = await trial_searcher(
        other_terms=[known.gene.symbol, "mutation"]
    )

    return {
        "annotations": known,
        "predictions": prediction,
        "literature": articles,
        "trials": trials
    }
```

### 4. Interpret Results Appropriately

```python
def interpret_expression_change(log2_fc):
    """Convert log2 fold change to interpretation"""
    if log2_fc > 2.0:
        return "Very strong increase (>4x)"
    elif log2_fc > 1.0:
        return "Strong increase (2-4x)"
    elif log2_fc > 0.5:
        return "Moderate increase (1.4-2x)"
    elif log2_fc < -2.0:
        return "Very strong decrease (<0.25x)"
    elif log2_fc < -1.0:
        return "Strong decrease (0.25-0.5x)"
    elif log2_fc < -0.5:
        return "Moderate decrease (0.5-0.7x)"
    else:
        return "Minimal change"
```

## Limitations and Considerations

### Technical Limitations

- **Human only**: GRCh38 reference genome
- **SNVs only**: No indels or structural variants
- **Exact coordinates**: Must have precise genomic position
- **Sequence context**: Requires reference sequence match

### Interpretation Caveats

- **Predictions not certainties**: Validate with functional studies
- **Context matters**: Cell type, developmental stage affect outcomes
- **Indirect effects**: May miss complex regulatory cascades
- **Population variation**: Individual genetic background influences

## Troubleshooting

### Common Issues

**"API key required"**

- Set environment variable or provide per-request
- Check key validity at AlphaGenome portal

**"Invalid sequence length"**

- Verify chromosome format (use "chr" prefix)
- Check position is within chromosome bounds
- Ensure ref/alt are single nucleotides

**"No results returned"**

- May be no genes in analysis window
- Try larger interval size
- Check if variant is in gene desert

**Installation issues**

- Ensure Python 3.10+
- Try `pip install --upgrade pip` first
- Check for conflicting protobuf versions

## Next Steps

- Explore [comprehensive variant annotations](03-get-comprehensive-variant-annotations.md)
- Learn about [article searches](01-find-articles-and-cbioportal-data.md) for variants
- Set up [logging and monitoring](05-logging-and-monitoring-with-bigquery.md)
