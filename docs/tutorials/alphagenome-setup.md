# AlphaGenome Setup and Usage Guide

This guide covers how to set up and use Google DeepMind's AlphaGenome with BioMCP for advanced variant effect prediction.

## Overview

AlphaGenome is a state-of-the-art AI model that predicts how genetic variants affect gene regulation. It analyzes variants in their genomic context to predict impacts on:

- Gene expression (RNA-seq)
- Chromatin accessibility (ATAC-seq, DNase-seq)
- Splicing patterns
- Promoter activity (CAGE)
- Transcription factor binding
- 3D chromatin structure

## Prerequisites

1. **BioMCP Installation**: Ensure BioMCP is installed and working
2. **Python 3.10+**: Required for AlphaGenome compatibility
3. **Git**: For cloning the AlphaGenome repository

## Setup Instructions

### Step 1: Get an AlphaGenome API Key

1. Visit [Google DeepMind AlphaGenome](https://deepmind.google.com/science/alphagenome)
2. Click "Get Started" or "Request Access"
3. Fill out the registration form (free for non-commercial use)
4. You'll receive an API key that looks like: `AIzaSy...`

### Step 2: Configure the API Key

You have three options for providing your API key:

#### Option A: Per-Request (Recommended for AI Assistants)

Include your API key directly in your request to the AI assistant:

```
"Predict effects of BRAF V600E. My AlphaGenome API key is YOUR_KEY_HERE"
```

See the [API Key Handling Guide](alphagenome-api-key-handling.md) for detailed examples.

#### Option B: Environment Variable (Personal Use)

Set it in your shell profile (`~/.zshrc` or `~/.bashrc`):

```bash
export ALPHAGENOME_API_KEY='your-api-key-here'
```

#### Option C: Using .env file (Development)

1. Create a `.env` file in your BioMCP directory:

   ```bash
   cd ~/code/biomcp  # or your BioMCP installation directory
   echo 'ALPHAGENOME_API_KEY=your-api-key-here' >> .env
   ```

2. Replace `your-api-key-here` with your actual API key

#### Option C: Claude Desktop Configuration

Add to your Claude Desktop MCP settings:

```json
"biomcp": {
  "command": "uv",
  "args": [
    "--directory",
    "/path/to/biomcp",
    "run",
    "biomcp",
    "run"
  ],
  "env": {
    "ALPHAGENOME_API_KEY": "your-api-key-here"
  }
}
```

### Step 3: Install AlphaGenome

1. Clone the AlphaGenome repository:

   ```bash
   cd ~/code  # or any directory you prefer
   git clone https://github.com/google-deepmind/alphagenome.git
   ```

2. Install AlphaGenome in BioMCP's environment:

   ```bash
   cd ~/code/biomcp  # Navigate to BioMCP directory
   uv pip install ~/code/alphagenome
   ```

3. Verify installation:
   ```bash
   uv run python -c "import alphagenome; print('AlphaGenome installed successfully')"
   ```

### Step 4: Test the Integration

Test via CLI:

```bash
uv run biomcp variant predict chr7 140753336 A T
```

Expected output:

```
## AlphaGenome Variant Effect Predictions

**Variant**: chr7:140753336 A>T
**Analysis window**: 131,072 bp

### Gene Expression
- **BRAF**: -0.85 log₂ fold change (↓ decreases expression)
...
```

## Usage Examples

### Basic Variant Analysis

```bash
# Analyze a known pathogenic variant (BRAF V600E)
biomcp variant predict chr7 140753336 A T

# Analyze with larger genomic context (1Mb window)
biomcp variant predict chr7 140753336 A T --interval 1048576

# Analyze with tissue-specific context
biomcp variant predict chr7 140753336 A T --tissue UBERON:0000310  # breast tissue

# Analyze with custom significance threshold
biomcp variant predict chr7 140753336 A T --threshold 0.3
```

### Using in Claude Desktop

Once configured, you can use these prompts in Claude:

#### Known Pathogenic Variants

- "Use alphagenome_predictor to analyze the regulatory effects of the BRAF V600E mutation (chr7:140753336 A>T)"
- "Predict how the TP53 R175H mutation (chr17:7675088 C>T) affects gene expression and chromatin accessibility"
- "What are the predicted regulatory impacts of the EGFR T790M mutation (chr7:55181378 C>T)?"

#### Non-coding Variant Analysis

- "Analyze this promoter variant using AlphaGenome: chr1:45797505 G>A in the MUTYH gene promoter"
- "Use alphagenome_predictor to assess this enhancer variant: chr8:128748315 T>C near the MYC gene"
- "Predict the regulatory effects of this 5' UTR variant: chr17:41244936 G>A in BRCA1"

#### Splicing Analysis

- "Use AlphaGenome to predict if this intronic variant affects splicing: chr2:215593426 A>G in the BARD1 gene"
- "Analyze these variants near splice sites for potential splicing alterations: chr11:108198135 C>T (ATM)"

#### Research Workflows

- "I found a variant of uncertain significance: chr9:21971076 C>T in CDKN2A. First use variant_getter to see known annotations, then use alphagenome_predictor to assess regulatory impacts"
- "Compare the predicted effects of these BRCA1 variants: chr17:41245237 G>A vs chr17:41244936 G>A"

#### Multi-variant Analysis

- "I have variants from whole genome sequencing. Analyze these for regulatory effects: chr3:178936091 G>A, chr12:25398285 C>T, chr19:11224301 G>T"

## Advanced Usage

### Tissue-Specific Predictions

AlphaGenome can provide tissue-specific predictions using UBERON ontology terms:

```python
# Common tissue codes:
# UBERON:0000310 - breast
# UBERON:0002107 - liver
# UBERON:0002367 - prostate
# UBERON:0000955 - brain
# UBERON:0002048 - lung
# UBERON:0001157 - colon

# Example: Liver-specific analysis
result = await alphagenome_predictor(
    chromosome="chr16",
    position=31356190,
    reference="G",
    alternate="A",
    tissue_types=["UBERON:0002107"]
)
```

### Interval Sizes

AlphaGenome supports specific interval sizes:

- 2,048 bp (2kb) - Very local effects
- 16,384 bp (16kb) - Local regulatory elements
- 131,072 bp (128kb) - Default, captures most regulatory elements
- 524,288 bp (512kb) - Extended regulatory landscape
- 1,048,576 bp (1Mb) - Long-range interactions

Choose based on your hypothesis:

- Promoter variants: 16kb
- Enhancer variants: 128kb-512kb
- Long-range regulatory: 1Mb

**Note**: If you request a size larger than 1Mb, the system automatically uses 1Mb. If you request a size between supported values, it rounds up to the next supported size.

### Interpreting Results

**Gene Expression (log₂ fold change)**:

- \> +1.0: Strong increase (2x or more)
- +0.5 to +1.0: Moderate increase
- -0.5 to +0.5: Minimal change
- -1.0 to -0.5: Moderate decrease
- < -1.0: Strong decrease (2x or less)

**Chromatin Accessibility**:

- Positive values: More open chromatin (increased accessibility)
- Negative values: More closed chromatin (decreased accessibility)

**Summary Statistics**:

- Total tracks: Number of cell types/conditions analyzed
- Significant changes: Tracks with |log₂| > 0.5 (default threshold)
- Custom threshold: You can adjust the significance threshold using the `--threshold` parameter

## Troubleshooting

### "AlphaGenome API key not found"

- Check your `.env` file exists and contains `ALPHAGENOME_API_KEY=your-key`
- Ensure you're running commands from the BioMCP directory
- Try: `cat .env | grep ALPHAGENOME` to verify

### "AlphaGenome not installed"

- Make sure you installed AlphaGenome: `uv pip install ~/code/alphagenome`
- Check installation: `uv pip list | grep alphagenome`

### "Sequence length X not supported"

- Use one of the supported sizes: 2048, 16384, 131072, 524288, 1048576
- The tool automatically rounds up to the nearest supported size

### Protobuf warnings

- You may see warnings like "Protobuf gencode version 5.27.2 is exactly one major version older..."
- These warnings are harmless and don't affect functionality
- They occur because AlphaGenome's proto files were compiled with an older protobuf version
- To suppress in your own code:
  ```python
  import warnings
  warnings.filterwarnings("ignore", message="Protobuf gencode version.*is exactly one major version older")
  ```

## Best Practices

1. **Start with default settings** - The 128kb window captures most regulatory elements
2. **Use tissue context when relevant** - Especially for tissue-specific diseases
3. **Combine with other tools** - Use `variant_getter` first for known annotations
4. **Consider multiple variants** - Analyze all variants in a gene for comprehensive view
5. **Document your findings** - Save important predictions for future reference
6. **Validate inputs** - Ensure chromosome format (chr1-22, chrX, chrY, chrM/chrMT) and valid nucleotides (A, C, G, T)
7. **Leverage caching** - Results are cached for 30 minutes to improve performance for repeated queries
8. **Adjust thresholds** - Use lower thresholds (e.g., 0.3) to detect subtle effects, higher (e.g., 1.0) for strong effects only

## Limitations

- Human genome only (GRCh38/hg38)
- Requires exact genomic coordinates
- Cannot analyze structural variants or complex indels
- Predictions are computational and should be validated experimentally
- API rate limits may apply for high-volume usage
- Chromosome format must include 'chr' prefix (e.g., chr1, not 1)
- Only standard nucleotides supported (A, C, G, T) - no ambiguity codes

## Further Resources

- [AlphaGenome Paper](https://deepmind.google/science/alphagenome)
- [BioMCP Variant Documentation](../cli/variants.md)
- [UBERON Tissue Ontology](https://www.ebi.ac.uk/ols/ontologies/uberon)
