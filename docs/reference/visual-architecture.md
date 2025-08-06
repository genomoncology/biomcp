# Visual Architecture Guide

## System Architecture

BioMCP follows a clean architecture pattern with three main layers:

### 1. User Interface Layer

- **biomcp CLI**: Command-line interface for direct usage
- **Claude Desktop**: AI assistant integration via MCP
- **Python SDK**: Programmatic access for custom applications

### 2. BioMCP Core Layer

- **MCP Server**: Handles Model Context Protocol communication
- **Cache System**: Smart caching for API responses
- **Router**: Unified query routing across data sources

### 3. Data Source Layer

- **PubMed/PubTator3**: Biomedical literature and annotations
- **ClinicalTrials.gov**: Clinical trial registry
- **MyVariant.info**: Genetic variant database
- **cBioPortal**: Cancer genomics data
- **NCI CTS API**: National Cancer Institute trial data
- **BioThings APIs**: Gene, drug, and disease information

## Data Flow

1. **Request Processing**:

   - User sends query via CLI, Claude, or SDK
   - BioMCP server receives and validates request
   - Router determines appropriate data source(s)

2. **Caching Strategy**:

   - Check cache for existing results
   - If cache miss, fetch from external API
   - Store results with appropriate TTL
   - Return formatted results to user

3. **Response Formatting**:
   - Raw API data is normalized
   - Domain-specific enrichment applied
   - Results formatted for consumption

## Architecture References

- [Detailed Architecture Diagrams](architecture-diagrams.md)
- [Quick Architecture Reference](quick-architecture.md)

## Key Architecture Patterns

### Domain Separation

Each data source has its own module with dedicated:

- Search functions
- Result parsers
- Error handlers
- Cache strategies

### Unified Interface

All domains expose consistent methods:

- `search()`: Query for multiple results
- `fetch()`: Get detailed record by ID
- Common parameter names across domains

### Smart Caching

- API responses cached 15-30 minutes
- Cache keys include query parameters
- Automatic cache invalidation on errors
- Per-domain cache configuration

### Error Resilience

- Graceful degradation when APIs unavailable
- Specific error messages for troubleshooting
- Automatic retries with exponential backoff
- Fallback to cached data when possible
