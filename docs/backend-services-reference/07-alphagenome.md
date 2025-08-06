# AlphaGenome API Reference

Google DeepMind's AlphaGenome provides AI-powered predictions of variant effects on gene regulation, chromatin accessibility, and splicing.

## Usage Guide

For a step-by-step tutorial on using AlphaGenome for variant effect prediction, see [How to Predict Variant Effects with AlphaGenome](../how-to-guides/04-predict-variant-effects-with-alphagenome.md).

## Overview

AlphaGenome predicts regulatory effects of genetic variants by analyzing:

- Gene expression changes in nearby genes
- Chromatin accessibility alterations
- Splicing pattern modifications
- Enhancer and promoter activity
- Transcription factor binding
- 3D chromatin interactions

**Note:** AlphaGenome is an optional integration requiring separate installation and API key.

## Authentication

### Obtaining an API Key

1. Visit [https://deepmind.google.com/science/alphagenome](https://deepmind.google.com/science/alphagenome)
2. Register for non-commercial research use
3. Accept terms of service
4. Receive API key via email

### API Key Usage

**Environment Variable:**

```bash
export ALPHAGENOME_API_KEY="your-key-here"
```

**Per-Request:**

```python
result = alphagenome_predictor(
    chromosome="chr7",
    position=140753336,
    reference="A",
    alternate="T",
    api_key="your-key-here"  # Overrides environment
)
```

## Installation

AlphaGenome requires separate installation:

```bash
# Clone and install
git clone https://github.com/google-deepmind/alphagenome.git
cd alphagenome
pip install .

# Verify installation
python -c "import alphagenome; print('AlphaGenome installed')"
```

## API Interface

### Prediction Endpoint

The AlphaGenome API is accessed through the BioMCP `alphagenome_predictor` tool.

#### Parameters

| Parameter                | Type      | Required | Description                       |
| ------------------------ | --------- | -------- | --------------------------------- |
| `chromosome`             | str       | Yes      | Chromosome (e.g., "chr7")         |
| `position`               | int       | Yes      | 1-based genomic position          |
| `reference`              | str       | Yes      | Reference allele                  |
| `alternate`              | str       | Yes      | Alternate allele                  |
| `interval_size`          | int       | No       | Analysis window (default: 131072) |
| `tissue_types`           | list[str] | No       | UBERON tissue codes               |
| `significance_threshold` | float     | No       | Log2FC threshold (default: 0.5)   |
| `api_key`                | str       | No       | AlphaGenome API key               |

#### Interval Sizes

| Size      | Use Case   | Description                    |
| --------- | ---------- | ------------------------------ |
| 2,048     | Promoter   | TSS and promoter variants      |
| 16,384    | Local      | Proximal regulatory elements   |
| 131,072   | Standard   | Enhancer-promoter interactions |
| 524,288   | Long-range | Distal regulatory elements     |
| 1,048,576 | TAD-level  | Topological domain effects     |

## Tissue Codes

AlphaGenome supports tissue-specific predictions using UBERON ontology:

| Tissue   | UBERON Code    | Description          |
| -------- | -------------- | -------------------- |
| Breast   | UBERON:0000310 | Mammary gland tissue |
| Liver    | UBERON:0002107 | Hepatic tissue       |
| Prostate | UBERON:0002367 | Prostate gland       |
| Brain    | UBERON:0000955 | Neural tissue        |
| Lung     | UBERON:0002048 | Pulmonary tissue     |
| Colon    | UBERON:0001155 | Colonic mucosa       |

## Response Format

### Gene Expression Predictions

```json
{
  "gene_expression": [
    {
      "gene_name": "BRAF",
      "gene_id": "ENSG00000157764",
      "distance_to_tss": 1234,
      "log2_fold_change": 1.25,
      "confidence": 0.89,
      "tissue": "UBERON:0000310"
    }
  ]
}
```

**Interpretation:**

- `log2_fold_change > 1.0`: Strong increase (2x+)
- `log2_fold_change > 0.5`: Moderate increase
- `log2_fold_change < -1.0`: Strong decrease (0.5x)
- `log2_fold_change < -0.5`: Moderate decrease

### Chromatin Accessibility

```json
{
  "chromatin_accessibility": [
    {
      "region_type": "enhancer",
      "coordinates": "chr7:140450000-140451000",
      "accessibility_change": 0.75,
      "peak_height_change": 1.2,
      "tissue": "UBERON:0000310"
    }
  ]
}
```

**Interpretation:**

- Positive values: Increased accessibility (open chromatin)
- Negative values: Decreased accessibility (closed chromatin)

### Splicing Predictions

```json
{
  "splicing": [
    {
      "event_type": "exon_skipping",
      "affected_exon": "ENST00000288602.6:exon14",
      "delta_psi": -0.35,
      "splice_site_strength_change": -2.1
    }
  ]
}
```

**PSI (Percent Spliced In):**

- `delta_psi > 0`: Increased exon inclusion
- `delta_psi < 0`: Increased exon skipping
- `|delta_psi| > 0.1`: Biologically significant

## Usage Examples

### Basic Prediction

```python
# Predict BRAF V600E effects
result = await alphagenome_predictor(
    chromosome="chr7",
    position=140753336,
    reference="A",
    alternate="T"
)

# Process results
for gene in result.gene_expression:
    if abs(gene.log2_fold_change) > 1.0:
        print(f"{gene.gene_name}: {gene.log2_fold_change:.2f} log2FC")
```

### Tissue-Specific Analysis

```python
# Compare effects across tissues
tissues = {
    "breast": "UBERON:0000310",
    "lung": "UBERON:0002048",
    "brain": "UBERON:0000955"
}

results = {}
for tissue_name, tissue_code in tissues.items():
    results[tissue_name] = await alphagenome_predictor(
        chromosome="chr17",
        position=7577120,
        reference="G",
        alternate="A",
        tissue_types=[tissue_code]
    )
```

### Promoter Variant Analysis

```python
# Use small window for promoter variants
result = await alphagenome_predictor(
    chromosome="chr7",
    position=5569100,  # Near ACTB promoter
    reference="C",
    alternate="T",
    interval_size=2048  # 2kb window
)

# Check for promoter effects
promoter_effects = [
    g for g in result.gene_expression
    if abs(g.distance_to_tss) < 1000
]
```

### Enhancer Variant Analysis

```python
# Use larger window for enhancer variants
result = await alphagenome_predictor(
    chromosome="chr8",
    position=128748315,  # MYC enhancer region
    reference="G",
    alternate="A",
    interval_size=524288  # 512kb window
)

# Analyze chromatin changes
enhancer_changes = [
    c for c in result.chromatin_accessibility
    if c.region_type == "enhancer" and abs(c.accessibility_change) > 0.5
]
```

## Best Practices

### 1. Choose Appropriate Interval Size

```python
def select_interval_size(variant_type):
    """Select interval based on variant type"""
    intervals = {
        "promoter": 2048,
        "splice_site": 16384,
        "enhancer": 131072,
        "intergenic": 524288,
        "structural": 1048576
    }
    return intervals.get(variant_type, 131072)
```

### 2. Handle Missing Predictions

```python
# Not all variants affect gene expression
if not result.gene_expression:
    print("No gene expression changes predicted")
    # Check chromatin or splicing effects instead
```

### 3. Filter by Significance

```python
# Focus on significant changes
significant_genes = [
    g for g in result.gene_expression
    if abs(g.log2_fold_change) > significance_threshold
    and g.confidence > 0.8
]
```

### 4. Validate Input

```python
def validate_variant(chr, pos, ref, alt):
    """Validate variant format"""
    # Check chromosome format
    if not chr.startswith("chr"):
        raise ValueError("Chromosome must start with 'chr'")

    # Check alleles
    valid_bases = set("ACGT")
    if ref not in valid_bases or alt not in valid_bases:
        raise ValueError("Invalid nucleotide")

    # Check position
    if pos < 1:
        raise ValueError("Position must be 1-based")
```

## Integration Patterns

### VUS Classification Pipeline

```python
async def classify_vus(variant):
    """Classify variant of unknown significance"""

    # 1. Predict regulatory effects
    predictions = await alphagenome_predictor(
        chromosome=variant.chr,
        position=variant.pos,
        reference=variant.ref,
        alternate=variant.alt
    )

    # 2. Score impact
    max_expression = max(
        abs(g.log2_fold_change) for g in predictions.gene_expression
    ) if predictions.gene_expression else 0

    max_chromatin = max(
        abs(c.accessibility_change) for c in predictions.chromatin_accessibility
    ) if predictions.chromatin_accessibility else 0

    # 3. Classify
    if max_expression > 2.0 or max_chromatin > 1.5:
        return "High regulatory impact"
    elif max_expression > 1.0 or max_chromatin > 0.75:
        return "Moderate regulatory impact"
    else:
        return "Low regulatory impact"
```

### Multi-Variant Analysis

```python
async def analyze_variant_set(variants, target_gene):
    """Analyze multiple variants affecting a gene"""

    results = []
    for variant in variants:
        prediction = await alphagenome_predictor(
            chromosome=variant["chr"],
            position=variant["pos"],
            reference=variant["ref"],
            alternate=variant["alt"]
        )

        # Find target gene effect
        for gene in prediction.gene_expression:
            if gene.gene_name == target_gene:
                results.append({
                    "variant": f"{variant['chr']}:{variant['pos']}",
                    "effect": gene.log2_fold_change,
                    "confidence": gene.confidence
                })
                break

    # Sort by effect size
    return sorted(results, key=lambda x: abs(x["effect"]), reverse=True)
```

## Limitations

### Technical Limitations

- **Species**: Human only (GRCh38)
- **Variant Types**: SNVs only (no indels/SVs)
- **Sequence Context**: Requires reference match
- **Computation Time**: 1-3 seconds per variant

### Biological Limitations

- **Cell Type**: Predictions are tissue-specific approximations
- **Environmental Factors**: Does not account for conditions
- **Epistasis**: Single variant effects only
- **Temporal**: No developmental stage consideration

## Error Handling

### Common Errors

```python
try:
    result = await alphagenome_predictor(...)
except AlphaGenomeError as e:
    if "API key" in str(e):
        # Handle missing/invalid key
        pass
    elif "Invalid sequence" in str(e):
        # Handle sequence errors
        pass
    elif "Rate limit" in str(e):
        # Handle rate limiting
        pass
```

### Retry Logic

```python
async def predict_with_retry(params, max_retries=3):
    """Retry on transient failures"""
    for attempt in range(max_retries):
        try:
            return await alphagenome_predictor(**params)
        except Exception as e:
            if attempt == max_retries - 1:
                raise
            await asyncio.sleep(2 ** attempt)  # Exponential backoff
```

## Performance Optimization

### Batch Processing

```python
async def batch_predict(variants, batch_size=10):
    """Process variants in batches"""
    results = []

    for i in range(0, len(variants), batch_size):
        batch = variants[i:i + batch_size]
        batch_results = await asyncio.gather(*[
            alphagenome_predictor(**v) for v in batch
        ])
        results.extend(batch_results)

        # Rate limiting
        if i + batch_size < len(variants):
            await asyncio.sleep(1)

    return results
```

### Caching Strategy

```python
from functools import lru_cache

@lru_cache(maxsize=1000)
def get_cached_prediction(chr, pos, ref, alt, interval):
    """Cache predictions for repeated queries"""
    return alphagenome_predictor(
        chromosome=chr,
        position=pos,
        reference=ref,
        alternate=alt,
        interval_size=interval
    )
```

## Support Resources

- **Documentation**: [AlphaGenome GitHub](https://github.com/google-deepmind/alphagenome)
- **Paper**: [Nature Publication](https://www.nature.com/alphagenome)
- **Support**: Via GitHub issues
- **Terms**: Non-commercial research use only
