# Integration Testing Guide

## Overview

BioMCP includes integration tests that make real API calls to external services. These tests verify that our integrations work correctly with live data but can be affected by API availability, rate limits, and data changes.

## Running Integration Tests

### Run All Tests (Including Integration)

```bash
make test
```

### Run Only Integration Tests

```bash
pytest -m integration
```

### Skip Integration Tests

```bash
pytest -m "not integration"
```

## Handling Flaky Tests

Integration tests may fail or skip for various reasons:

### 1. API Unavailability

**Symptom**: Tests skip with "API returned no data" message
**Cause**: The external service is down or experiencing issues
**Action**: Re-run tests later or check service status

### 2. Rate Limiting

**Symptom**: Multiple test failures after initial successes
**Cause**: Too many requests in a short time
**Action**: Run tests with delays between them or use API tokens

### 3. Data Changes

**Symptom**: Assertions about specific data fail
**Cause**: The external data has changed (e.g., new mutations discovered)
**Action**: Update tests to use more flexible assertions

## Test Design Principles

### 1. Graceful Skipping

Tests should skip rather than fail when:

- API returns no data
- Service is unavailable
- Rate limits are hit

Example:

```python
if not data or data.total_count == 0:
    pytest.skip("API returned no data - possible service issue")
```

### 2. Flexible Assertions

Avoid assertions on specific data values that might change:

❌ **Bad**: Expecting exact mutation counts

```python
assert summary.total_mutations == 1234
```

✅ **Good**: Checking data exists and has reasonable structure

```python
assert summary.total_mutations > 0
assert hasattr(summary, 'hotspots')
```

### 3. Retry Logic

For critical tests, implement retry with delay:

```python
async def fetch_with_retry(client, resource, max_attempts=2, delay=1.0):
    for attempt in range(max_attempts):
        result = await client.get(resource)
        if result and result.data:
            return result
        if attempt < max_attempts - 1:
            await asyncio.sleep(delay)
    return None
```

### 4. Cache Management

Clear caches before tests to ensure fresh data:

```python
from biomcp.utils.request_cache import clear_cache
await clear_cache()
```

## Common Integration Test Patterns

### Testing Search Functionality

```python
@pytest.mark.integration
async def test_gene_search(self):
    client = SearchClient()
    results = await client.search("BRAF")

    # Flexible assertions
    assert results is not None
    if results.count > 0:
        assert results.items[0].gene_symbol == "BRAF"
    else:
        pytest.skip("No results returned - API may be unavailable")
```

### Testing Data Retrieval

```python
@pytest.mark.integration
async def test_variant_details(self):
    client = VariantClient()
    variant = await client.get_variant("rs121913529")

    if not variant:
        pytest.skip("Variant not found - may have been removed from database")

    # Check structure, not specific values
    assert hasattr(variant, 'chromosome')
    assert hasattr(variant, 'position')
```

## Debugging Failed Integration Tests

### 1. Enable Debug Logging

```bash
BIOMCP_LOG_LEVEL=DEBUG pytest tests/integration/test_failing.py -v
```

### 2. Check API Status

Many services provide status pages:

- PubMed: https://www.ncbi.nlm.nih.gov/home/about/website-updates/
- ClinicalTrials.gov: https://clinicaltrials.gov/about/announcements
- cBioPortal: https://www.cbioportal.org/

### 3. Inspect Response Data

Add debugging output to see actual responses:

```python
if not expected_data:
    print(f"Unexpected response: {response}")
    pytest.skip("Data structure changed")
```

## Environment Variables for Testing

### API Tokens

Some services provide higher rate limits with authentication:

```bash
export CBIO_TOKEN="your-token-here"
export PUBMED_API_KEY="your-key-here"
```

### Offline Mode

Test offline behavior:

```bash
export BIOMCP_OFFLINE=true
pytest tests/
```

### Custom Timeouts

Adjust timeouts for slow connections:

```bash
export BIOMCP_REQUEST_TIMEOUT=60
pytest tests/integration/
```

## CI/CD Considerations

### 1. Separate Test Runs

Run integration tests separately in CI:

```yaml
- name: Unit Tests
  run: pytest -m "not integration"

- name: Integration Tests
  run: pytest -m integration
  continue-on-error: true
```

### 2. Scheduled Runs

Run integration tests on a schedule to detect API changes:

```yaml
on:
  schedule:
    - cron: "0 6 * * *" # Daily at 6 AM
```

### 3. Result Monitoring

Track integration test success rates over time to identify patterns.

## Best Practices

1. **Keep integration tests focused** - Test integration points, not business logic
2. **Use reasonable timeouts** - Don't wait forever for slow APIs
3. **Document expected failures** - Add comments explaining why tests might skip
4. **Monitor external changes** - Subscribe to API change notifications
5. **Provide escape hatches** - Allow skipping integration tests when needed
