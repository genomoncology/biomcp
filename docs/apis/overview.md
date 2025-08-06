# API Reference Overview

BioMCP provides multiple interfaces for programmatic access to biomedical data. This reference covers the Python SDK, MCP protocol implementation, and HTTP API endpoints.

## Available APIs

### 1. Python SDK

The Python SDK provides async/await interfaces for all BioMCP functionality:

- **Client API**: High-level client for all domains
- **Domain-specific APIs**: Specialized interfaces for articles, trials, variants
- **Streaming API**: For real-time data processing
- **Batch API**: For bulk operations

See [Python SDK Reference](python-sdk.md) for detailed documentation.

### 2. MCP Protocol

BioMCP implements the Model Context Protocol for AI assistant integration:

- **24 specialized tools** for biomedical research
- **Unified search** across all domains
- **Sequential thinking** for complex queries
- **Streaming responses** for large datasets

See [MCP Tools Reference](../user-guides/02-mcp-tools-reference.md) for implementation details.

### 3. HTTP REST API

When running in HTTP mode, BioMCP exposes RESTful endpoints:

- **Search endpoints** for each domain
- **Fetch endpoints** for detailed records
- **Health monitoring** endpoints
- **WebSocket support** for streaming

See [Transport Protocol Guide](../developer-guides/04-transport-protocol.md) for endpoint documentation.

## Common Patterns

### Authentication

Most endpoints work without authentication. API keys enable enhanced features:

```python
# Python SDK
client = BioMCPClient(
    nci_api_key="your-key",
    alphagenome_api_key="your-key"
)

# HTTP API
headers = {
    "X-NCI-API-Key": "your-key",
    "X-AlphaGenome-API-Key": "your-key"
}
```

### Error Handling

All APIs use consistent error codes:

| Code | Meaning      | Action             |
| ---- | ------------ | ------------------ |
| 400  | Bad Request  | Check parameters   |
| 401  | Unauthorized | Check API key      |
| 404  | Not Found    | Verify ID exists   |
| 429  | Rate Limited | Retry with backoff |
| 500  | Server Error | Retry later        |

### Pagination

Standard pagination across all APIs:

```python
# Python SDK
results = await client.search(
    domain="article",
    page=1,
    page_size=20
)

# HTTP API
GET /api/articles?page=1&page_size=20
```

### Response Formats

All APIs support multiple response formats:

- **JSON**: Default, structured data
- **JSONL**: Streaming line-delimited JSON
- **Markdown**: Human-readable formatting
- **CSV**: Tabular data export

## Rate Limits

| API                | Without Key | With Key     |
| ------------------ | ----------- | ------------ |
| PubMed/PubTator3   | 3 req/sec   | 10 req/sec   |
| ClinicalTrials.gov | 50 req/min  | 50 req/min   |
| BioThings          | 3 req/sec   | 10 req/sec   |
| NCI                | N/A         | 1000 req/day |
| AlphaGenome        | N/A         | 100 req/day  |

## Next Steps

- [Python SDK Reference](python-sdk.md) - Detailed Python API documentation
- [MCP Tools Reference](../user-guides/02-mcp-tools-reference.md) - MCP implementation details
- [Transport Protocol Guide](../developer-guides/04-transport-protocol.md) - REST endpoint documentation
- [Error Codes Reference](error-codes.md) - Complete error code listing
