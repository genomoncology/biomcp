# Testing Guide

## Running Tests

### All Tests (Local Development)

```bash
make test
```

### Unit Tests Only (Excluding Integration Tests)

```bash
uv run python -m pytest tests -m "not integration"
```

### Integration Tests Only

```bash
uv run python -m pytest tests -m "integration"
```

## Test Categories

### Unit Tests

- Run without external API calls
- Fast and reliable
- Always run in CI

### Integration Tests

- Make real API calls to external services (PubMed, cBioPortal, etc.)
- Marked with `@pytest.mark.integration`
- May fail due to:
  - Network issues
  - API rate limiting
  - Service availability
- Run separately in CI with `continue-on-error: true`

## CI/CD Testing Strategy

1. **Pull Requests & Main Branch**:

   - Run unit tests only (`-m "not integration"`)
   - Ensures fast, reliable CI runs
   - Prevents flaky test failures

2. **Integration Tests**:
   - Run in a separate optional job
   - Allowed to fail without blocking CI
   - Useful for monitoring API availability

## Writing New Tests

### Unit Test Example

```python
@pytest.mark.asyncio
async def test_search_articles():
    """Test article search functionality."""
    # Mock external API calls
    with patch("biomcp.articles.search.fetch_articles") as mock_fetch:
        mock_fetch.return_value = {"results": [...]}
        # Test code here
```

### Integration Test Example

```python
@pytest.mark.asyncio
@pytest.mark.integration  # This marks it as an integration test
async def test_real_api_call():
    """Test real API interaction."""
    # Makes actual API calls
    result = await fetch_from_pubmed("BRAF")
    assert result is not None
```

## Troubleshooting Test Failures

### Integration Test Failures

If integration tests fail in CI:

1. Check if the external API is available
2. Look for rate limiting messages
3. Verify network connectivity
4. These failures don't block merges

### Running Tests with Coverage

```bash
make cov
```

This generates an HTML coverage report in `htmlcov/`.
