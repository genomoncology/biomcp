# Migration Examples: Direct HTTP to Centralized Client

This guide provides examples of migrating from direct HTTP library usage to the centralized BioMCP HTTP client.

## Basic GET Request

### Before (httpx)

```python
import httpx

async def get_gene_info(gene_symbol: str) -> dict:
    async with httpx.AsyncClient() as client:
        response = await client.get(
            f"https://api.example.com/genes/{gene_symbol}",
            timeout=30.0
        )
        response.raise_for_status()
        return response.json()
```

### After (Centralized Client)

```python
from biomcp import http_client

async def get_gene_info(gene_symbol: str) -> dict | None:
    data, error = await http_client.request_api(
        url=f"https://api.example.com/genes/{gene_symbol}",
        request={},
        domain="example",
        endpoint_key="example_genes"
    )
    if error:
        logger.error(f"Failed to fetch gene {gene_symbol}: {error.message}")
        return None
    return data
```

## POST Request with JSON Body

### Before (httpx)

```python
async def search_variants(query: dict) -> list:
    async with httpx.AsyncClient() as client:
        response = await client.post(
            "https://api.example.com/variants/search",
            json=query,
            headers={"Authorization": "Bearer token123"}
        )
        response.raise_for_status()
        return response.json()
```

### After (Centralized Client)

```python
async def search_variants(query: dict) -> list:
    # Headers can be passed via adapter or request
    data, error = await http_client.request_api(
        url="https://api.example.com/variants/search",
        request=query,
        method="POST",
        domain="example",
        endpoint_key="example_variant_search"
    )
    if error:
        return []
    return data
```

## Retry Logic

### Before (Manual Retry)

```python
import asyncio
import httpx

async def fetch_with_retry(url: str, max_retries: int = 3):
    for attempt in range(max_retries):
        try:
            async with httpx.AsyncClient() as client:
                response = await client.get(url)
                response.raise_for_status()
                return response.json()
        except httpx.HTTPStatusError as e:
            if e.response.status_code in [429, 500, 502, 503, 504]:
                if attempt < max_retries - 1:
                    await asyncio.sleep(2 ** attempt)
                    continue
            raise
        except httpx.RequestError:
            if attempt < max_retries - 1:
                await asyncio.sleep(2 ** attempt)
                continue
            raise
```

### After (Built-in Retry)

```python
async def fetch_with_retry(url: str):
    # Retry is built into the centralized client
    data, error = await http_client.request_api(
        url=url,
        request={},
        domain="example",
        enable_retry=True  # Default is True
    )
    return data, error
```

## Authenticated Requests

### Before (Direct Headers)

```python
class APIClient:
    def __init__(self, api_key: str):
        self.headers = {"Authorization": f"Bearer {api_key}"}

    async def get_data(self, endpoint: str):
        async with httpx.AsyncClient() as client:
            response = await client.get(
                f"https://api.example.com/{endpoint}",
                headers=self.headers
            )
            return response.json()
```

### After (Using Adapter)

```python
from biomcp import http_client
import json

class APIAdapter:
    def __init__(self, api_key: str):
        self.base_url = "https://api.example.com"
        self.headers = {"Authorization": f"Bearer {api_key}"}

    async def get_data(self, endpoint: str):
        # Pass headers through request
        request_data = {
            "_headers": json.dumps(self.headers)
        }

        data, error = await http_client.request_api(
            url=f"{self.base_url}/{endpoint}",
            request=request_data,
            domain="example"
        )
        return data, error
```

## Parallel Requests

### Before (asyncio.gather)

```python
async def fetch_multiple_genes(gene_list: list[str]):
    async with httpx.AsyncClient() as client:
        tasks = [
            client.get(f"https://api.example.com/genes/{gene}")
            for gene in gene_list
        ]
        responses = await asyncio.gather(*tasks, return_exceptions=True)

        results = []
        for resp in responses:
            if isinstance(resp, Exception):
                results.append(None)
            else:
                results.append(resp.json())
        return results
```

### After (Same Pattern, Better Error Handling)

```python
async def fetch_multiple_genes(gene_list: list[str]):
    tasks = [
        http_client.request_api(
            url=f"https://api.example.com/genes/{gene}",
            request={},
            domain="example"
        )
        for gene in gene_list
    ]

    results = await asyncio.gather(*tasks)

    # Process results with consistent error handling
    gene_data = []
    for gene, (data, error) in zip(gene_list, results):
        if error:
            logger.warning(f"Failed to fetch {gene}: {error.message}")
            gene_data.append(None)
        else:
            gene_data.append(data)

    return gene_data
```

## Session Management

### Before (Reusing Client)

```python
class DataFetcher:
    def __init__(self):
        self.client = httpx.AsyncClient(timeout=30.0)

    async def fetch_many(self, urls: list[str]):
        results = []
        for url in urls:
            resp = await self.client.get(url)
            results.append(resp.json())
        return results

    async def close(self):
        await self.client.aclose()
```

### After (Connection Pooling is Automatic)

```python
class DataFetcher:
    async def fetch_many(self, urls: list[str]):
        # Connection pooling is handled automatically
        results = []
        for url in urls:
            data, error = await http_client.request_api(
                url=url,
                request={},
                domain="example"
            )
            if error:
                results.append(None)
            else:
                results.append(data)
        return results
```

## Error Type Handling

### Before (Multiple Exception Types)

```python
try:
    response = await client.get(url)
    response.raise_for_status()
    data = response.json()
except httpx.TimeoutException:
    logger.error("Request timed out")
    return None
except httpx.HTTPStatusError as e:
    logger.error(f"HTTP {e.response.status_code}")
    return None
except httpx.RequestError as e:
    logger.error(f"Request failed: {e}")
    return None
except json.JSONDecodeError:
    logger.error("Invalid JSON response")
    return None
```

### After (Unified Error Handling)

```python
data, error = await http_client.request_api(url, {}, domain="example")

if error:
    # All error types are normalized to RequestError
    logger.error(f"Request failed: {error.code} - {error.message}")
    if error.details:
        logger.debug(f"Error details: {error.details}")
    return None

return data
```

## Best Practices Summary

1. **Always check for errors**: The centralized client returns `(data, error)` tuples
2. **Use domain-specific adapters**: For complex APIs, create an adapter class
3. **Leverage built-in features**: Retry, caching, and rate limiting are automatic
4. **Register endpoints**: Add new endpoints to the registry for tracking
5. **Set appropriate cache TTLs**: Based on how often the data changes
6. **Use endpoint keys**: For better metrics and debugging
