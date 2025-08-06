# Troubleshooting Guide

This guide helps you resolve common issues with BioMCP installation, configuration, and usage.

## Installation Issues

### Prerequisites Not Met

**macOS:**

```bash
# Install uv (recommended)
brew install uv

# Or using the official installer
curl -LsSf https://astral.sh/uv/install.sh | sh

# Install Node.js for npx (if needed)
brew install node
```

**Linux:**

```bash
# Install uv
curl -LsSf https://astral.sh/uv/install.sh | sh

# Install Node.js
curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -
sudo apt-get install -y nodejs
```

**Windows:**

```powershell
# Install uv
powershell -c "irm https://astral.sh/uv/install.ps1 | iex"

# Install Node.js from https://nodejs.org
```

### "Command not found" Error

After installing BioMCP, if you get "command not found":

1. **Restart your terminal** - PATH updates require a new session

2. **Check installation location:**

   ```bash
   # For uv tool install
   ls ~/.local/bin/biomcp

   # For pip install
   which biomcp
   ```

3. **Add to PATH manually:**

   ```bash
   # Add to ~/.bashrc or ~/.zshrc
   export PATH="$HOME/.local/bin:$PATH"
   ```

4. **Reinstall with force:**

   ```bash
   uv tool install biomcp --force
   ```

5. **Use full path:**
   ```bash
   ~/.local/bin/biomcp --version
   ```

### Python Version Issues

BioMCP requires Python 3.10 or higher:

```bash
# Check Python version
python --version

# If too old, install newer version
# macOS
brew install python@3.11

# Linux
sudo apt update
sudo apt install python3.11

# Use pyenv for version management
pyenv install 3.11.8
pyenv local 3.11.8
```

## Configuration Issues

### API Key Not Working

**Environment Variable Not Set:**

```bash
# Check if set
echo $NCI_API_KEY

# Set temporarily
export NCI_API_KEY="your-key-here"

# Set permanently in ~/.bashrc or ~/.zshrc
echo 'export NCI_API_KEY="your-key-here"' >> ~/.bashrc
source ~/.bashrc
```

**Wrong API Key Format:**

- NCI keys: Should be 36 characters (UUID format)
- AlphaGenome: Alphanumeric string
- cBioPortal: JWT token format

**API Key Permissions:**

```bash
# Test NCI API key
biomcp health check --verbose

# Test specific API
curl -H "X-API-KEY: $NCI_API_KEY" \
  "https://cts.nlm.nih.gov/api/v2/trials?size=1"
```

### SSL Certificate Errors

**Update certificates:**

```bash
# Python certificates
pip install --upgrade certifi

# System certificates (macOS)
brew install ca-certificates

# System certificates (Linux)
sudo apt-get update
sudo apt-get install ca-certificates
```

**Corporate proxy issues:**

```bash
# Set proxy environment variables
export HTTP_PROXY="http://proxy.company.com:8080"
export HTTPS_PROXY="http://proxy.company.com:8080"
export NO_PROXY="localhost,127.0.0.1"

# Configure pip for proxy
pip config set global.proxy http://proxy.company.com:8080
```

## Search Issues

### No Results Found

**1. Check gene symbol:**

```bash
# Wrong: common names
biomcp article search --gene HER2  # ❌

# Correct: official HGNC symbol
biomcp article search --gene ERBB2  # ✅

# Find correct symbol
biomcp gene get HER2  # Will suggest ERBB2
```

**2. Too restrictive filters:**

```bash
# Too specific - may return nothing
biomcp article search --gene BRAF --disease "stage IV melanoma" \
  --chemical "dabrafenib and trametinib combination"

# Better - broader search
biomcp article search --gene BRAF --disease melanoma \
  --keyword "dabrafenib trametinib"
```

**3. Check data availability:**

```bash
# Test if gene exists in database
biomcp gene get YOUR_GENE

# Test if disease term is recognized
biomcp disease get "your disease term"
```

### Location Search Not Working

Location searches require coordinates:

```bash
# Wrong - city name only
biomcp trial search --condition cancer --city "New York"  # ❌

# Correct - with coordinates
biomcp trial search --condition cancer \
  --latitude 40.7128 --longitude -74.0060 --distance 50  # ✅
```

Common coordinates:

- New York: 40.7128, -74.0060
- Los Angeles: 34.0522, -118.2437
- Chicago: 41.8781, -87.6298
- Houston: 29.7604, -95.3698
- Boston: 42.3601, -71.0589

### Preprint Search Issues

**Preprints not appearing:**

```bash
# Check if preprints are being excluded
biomcp article search --gene BRAF --no-preprints  # Excludes preprints

# Include preprints (default)
biomcp article search --gene BRAF  # Includes preprints
```

**DOI not found:**

```bash
# Ensure correct DOI format
biomcp article get "10.1101/2024.01.20.23288905"  # bioRxiv format

# Not all preprints are indexed immediately
# Try searching by title/keywords instead
```

## Performance Issues

### Slow Searches

**1. Reduce result count:**

```bash
# Default may be too high
biomcp article search --gene TP53 --limit 100  # Slow

# Reduce for faster results
biomcp article search --gene TP53 --limit 10   # Fast
```

**2. Use specific filters:**

```bash
# Broad search - slow
biomcp trial search --condition cancer

# Specific search - faster
biomcp trial search --condition "melanoma" --phase PHASE3 \
  --status RECRUITING --country "United States"
```

**3. Check API health:**

```bash
# See which APIs are slow
biomcp health check --verbose

# Check specific API
biomcp health check --apis-only
```

### Timeout Errors

**Increase timeout for slow networks:**

```bash
# Set environment variable
export BIOMCP_TIMEOUT=300  # 5 minutes

# Or use configuration file
echo "timeout: 300" > ~/.biomcp/config.yml
```

**For specific operations:**

```python
# In Python scripts
import asyncio
asyncio.set_event_loop_policy(asyncio.WindowsSelectorEventLoopPolicy())
```

### Memory Issues

**Large result sets:**

```bash
# Process in batches
for i in {1..10}; do
  biomcp article search --gene BRCA1 --page $i --limit 100
done

# Use streaming where available
biomcp article search --gene TP53 --format jsonl | \
  while read line; do
    echo "$line" | jq '.pmid'
  done
```

## MCP Server Issues

### Testing Server Connectivity

**1. Test with MCP Inspector:**

```bash
npx @modelcontextprotocol/inspector uv run --with biomcp-python biomcp run
```

Open http://127.0.0.1:6274 and verify:

- Tools list loads
- Can invoke a simple tool like `gene_getter`

**2. Test with curl (HTTP mode):**

```bash
# Start server in HTTP mode
biomcp run --mode http --port 8000

# Test health endpoint
curl http://localhost:8000/health

# Test MCP endpoint
curl -X POST http://localhost:8000/mcp \
  -H "Content-Type: application/json" \
  -d '{"method": "tools/list"}'
```

### Claude Desktop Integration Issues

**Server not appearing:**

1. Check configuration file location:

   - macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
   - Windows: `%APPDATA%\Claude\claude_desktop_config.json`

2. Validate JSON syntax:

   ```bash
   # macOS
   cat ~/Library/Application\ Support/Claude/claude_desktop_config.json | jq .
   ```

3. Check server starts correctly:
   ```bash
   # Test the exact command from config
   uv run --with biomcp-python biomcp run
   ```

**Server crashes:**
Check logs:

```bash
# Enable debug logging
export BIOMCP_LOG_LEVEL=DEBUG
uv run --with biomcp-python biomcp run
```

Common fixes:

- Update to latest version: `uv tool install biomcp --force`
- Clear cache: `rm -rf ~/.biomcp/cache`
- Check port conflicts: `lsof -i :8000`

## Data Quality Issues

### Outdated Results

**Check data freshness:**

```bash
# See when databases were last updated
biomcp health check --verbose | grep "Last updated"
```

**Clear cache if needed:**

```bash
# Remove cached results
rm -rf ~/.biomcp/cache

# Or set cache TTL
export BIOMCP_CACHE_TTL=900  # 15 minutes
```

### Missing Annotations

**PubTator3 annotations missing:**

- Some newer articles may not be fully annotated yet
- Try searching by PMID directly
- Check if article is indexed: search by title

**Variant annotations incomplete:**

- Not all variants have all annotation types
- Rare variants may lack population frequencies
- Novel variants won't have ClinVar data

## Error Messages

### Common Error Codes

**HTTP 429 - Rate Limit Exceeded:**

```bash
# Add delay between requests
biomcp article search --gene BRAF --delay 1000  # 1 second

# Or reduce parallel requests
export BIOMCP_MAX_CONCURRENT=2
```

**HTTP 404 - Not Found:**

- Check identifier format (PMID, NCT ID, etc.)
- Verify record exists in source database
- Try alternative identifiers

**HTTP 500 - Server Error:**

- External API may be down
- Check status: `biomcp health check`
- Try again later

### Debugging

**Enable verbose logging:**

```bash
# Set log level
export BIOMCP_LOG_LEVEL=DEBUG

# Run with verbose output
biomcp article search --gene BRAF --verbose

# Check log files
tail -f ~/.biomcp/logs/biomcp.log
```

**Report bugs:**
Include when reporting issues:

1. BioMCP version: `biomcp --version`
2. Full error message and stack trace
3. Command that caused the error
4. Operating system and Python version
5. Relevant environment variables

Report at: https://github.com/genomoncology/biomcp/issues

## Getting Help

### Quick Checks

1. **Check FAQ first**: [Frequently Asked Questions](faq-condensed.md)
2. **Search existing issues**: [GitHub Issues](https://github.com/genomoncology/biomcp/issues)
3. **Check examples**: [How-to Guides](how-to-guides/01-find-articles-and-cbioportal-data.md)

### Community Support

- Issue Tracker: Report bugs, request features
- Documentation: PRs welcome for improvements

### Professional Support

For commercial support, contact: support@genomoncology.com

---

_Still having issues? [Open a GitHub issue](https://github.com/genomoncology/biomcp/issues/new) with details._
