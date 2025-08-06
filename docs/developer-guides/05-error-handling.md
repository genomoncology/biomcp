# Error Handling Guide

## Overview

BioMCP uses a consistent error handling pattern across all HTTP operations. This guide explains the error types, when they occur, and how to handle them.

## Error Structure

All HTTP operations return a tuple: `(data, error)` where one is always `None`.

```python
data, error = await http_client.request_api(...)
if error:
    # Handle error
    logger.error(f"Request failed: {error.code} - {error.message}")
else:
    # Process data
    process_result(data)
```

## Error Types

### Network Errors

- **When**: Connection timeout, DNS resolution failure, network unreachable
- **Error Code**: Various HTTP client exceptions
- **Handling**: Retry with exponential backoff or fail gracefully

### HTTP Status Errors

- **When**: Server returns 4xx or 5xx status codes
- **Error Codes**:
  - `400-499`: Client errors (bad request, unauthorized, not found)
  - `500-599`: Server errors (internal error, service unavailable)
- **Handling**:
  - 4xx: Fix request parameters or authentication
  - 5xx: Retry with backoff or use cached data

### Circuit Breaker Errors

- **When**: Too many consecutive failures to a domain
- **Error**: Circuit breaker opens to prevent cascading failures
- **Handling**: Wait for recovery timeout or use alternative data source

### Offline Mode Errors

- **When**: `BIOMCP_OFFLINE=true` and no cached data available
- **Error**: Request blocked in offline mode
- **Handling**: Use cached data only or inform user about offline status

### Parse Errors

- **When**: Response is not valid JSON or doesn't match expected schema
- **Error**: JSON decode error or validation error
- **Handling**: Log error and treat as service issue

## Best Practices

### 1. Always Check Errors

```python
# ❌ Bad - ignoring error
data, _ = await http_client.request_api(...)
process(data)  # data might be None!

# ✅ Good - checking error
data, error = await http_client.request_api(...)
if error:
    logger.warning(f"Failed to fetch data: {error}")
    return None
process(data)
```

### 2. Provide Context in Error Messages

```python
# ❌ Bad - generic error
if error:
    logger.error("Request failed")

# ✅ Good - contextual error
if error:
    logger.error(f"Failed to fetch gene {gene_id} from cBioPortal: {error.message}")
```

### 3. Graceful Degradation

```python
async def get_variant_with_fallback(variant_id: str):
    # Try primary source
    data, error = await primary_source.get_variant(variant_id)
    if not error:
        return data

    logger.warning(f"Primary source failed: {error}, trying secondary")

    # Try secondary source
    data, error = await secondary_source.get_variant(variant_id)
    if not error:
        return data

    # Use cached data as last resort
    return get_cached_variant(variant_id)
```

### 4. User-Friendly Error Messages

```python
def format_error_for_user(error: RequestError) -> str:
    if error.code >= 500:
        return "The service is temporarily unavailable. Please try again later."
    elif error.code == 404:
        return "The requested data was not found."
    elif error.code == 401:
        return "Authentication required. Please check your credentials."
    elif "OFFLINE" in str(error):
        return "You are in offline mode. Only cached data is available."
    else:
        return "An error occurred while fetching data. Please try again."
```

## Testing Error Conditions

### 1. Simulate Network Errors

```python
with patch("biomcp.http_client.call_http") as mock:
    mock.side_effect = Exception("Network error")
    data, error = await client.fetch_data()
    assert error is not None
    assert data is None
```

### 2. Test Circuit Breaker

```python
# Simulate multiple failures
for _ in range(5):
    with patch("biomcp.http_client.call_http") as mock:
        mock.return_value = (500, "Server Error")
        await client.fetch_data()

# Circuit should be open
data, error = await client.fetch_data()
assert error is not None
assert "circuit" in error.message.lower()
```

### 3. Test Offline Mode

```python
with patch.dict(os.environ, {"BIOMCP_OFFLINE": "true"}):
    data, error = await client.fetch_data()
    # Should only return cached data or error
```

## Common Patterns

### Retry with Backoff

The centralized HTTP client automatically retries with exponential backoff for:

- Network errors
- 5xx server errors
- Rate limit errors (429)

### Caching

Failed requests don't overwrite cached data, ensuring availability during outages.

### Rate Limiting

Requests are automatically rate-limited per domain to prevent overwhelming services.

## Debugging

Enable debug logging to see all HTTP requests and errors:

```python
import logging
logging.getLogger("biomcp.http_client").setLevel(logging.DEBUG)
```

This will show:

- All HTTP requests with URLs and methods
- Response status codes and times
- Error details and retry attempts
- Circuit breaker state changes
