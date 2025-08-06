# BioMCP Architecture Diagrams

This page describes BioMCP's architecture, data flows, and workflows.

## System Architecture Overview

BioMCP consists of three main layers:

### Client Layer

- **CLI Interface**: Command-line tool for direct interaction
- **Claude Desktop**: AI assistant integration via MCP protocol
- **Python SDK**: Programmatic access for custom applications
- **Custom MCP Clients**: Any MCP-compatible client

### BioMCP Core

- **MCP Server**: Handles protocol communication
- **Request Router**: Directs queries to appropriate handlers
- **Cache Layer**: Intelligent caching for API responses
- **Domain Handlers**: Specialized processors for each data type
  - Articles Handler (PubMed/PubTator3)
  - Trials Handler (ClinicalTrials.gov, NCI)
  - Variants Handler (MyVariant.info)
  - Genes Handler (MyGene.info)

### External APIs

- **PubMed/PubTator3**: Biomedical literature
- **ClinicalTrials.gov**: US clinical trials registry
- **NCI CTS API**: National Cancer Institute trials
- **MyVariant.info**: Genetic variant annotations
- **MyGene.info**: Gene information
- **cBioPortal**: Cancer genomics data
- **AlphaGenome**: Variant effect predictions

## Data Flow Architecture

1. **User Request**: Query submitted via CLI, Claude, or SDK
2. **Cache Check**: System checks for cached results
3. **API Request**: If cache miss, fetch from external API
4. **Result Processing**: Normalize and enrich data
5. **Cache Storage**: Store results for future use
6. **Response Delivery**: Return formatted results to user

## Key Workflows

### Search Workflow

1. **Think Tool**: Plan search strategy
2. **Execute Search**: Query relevant data sources
3. **Enrich Results**: Add contextual information
4. **Combine Data**: Merge results from multiple sources
5. **Format Output**: Present in user-friendly format

### Article Search Pipeline

1. **Query Processing**: Parse user input
2. **Entity Recognition**: Normalize gene/disease names
3. **PubTator3 Search**: Query literature database
4. **Preprint Integration**: Include bioRxiv/medRxiv if enabled
5. **cBioPortal Enrichment**: Add cancer genomics data for genes
6. **Result Merging**: Combine all data sources

### Clinical Trial Matching

1. **Patient Profile**: Parse eligibility criteria
2. **Location Filter**: Geographic constraints
3. **Molecular Profile**: Mutation requirements
4. **Prior Treatments**: Treatment history matching
5. **Scoring Algorithm**: Rank trials by relevance
6. **Contact Extraction**: Retrieve site information

### Variant Interpretation

1. **Input Parsing**: Process VCF/MAF files
2. **Batch Processing**: Group variants efficiently
3. **Annotation Gathering**:
   - Clinical significance from MyVariant.info
   - Population frequency data
   - In silico predictions
   - Literature evidence
   - Clinical trial associations
4. **AlphaGenome Integration**: Regulatory predictions (optional)
5. **Tier Classification**: Categorize by clinical relevance
6. **Report Generation**: Create interpretation summary

## Architecture Patterns

### Caching Strategy

- **Multi-tier Cache**: Memory → Disk → External
- **Smart TTL**: Domain-specific expiration times
- **Cache Key Generation**: Include all query parameters
- **Invalidation Logic**: Clear on errors or updates

### Error Handling

- **Retry Logic**: Exponential backoff for transient errors
- **Rate Limiting**: Respect API limits with queuing
- **Graceful Degradation**: Return partial results when possible
- **Clear Error Messages**: Help users troubleshoot issues

### Authentication Flow

1. Check for user-provided API key
2. Fall back to environment variable
3. Use public access if no key available
4. Handle authentication errors gracefully

### Performance Optimization

- **Request Batching**: Combine multiple queries
- **Parallel Execution**: Concurrent API calls
- **Connection Pooling**: Reuse HTTP connections
- **Result Streaming**: Return data as available

## Deployment Options

### Local Development

- Single process with in-memory cache
- Direct file system access
- Simple configuration

### Docker Deployment

- Containerized application
- Volume-mounted cache
- Environment-based configuration

### Cloud Deployment

- Load-balanced instances
- Shared Redis cache
- Auto-scaling capabilities
- Monitoring integration

## Creating Documentation Diagrams

For visual diagrams, we recommend:

1. **ASCII Art**: Universal compatibility

   - Use tools like asciiflow.com
   - Store in `docs/assets/` directory

2. **Screenshots**: For complex UIs

   - Annotate with arrows/labels
   - Save as PNG in `docs/assets/`

3. **External Tools**:
   - draw.io for flowcharts
   - Lucidchart for professional diagrams
   - Export as static images

## ASCII System Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              USER INTERFACES                             │
├────────────────┬───────────────────┬───────────────┬───────────────────┤
│                │                   │               │                   │
│   CLI Tool     │  Claude Desktop   │  Python SDK   │   Custom Client   │
│  (biomcp)      │   (MCP Client)    │   (async)     │    (your app)     │
│                │                   │               │                   │
└────────┬───────┴─────────┬─────────┴───────┬───────┴───────────┬───────┘
         │                 │                 │                   │
         └─────────────────┴─────────────────┴───────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                            BioMCP CORE SERVER                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐  │
│  │   Router    │  │ Rate Limiter │  │ Cache Manager│  │   Logger   │  │
│  │             │  │              │  │              │  │            │  │
│  └──────┬──────┘  └──────────────┘  └──────────────┘  └────────────┘  │
│         │                                                               │
│         ▼                                                               │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                      Domain Handlers                             │   │
│  ├─────────────┬─────────────┬─────────────┬──────────────────────┤   │
│  │  Articles   │   Trials    │  Variants   │  Genes/Drugs/Disease │   │
│  │  Handler    │   Handler   │  Handler    │      Handler         │   │
│  └──────┬──────┴──────┬──────┴──────┬──────┴──────────┬───────────┘   │
│         │             │             │                 │                 │
└─────────┼─────────────┼─────────────┼─────────────────┼─────────────────┘
          │             │             │                 │
          ▼             ▼             ▼                 ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                          EXTERNAL DATA SOURCES                           │
├─────────────┬─────────────┬─────────────┬──────────────────────────────┤
│             │             │             │                              │
│  PubMed/    │ Clinical    │ MyVariant   │        BioThings Suite       │
│  PubTator3  │ Trials.gov  │   .info     │  (MyGene/MyDisease/MyChem)  │
│             │    + NCI    │             │                              │
│             │             │             │                              │
├─────────────┴─────────────┴─────────────┴──────────────────────────────┤
│                                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                 │
│  │  cBioPortal  │  │  AlphaGenome │  │  Europe PMC  │                 │
│  │   (Cancer)   │  │ (Predictions)│  │  (Preprints) │                 │
│  └──────────────┘  └──────────────┘  └──────────────┘                 │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

See also: [Quick Architecture Reference](quick-architecture.md)

## Next Steps

- View the [Quick Architecture Guide](quick-architecture.md) for a concise overview
- Check [Developer Guides](../developer-guides/01-server-deployment.md) for implementation details
- See [API Reference](../apis/overview.md) for detailed specifications
