# Error Codes Reference

This document provides a comprehensive list of error codes returned by BioMCP APIs, their meanings, and recommended actions.

## HTTP Status Codes

### Success Codes (2xx)

| Code | Status     | Description                              |
| ---- | ---------- | ---------------------------------------- |
| 200  | OK         | Request successful                       |
| 201  | Created    | Resource created successfully            |
| 204  | No Content | Request successful, no content to return |

### Client Error Codes (4xx)

| Code | Status               | Description                | Action                                 |
| ---- | -------------------- | -------------------------- | -------------------------------------- |
| 400  | Bad Request          | Invalid request parameters | Check parameter format and values      |
| 401  | Unauthorized         | Missing or invalid API key | Verify API key is correct              |
| 403  | Forbidden            | Access denied to resource  | Check permissions for API key          |
| 404  | Not Found            | Resource not found         | Verify ID exists and is correct format |
| 409  | Conflict             | Resource conflict          | Check for duplicate requests           |
| 422  | Unprocessable Entity | Validation error           | Review validation errors in response   |
| 429  | Too Many Requests    | Rate limit exceeded        | Implement backoff and retry            |

### Server Error Codes (5xx)

| Code | Status                | Description                     | Action                            |
| ---- | --------------------- | ------------------------------- | --------------------------------- |
| 500  | Internal Server Error | Server error                    | Retry with exponential backoff    |
| 502  | Bad Gateway           | Upstream service error          | Wait and retry                    |
| 503  | Service Unavailable   | Service temporarily unavailable | Check service status, retry later |
| 504  | Gateway Timeout       | Request timeout                 | Retry with smaller request        |

## BioMCP-Specific Error Codes

### Article Errors (1xxx)

| Code | Error                | Description                 | Example                        |
| ---- | -------------------- | --------------------------- | ------------------------------ |
| 1001 | INVALID_PMID         | Invalid PubMed ID format    | "abc123" instead of "12345678" |
| 1002 | ARTICLE_NOT_FOUND    | Article does not exist      | PMID not in PubMed             |
| 1003 | DOI_NOT_FOUND        | DOI cannot be resolved      | Invalid or non-existent DOI    |
| 1004 | PUBTATOR_ERROR       | PubTator3 annotation failed | Service temporarily down       |
| 1005 | PREPRINT_NOT_INDEXED | Preprint not yet indexed    | Recently submitted preprint    |

### Trial Errors (2xxx)

| Code | Error            | Description                    | Example                      |
| ---- | ---------------- | ------------------------------ | ---------------------------- |
| 2001 | INVALID_NCT_ID   | Invalid NCT ID format          | Missing "NCT" prefix         |
| 2002 | TRIAL_NOT_FOUND  | Trial does not exist           | NCT ID not registered        |
| 2003 | INVALID_LOCATION | Invalid geographic coordinates | Latitude > 90                |
| 2004 | NCI_API_REQUIRED | NCI API key required           | Using NCI source without key |
| 2005 | INVALID_STATUS   | Invalid trial status           | Status not recognized        |

### Variant Errors (3xxx)

| Code | Error                | Description                       | Example                |
| ---- | -------------------- | --------------------------------- | ---------------------- |
| 3001 | INVALID_HGVS         | Invalid HGVS notation             | Malformed HGVS string  |
| 3002 | VARIANT_NOT_FOUND    | Variant not in database           | Novel variant          |
| 3003 | INVALID_ASSEMBLY     | Invalid genome assembly           | Not hg19 or hg38       |
| 3004 | COORDINATE_MISMATCH  | Coordinates don't match reference | Position out of range  |
| 3005 | ALPHAGENOME_REQUIRED | AlphaGenome API key required      | Prediction without key |

### Gene/Drug/Disease Errors (4xxx)

| Code | Error                 | Description                 | Example                  |
| ---- | --------------------- | --------------------------- | ------------------------ |
| 4001 | GENE_NOT_FOUND        | Gene symbol not recognized  | Non-standard symbol      |
| 4002 | DRUG_NOT_FOUND        | Drug/chemical not found     | Misspelled drug name     |
| 4003 | DISEASE_NOT_FOUND     | Disease term not recognized | Non-standard terminology |
| 4004 | SPECIES_NOT_SUPPORTED | Only human genes supported  | Requesting mouse gene    |
| 4005 | AMBIGUOUS_QUERY       | Multiple matches found      | Common drug name         |

### Authentication Errors (5xxx)

| Code | Error                    | Description                        | Action              |
| ---- | ------------------------ | ---------------------------------- | ------------------- |
| 5001 | API_KEY_INVALID          | API key format invalid             | Check key format    |
| 5002 | API_KEY_EXPIRED          | API key has expired                | Renew API key       |
| 5003 | API_KEY_REVOKED          | API key was revoked                | Contact support     |
| 5004 | INSUFFICIENT_PERMISSIONS | API key lacks required permissions | Upgrade API key     |
| 5005 | IP_NOT_ALLOWED           | IP address not whitelisted         | Add IP to whitelist |

### Rate Limit Errors (6xxx)

| Code | Error                | Description                  | Headers                      |
| ---- | -------------------- | ---------------------------- | ---------------------------- |
| 6001 | RATE_LIMIT_EXCEEDED  | Too many requests            | X-RateLimit-Remaining: 0     |
| 6002 | DAILY_LIMIT_EXCEEDED | Daily quota exceeded         | X-RateLimit-Reset: timestamp |
| 6003 | CONCURRENT_LIMIT     | Too many concurrent requests | X-Concurrent-Limit: 10       |
| 6004 | BURST_LIMIT_EXCEEDED | Short-term rate limit        | Retry-After: 60              |

### Validation Errors (7xxx)

| Code | Error                  | Description                 | Example                         |
| ---- | ---------------------- | --------------------------- | ------------------------------- |
| 7001 | MISSING_REQUIRED_FIELD | Required parameter missing  | Missing gene for variant search |
| 7002 | INVALID_FIELD_TYPE     | Wrong parameter type        | String instead of integer       |
| 7003 | VALUE_OUT_OF_RANGE     | Value outside allowed range | Page number < 1                 |
| 7004 | INVALID_ENUM_VALUE     | Invalid enumeration value   | Phase "PHASE5"                  |
| 7005 | MUTUALLY_EXCLUSIVE     | Conflicting parameters      | Both PMID and DOI provided      |

### External Service Errors (8xxx)

| Code | Error                      | Description              | Service          |
| ---- | -------------------------- | ------------------------ | ---------------- |
| 8001 | PUBMED_UNAVAILABLE         | PubMed API down          | NCBI E-utilities |
| 8002 | CLINICALTRIALS_UNAVAILABLE | ClinicalTrials.gov down  | CT.gov API       |
| 8003 | BIOTHINGS_UNAVAILABLE      | BioThings API down       | MyGene/MyVariant |
| 8004 | CBIOPORTAL_UNAVAILABLE     | cBioPortal unavailable   | cBioPortal API   |
| 8005 | EXTERNAL_TIMEOUT           | External service timeout | Any external API |

## Error Response Format

### Standard Error Response

```json
{
  "error": {
    "code": 1002,
    "type": "ARTICLE_NOT_FOUND",
    "message": "Article with PMID 99999999 not found",
    "details": {
      "pmid": "99999999",
      "searched_in": ["pubmed", "pmc", "preprints"]
    }
  },
  "request_id": "req_abc123",
  "timestamp": "2024-03-15T10:30:00Z"
}
```

### Validation Error Response

```json
{
  "error": {
    "code": 7001,
    "type": "MISSING_REQUIRED_FIELD",
    "message": "Validation failed",
    "details": {
      "errors": [
        {
          "field": "gene",
          "message": "Gene symbol is required for variant search"
        },
        {
          "field": "assembly",
          "message": "Assembly must be 'hg19' or 'hg38'"
        }
      ]
    }
  }
}
```

### Rate Limit Error Response

```json
{
  "error": {
    "code": 6001,
    "type": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit of 180 requests per minute exceeded",
    "details": {
      "limit": 180,
      "remaining": 0,
      "reset": 1710504000,
      "retry_after": 45
    }
  },
  "headers": {
    "X-RateLimit-Limit": "180",
    "X-RateLimit-Remaining": "0",
    "X-RateLimit-Reset": "1710504000",
    "Retry-After": "45"
  }
}
```

## Error Handling Best Practices

### 1. Implement Exponential Backoff

```python
import time
import random

def exponential_backoff(attempt: int, base_delay: float = 1.0):
    """Calculate exponential backoff with jitter."""
    delay = base_delay * (2 ** attempt)
    jitter = random.uniform(0, delay * 0.1)
    return delay + jitter

# Usage
for attempt in range(5):
    try:
        response = await client.search(...)
        break
    except RateLimitError:
        delay = exponential_backoff(attempt)
        time.sleep(delay)
```

### 2. Handle Specific Error Types

```python
try:
    article = await client.articles.get(pmid)
except BioMCPError as e:
    if e.code == 1002:  # ARTICLE_NOT_FOUND
        # Try alternative sources
        article = await search_preprints(pmid)
    elif e.code == 6001:  # RATE_LIMIT_EXCEEDED
        # Wait and retry
        time.sleep(e.retry_after)
        article = await client.articles.get(pmid)
    else:
        # Log and re-raise
        logger.error(f"Unexpected error: {e}")
        raise
```

### 3. Parse Error Details

```python
def handle_validation_error(error_response):
    """Extract and handle validation errors."""
    if error_response["error"]["type"] == "VALIDATION_ERROR":
        for error in error_response["error"]["details"]["errors"]:
            field = error["field"]
            message = error["message"]
            print(f"Validation error on {field}: {message}")
```

### 4. Monitor Rate Limits

```python
class RateLimitMonitor:
    def __init__(self):
        self.limits = {}

    def update_from_headers(self, headers):
        """Update rate limit state from response headers."""
        self.limits["remaining"] = int(headers.get("X-RateLimit-Remaining", 0))
        self.limits["reset"] = int(headers.get("X-RateLimit-Reset", 0))

        if self.limits["remaining"] < 10:
            logger.warning(f"Rate limit low: {self.limits['remaining']} remaining")

    def should_delay(self):
        """Check if we should delay before next request."""
        return self.limits.get("remaining", 100) < 5
```

## Common Error Scenarios

### Scenario 1: Gene Symbol Not Found

**Error:**

```json
{
  "error": {
    "code": 4001,
    "type": "GENE_NOT_FOUND",
    "message": "Gene symbol 'HER2' not found. Did you mean 'ERBB2'?",
    "details": {
      "query": "HER2",
      "suggestions": ["ERBB2", "ERBB2IP"]
    }
  }
}
```

**Solution:**

```python
try:
    gene = await client.genes.get("HER2")
except GeneNotFoundError as e:
    if e.suggestions:
        # Try first suggestion
        gene = await client.genes.get(e.suggestions[0])
```

### Scenario 2: Location Search Without Coordinates

**Error:**

```json
{
  "error": {
    "code": 7001,
    "type": "MISSING_REQUIRED_FIELD",
    "message": "Latitude and longitude required for location search",
    "details": {
      "hint": "Use geocoding service to convert city names to coordinates"
    }
  }
}
```

**Solution:**

```python
# Use a geocoding service first
coords = await geocode("Boston, MA")
trials = await client.trials.search(
    conditions=["cancer"],
    lat=coords.lat,
    long=coords.long,
    distance=50
)
```

### Scenario 3: API Key Required

**Error:**

```json
{
  "error": {
    "code": 2004,
    "type": "NCI_API_REQUIRED",
    "message": "NCI API key required for this operation",
    "details": {
      "get_key_url": "https://api.cancer.gov",
      "feature": "biomarker_search"
    }
  }
}
```

**Solution:**

```python
# Initialize client with API key
client = BioMCPClient(nci_api_key=os.getenv("NCI_API_KEY"))

# Or provide per-request
trials = await client.trials.search(
    source="nci",
    conditions=["melanoma"],
    api_key="your-nci-key"
)
```

## Debugging Tips

### 1. Enable Debug Logging

```python
import logging

logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger("biomcp")
```

### 2. Inspect Raw Responses

```python
# Enable raw response mode
client = BioMCPClient(debug=True)

# Access raw response
response = await client.articles.search(genes=["BRAF"])
print(response.raw_response)
```

### 3. Capture Request IDs

```python
try:
    result = await client.search(...)
except BioMCPError as e:
    print(f"Request ID: {e.request_id}")
    # Include request_id when reporting issues
```

## Support

For error codes not listed here or persistent issues:

1. Check [FAQ](../faq-condensed.md) for common issues
2. Search [GitHub Issues](https://github.com/genomoncology/biomcp/issues)
3. Report new issues with:
   - Error code and message
   - Request ID if available
   - Minimal code to reproduce
   - BioMCP version
