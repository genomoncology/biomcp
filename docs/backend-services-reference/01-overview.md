# Backend Services Reference Overview

BioMCP integrates with multiple biomedical databases and services to provide comprehensive research capabilities. This reference documents the underlying APIs and their capabilities.

## Service Categories

### Literature and Publications

- **[PubTator3](06-pubtator3.md)**: Biomedical literature with entity annotations
- **Europe PMC**: Preprints from bioRxiv and medRxiv

### Clinical Trials

- **[ClinicalTrials.gov](04-clinicaltrials-gov.md)**: U.S. and international clinical trials registry
- **[NCI CTS API](05-nci-cts-api.md)**: National Cancer Institute's enhanced trial search

### Biomedical Annotations

- **[BioThings Suite](02-biothings-suite.md)**:
  - MyGene.info - Gene annotations
  - MyVariant.info - Variant annotations
  - MyDisease.info - Disease ontology
  - MyChem.info - Drug/chemical data

### Cancer Genomics

- **[cBioPortal](03-cbioportal.md)**: Cancer genomics portal with mutation data
- **TCGA**: The Cancer Genome Atlas (via MyVariant.info)

### Variant Effect Prediction

- **[AlphaGenome](07-alphagenome.md)**: Google DeepMind's AI for regulatory predictions

## API Authentication

| Service            | Authentication Required | Type    | Rate Limits         |
| ------------------ | ----------------------- | ------- | ------------------- |
| PubTator3          | No                      | Public  | 3 requests/second   |
| ClinicalTrials.gov | No                      | Public  | 50,000 requests/day |
| NCI CTS API        | Yes                     | API Key | 1,000 requests/day  |
| BioThings APIs     | No                      | Public  | 1,000 requests/hour |
| cBioPortal         | Optional                | Token   | Higher with token   |
| AlphaGenome        | Yes                     | API Key | Contact provider    |

## Data Flow Architecture

```
User Query → BioMCP Tools → Backend APIs → Unified Response

Example Flow:
1. User: "Find articles about BRAF mutations"
2. BioMCP: article_searcher tool
3. APIs Called:
   - PubTator3 (articles)
   - cBioPortal (mutation data)
   - Europe PMC (preprints)
4. Response: Integrated results with citations
```

## Service Reliability

### Primary Services

- **PubTator3**: 99.9% uptime, updated daily
- **ClinicalTrials.gov**: 99.5% uptime, updated daily
- **BioThings APIs**: 99.9% uptime, real-time data

### Fallback Strategies

- Cache frequently accessed data
- Implement exponential backoff
- Use alternative endpoints when available

## Common Integration Patterns

### 1. Entity Recognition Enhancement

```
PubTator3 → Extract entities → BioThings → Get detailed annotations
```

### 2. Variant to Trial Pipeline

```
MyVariant.info → Get gene → ClinicalTrials.gov → Find relevant trials
```

### 3. Comprehensive Gene Analysis

```
MyGene.info → Basic info
cBioPortal → Cancer mutations
PubTator3 → Literature
AlphaGenome → Predictions
```

## Performance Considerations

### Response Times (typical)

- PubTator3: 200-500ms
- ClinicalTrials.gov: 300-800ms
- BioThings APIs: 100-300ms
- cBioPortal: 200-600ms
- AlphaGenome: 1-3 seconds

### Optimization Strategies

1. **Batch requests** when APIs support it
2. **Cache static data** (gene names, ontologies)
3. **Parallelize independent** API calls
4. **Use pagination** for large result sets

## Error Handling

### Common Error Types

- **Rate Limiting**: 429 errors, implement backoff
- **Invalid Parameters**: 400 errors, validate inputs
- **Service Unavailable**: 503 errors, retry with delay
- **Authentication**: 401 errors, check API keys

### Error Response Format

```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "API rate limit exceeded",
    "retry_after": 3600
  }
}
```

## Data Formats

### Input Formats

- **Identifiers**: HGNC symbols, rsIDs, NCT numbers, PMIDs
- **Coordinates**: GRCh38 genomic positions
- **Terms**: MeSH, MONDO, HPO ontologies

### Output Formats

- **JSON**: Primary format for all APIs
- **XML**: Available for some services
- **TSV/CSV**: Export options for bulk data

## Update Frequencies

| Service            | Update Frequency | Data Lag   |
| ------------------ | ---------------- | ---------- |
| PubTator3          | Daily            | 1-2 days   |
| ClinicalTrials.gov | Daily            | Real-time  |
| NCI CTS            | Daily            | 1 day      |
| BioThings          | Real-time        | Minutes    |
| cBioPortal         | Quarterly        | 3-6 months |

## Best Practices

### 1. API Key Management

- Store keys securely
- Rotate keys periodically
- Monitor usage against limits

### 2. Error Recovery

- Implement retry logic
- Log failed requests
- Provide fallback data

### 3. Data Validation

- Verify gene symbols
- Validate genomic coordinates
- Check identifier formats

### 4. Performance

- Cache when appropriate
- Batch similar requests
- Use appropriate page sizes

## Getting Started

1. Review individual service documentation
2. Obtain necessary API keys
3. Test endpoints with sample data
4. Implement error handling
5. Monitor usage and performance

## Support Resources

- **PubTator3**: [Support Forum](https://www.ncbi.nlm.nih.gov/research/pubtator3/)
- **ClinicalTrials.gov**: [Help Desk](https://clinicaltrials.gov/help)
- **BioThings**: [Documentation](https://docs.biothings.io/)
- **cBioPortal**: [User Guide](https://docs.cbioportal.org/)
- **NCI**: [API Support](https://api.cancer.gov/support)
