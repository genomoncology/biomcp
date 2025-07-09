# BioMCP Test Suite Performance Analysis

## Executive Summary

The BioMCP test suite consists of 422 tests (389 unit tests + 33 integration tests). The total runtime is approximately 66 seconds (41.35s for unit tests + 24.67s for integration tests). While this is reasonable for a comprehensive test suite, there are several opportunities for optimization.

## Slowest Tests Identified

### Unit Tests (Top 10 Slowest)

1. **5.18s** - `test_search_articles_unified_markdown_output` (test_unified.py)
2. **3.24s** - `test_unified_search_execution` (test_router.py)
3. **2.28s** - `test_pten_r173_search_limitations` (test_pten_r173_search.py)
4. **2.22s** - `test_specific_pten_papers_not_found` (test_pten_r173_search.py)
5. **2.11s** - `test_search_variants_complex` (variants/test_search.py)
6. **1.91s** - `test_mcp_unified_query_integration` (test_mcp_integration.py)
7. **1.77s** - `test_search` (articles/test_search.py)
8. **1.76s** - `test_search_variants_with_limit` (variants/test_search.py)
9. **1.74s** - `test_search_variants_no_results` (variants/test_search.py)
10. **1.10s** - `test_rate_limit_replenishment` (utils/test_rate_limiter.py)

### Integration Tests (Top 5 Slowest)

1. **4.22s** - `test_search_braf_v600e` (cBioPortal mutation search)
2. **3.88s** - `test_search_specific_mutation_srsf2_f57y` (cBioPortal mutation search)
3. **1.99s** - `test_aggregate_all_sources` (External variant aggregator)
4. **1.95s** - `test_variant_population_frequencies` (1000 Genomes)
5. **1.66s** - `test_error_handling_resilience` (External variant aggregator)

## Performance Bottleneck Categories

### 1. **Complex Mock Setup Tests** (5-3 seconds)

- Tests that set up multiple mocks and complex test data
- Example: `test_search_articles_unified_markdown_output`
- These tests mock multiple services (PubMed, preprints, cBioPortal)

### 2. **Search Simulation Tests** (2-3 seconds)

- Tests that simulate full search workflows
- Example: `test_unified_search_execution`, `test_pten_r173_search_limitations`
- These likely process large amounts of test data

### 3. **Real API Integration Tests** (1-4 seconds)

- Tests marked with `@pytest.mark.integration` that make actual API calls
- Examples: cBioPortal, 1000 Genomes, TCGA integration tests
- Network latency and API response times contribute to slowness

### 4. **Rate Limiter Tests** (1+ second)

- Tests that use `asyncio.sleep()` to test timing behavior
- Examples: `test_rate_limit_replenishment`, `test_wait_if_needed`
- These tests intentionally wait to verify rate limiting

### 5. **Variant Search Tests** (1-2 seconds)

- Complex variant search tests with multiple filters
- Processing and filtering large result sets

## Optimization Recommendations

### 1. **Parallelize Integration Tests**

```bash
# Use pytest-xdist for parallel test execution
uv add pytest-xdist --dev
pytest -n auto -m "integration"
```

### 2. **Mock External Services More Efficiently**

- Create shared fixtures for common mock setups
- Use `pytest.fixture(scope="module")` for expensive mock data
- Example refactoring:

```python
@pytest.fixture(scope="module")
def pubmed_mock_data():
    # Load once, use many times
    return load_test_data("pubmed_results.json")
```

### 3. **Reduce Sleep Times in Rate Limiter Tests**

- Use time mocking instead of real sleeps
- Example using `freezegun` or `time-machine`:

```python
from time_machine import travel

@travel("+1.1s")
async def test_rate_limit_replenishment():
    # Test without actual sleep
```

### 4. **Cache Test Data**

- Load large test datasets once per session
- Use `pytest.fixture(scope="session")` for shared test data

### 5. **Split Integration Tests**

- Create separate test suites for different external services
- Run critical path tests frequently, comprehensive tests in CI only

```bash
# Quick tests
pytest -m "not integration and not slow"

# Full suite in CI
pytest
```

### 6. **Add Test Timing Markers**

```python
# Mark slow tests explicitly
@pytest.mark.slow
@pytest.mark.timeout(30)
async def test_comprehensive_search():
    pass
```

### 7. **Profile Individual Slow Tests**

```bash
# Profile specific test
pytest tests/tdd/articles/test_unified.py::TestUnifiedSearch::test_search_articles_unified_markdown_output --profile
```

## Quick Wins

1. **Immediate**: Add pytest-xdist for parallel execution (potential 40-50% reduction)
2. **Short-term**: Mock time-based tests instead of using real sleeps
3. **Medium-term**: Refactor complex mock setups into shared fixtures
4. **Long-term**: Consider test data factories for generating test data on-demand

## Monitoring Test Performance

Add to CI/CD pipeline:

```yaml
- name: Run tests with timing
  run: |
    pytest --durations=20 --junit-xml=test-results.xml
    # Alert if any test takes > 5 seconds
```

## Conclusion

The test suite is well-structured with clear separation between unit and integration tests. The main performance bottlenecks are:

1. Complex mock setups (can be optimized with better fixtures)
2. Real API calls in integration tests (necessary but can be parallelized)
3. Time-based tests (can be mocked)

Implementing the recommended optimizations could reduce total test time by 30-50% while maintaining test coverage and reliability.
