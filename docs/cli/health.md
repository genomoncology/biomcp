# Health Check CLI Documentation

The Health Check CLI module provides commands for checking the health of API endpoints and system resources used by BioMCP.

> **Tip**: Use the `--help` flag with any command (e.g., `biomcp health check --help`) to see the most up-to-date options directly from the tool.

## Check Command (`check`)

Run a comprehensive health check on API endpoints and system resources.

### Usage

```bash
biomcp health check [OPTIONS]
```

#### Options

- `--api-only`: Check only API endpoints. [default: False]
- `--system-only`: Check only system health. [default: False]
- `-v, --verbose`: Show detailed error information and API responses. [default: False]
- `--help`: Show help message and exit.

### API Endpoints Checked

The health check command tests connectivity and responses from all external APIs that BioMCP depends on:

1. **PubTator3 API**:
   - Autocomplete endpoint
   - Publications export endpoint
   - Search endpoint

2. **ClinicalTrials.gov API**:
   - Studies search endpoint
   - Individual study retrieval endpoint

3. **MyVariant.info API**:
   - Query endpoint
   - Variant retrieval endpoint

### System Health Checks

When checking system health, the command evaluates:

- **Network connectivity**: Tests basic internet connectivity
- **System resources**: Monitors CPU usage, memory availability, and disk space
- **Python environment**: Reports Python version and critical dependencies

> **Note**: For full system resource checks, the `psutil` package is required. If not installed, the command will still run but will indicate that `psutil` is missing.

### Examples

Run a complete health check (API endpoints and system resources):

```bash
biomcp health check
```

Check only API endpoints:

```bash
biomcp health check --api-only
```

Check only system resources:

```bash
biomcp health check --system-only
```

Show detailed error information for any failing checks:

```bash
biomcp health check --verbose
```

Combine options as needed:

```bash
biomcp health check --api-only --verbose
```

### Output

The command displays results in formatted tables:

1. **API Endpoints Health**: Shows the status of each API endpoint (200 OK or error code)
2. **System Resources**: Displays CPU, memory, and disk usage statistics
3. **Network & Environment**: Shows network connectivity status and Python environment details

In verbose mode, detailed error information is displayed for any failing endpoints, which can help diagnose API-related issues.

### Exit Status

The command provides a summary of overall health status:
- "✓ All systems operational!" when all checks pass
- "⚠ Some health checks failed." when one or more checks fail
