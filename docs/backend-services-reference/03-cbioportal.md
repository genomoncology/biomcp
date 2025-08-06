# cBioPortal Integration

BioMCP integrates with [cBioPortal](https://www.cbioportal.org/), a comprehensive cancer genomics portal that provides visualization and analysis tools for large-scale cancer genomics datasets.

## Overview

The cBioPortal integration enhances article searches by automatically including relevant cancer genomics data when searching for genes. This integration provides:

1. **Gene-level summaries** - Mutation frequency and distribution across cancer studies
2. **Mutation-specific searches** - Find studies containing specific mutations (e.g., BRAF V600E)
3. **Cancer type resolution** - Accurate cancer type categorization using cBioPortal's API

## How It Works

### Automatic Integration

When you search for articles with a gene parameter, BioMCP automatically queries cBioPortal to provide additional context:

```python
# Basic gene search includes cBioPortal summary
search(domain="article", genes=["BRAF"], diseases=["melanoma"])
```

This returns:

- Standard PubMed/PubTator3 article results
- cBioPortal summary showing mutation frequency across cancer studies
- Top cancer types where the gene is mutated

### Mutation-Specific Searches

To search for specific mutations, include the mutation notation in keywords:

```python
# Search for BRAF V600E mutation
search(domain="article", genes=["BRAF"], keywords=["V600E"])

# Search for SRSF2 F57Y mutation
search(domain="article", genes=["SRSF2"], keywords=["F57Y"])

# Use wildcards for mutation patterns (e.g., any amino acid at position 57)
search(domain="article", genes=["SRSF2"], keywords=["F57*"])
```

Mutation-specific searches return:

- Total number of studies in cBioPortal
- Number of studies containing the mutation
- Top studies ranked by mutation count
- Cancer type distribution

## Example Output

### Gene-Level Summary

```
### cBioPortal Summary for BRAF
- **Mutation Frequency**: 76.7% (368 mutations in 480 samples)
- **Top Cancer Types**: Melanoma (45%), Thyroid (23%), Colorectal (18%)
- **Top Mutations**: V600E (89%), V600K (7%), G469A (2%)
```

### Mutation-Specific Results

```
### cBioPortal Mutation Search: BRAF
**Specific Mutation**: V600E
- **Total Studies**: 2340
- **Studies with Mutation**: 170
- **Total Mutations Found**: 5780

**Top Studies by Mutation Count:**
| Count | Study ID | Cancer Type | Study Name |
|-------|----------|-------------|------------|
|   804 | msk_met_2021 | Mixed Cancer Types | MSK MetTropism (MSK, Cell 2021) |
|   555 | msk_chord_2024 | Mixed Cancer Types | MSK-CHORD (MSK, Nature 2024) |
|   295 | msk_impact_2017 | Mixed Cancer Types | MSK-IMPACT Clinical Sequencing Cohort |
```

## Supported Mutation Notations

The integration recognizes standard protein change notation:

- **Specific mutations**: `V600E`, `F57Y`, `T790M`
- **Wildcard patterns**: `F57*` (matches F57Y, F57L, etc.)
- **Multiple mutations**: Include multiple keywords for OR search

## API Details

### Endpoints Used

1. **Gene Information**: `/api/genes/{gene}`
2. **Cancer Types**: `/api/cancer-types`
3. **Mutation Data**: `/api/mutations/fetch`
4. **Study Information**: `/api/studies`

### Rate Limiting

- Conservative rate limit of 5 requests/second
- Results cached for 15-30 minutes (mutations) or 24 hours (cancer types)

### Authentication

Optional authentication via environment variable:

```bash
export CBIO_TOKEN="your-api-token"
```

Public cBioPortal instance works without authentication but may have rate limits.

## CLI Usage

For detailed command-line options for searching articles with cBioPortal integration, see the [CLI User Guide](../user-guides/01-command-line-interface.md#article-commands).

## Performance Considerations

1. **Caching**: Results are cached to minimize API calls

   - Gene summaries: 15 minutes
   - Mutation searches: 30 minutes
   - Cancer types: 24 hours

2. **Graceful Degradation**: If cBioPortal is unavailable, searches continue without the additional data

3. **Parallel Processing**: API calls are made in parallel with article searches for optimal performance

## Limitations

1. Only works with valid HUGO gene symbols
2. Mutation searches require exact protein change notation
3. Limited to mutations in cBioPortal's curated studies
4. Rate limits may apply for high-volume usage

## Error Handling

The integration handles various error scenarios:

- Invalid gene symbols are validated before API calls
- Network timeouts fall back to article-only results
- API errors are logged but don't block search results
