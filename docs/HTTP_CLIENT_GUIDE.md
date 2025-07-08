# BioMCP HTTP Client Guide

## Overview

BioMCP uses a centralized HTTP client for all external API calls. This provides:

- Consistent error handling and retry logic
- Request/response caching
- Rate limiting per domain
- Circuit breaker for fault tolerance
- Offline mode support
- Comprehensive endpoint tracking

## Migration from Direct HTTP Libraries

### Before (Direct httpx usage):

```python
import httpx

async def fetch_gene(gene: str):
    async with httpx.AsyncClient() as client:
        response = await client.get(f"https://api.example.com/genes/{gene}")
        response.raise_for_status()
        return response.json()
```

### After (Centralized client):

```python
from biomcp import http_client

async def fetch_gene(gene: str):
    data, error = await http_client.request_api(
        url=f"https://api.example.com/genes/{gene}",
        request={},
        domain="example"
    )
    if error:
        # Handle error consistently
        return None
    return data
```

## Error Handling

The centralized client uses a consistent error handling pattern:

```python
result, error = await http_client.request_api(...)

if error:
    # error is a RequestError object with:
    # - error.code: HTTP status code or error type
    # - error.message: Human-readable error message
    # - error.details: Additional context
    logger.error(f"Request failed: {error.message}")
    return None  # or handle appropriately
```

### Error Handling Guidelines

1. **For optional data**: Return `None` when the data is not critical
2. **For required data**: Raise an exception or return an error to the caller
3. **For batch operations**: Collect errors and report at the end
4. **For user-facing operations**: Provide clear, actionable error messages

## Creating Domain-Specific Adapters

For complex APIs, create an adapter class:

```python
from biomcp import http_client
from biomcp.http_client import RequestError

class MyAPIAdapter:
    """Adapter for MyAPI using centralized HTTP client."""

    def __init__(self):
        self.base_url = "https://api.example.com"

    async def get_resource(self, resource_id: str) -> tuple[dict | None, RequestError | None]:
        """Fetch a resource by ID.

        Returns:
            Tuple of (data, error) where one is always None
        """
        return await http_client.request_api(
            url=f"{self.base_url}/resources/{resource_id}",
            request={},
            domain="example",
            endpoint_key="example_resources"
        )
```

## Configuration

### Cache TTL (Time To Live)

```python
# Cache for 1 hour (3600 seconds)
data, error = await http_client.request_api(
    url=url,
    request=request,
    cache_ttl=3600
)

# Disable caching for this request
data, error = await http_client.request_api(
    url=url,
    request=request,
    cache_ttl=0
)
```

### Rate Limiting

Rate limits are configured per domain in `http_client.py`:

```python
# Default rate limits
rate_limits = {
    "ncbi.nlm.nih.gov": 20,  # 20 requests/second
    "clinicaltrials.gov": 10,  # 10 requests/second
    "myvariant.info": 1000/3600,  # 1000 requests/hour
}
```

### Circuit Breaker

The circuit breaker prevents cascading failures:

- **Closed**: Normal operation
- **Open**: Failing fast after threshold exceeded
- **Half-Open**: Testing if service recovered

Configure thresholds:

```python
CIRCUIT_BREAKER_FAILURE_THRESHOLD = 5  # Open after 5 failures
CIRCUIT_BREAKER_RECOVERY_TIMEOUT = 60  # Try again after 60 seconds
```

## Offline Mode

Enable offline mode to only serve cached responses:

```bash
export BIOMCP_OFFLINE=true
biomcp run
```

In offline mode:

- Only cached responses are returned
- No external HTTP requests are made
- Missing cache entries return None with appropriate error

## Performance Tuning

### Connection Pooling

The HTTP client maintains connection pools per domain:

```python
# Configure in http_client_simple.py
limits = httpx.Limits(
    max_keepalive_connections=20,
    max_connections=100,
    keepalive_expiry=30
)
```

### Concurrent Requests

For parallel requests to the same API:

```python
import asyncio

# Fetch multiple resources concurrently
tasks = [
    http_client.request_api(f"/resource/{i}", {}, domain="example")
    for i in range(10)
]
results = await asyncio.gather(*tasks)
```

## Monitoring and Debugging

### Request Metrics

The client tracks metrics per endpoint:

- Request count
- Error count
- Cache hit/miss ratio
- Average response time

Access metrics:

```python
from biomcp.http_client import get_metrics
metrics = get_metrics()
```

### Debug Logging

Enable debug logging to see all HTTP requests:

```python
import logging
logging.getLogger("biomcp.http_client").setLevel(logging.DEBUG)
```

## Best Practices

1. **Always use the centralized client** for external HTTP calls
2. **Register new endpoints** in the endpoint registry
3. **Set appropriate cache TTLs** based on data volatility
4. **Handle errors gracefully** with user-friendly messages
5. **Test with offline mode** to ensure cache coverage
6. **Monitor rate limits** to avoid API throttling
7. **Use domain-specific adapters** for complex APIs

## Endpoint Registration

Register new endpoints in `endpoint_registry.py`:

```python
registry.register(
    "my_api_endpoint",
    EndpointInfo(
        url="https://api.example.com/v1/data",
        category=EndpointCategory.BIOMEDICAL_LITERATURE,
        data_types=[DataType.RESEARCH_ARTICLES],
        description="My API for fetching data",
        compliance_notes="Public API, no PII",
        rate_limit="100 requests/minute"
    )
)
```

This ensures the endpoint is documented and tracked properly.
