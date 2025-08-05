# Contributing and Testing Guide

This guide covers how to contribute to BioMCP and run the comprehensive test suite.

## Getting Started

### Prerequisites

- Python 3.10 or higher
- [uv](https://docs.astral.sh/uv/) package manager
- Git
- Node.js (for MCP Inspector)

### Initial Setup

1. **Fork and clone the repository:**

```bash
git clone https://github.com/YOUR_USERNAME/biomcp.git
cd biomcp
```

2. **Install dependencies and setup:**

```bash
# Recommended: Use make for complete setup
make install

# Alternative: Manual setup
uv sync --all-extras
uv run pre-commit install
```

3. **Verify installation:**

```bash
# Run server
biomcp run

# Run tests
make test-offline
```

## Development Workflow

### 1. Create Feature Branch

```bash
git checkout -b feature/your-feature-name
```

### 2. Make Changes

Follow these principles:

- **Keep changes minimal and focused**
- **Follow existing code patterns**
- **Add tests for new functionality**
- **Update documentation as needed**

### 3. Quality Checks

**MANDATORY: Run these before considering work complete:**

```bash
# Step 1: Code quality checks
make check

# This runs:
# - ruff check (linting)
# - ruff format (code formatting)
# - mypy (type checking)
# - pre-commit hooks
# - deptry (dependency analysis)
```

### 4. Run Tests

```bash
# Step 2: Run appropriate test suite
make test          # Full suite (requires network)
# OR
make test-offline  # Unit tests only (no network)
```

**Both quality checks and tests MUST pass before submitting changes.**

## Testing Strategy

### Test Categories

#### Unit Tests

- Fast, reliable tests without external dependencies
- Mock all external API calls
- Always run in CI/CD

```python
# Example unit test
@patch('httpx.AsyncClient.get')
async def test_article_search(mock_get):
    mock_get.return_value.json.return_value = {"results": [...]}
    result = await article_searcher(genes=["BRAF"])
    assert len(result) > 0
```

#### Integration Tests

- Test real API interactions
- May fail due to network/API issues
- Run separately in CI with `continue-on-error`

```python
# Example integration test
@pytest.mark.integration
async def test_real_pubmed_search():
    result = await article_searcher(genes=["TP53"], limit=5)
    assert len(result) == 5
    assert all("TP53" in r.text for r in result)
```

### Running Tests

#### Command Options

```bash
# Run all tests
make test
uv run python -m pytest

# Run only unit tests (fast, offline)
make test-offline
uv run python -m pytest -m "not integration"

# Run only integration tests
uv run python -m pytest -m "integration"

# Run specific test file
uv run python -m pytest tests/tdd/test_article_search.py

# Run with coverage
make cov
uv run python -m pytest --cov --cov-report=html

# Run tests verbosely
uv run python -m pytest -v

# Run tests and stop on first failure
uv run python -m pytest -x
```

#### Test Discovery

Tests are organized in:

- `tests/tdd/` - Unit and integration tests
- `tests/bdd/` - Behavior-driven development tests
- `tests/data/` - Test fixtures and sample data

### Writing Tests

#### Test Structure

```python
import pytest
from unittest.mock import patch, AsyncMock
from biomcp.articles import article_searcher

class TestArticleSearch:
    """Test article search functionality"""

    @pytest.fixture
    def mock_response(self):
        """Sample API response"""
        return {
            "results": [
                {"pmid": "12345", "title": "BRAF in melanoma"}
            ]
        }

    @patch('httpx.AsyncClient.get')
    async def test_basic_search(self, mock_get, mock_response):
        """Test basic article search"""
        # Setup
        mock_get.return_value = AsyncMock()
        mock_get.return_value.json.return_value = mock_response

        # Execute
        result = await article_searcher(genes=["BRAF"])

        # Assert
        assert len(result) == 1
        assert "BRAF" in result[0].title
```

#### Async Testing

```python
import pytest
import asyncio

@pytest.mark.asyncio
async def test_async_function():
    """Test async functionality"""
    result = await some_async_function()
    assert result is not None

# Or use pytest-asyncio fixtures
@pytest.fixture
async def async_client():
    async with AsyncClient() as client:
        yield client
```

#### Mocking External APIs

```python
from unittest.mock import patch, MagicMock

@patch('biomcp.integrations.pubmed.search')
def test_with_mock(mock_search):
    # Configure mock
    mock_search.return_value = [{
        "pmid": "12345",
        "title": "Test Article"
    }]

    # Test code that uses the mocked function
    result = search_articles("BRAF")

    # Verify mock was called correctly
    mock_search.assert_called_once_with("BRAF")
```

## MCP Inspector Testing

The MCP Inspector provides an interactive way to test MCP tools.

### Setup

```bash
# Install inspector
npm install -g @modelcontextprotocol/inspector

# Run BioMCP with inspector
make inspector
# OR
npx @modelcontextprotocol/inspector uv run --with biomcp-python biomcp run
```

### Testing Tools

1. **Connect to server** in the inspector UI
2. **View available tools** in the tools panel
3. **Test individual tools** with sample inputs

#### Example Tool Tests

```javascript
// Test article search
{
  "tool": "article_searcher",
  "arguments": {
    "genes": ["BRAF"],
    "diseases": ["melanoma"],
    "limit": 5
  }
}

// Test trial search
{
  "tool": "trial_searcher",
  "arguments": {
    "conditions": ["lung cancer"],
    "recruiting_status": "OPEN",
    "limit": 10
  }
}

// Test think tool (ALWAYS first!)
{
  "tool": "think",
  "arguments": {
    "thought": "Planning to search for BRAF mutations",
    "thoughtNumber": 1,
    "nextThoughtNeeded": true
  }
}
```

### Debugging with Inspector

1. **Check request/response**: View raw MCP messages
2. **Verify parameters**: Ensure correct argument format
3. **Test error handling**: Try invalid inputs
4. **Monitor performance**: Check response times

## Code Style and Standards

### Python Style

- **Formatter**: ruff (line length: 79)
- **Type hints**: Required for all functions
- **Docstrings**: Google style for all public functions

```python
def search_articles(
    genes: list[str],
    limit: int = 10
) -> list[Article]:
    """Search for articles by gene names.

    Args:
        genes: List of gene symbols to search
        limit: Maximum number of results

    Returns:
        List of Article objects

    Raises:
        ValueError: If genes list is empty
    """
    if not genes:
        raise ValueError("Genes list cannot be empty")
    # Implementation...
```

### Pre-commit Hooks

Automatically run on commit:

- ruff formatting
- ruff linting
- mypy type checking
- File checks (YAML, TOML, merge conflicts)

Manual run:

```bash
uv run pre-commit run --all-files
```

## Continuous Integration

### GitHub Actions Workflow

The CI pipeline runs:

1. **Linting and Formatting**
2. **Type Checking**
3. **Unit Tests** (required to pass)
4. **Integration Tests** (allowed to fail)
5. **Coverage Report**

### CI Configuration

```yaml
# .github/workflows/test.yml structure
jobs:
  test:
    strategy:
      matrix:
        python-version: ["3.10", "3.11", "3.12"]
    steps:
      - uses: actions/checkout@v4
      - uses: astral-sh/setup-uv@v2
      - run: make check
      - run: make test-offline
```

## Debugging and Troubleshooting

### Common Issues

#### Test Failures

```bash
# Run failed test with more details
uv run python -m pytest -vvs tests/path/to/test.py::test_name

# Debug with print statements
uv run python -m pytest -s  # Don't capture stdout

# Use debugger
uv run python -m pytest --pdb  # Drop to debugger on failure
```

#### Integration Test Issues

Common causes:

- **Rate limiting**: Add delays or use mocks
- **API changes**: Update test expectations
- **Network issues**: Check connectivity
- **API keys**: Ensure valid keys for NCI tests

#### Type Checking Errors

```bash
# Check specific file
uv run mypy src/biomcp/specific_file.py

# Ignore specific error
# type: ignore[error-code]

# Show error codes
uv run mypy --show-error-codes
```

### Performance Testing

```python
import time
import pytest

@pytest.mark.performance
def test_search_performance():
    """Ensure search completes within time limit"""
    start = time.time()
    result = search_articles("TP53", limit=100)
    duration = time.time() - start

    assert duration < 5.0  # Should complete in 5 seconds
    assert len(result) == 100
```

## Submitting Changes

### Pull Request Process

1. **Ensure all checks pass:**

```bash
make check && make test
```

2. **Update documentation** if needed

3. **Commit with clear message:**

```bash
git add .
git commit -m "feat: add support for variant batch queries

- Add batch_variant_search function
- Update tests for batch functionality
- Document batch size limits"
```

4. **Push to your fork:**

```bash
git push origin feature/your-feature-name
```

5. **Create Pull Request** with:
   - Clear description of changes
   - Link to related issues
   - Test results summary

### Code Review Guidelines

Your PR will be reviewed for:

- **Code quality** and style consistency
- **Test coverage** for new features
- **Documentation** updates
- **Performance** impact
- **Security** considerations

## Best Practices

### DO:

- Write tests for new functionality
- Follow existing patterns
- Keep PRs focused and small
- Update documentation
- Run full test suite locally

### DON'T:

- Skip tests to "save time"
- Mix unrelated changes in one PR
- Ignore linting warnings
- Commit sensitive data
- Break existing functionality

## Additional Resources

- [MCP Documentation](https://modelcontextprotocol.org)
- [pytest Documentation](https://docs.pytest.org)
- [Type Hints Guide](https://mypy.readthedocs.io)
- [Ruff Documentation](https://docs.astral.sh/ruff)

## Getting Help

- **GitHub Issues**: Report bugs or request features
- **Discussions**: Ask questions or share ideas
- **Pull Requests**: Submit contributions
- **Documentation**: Check existing docs first

Remember: Quality over speed. Take time to write good tests and clean code!
