# AlphaGenome Prompt Examples

This guide provides example prompts for using AlphaGenome through BioMCP to analyze genetic variants. These prompts are designed for use with AI assistants like Claude that have BioMCP integrated.

## Including Your API Key

When using AlphaGenome through an AI assistant, include your API key in your request:

```
Analyze BRAF V600E mutation effects. My AlphaGenome API key is YOUR_KEY_HERE
```

Or for multiple analyses:

```
My AlphaGenome API key is YOUR_KEY_HERE. Please analyze these variants:
1. BRAF V600E (chr7:140753336 A>T)
2. TP53 R273H (chr17:7577121 C>T)
```

## Basic Variant Analysis

### Known Pathogenic Variants

**BRAF V600E (Melanoma)**

```
Use alphagenome_predictor to analyze the regulatory effects of the BRAF V600E mutation (chr7:140753336 A>T)
```

**TP53 Hotspot Mutations**

```
Predict how the TP53 R175H mutation (chr17:7675088 C>T) affects gene expression and chromatin accessibility
```

**EGFR Resistance Mutation**

```
What are the predicted regulatory impacts of the EGFR T790M mutation (chr7:55181378 C>T)?
```

### Non-coding Variant Analysis

**Promoter Variants**

```
Analyze this promoter variant using AlphaGenome: chr1:45797505 G>A in the MUTYH gene promoter
```

**Enhancer Variants**

```
Use alphagenome_predictor to assess this enhancer variant: chr8:128748315 T>C near the MYC gene
```

**UTR Variants**

```
Predict the regulatory effects of this 5' UTR variant: chr17:41244936 G>A in BRCA1
```

## Research-Oriented Analysis

### Variant Prioritization

**Multiple Variant Screening**

```
I have a list of variants from whole genome sequencing. Can you use alphagenome_predictor to analyze these and identify which ones likely have the strongest regulatory effects:
- chr3:178936091 G>A
- chr12:25398285 C>T
- chr19:11224301 G>T
```

**Regulatory Impact Ranking**

```
Analyze these non-coding variants and rank them by predicted regulatory impact:
1. chr5:1282543 C>T (TERT promoter)
2. chr8:128750412 A>G (MYC enhancer)
3. chr17:7571720 G>A (TP53 promoter)
```

### Splicing Analysis

**Intronic Variants**

```
Use AlphaGenome to predict if this intronic variant affects splicing: chr2:215593426 A>G in the BARD1 gene
```

**Splice Site Variants**

```
Analyze these variants near splice sites for potential splicing alterations:
- chr11:108198135 C>T (ATM gene, +5 position)
- chr13:32340700 G>A (BRCA2 gene, -3 position)
```

### Tissue-Specific Analysis

**Breast Tissue Analysis**

```
Predict the effects of chr7:140753336 A>T specifically in breast tissue (UBERON:0000310)
```

**Liver-Specific Effects**

```
Use alphagenome_predictor with liver tissue context (UBERON:0002107) for this variant: chr16:31356190 G>A in the FTO gene
```

**Multi-Tissue Comparison**

```
Compare the effects of chr12:25398285 C>T across:
- Brain tissue (UBERON:0000955)
- Liver tissue (UBERON:0002107)
- Lung tissue (UBERON:0002048)
```

## Clinical Research Workflows

### Variant of Uncertain Significance (VUS) Analysis

**Complete VUS Workup**

```
I found a variant of uncertain significance in a cancer patient: chr9:21971076 C>T in CDKN2A.
1. First use variant_getter to see known annotations
2. Then use alphagenome_predictor to assess potential regulatory impacts
3. Search for articles about similar CDKN2A variants
```

### Pharmacogenomics

**Drug Metabolism Variants**

```
Analyze how the CYP2D6 variant chr22:42130692 G>A might affect drug metabolism gene expression using AlphaGenome
```

**Warfarin Sensitivity**

```
Predict the regulatory effects of VKORC1 variant chr16:31107689 G>A on warfarin sensitivity
```

### Rare Disease Investigation

**Mitochondrial Disease**

```
This patient has a rare variant chr15:89859516 C>T in the POLG gene. Use alphagenome_predictor to understand if it might affect mitochondrial DNA polymerase expression
```

**Neurodevelopmental Disorders**

```
Analyze this de novo variant in a child with developmental delay: chr2:166199235 C>G in the SCN1A gene
```

## Comparative Analysis

### Multiple Variant Comparison

**Gene-Wide Analysis**

```
Compare the predicted regulatory effects of these three BRCA1 variants using AlphaGenome:
- chr17:41245237 G>A (promoter)
- chr17:41244936 G>A (5' UTR)
- chr17:41243451 T>C (intron 2)
```

**Hotspot Comparison**

```
I'm studying why some TP53 mutations are more severe than others. Use alphagenome_predictor to compare these hotspot mutations:
- R175H (chr17:7675088 C>T)
- R248W (chr17:7674220 G>A)
- R273H (chr17:7673802 C>T)
```

### Allele-Specific Analysis

**Alternative Alleles**

```
This GWAS hit is at chr5:1280000. Use alphagenome_predictor to analyze all possible variants:
- chr5:1280000 A>G
- chr5:1280000 A>C
- chr5:1280000 A>T
Which alternate allele has the strongest predicted effect?
```

## Advanced Research Prompts

### Long-Range Regulatory Analysis

**Extended Window Analysis**

```
Use alphagenome_predictor with --interval 1048576 to analyze long-range regulatory effects of chr8:128750000 A>G near the MYC oncogene
```

**TAD Boundary Variants**

```
Analyze this variant at a TAD boundary with maximum interval: chr3:186500000 C>T (use 1Mb window)
```

### Compound Heterozygote Analysis

**Trans Configuration**

```
Analyze these two variants in trans in the same gene:
- Maternal: chr11:47342697 C>T (MYBPC3)
- Paternal: chr11:47380142 G>A (MYBPC3)
What are their individual regulatory effects?
```

### Cancer Research

**Driver vs Passenger**

```
Help distinguish driver from passenger mutations. Analyze these variants from a tumor:
1. chr7:140753336 A>T (BRAF V600E)
2. chr3:41266101 C>T (CTNNB1 S33F)
3. chr1:115256529 G>A (NRAS Q61R)
Which show the strongest regulatory effects?
```

**Tumor Suppressor Analysis**

```
Analyze non-coding variants near tumor suppressors:
- chr17:7565097 C>T (TP53 promoter)
- chr13:32316461 A>G (BRCA2 promoter)
- chr17:41196312 G>C (BRCA1 promoter)
```

## Integration with Other BioMCP Tools

### Full Variant Characterization

**Literature + Prediction**

```
1. Search for articles about KRAS G12D mutations using article_searcher
2. Then use alphagenome_predictor to analyze chr12:25245350 C>T
3. Compare the literature findings with AlphaGenome predictions
```

**Database + AI Analysis**

```
1. Use variant_searcher to find pathogenic variants in the BRCA2 gene
2. Pick the top 3 results
3. Analyze each with alphagenome_predictor
4. Which has the strongest predicted regulatory impact?
```

### Clinical Trial Context

**Treatment Target Analysis**

```
1. Find clinical trials for BRAF V600E melanoma
2. Use AlphaGenome to understand why this mutation (chr7:140753336 A>T) is so impactful
3. Does the regulatory effect explain the treatment response?
```

## Tips for Effective Prompts

### Required Information

- **Chromosome**: Use "chr" prefix (e.g., chr7, chrX)
- **Position**: 1-based coordinate from reference genome
- **Reference allele**: Current base(s) at that position
- **Alternate allele**: Changed base(s)

### Optional Parameters

- **Interval size**: 2048, 16384, 131072, 524288, or 1048576
- **Tissue type**: UBERON ontology terms
- **Multiple variants**: Analyze in single prompt for comparison

### Best Practices

1. **Be specific** - Include exact coordinates
2. **Provide context** - Mention gene names and known effects
3. **Ask for interpretation** - Request specific insights
4. **Combine tools** - Use multiple BioMCP tools for comprehensive analysis
5. **Consider mechanism** - Ask about expression, splicing, or chromatin

## Example Multi-Step Workflow

```
I'm investigating a patient with suspected hereditary cancer syndrome. They have these variants:

1. First, check each variant in databases:
   - Use variant_getter on chr17:41245237 G>A
   - Use variant_getter on chr13:32340700 G>A
   - Use variant_getter on chr11:108198135 C>T

2. Then predict regulatory effects:
   - Use alphagenome_predictor on each variant
   - Compare which has the strongest impact

3. Search literature:
   - Find articles about each affected gene
   - Look for similar cases

4. Summarize findings:
   - Which variant is most likely pathogenic?
   - What functional evidence supports this?
```

This structured approach combines AlphaGenome's predictive power with BioMCP's database access for comprehensive variant analysis.
