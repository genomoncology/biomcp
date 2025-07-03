# Variants CLI Documentation

The Variants CLI allows users to search for and retrieve genetic variant information using the MyVariant.info API.

> **API Documentation**: For details about the underlying API, see the [MyVariant.info API Documentation](../apis/myvariant_info.md).
>
> **Tip**: Use the `--help` flag with any command (e.g., `biomcp variant search --help`) to see the most up-to-date options directly from the tool.

## Search Command (`search`)

Search for genetic variants using multiple parameters and filters. At least one search parameter (like gene, hgvsp, rsid, region) is required.

### Usage

```bash
biomcp variant search [OPTIONS]
```

#### Basic Search Parameters

- `-g, --gene TEXT`: Gene symbol to search for (e.g., BRAF, TP53).
- `--hgvsp TEXT`: Protein change notation using HGVS format (e.g., "p.Val600Glu", "p.V600E"). Often used with `--gene`.
- `--hgvsc TEXT`: cDNA change notation using HGVS format (e.g., "c.1799T>A"). Often used with `--gene`.
- `--rsid TEXT`: dbSNP rsID (e.g., "rs113488022").
- `--region TEXT`: Genomic region in format chr:start-end (e.g., "chr7:140453100-140453200").

#### Clinical and Functional Filters

- `-s, --significance [pathogenic|likely_pathogenic|uncertain_significance|likely_benign|benign]`: Filter by ClinVar clinical significance. Case-insensitive.
- `--min-frequency FLOAT`: Minimum gnomAD exome allele frequency (0.0 to 1.0).
- `--max-frequency FLOAT`: Maximum gnomAD exome allele frequency (0.0 to 1.0).
- `--cadd FLOAT`: Minimum CADD phred score (e.g., 15, 20). Filters for variants with score >= value.
- `--polyphen [D|P|B]`: Filter by PolyPhen-2 prediction (D: Probably damaging, P: Possibly damaging, B: Benign). Case-insensitive.
- `--sift [D|T]`: Filter by SIFT prediction (D: Deleterious, T: Tolerated). Case-insensitive.

#### Output and Source Options

- `--sources TEXT`: Comma-separated list of specific data sources to include in the results (e.g., "clinvar,dbsnp,cosmic"). See MyVariant.info docs for source names. Adding sources can increase the detail in the output.
- `--size INTEGER`: Maximum number of results to return [default: 40].
- `--offset INTEGER`: Result offset for pagination [default: 0]. Use with `--size` for paging.
- `--sort TEXT`: Field to sort results by, using MyVariant.info syntax (e.g., "cadd.phred:desc").
- `-j, --json`: Render output in JSON format instead of Markdown.
- `--help`: Show help message and exit.

#### Examples

Search for a variant by gene and protein change:

```bash
biomcp variant search --gene BRAF --hgvsp p.V600E
```

Search for pathogenic variants in TP53:

```bash
biomcp variant search --gene TP53 --significance pathogenic
```

Search for rare (max freq 0.1%) BRAF variants with high CADD score:

```bash
biomcp variant search --gene BRAF --max-frequency 0.001 --cadd 20
```

Search by genomic region:

```bash
biomcp variant search --region chr7:140453130-140453140
```

Search by rsID and request extra data from COSMIC:

```bash
biomcp variant search --rsid rs113488022 --sources cosmic
```

Get results as JSON:

```bash
biomcp variant search --gene BRAF --hgvsp p.V600E --json
```

## Get Command (`get`)

Retrieve detailed information about a single specific variant by its identifier.

### Usage

```bash
biomcp variant get [OPTIONS] VARIANT_ID
```

#### Arguments

- `VARIANT_ID`: The variant identifier. This can be a MyVariant.info ID (HGVS format, e.g., "chr7:g.140453136A>T") or a dbSNP rsID (e.g., "rs113488022"). [required]

#### Options

- `-j, --json`: Render output in JSON format instead of Markdown.
- `--help`: Show help message and exit.

#### Examples

Get a variant by HGVS ID:

```bash
biomcp variant get chr7:g.140453136A>T
```

Get a variant by rsID:

```bash
biomcp variant get rs113488022
```

Get a variant by rsID as JSON:

```bash
biomcp variant get rs113488022 --json
```

## Predict Command (`predict`)

Predict variant effects on gene regulation using Google DeepMind's AlphaGenome model. This advanced feature uses AI to predict how genetic variants affect gene expression, chromatin accessibility, splicing, and other regulatory mechanisms.

### Prerequisites

1. **Install AlphaGenome** (optional dependency):

   ```bash
   git clone https://github.com/google-deepmind/alphagenome.git
   cd alphagenome && pip install .
   ```

2. **API Key**: Get a free API key from [DeepMind AlphaGenome](https://deepmind.google.com/science/alphagenome) and set:
   ```bash
   export ALPHAGENOME_API_KEY='your-api-key'
   ```

### Usage

```bash
biomcp variant predict [OPTIONS] CHROMOSOME POSITION REFERENCE ALTERNATE
```

#### Arguments

- `CHROMOSOME`: Chromosome name (e.g., chr7, chrX) [required]
- `POSITION`: 1-based genomic position [required]
- `REFERENCE`: Reference allele(s) (e.g., A, ATG) [required]
- `ALTERNATE`: Alternate allele(s) (e.g., T, A) [required]

#### Options

- `-i, --interval INTEGER`: Size of genomic interval to analyze in base pairs (default: 100000, max: 1000000)
- `-t, --tissue TEXT`: UBERON ontology terms for tissue-specific predictions. Can be used multiple times.
- `--help`: Show help message and exit

#### Examples

Predict effects of BRAF V600E mutation:

```bash
biomcp variant predict chr7 140753336 A T
```

Predict with tissue-specific context (breast tissue):

```bash
biomcp variant predict chr7 140753336 A T --tissue UBERON:0002367
```

Use larger analysis window (500kb):

```bash
biomcp variant predict chr7 140753336 A T --interval 500000
```

Multiple tissue contexts:

```bash
biomcp variant predict chr7 140753336 A T --tissue UBERON:0002367 --tissue UBERON:0001157
```

### Output

AlphaGenome predictions include:

- **Gene Expression**: Log₂ fold changes in RNA-seq signals
- **Chromatin Accessibility**: Changes in ATAC-seq/DNase-seq signals
- **Splicing**: Potential splice site alterations
- **Promoter Activity**: CAGE signal changes
- **Summary Statistics**: Number of affected regulatory tracks

Results show the most significant effects across all analyzed regulatory modalities, helping understand the variant's potential functional impact.

> **📚 Further Reading**: For detailed setup instructions and advanced usage examples, see the [AlphaGenome Setup Guide](../tutorials/alphagenome-setup.md) and [AlphaGenome Prompt Examples](../tutorials/alphagenome-prompts.md).

## Output Format

By default, both search and get output variant information in Markdown format, designed for readability. This includes key annotations and automatically generated links to external databases like dbSNP, ClinVar, Ensembl, UCSC Genome Browser, etc., where applicable.

Use the `--json` flag to get the raw data (with injected URLs) as a JSON object, which is useful for scripting or integration with other tools. The specific fields returned by default in a search focus on common identifiers and annotations; use `--sources` to request more comprehensive data for specific databases. The get command retrieves all available default fields plus database links.
